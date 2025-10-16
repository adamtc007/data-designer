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
    pub data_type: String,
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
}

// Test helper for WASM
pub async fn test_api_endpoint(endpoint: &str) -> bool {
    let client = Client::new();
    let url = format!("{}/api/health", endpoint);

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            wasm_utils::console_log(&format!("✅ API endpoint {} returned status: {}", endpoint, status));
            status >= 200 && status < 300
        }
        Err(e) => {
            wasm_utils::console_log(&format!("❌ API endpoint {} unreachable: {:?}", endpoint, e));
            false
        }
    }
}