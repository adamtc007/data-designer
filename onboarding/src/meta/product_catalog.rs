use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCatalog { pub products: Vec<Product> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String, // "GlobalCustody@v3"
    pub services: Vec<Service>,
    pub resources: Option<Vec<ResourceBinding>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    #[serde(rename = "serviceId")]
    pub service_id: String,
    #[serde(default)]
    pub options: Vec<ServiceOption>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOption {
    pub id: String,
    pub prompt: String,
    #[serde(rename="type")]
    pub kind: String, // select, multiselect, etc.
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default, rename = "requiredForResources")]
    pub required_for_resources: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBinding {
    pub r#type: String, // "custody-core:v1"
    pub implements: Vec<String>, // serviceIds
}
