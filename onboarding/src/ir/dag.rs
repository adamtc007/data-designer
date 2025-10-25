use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub instance_id: String,
    pub cbu_id: String,
    pub products: Vec<String>,
    pub steps: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: TaskKind,
    pub needs: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskKind {
    SolicitData { options: Vec<String>, attrs: Vec<String>, audience: String },
    ResourceOp { resource: String, op: String },
}
