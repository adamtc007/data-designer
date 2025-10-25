use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules { pub rules: Vec<Rule> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub language: String, // "cue" | "jsonlogic" | "cel" | "rust-fn"
    pub expr: String,
}
