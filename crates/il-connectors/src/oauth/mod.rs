//! Generic OAuth2 authorization-code broker with self-registered providers.

mod routes;

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use il_sdk::{
    NodeError, SafeHttp, Services,
    services::{Credential, Store},
};
use serde_json::{Value, json};
use url::Url;

/// OAuth2 endpoints and scopes for one app.
#[derive(Clone)]
pub struct Provider {
    pub id: &'static str,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub client_id_env: &'static str,
    pub client_secret_env: &'static str,
    pub redirect_uri_env: &'static str,
}

/// A provider factory contributed by its connector module.
pub struct ProviderFactory(pub fn() -> Provider);
inventory::collect!(ProviderFactory);

/// Discovers a provider without a hand-maintained registry.
pub fn provider(id: &str) -> Option<Provider> {
    inventory::iter::<ProviderFactory>.into_iter().map(|factory| (factory.0)()).find(|candidate| candidate.id == id)
}

/// Start/callback/refresh operations over the encrypted credential store.
pub struct Broker {
    provider: Provider,
    store: Arc<dyn Store>,
    http: SafeHttp,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl Broker {
    /// Loads a registered provider and its client credentials from the environment.
    pub fn from_env(id: &str, store: Arc<dyn Store>, http: SafeHttp) -> Result<Self, String> {
        let provider = provider(id).ok_or_else(|| format!("OAuth provider not found: {id}"))?;
        let client_id =
            std::env::var(provider.client_id_env).map_err(|_| format!("{} is unset", provider.client_id_env))?;
        let client_secret = std::env::var(provider.client_secret_env)
            .map_err(|_| format!("{} is unset", provider.client_secret_env))?;
        let redirect_uri =
            std::env::var(provider.redirect_uri_env).map_err(|_| format!("{} is unset", provider.redirect_uri_env))?;
        Ok(Self { provider, store, http, client_id, client_secret, redirect_uri })
    }

    /// Starts consent and saves an expiring one-time CSRF state.
    pub async fn start(&self, workspace_id: &str) -> Result<Url, String> {
        let state = format!("{}{}", ulid::Ulid::new(), ulid::Ulid::new());
        let key = format!("oauth_state:{state}");
        self.store
            .kv_put(
                &key,
                &json!({"workspaceId":workspace_id,"provider":self.provider.id,
            "expiresAt":Utc::now().timestamp()+600,"used":false}),
            )
            .await
            .map_err(|e| e.to_string())?;
        let mut url = Url::parse(&self.provider.auth_url).map_err(|e| e.to_string())?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("state", &state)
            .append_pair("scope", &self.provider.scopes.join(" "))
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
        Ok(url)
    }

    /// Exchanges a callback code and stores tokens through the encrypted Store.
    pub async fn callback(&self, state: &str, code: &str) -> Result<(String, String), String> {
        let key = format!("oauth_state:{state}");
        let mut saved = self.store.kv_get(&key).await.map_err(|e| e.to_string())?.ok_or("unknown OAuth state")?;
        if saved["used"] == true
            || saved["provider"] != self.provider.id
            || saved["expiresAt"].as_i64().unwrap_or(0) < Utc::now().timestamp()
        {
            return Err("expired or used OAuth state".into());
        }
        let workspace_id = saved["workspaceId"].as_str().ok_or("invalid OAuth state")?.to_owned();
        saved["used"] = json!(true);
        self.store.kv_put(&key, &saved).await.map_err(|e| e.to_string())?;
        let response = self
            .http
            .post(&self.provider.token_url)
            .map_err(|e| e.to_string())?
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let body: Value = response.json().await.map_err(|e| e.to_string())?;
        let access = body["access_token"].as_str().ok_or("OAuth token response has no access token")?;
        let refresh = body["refresh_token"].as_str().ok_or("OAuth token response has no refresh token")?;
        let id = format!("cred_{}", ulid::Ulid::new());
        let credential = Credential {
            id: id.clone(),
            app: self.provider.id.into(),
            secret: json!({"access_token":access,"refresh_token":refresh,
                "expires_at":Utc::now().timestamp()+body["expires_in"].as_i64().unwrap_or(3600)}),
            meta: json!({"scope":body["scope"]}),
        };
        self.store.put_credential(&workspace_id, &credential).await.map_err(|e| e.to_string())?;
        Ok((workspace_id, id))
    }

    /// Refreshes an existing credential and preserves its refresh token.
    pub async fn refresh(&self, workspace_id: &str, credential_id: &str) -> Result<(), String> {
        let mut credential = self
            .store
            .credential(workspace_id, credential_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("credential not found")?;
        if credential.app != self.provider.id {
            return Err("credential provider mismatch".into());
        }
        let refresh = credential.secret["refresh_token"].as_str().ok_or("missing refresh token")?;
        let response = self
            .http
            .post(&self.provider.token_url)
            .map_err(|e| e.to_string())?
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let body: Value = response.json().await.map_err(|e| e.to_string())?;
        let access = body["access_token"].as_str().ok_or("OAuth token response has no access token")?;
        credential.secret["access_token"] = json!(access);
        if let Some(refresh) = body["refresh_token"].as_str() {
            credential.secret["refresh_token"] = json!(refresh);
        }
        credential.secret["expires_at"] = json!(Utc::now().timestamp() + body["expires_in"].as_i64().unwrap_or(3600));
        self.store.put_credential(workspace_id, &credential).await.map_err(|e| e.to_string())
    }
}

/// Seconds before expiry at which a token is refreshed.
const REFRESH_EARLY_SECS: i64 = 60;

/// The step's Google access token, refreshed first when it expires within a minute.
pub async fn google_token(
    services: &Services,
    workspace_id: &str,
    credential_ids: &BTreeMap<String, String>,
) -> Result<String, NodeError> {
    let id = credential_ids.get("google").ok_or_else(|| NodeError::MissingCredential("google".into()))?;
    let broker = |app: &str| Broker::from_env(app, services.store.clone(), services.http.clone());
    fresh_token(services.store.as_ref(), workspace_id, id, broker).await
}

/// `credential`'s access token, refreshed through `broker` when it is about to expire. Without a
/// broker (no client secret configured) the saved token is used as it is.
async fn fresh_token(
    store: &dyn Store,
    workspace_id: &str,
    credential_id: &str,
    broker: impl FnOnce(&str) -> Result<Broker, String>,
) -> Result<String, NodeError> {
    let load = async || {
        store
            .credential(workspace_id, credential_id)
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .ok_or_else(|| NodeError::MissingCredential(credential_id.into()))
    };
    let mut credential = load().await?;
    let expires_at = credential.secret["expires_at"].as_i64().unwrap_or(i64::MAX);
    if expires_at - Utc::now().timestamp() <= REFRESH_EARLY_SECS {
        match broker(&credential.app) {
            Ok(broker) => {
                broker.refresh(workspace_id, credential_id).await.map_err(|e| {
                    NodeError::Failed(format!(
                        "Google sign-in expired and could not be renewed ({e}); reconnect it on Connections"
                    ))
                })?;
                credential = load().await?;
            }
            Err(e) => tracing::warn!(credential_id, error = %e, "can't refresh an expiring token"),
        }
    }
    credential.secret["access_token"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| NodeError::Failed("missing Google access token".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c01_google_provider_is_registered() {
        assert_eq!(provider("google").unwrap().id, "google");
    }
    use il_sdk::{services::Store, testkit::MemoryStore};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn c01_oauth_state_callback_and_refresh() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"access_token":"a1","refresh_token":"r1","expires_in":3600})),
            )
            .mount(&server)
            .await;
        let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
        let broker = Broker {
            provider: Provider {
                id: "test",
                auth_url: format!("{}/auth", server.uri()),
                token_url: format!("{}/token", server.uri()),
                scopes: vec!["mail".into()],
                client_id_env: "",
                client_secret_env: "",
                redirect_uri_env: "",
            },
            store: store.clone(),
            http: SafeHttp::permissive_for_tests(),
            client_id: "client".into(),
            client_secret: "secret".into(),
            redirect_uri: "https://app.example/callback".into(),
        };
        let url = broker.start("ws_1").await.unwrap();
        let state = url.query_pairs().find(|(name, _)| name == "state").unwrap().1.to_string();
        let (_, id) = broker.callback(&state, "code").await.unwrap();
        assert!(broker.callback(&state, "code").await.is_err());
        broker.refresh("ws_1", &id).await.unwrap();
        assert_eq!(store.credential("ws_1", &id).await.unwrap().unwrap().secret["refresh_token"], "r1");
    }

    #[tokio::test]
    async fn c07_tokens_are_refreshed_shortly_before_they_expire() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"fresh","expires_in":3600})))
            .expect(1)
            .mount(&server)
            .await;
        let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
        let make = |id: &str, expires_at: i64| Credential {
            id: id.into(),
            app: "test".into(),
            secret: json!({"access_token":"old","refresh_token":"r1","expires_at":expires_at}),
            meta: json!({}),
        };
        let now = Utc::now().timestamp();
        store.put_credential("ws_1", &make("cred_soon", now + 30)).await.unwrap();
        store.put_credential("ws_1", &make("cred_later", now + 3000)).await.unwrap();
        let broker = |_: &str| {
            Ok(Broker {
                provider: Provider {
                    id: "test",
                    auth_url: format!("{}/auth", server.uri()),
                    token_url: format!("{}/token", server.uri()),
                    scopes: vec![],
                    client_id_env: "",
                    client_secret_env: "",
                    redirect_uri_env: "",
                },
                store: store.clone(),
                http: SafeHttp::permissive_for_tests(),
                client_id: "client".into(),
                client_secret: "secret".into(),
                redirect_uri: String::new(),
            })
        };
        assert_eq!(fresh_token(store.as_ref(), "ws_1", "cred_soon", broker).await.unwrap(), "fresh");
        let saved = store.credential("ws_1", "cred_soon").await.unwrap().unwrap();
        assert_eq!(
            (saved.secret["access_token"].as_str(), saved.secret["refresh_token"].as_str()),
            (Some("fresh"), Some("r1"))
        );
        assert_eq!(fresh_token(store.as_ref(), "ws_1", "cred_later", broker).await.unwrap(), "old", "not yet due");
        let no_broker = |_: &str| Err("GOOGLE_CLIENT_SECRET is unset".to_string());
        let other = make("cred_kept", now);
        store.put_credential("ws_1", &other).await.unwrap();
        assert_eq!(
            fresh_token(store.as_ref(), "ws_1", "cred_kept", no_broker).await.unwrap(),
            "old",
            "no client secret"
        );
    }
}
