//! Config: `config/interlock.toml` (git-ignored) with `IL_…` env overrides. Each module reads
//! its own section via [`Config::section`]; secrets come only from env.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// `[server]` section.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub public_url: Option<String>,
    pub public_requests_per_minute: u32,
    pub demo_sessions_per_hour: u32,
    pub templates_dir: String,
    /// Requests without a session use the local workspace (self-hosted). Turn off on a public deploy.
    pub local_workspace: bool,
    /// Behind a reverse proxy: rate limits use the client address it appends to `X-Forwarded-For`.
    pub trust_proxy: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            cors_origins: vec![],
            public_url: None,
            public_requests_per_minute: 120,
            demo_sessions_per_hour: 10,
            templates_dir: "fixtures/templates".into(),
            local_workspace: true,
            trust_proxy: false,
        }
    }
}

/// Loaded configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    /// `fake` (tests, offline) or `real`.
    pub providers: String,
    /// The whole file as JSON, for module sections.
    pub raw: Value,
}

impl Config {
    /// Loads from `IL_CONFIG`, else `config/interlock.toml`, else the checked-in example (so a fresh
    /// clone runs with working model roles), else defaults; then applies env overrides.
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("IL_CONFIG").map(PathBuf::from).unwrap_or_else(|_| {
            let own = PathBuf::from("config/interlock.toml");
            let example = PathBuf::from("config/interlock.example.toml");
            if own.exists() || !example.exists() { own } else { example }
        });
        let raw = if path.exists() { read(&path)? } else { Value::Object(Default::default()) };
        Self::from_value(raw)
    }

    /// Builds config from an already-parsed document plus env overrides.
    pub fn from_value(raw: Value) -> anyhow::Result<Self> {
        let mut server: ServerConfig = section(&raw, "server")?;
        if let Ok(host) = std::env::var("IL_HOST") {
            server.host = host;
        }
        if let Ok(port) = std::env::var("IL_PORT") {
            server.port = port.parse().context("IL_PORT must be a port number")?;
        }
        let providers = std::env::var("IL_PROVIDERS").unwrap_or_else(|_| "fake".into());
        Ok(Self { server, providers, raw })
    }

    /// Deserializes a top-level section (missing → `Default`).
    pub fn section<T: DeserializeOwned + Default>(&self, name: &str) -> anyhow::Result<T> {
        section(&self.raw, name)
    }
}

fn read(path: &Path) -> anyhow::Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let doc: toml::Value = toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(serde_json::to_value(doc)?)
}

fn section<T: DeserializeOwned + Default>(raw: &Value, name: &str) -> anyhow::Result<T> {
    match raw.get(name) {
        Some(v) => serde_json::from_value(v.clone()).with_context(|| format!("invalid [{name}] section")),
        None => Ok(T::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/interlock.example.toml");
        let cfg = Config::from_value(read(&path).unwrap()).unwrap();
        assert!(cfg.raw.get("models").is_some());
        assert!(cfg.server.port > 0);
    }
}
