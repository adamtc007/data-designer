use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub states: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub name: String,          // e.g. "Configure"
    pub from: String,
    pub to: String,
    pub requires_attrs: Vec<String>,
    pub op: Op,
    pub yields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Op {
    Http { method: String, url: String, body_template: serde_json::Value },
    Grpc { endpoint: String, method: String, payload_template: serde_json::Value },
    Kafka { topic: String, key_template: Option<String>, value_template: serde_json::Value },
}
