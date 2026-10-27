//! Agent tool for a selected OpenAPI operation.

use async_trait::async_trait;
use il_sdk::{
    SafeHttp,
    services::{AgentTool, Credential, ToolError},
};
use serde_json::{Map, Value, json};

use super::{request, spec::Operation};

/// A selected operation exposed to an Agent.
pub struct ApiTool {
    operation: Operation,
    http: SafeHttp,
    credential: Option<Credential>,
}

impl ApiTool {
    /// Creates a tool only when its risk policy allows the operation.
    pub fn new(
        operation: Operation,
        http: SafeHttp,
        credential: Option<Credential>,
        allow_risky: bool,
    ) -> Result<Self, ToolError> {
        if operation.risky && !allow_risky {
            return Err(ToolError::InvalidArguments("destructive operation requires allowRisky".into()));
        }
        Ok(Self { operation, http, credential })
    }
}

#[async_trait]
impl AgentTool for ApiTool {
    fn name(&self) -> &str {
        &self.operation.id
    }
    fn description(&self) -> &str {
        &self.operation.description
    }
    fn schema(&self) -> Value {
        let mut properties = Map::new();
        let mut required = Vec::new();
        for parameter in &self.operation.parameters {
            properties.insert(parameter.name.clone(), parameter.schema.clone());
            if parameter.required {
                required.push(parameter.name.clone());
            }
        }
        if let Some(schema) = &self.operation.body_schema {
            properties.insert("body".into(), schema.clone());
        }
        json!({"type":"object","properties":properties,"required":required})
    }
    async fn invoke(&self, args: Value) -> Result<Value, ToolError> {
        request::invoke(&self.http, &self.operation, &args, self.credential.as_ref()).await.map_err(ToolError::Failed)
    }
}

#[cfg(test)]
mod tests {
    use il_sdk::services::{AgentTool, Credential};
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn k02_agent_tool_uses_credential_and_enforces_risk() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/invoices/42"))
            .and(header("x-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"paid":true})))
            .mount(&server)
            .await;
        let operation = Operation {
            id: "getInvoice".into(),
            description: "Get invoice".into(),
            method: "GET".into(),
            base_url: server.uri(),
            path: "/invoices/{invoiceId}".into(),
            parameters: vec![super::super::spec::Parameter {
                name: "invoiceId".into(),
                location: "path".into(),
                required: true,
                schema: json!({"type":"string"}),
            }],
            body_schema: None,
            example_body: None,
            fixed_query: None,
            risky: false,
        };
        let credential = Credential {
            id: "cred_test".into(),
            app: "openapi".into(),
            secret: json!({"api_key":"test-key"}),
            meta: json!({}),
        };
        let tool = ApiTool::new(operation.clone(), SafeHttp::permissive_for_tests(), Some(credential), false).unwrap();
        assert_eq!(tool.schema()["required"], json!(["invoiceId"]));
        assert_eq!(tool.invoke(json!({"invoiceId":"42"})).await.unwrap()["paid"], true);
        let mut destructive = operation;
        destructive.risky = true;
        assert!(ApiTool::new(destructive, SafeHttp::permissive_for_tests(), None, false).is_err());
    }
}
