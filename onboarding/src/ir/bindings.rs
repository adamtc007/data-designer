use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bindings {
    pub tasks: BTreeMap<String, Binding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Binding {
    #[serde(rename="http")]
    Http {
        method: String,
        url: String,
        body: serde_json::Value,
        headers: Option<std::collections::BTreeMap<String,String>>,
    },
    #[serde(other)]
    None,
}
