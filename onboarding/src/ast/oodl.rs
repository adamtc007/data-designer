use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardIntent {
    pub instance_id: String,
    pub cbu_id: String,
    pub products: Vec<String>, // e.g. ["GlobalCustody@v3"]
}
