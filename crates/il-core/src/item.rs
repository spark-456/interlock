//! Items and file references (PRD §8.2).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Json;

/// The unit of data that flows between nodes. Each port carries `Vec<Item>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Item {
    pub json: Json,
    #[serde(default)]
    pub files: BTreeMap<String, FileRef>,
}

impl Item {
    /// An item with JSON only.
    pub fn json(json: Json) -> Self {
        Self { json, files: BTreeMap::new() }
    }

    /// Adds a file under `key`.
    pub fn with_file(mut self, key: impl Into<String>, file: FileRef) -> Self {
        self.files.insert(key.into(), file);
        self
    }
}

/// A pointer to a file held by the `FileStore`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileRef {
    pub id: String,
    pub mime: String,
    #[ts(type = "number")]
    pub bytes: u64,
    pub name: String,
    #[serde(default)]
    pub preview_url: Option<String>,
}
