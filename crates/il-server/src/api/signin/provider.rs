//! What differs between sign-in providers: endpoints, scopes, the token exchange and the profile.

use axum::http::header;
use il_core::dto::UserSummary;
use serde_json::{Value, json};

use crate::wiring::AppState;

/// A sign-in provider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    Github,
    Google,
}

/// The person a provider vouches for: a stable id (the workspace hangs off it) and how to show them.
pub struct Profile {
    pub workspace_id: String,
    pub user: UserSummary,
}

impl Provider {
    pub const ALL: [Provider; 2] = [Provider::Google, Provider::Github];

    pub fn id(self) -> &'static str {
        match self {
            Provider::Github => "github",
            Provider::Google => "google",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Provider::Github => "GitHub",
            Provider::Google => "Google",
        }
    }

    /// `[auth.<id>] <key>`, which tests use to point the provider at a mock server.
    fn setting<'a>(self, state: &'a AppState, key: &str) -> Option<&'a str> {
        state.config.raw.get("auth")?.get(self.id())?.get(key)?.as_str().filter(|s| !s.is_empty())
    }

    /// The OAuth client id: `[auth.<id>] client_id`, or for Google the `GOOGLE_CLIENT_ID` that app
    /// connections already use.
    pub fn client_id(self, state: &AppState) -> Option<String> {
        self.setting(state, "client_id").map(str::to_owned).or_else(|| match self {
            Provider::Google => std::env::var("GOOGLE_CLIENT_ID").ok().filter(|s| !s.is_empty()),
            Provider::Github => None,
        })
    }

    pub fn secret(self, state: &AppState) -> Option<&str> {
        match self {
            Provider::Github => state.github_secret.as_deref(),
            Provider::Google => state.google_secret.as_deref(),
        }
    }

    pub fn configured(self, state: &AppState) -> bool {
        self.client_id(state).is_some() && self.secret(state).is_some()
    }

    pub fn authorize_url(self, state: &AppState) -> &str {
        self.setting(state, "authorize_url").unwrap_or(match self {
            Provider::Github => "https://github.com/login/oauth/authorize",
            Provider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        })
    }

    /// Extra authorize parameters. Google sign-in asks only who the person is; Gmail and Drive
    /// access are separate connections.
    pub fn authorize_params(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Provider::Github => &[("scope", "read:user")],
            Provider::Google => {
                &[("scope", "openid email profile"), ("response_type", "code"), ("prompt", "select_account")]
            }
        }
    }

    /// Exchanges the authorization code for an access token.
    pub async fn access_token(self, state: &AppState, code: &str, redirect_uri: &str) -> Result<String, &'static str> {
        let (client_id, secret) =
            (self.client_id(state).ok_or("unconfigured")?, self.secret(state).ok_or("unconfigured")?);
        let http = &state.services.http;
        let request = match self {
            Provider::Github => {
                let url = self.setting(state, "token_url").unwrap_or("https://github.com/login/oauth/access_token");
                http.post(url).map_err(|_| "request")?.header(header::ACCEPT, "application/json").json(&json!({
                    "client_id": client_id, "client_secret": secret, "code": code, "redirect_uri": redirect_uri
                }))
            }
            Provider::Google => {
                let url = self.setting(state, "token_url").unwrap_or("https://oauth2.googleapis.com/token");
                http.post(url).map_err(|_| "request")?.form(&[
                    ("client_id", client_id.as_str()),
                    ("client_secret", secret),
                    ("code", code),
                    ("redirect_uri", redirect_uri),
                    ("grant_type", "authorization_code"),
                ])
            }
        };
        let reply = request.send().await.map_err(|_| "token")?;
        if !reply.status().is_success() {
            return Err("token");
        }
        let body: Value = reply.json().await.map_err(|_| "token")?;
        body["access_token"].as_str().map(str::to_owned).ok_or("token")
    }

    /// Reads the signed-in person's profile with the access token.
    pub async fn profile(self, state: &AppState, access_token: &str) -> Result<Profile, &'static str> {
        let url = self.setting(state, "user_url").unwrap_or(match self {
            Provider::Github => "https://api.github.com/user",
            Provider::Google => "https://openidconnect.googleapis.com/v1/userinfo",
        });
        let reply = state
            .services
            .http
            .get(url)
            .map_err(|_| "request")?
            .bearer_auth(access_token)
            .header(header::USER_AGENT, "Interlock")
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| "profile")?;
        if !reply.status().is_success() {
            return Err("profile");
        }
        let user: Value = reply.json().await.map_err(|_| "profile")?;
        let text = |key: &str| user[key].as_str().filter(|s| !s.is_empty()).map(str::to_owned);
        match self {
            Provider::Github => {
                let id = user["id"].as_u64().ok_or("profile")?;
                let login = text("login").ok_or("profile")?;
                Ok(Profile {
                    workspace_id: format!("ws_gh_{id}"),
                    user: UserSummary {
                        id: format!("github:{id}"),
                        name: text("name").unwrap_or(login),
                        avatar_url: text("avatar_url"),
                    },
                })
            }
            Provider::Google => {
                let sub = text("sub").filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric())).ok_or("profile")?;
                let name = text("name").or_else(|| text("email")).unwrap_or_else(|| "Google user".into());
                Ok(Profile {
                    workspace_id: format!("ws_gg_{sub}"),
                    user: UserSummary { id: format!("google:{sub}"), name, avatar_url: text("picture") },
                })
            }
        }
    }
}
