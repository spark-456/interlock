//! Live smoke test against Token Factory: `cargo run -p il-ai --example ping [config.toml]`.
//! Needs `NEBIUS_API_KEY`. Checks the configured model ids, then makes a chat, a JSON and an embedding call.

use il_ai::Gateway;
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelGateway, ModelRole};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "config/interlock.example.toml".into());
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(&path)?)?;
    let models = serde_json::to_value(config.get("models").ok_or("no [models] section")?)?;
    let gw = Gateway::from_config(&models, false)?;

    let missing = gw.check().await;
    println!(
        "model check: {}",
        if missing.is_empty() { "all configured models found".into() } else { format!("missing {missing:?}") }
    );

    let ask = |text: &str| ChatRequest { messages: vec![ChatMessage::user(text)], ..Default::default() };
    for role in [ModelRole::Fast, ModelRole::Agent, ModelRole::Architect] {
        let r = gw.chat(role, ask("Reply with the single word: pong")).await?;
        println!(
            "{role:?} ({}): {:?} — {} + {} tokens, ${:.6}",
            r.model,
            r.content.trim(),
            r.usage.prompt_tokens,
            r.usage.completion_tokens,
            r.usage.cost_usd
        );
    }

    let schema = json!({"type": "object", "required": ["ok"], "properties": {"ok": {"type": "boolean"}}});
    let r = gw
        .chat(ModelRole::Fast, ChatRequest { json_schema: Some(schema), ..ask("Is water wet? Answer as JSON.") })
        .await?;
    println!("JSON: {}", r.content);

    let v = gw.embed(&["Interlock".into()]).await?;
    println!("embedding: {} dimensions", v.first().map_or(0, Vec::len));
    Ok(())
}
