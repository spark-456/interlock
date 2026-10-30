//! `il-server [serve]` runs the server (with `IL_BUNDLE=<folder>`, an exported bundle: see
//! `docs/EXPORT.md`); `il-server catalog` prints the node catalog as JSON (used by `cargo xtask catalog`).

use anyhow::Context;
use il_core::dto::CatalogResponse;
use il_server::{Config, app};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    match cmd.as_str() {
        "catalog" => {
            il_server::wiring::link_all();
            let catalog = CatalogResponse { nodes: il_sdk::registry::catalog() };
            println!("{}", serde_json::to_string_pretty(&catalog)?);
            Ok(())
        }
        "serve" => serve().await,
        other => anyhow::bail!("unknown command `{other}` (expected `serve` or `catalog`)"),
    }
}

async fn serve() -> anyhow::Result<()> {
    let json_logs = std::env::var("IL_LOG_JSON").is_ok();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=warn"));
    if json_logs {
        tracing_subscriber::fmt().with_env_filter(filter).json().init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
    let config = Config::load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let state = il_server::wiring::build_state_persistent(config).await?;
    if let Some(dir) = std::env::var_os("IL_BUNDLE") {
        il_server::bundle::load(&state, std::path::Path::new(&dir), |name| std::env::var(name).ok()).await?;
    }
    il_server::wiring::recover_startup(&state).await?;
    tracing::info!(nodes = il_sdk::registry::catalog().len(), "node catalog loaded");
    let listener = tokio::net::TcpListener::bind(&addr).await.with_context(|| format!("binding {addr}"))?;
    tracing::info!("Interlock listening on http://{addr}");
    axum::serve(listener, app(state).into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
