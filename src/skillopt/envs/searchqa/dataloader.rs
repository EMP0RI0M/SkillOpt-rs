// Auto-synthesized Rust implementation for skillopt/envs/searchqa/dataloader.py
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/* SearchQA task dataloader. */

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQADataLoader {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

impl SearchQADataLoader {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn _load_items(path: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::Value::Null)
}

pub fn load_raw_items(data_path: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::Value::Null)
}
