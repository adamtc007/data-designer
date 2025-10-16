use axum::{
    extract::{Path, Json},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use tower_http::cors::CorsLayer;

// Template data structures matching the WASM client
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
pub struct HealthCheckResponse {
    pub status: String,
    pub message: String,
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

const TEMPLATES_FILE_PATH: &str = "../resource_templates.json";

fn create_template_router() -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/templates", get(get_all_templates))
        .route("/api/templates/:id", get(get_template))
        .route("/api/templates/:id", put(upsert_template))
        .layer(CorsLayer::permissive()) // Enable CORS for browser requests
}

#[axum::debug_handler]
async fn health_check() -> Result<ResponseJson<HealthCheckResponse>, StatusCode> {
    info!("Health check endpoint called");

    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        message: "Template API is running".to_string(),
    };

    Ok(ResponseJson(response))
}

async fn get_all_templates() -> Result<ResponseJson<GetAllTemplatesResponse>, StatusCode> {
    info!("Getting all templates from file: {}", TEMPLATES_FILE_PATH);

    match load_templates_from_file().await {
        Ok(templates) => {
            info!("Successfully loaded {} templates", templates.len());
            let response = GetAllTemplatesResponse { templates };
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("Failed to load templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_template(Path(id): Path<String>) -> Result<ResponseJson<ResourceTemplate>, StatusCode> {
    info!("Getting template with id: {}", id);

    match load_templates_from_file().await {
        Ok(templates) => {
            match templates.get(&id) {
                Some(template) => {
                    info!("Successfully found template: {}", id);
                    Ok(ResponseJson(template.clone()))
                }
                None => {
                    error!("Template not found: {}", id);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        Err(e) => {
            error!("Failed to load templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn upsert_template(
    Path(id): Path<String>,
    Json(template): Json<ResourceTemplate>,
) -> Result<ResponseJson<UpsertTemplateResponse>, (StatusCode, String)> {
    info!("Upserting template with id: {}", id);

    match load_templates_from_file().await {
        Ok(mut templates) => {
            // Update or insert the template
            let mut updated_template = template;
            updated_template.id = id.clone(); // Ensure ID matches URL path

            let is_new = !templates.contains_key(&id);
            templates.insert(id.clone(), updated_template);

            // Save back to file
            match save_templates_to_file(&templates).await {
                Ok(_) => {
                    let message = if is_new {
                        format!("Template '{}' created successfully", id)
                    } else {
                        format!("Template '{}' updated successfully", id)
                    };

                    info!("{}", message);

                    let response = UpsertTemplateResponse {
                        success: true,
                        message,
                    };
                    Ok(ResponseJson(response))
                }
                Err(e) => {
                    error!("Failed to save templates: {}", e);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save templates: {}", e)))
                }
            }
        }
        Err(e) => {
            error!("Failed to load templates: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load templates: {}", e)))
        }
    }
}

async fn load_templates_from_file() -> Result<HashMap<String, ResourceTemplate>, anyhow::Error> {
    let content = fs::read_to_string(TEMPLATES_FILE_PATH).await?;
    let templates: HashMap<String, ResourceTemplate> = serde_json::from_str(&content)?;
    Ok(templates)
}

async fn save_templates_to_file(templates: &HashMap<String, ResourceTemplate>) -> Result<(), anyhow::Error> {
    let content = serde_json::to_string_pretty(templates)?;
    fs::write(TEMPLATES_FILE_PATH, content).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create HTTP template API router
    let template_router = create_template_router();

    // Server address
    let http_addr = "0.0.0.0:3030".parse::<std::net::SocketAddr>()?;

    info!("🚀 Starting Template API server on {}", http_addr);
    info!("📁 Template file: {}", TEMPLATES_FILE_PATH);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    info!("✅ Template API server ready!");

    axum::serve(listener, template_router).await?;

    Ok(())
}