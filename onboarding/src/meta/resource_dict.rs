use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDicts {
    #[serde(rename = "resourceTypes")]
    pub resource_types: Vec<ResourceType>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceType {
    pub id: String, // "custody-core:v1"
    pub dictionary: Dictionary,
    pub lifecycle: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dictionary { pub attrs: Vec<Attr> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attr {
    pub key: String,
    #[serde(rename="type")]
    pub r#type: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub derive: Option<Derive>,
    pub source: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derive { pub rule: String }
