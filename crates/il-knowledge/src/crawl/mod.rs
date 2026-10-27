//! Tavily crawl client with an enforced per-source page cap.

use il_sdk::{SafeHttp, services::knowledge::KnowledgeError};
use serde_json::{Value, json};

/// One crawled page.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub url: String,
    pub text: String,
}

/// Uses Tavily's documented Crawl endpoint.
#[derive(Clone)]
pub struct Tavily {
    http: SafeHttp,
    crawl_endpoint: String,
    extract_endpoint: String,
    key: String,
}

impl Tavily {
    /// Creates a production client. The key is read only from the environment.
    pub fn from_env(http: SafeHttp) -> Result<Self, KnowledgeError> {
        let key =
            std::env::var("TAVILY_API_KEY").map_err(|_| KnowledgeError::Other("TAVILY_API_KEY is not set".into()))?;
        Ok(Self {
            http,
            crawl_endpoint: "https://api.tavily.com/crawl".into(),
            extract_endpoint: "https://api.tavily.com/extract".into(),
            key,
        })
    }

    /// Injects a local endpoint for wiremock tests.
    #[cfg(test)]
    pub fn for_test(base_url: String) -> Self {
        Self {
            http: SafeHttp::permissive_for_tests(),
            crawl_endpoint: format!("{base_url}/crawl"),
            extract_endpoint: format!("{base_url}/extract"),
            key: "test".into(),
        }
    }

    /// Crawls at most 50 pages from `url` and returns their extracted text.
    pub async fn crawl(&self, url: &str, limit: u32) -> Result<Vec<Page>, KnowledgeError> {
        self.http.validate_destination(url).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let limit = limit.clamp(1, 50);
        let response = self.http.post(&self.crawl_endpoint).map_err(|e| KnowledgeError::Other(e.to_string()))?
            .bearer_auth(&self.key)
            .json(&json!({"url":url,"limit":limit,"max_depth":2,"max_breadth":20,"allow_external":false,"format":"markdown"}))
            .send().await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?
            .error_for_status().map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let body: Value = response.json().await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let pages =
            body["results"].as_array().ok_or_else(|| KnowledgeError::Ingest("Tavily returned no results".into()))?;
        Ok(pages
            .iter()
            .take(limit as usize)
            .filter_map(|page| {
                Some(Page { url: page["url"].as_str()?.into(), text: page["raw_content"].as_str()?.into() })
            })
            .collect())
    }

    /// Extracts URLs in batches of at most 20.
    pub async fn extract(&self, urls: &[String]) -> Result<Vec<Page>, KnowledgeError> {
        let mut pages = Vec::new();
        for batch in urls.chunks(20) {
            for url in batch {
                self.http.validate_destination(url).await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
            }
            let response = self
                .http
                .post(&self.extract_endpoint)
                .map_err(|e| KnowledgeError::Other(e.to_string()))?
                .bearer_auth(&self.key)
                .json(&json!({"urls":batch,"extract_depth":"basic","format":"markdown"}))
                .send()
                .await
                .map_err(|e| KnowledgeError::Ingest(e.to_string()))?
                .error_for_status()
                .map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
            let body: Value = response.json().await.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
            let results = body["results"]
                .as_array()
                .ok_or_else(|| KnowledgeError::Ingest("Tavily returned no results".into()))?;
            pages.extend(results.iter().filter_map(|page| {
                Some(Page { url: page["url"].as_str()?.into(), text: page["raw_content"].as_str()?.into() })
            }));
        }
        Ok(pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn k01_crawl_caps_pages_and_disables_external_links() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/crawl"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results":[
                {"url":"https://example.com/a","raw_content":"A"},
                {"url":"https://example.com/b","raw_content":"B"}
            ]})))
            .mount(&server)
            .await;
        let client = Tavily::for_test(server.uri());
        let pages = client.crawl("https://example.com", 1).await.unwrap();
        assert_eq!(pages.len(), 1);
        let requests = server.received_requests().await.unwrap();
        let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["limit"], 1);
        assert_eq!(body["allow_external"], false);
    }

    #[tokio::test]
    async fn k01_extract_uses_documented_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/extract"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results":[
                {"url":"https://example.com/a","raw_content":"Policy"}
            ]})))
            .mount(&server)
            .await;
        let client = Tavily::for_test(server.uri());
        let pages = client.extract(&["https://example.com/a".into()]).await.unwrap();
        assert_eq!(pages[0].text, "Policy");
    }
}
