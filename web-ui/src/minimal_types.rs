use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Minimal resource sheet record for web demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSheetRecord {
    pub resource_id: String,
    pub resource_type: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub client_id: Option<String>,
    pub product_id: Option<String>,
    pub status: String,
    pub json_data: JsonValue,
    pub metadata: JsonValue,
    pub created_by: String,
    pub tags: JsonValue,
}