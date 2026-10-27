//! Outbound notifier for Slack, Discord and Telegram approval channels.

use std::sync::Arc;

use async_trait::async_trait;
use il_sdk::prelude::{Value, json};
use il_sdk::services::{Notification, Notifier, NotifyError, Services};

/// Sends external notifications using credentials from the workspace store.
pub struct MessagingNotifier {
    services: Services,
    fallback: Arc<dyn Notifier>,
}

impl MessagingNotifier {
    /// Keeps the existing notifier for in-app and other channels.
    pub fn new(services: Services, fallback: Arc<dyn Notifier>) -> Self {
        Self { services, fallback }
    }

    async fn secret(&self, note: &Notification) -> Result<Value, NotifyError> {
        let id = note.credential_id.as_deref().ok_or_else(|| NotifyError::Send("credential is required".into()))?;
        let credential = self
            .services
            .store
            .credential(&note.workspace_id, id)
            .await
            .map_err(|e| NotifyError::Send(e.to_string()))?
            .ok_or_else(|| NotifyError::Send("credential not found".into()))?;
        if credential.app != note.channel {
            return Err(NotifyError::Send("credential app mismatch".into()));
        }
        Ok(credential.secret)
    }
}

#[async_trait]
impl Notifier for MessagingNotifier {
    async fn send(&self, note: &Notification) -> Result<Option<String>, NotifyError> {
        match note.channel.as_str() {
            "slack" => {
                let secret = self.secret(note).await?;
                let token = secret["access_token"]
                    .as_str()
                    .or_else(|| secret["bot_token"].as_str())
                    .ok_or_else(|| NotifyError::Send("Slack token missing".into()))?;
                let channel =
                    note.target.as_deref().ok_or_else(|| NotifyError::Send("Slack channel missing".into()))?;
                let text = format!("{}\n{}", note.title, note.body);
                let blocks = note.approval_id.as_ref().map(|id| json!([
                    {"type":"section","text":{"type":"mrkdwn","text":text}},
                    {"type":"actions","elements":[
                        {"type":"button","text":{"type":"plain_text","text":"Approve"},"style":"primary","action_id":"approve","value":id},
                        {"type":"button","text":{"type":"plain_text","text":"Reject"},"style":"danger","action_id":"reject","value":id}
                    ]}
                ]));
                let response =
                    super::post_message(&self.services.http, "https://slack.com/api", token, channel, &text, blocks)
                        .await
                        .map_err(|e| NotifyError::Send(e.to_string()))?;
                Ok(response["ts"].as_str().map(str::to_owned))
            }
            "discord" => {
                let secret = self.secret(note).await?;
                let url =
                    secret["url"].as_str().ok_or_else(|| NotifyError::Send("Discord webhook URL missing".into()))?;
                let parsed =
                    url::Url::parse(url).map_err(|_| NotifyError::Send("invalid Discord webhook URL".into()))?;
                if parsed.scheme() != "https"
                    || parsed.host_str() != Some("discord.com")
                    || !parsed.path().starts_with("/api/webhooks/")
                {
                    return Err(NotifyError::Send("invalid Discord webhook URL".into()));
                }
                let mut parsed = parsed;
                parsed.query_pairs_mut().append_pair("wait", "true");
                let response = self
                    .services
                    .http
                    .post(parsed.as_str())
                    .map_err(|_| NotifyError::Send("invalid Discord destination".into()))?
                    .json(&json!({"content":format!("{}\n{}", note.title, note.body)}))
                    .send()
                    .await
                    .map_err(|_| NotifyError::Send("Discord request failed".into()))?;
                if !response.status().is_success() {
                    return Err(NotifyError::Send(format!("Discord HTTP {}", response.status())));
                }
                let body: Value =
                    response.json().await.map_err(|_| NotifyError::Send("invalid Discord response".into()))?;
                Ok(body["id"].as_str().map(str::to_owned))
            }
            "telegram" => {
                let secret = self.secret(note).await?;
                let token =
                    secret["token"].as_str().ok_or_else(|| NotifyError::Send("Telegram bot token missing".into()))?;
                let chat_id =
                    note.target.as_deref().ok_or_else(|| NotifyError::Send("Telegram chat missing".into()))?;
                let url = format!("https://api.telegram.org/bot{token}/sendMessage");
                let response = self
                    .services
                    .http
                    .post(&url)
                    .map_err(|_| NotifyError::Send("invalid Telegram destination".into()))?
                    .json(&json!({"chat_id":chat_id,"text":format!("{}\n{}",note.title,note.body)}))
                    .send()
                    .await
                    .map_err(|_| NotifyError::Send("Telegram request failed".into()))?;
                if !response.status().is_success() {
                    return Err(NotifyError::Send(format!("Telegram HTTP {}", response.status())));
                }
                let body: Value =
                    response.json().await.map_err(|_| NotifyError::Send("invalid Telegram response".into()))?;
                if body["ok"] != true {
                    return Err(NotifyError::Send("Telegram rejected message".into()));
                }
                Ok(body["result"]["message_id"].as_i64().map(|id| id.to_string()))
            }
            _ => self.fallback.send(note).await,
        }
    }
}
