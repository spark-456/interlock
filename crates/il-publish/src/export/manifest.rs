//! `interlock.bundle.json`: what a bundle holds, read back by the engine when it loads the bundle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bundle format version; the loader refuses newer ones.
pub const FORMAT: u32 = 1;
/// The manifest's path inside the bundle.
pub const PATH: &str = "interlock.bundle.json";
/// Prefix of a setting whose value lives in a file under `prompts/`.
pub const PROMPT_REF: &str = "@prompts/";

/// The bundle manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: u32,
    pub exported_at: DateTime<Utc>,
    pub engine: Engine,
    /// Flow definitions, as paths inside the bundle.
    pub flows: Vec<String>,
    pub files: Vec<FileSpec>,
    pub connections: Vec<Connection>,
    pub webhooks: Vec<Webhook>,
    /// Every environment variable the bundle reads, as listed in `.env.example`.
    pub env: Vec<EnvVar>,
}

/// The engine that runs the bundle: built from `source` at `git_ref`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Engine {
    pub version: String,
    pub source: String,
    pub git_ref: String,
}

/// A file a flow refers to by id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSpec {
    /// The id the exported flow uses; the loader points it at the stored copy.
    pub id: String,
    pub path: String,
    pub name: String,
    pub mime: String,
}

/// A connection (credential) a flow's steps use. Only its shape is exported; the values come from env.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    /// The credential id the flow's steps are bound to.
    pub id: String,
    pub app: String,
    pub label: String,
    /// Signs in with OAuth: the loader stores the refresh token and the engine fetches access tokens.
    pub oauth: bool,
    pub fields: Vec<Field>,
    /// Names of the steps that use it.
    pub used_by: Vec<String>,
}

/// One secret field of a connection and the env var that holds it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub key: String,
    pub env: String,
}

/// A webhook trigger; its token comes from `INTERLOCK_WEBHOOK_TOKEN`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub flow_id: String,
    pub node_id: String,
    pub name: String,
}

/// An environment variable the bundle reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub required: bool,
    /// One line for `.env.example` and the README.
    pub about: String,
}
