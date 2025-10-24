use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use crate::wasm_utils;

#[derive(Error, Debug)]
pub enum HttpApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Server error: {status}")]
    ServerError { status: u16 },
    #[error("Not connected to server")]
    NotConnected,
}

pub type Result<T> = std::result::Result<T, HttpApiError>;

// API data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub message: String,
}

// Template management structures for the file-based system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    pub id: String,
    pub description: String,
    pub attributes: Vec<TemplateAttribute>,
    pub dsl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAttribute {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    #[serde(rename = "allowedValues")]
    pub allowed_values: Option<Vec<String>>,
    pub ui: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAllTemplatesResponse {
    pub templates: HashMap<String, ResourceTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTemplateResponse {
    pub success: bool,
    pub message: String,
}

// Private Attributes API data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateAttributeDefinition {
    pub id: Option<i32>,
    pub attribute_name: String,
    pub description: String,
    pub data_type: String,
    pub visibility_scope: String,
    pub attribute_class: String,
    pub source_attributes: Vec<String>,
    pub filter_expression: Option<String>,
    pub transformation_logic: Option<String>,
    pub regex_pattern: Option<String>,
    pub validation_tests: Option<String>,
    pub materialization_strategy: String,
    pub derivation_rule_ebnf: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrivateAttributeRequest {
    pub attribute_name: String,
    pub description: String,
    pub data_type: String,
    pub source_attributes: Vec<String>,
    pub filter_expression: Option<String>,
    pub transformation_logic: Option<String>,
    pub regex_pattern: Option<String>,
    pub validation_tests: Option<String>,
    pub materialization_strategy: String,
    pub derivation_rule_ebnf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrivateAttributeResponse {
    pub success: bool,
    pub attribute_id: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPrivateAttributesResponse {
    pub attributes: Vec<PrivateAttributeDefinition>,
}

#[derive(Debug, Clone)]
pub struct DataDesignerHttpClient {
    client: Client,
    base_url: String,
    connected: bool,
}

impl DataDesignerHttpClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();

        Self {
            client,
            base_url: base_url.to_string(),
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        wasm_utils::console_log(&format!("🔌 Connecting to API at: {}", self.base_url));

        // Test connection with health check
        match self.health_check().await {
            Ok(_) => {
                self.connected = true;
                wasm_utils::console_log("✅ Successfully connected to API");
                Ok(())
            }
            Err(e) => {
                self.connected = false;
                wasm_utils::console_log(&format!("❌ Failed to connect: {:?}", e));
                Err(e)
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        wasm_utils::console_log("🔌 Disconnected from API");
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if connected {
            wasm_utils::console_log("✅ API client marked as connected");
        } else {
            wasm_utils::console_log("❌ API client marked as disconnected");
        }
    }

    pub async fn health_check(&self) -> Result<HealthCheckResponse> {
        let url = format!("{}/api/health", self.base_url);

        wasm_utils::console_log(&format!("🏥 Health check: {}", url));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let health_response: HealthCheckResponse = response.json().await?;
        Ok(health_response)
    }

    // Template management methods for the file-based system
    pub async fn get_all_templates(&self) -> Result<GetAllTemplatesResponse> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/templates", self.base_url);

        wasm_utils::console_log(&format!("📋 Fetching all templates: {}", url));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let templates_response: GetAllTemplatesResponse = response.json().await?;
        Ok(templates_response)
    }

    pub async fn get_template(&self, id: &str) -> Result<ResourceTemplate> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/templates/{}", self.base_url, id);

        wasm_utils::console_log(&format!("📄 Fetching template {}: {}", id, url));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let template: ResourceTemplate = response.json().await?;
        Ok(template)
    }

    pub async fn upsert_template(&self, id: &str, template: ResourceTemplate) -> Result<UpsertTemplateResponse> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/templates/{}", self.base_url, id);

        wasm_utils::console_log(&format!("💾 Saving template {}: {}", id, url));

        let response = self.client
            .put(&url)
            .json(&template)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let upsert_response: UpsertTemplateResponse = response.json().await?;
        Ok(upsert_response)
    }

    pub async fn create_template_from_baseline(&self, new_id: &str) -> Result<ResourceTemplate> {
        wasm_utils::console_log(&format!("✨ Creating new template {} from baseline", new_id));

        // First get the baseline template
        let baseline = self.get_template("baseline_template").await?;

        // Create a new template based on baseline
        let new_template = ResourceTemplate {
            id: new_id.to_string(),
            description: format!("New template based on baseline: {}", new_id),
            attributes: baseline.attributes,
            dsl: baseline.dsl,
        };

        Ok(new_template)
    }

    // Private Attributes API methods

    pub async fn get_private_attributes(&self) -> Result<GetPrivateAttributesResponse> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/private-attributes", self.base_url);

        wasm_utils::console_log(&format!("📊 Fetching private attributes: {}", url));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let attributes_response: GetPrivateAttributesResponse = response.json().await?;
        wasm_utils::console_log(&format!("✅ Retrieved {} private attributes", attributes_response.attributes.len()));
        Ok(attributes_response)
    }

    pub async fn create_private_attribute(&self, request: CreatePrivateAttributeRequest) -> Result<CreatePrivateAttributeResponse> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/private-attributes", self.base_url);

        wasm_utils::console_log(&format!("➕ Creating private attribute '{}': {}", request.attribute_name, url));

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let create_response: CreatePrivateAttributeResponse = response.json().await?;
        wasm_utils::console_log(&format!("✅ Created private attribute with ID: {}", create_response.attribute_id));
        Ok(create_response)
    }

    pub async fn get_private_attribute(&self, id: i32) -> Result<PrivateAttributeDefinition> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/private-attributes/{}", self.base_url, id);

        wasm_utils::console_log(&format!("📄 Fetching private attribute {}: {}", id, url));

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let attribute: PrivateAttributeDefinition = response.json().await?;
        Ok(attribute)
    }

    pub async fn update_private_attribute(&self, id: i32, request: CreatePrivateAttributeRequest) -> Result<CreatePrivateAttributeResponse> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/private-attributes/{}", self.base_url, id);

        wasm_utils::console_log(&format!("✏️ Updating private attribute {}: {}", id, url));

        let response = self.client
            .put(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        let update_response: CreatePrivateAttributeResponse = response.json().await?;
        wasm_utils::console_log(&format!("✅ Updated private attribute {}", id));
        Ok(update_response)
    }

    pub async fn delete_private_attribute(&self, id: i32) -> Result<()> {
        if !self.connected {
            return Err(HttpApiError::NotConnected);
        }

        let url = format!("{}/api/private-attributes/{}", self.base_url, id);

        wasm_utils::console_log(&format!("🗑️ Deleting private attribute {}: {}", id, url));

        let response = self.client
            .delete(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpApiError::ServerError {
                status: response.status().as_u16(),
            });
        }

        wasm_utils::console_log(&format!("✅ Deleted private attribute {}", id));
        Ok(())
    }
}

// Test helper for WASM
pub async fn test_api_endpoint(endpoint: &str) -> bool {
    let client = Client::new();
    let url = format!("{}/api/health", endpoint);

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            wasm_utils::console_log(&format!("✅ API endpoint {} returned status: {}", endpoint, status));
            (200..300).contains(&status)
        }
        Err(e) => {
            wasm_utils::console_log(&format!("❌ API endpoint {} unreachable: {:?}", endpoint, e));
            false
        }
    }
}