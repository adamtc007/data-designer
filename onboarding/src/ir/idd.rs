use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idd {
    pub schema: BTreeMap<String, AttrSchema>,
    pub values: BTreeMap<String, serde_json::Value>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrSchema {
    pub r#type: String,
    pub required: bool,
    pub provenance: Vec<String>,
    pub default: Option<serde_json::Value>,
}
