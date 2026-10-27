//! Live architect run: `cargo run -p il-server --example architect -- "<goal>" [config.toml]`.
//! Needs `NEBIUS_API_KEY`. Builds a flow from the goal with the real Architect model and the
//! registered node catalog, printing the build log and the finished flow.

use std::sync::Arc;
use std::time::Duration;

use il_architect::Architect;
use il_architect::catalog::Catalog;
use il_core::BuildEventKind;
use il_core::dto::BuildRequest;
use il_sdk::services::ArchitectService;
use il_sdk::testkit::FakeServices;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let goal = args.next().ok_or_else(|| anyhow::anyhow!("usage: architect \"<goal>\" [config.toml]"))?;
    let path = args.next().unwrap_or_else(|| "config/interlock.example.toml".into());
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(&path)?)?;
    let models = serde_json::to_value(config.get("models").ok_or_else(|| anyhow::anyhow!("no [models] section"))?)?;

    il_server::wiring::link_all();
    let fakes = FakeServices::default();
    let mut services = fakes.services();
    services.models = Arc::new(il_ai::Gateway::from_config(&models, false)?);
    let architect = Architect::new(services.clone(), Catalog::registered());
    let req = BuildRequest { goal, kind: None, attachments: vec![], hints: vec![] };
    let id = architect.start_build("ws_live", req).await?;

    let mut after = 0;
    loop {
        for ev in services.store.build_events(&id, after).await? {
            after = ev.seq;
            match ev.kind {
                BuildEventKind::Ready { flow_id } => {
                    let flow = services.store.get_flow(&flow_id).await?;
                    println!("{}", serde_json::to_string_pretty(&flow)?);
                    return Ok(());
                }
                BuildEventKind::Failed { error } => anyhow::bail!("build failed: {error}"),
                BuildEventKind::Question { questions } => anyhow::bail!("the architect asked: {questions:?}"),
                kind => println!("{}", serde_json::to_string(&kind)?),
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
