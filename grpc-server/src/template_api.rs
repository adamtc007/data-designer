use axum::{
    extract::{Path, Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use tower_http::cors::CorsLayer;
use sqlx::PgPool;
use data_designer_core::cbu_dsl::CbuDslParser;
use data_designer_core::lisp_cbu_dsl::LispCbuParser;
use data_designer_core::dsl_utils;

// Import gRPC types for HTTP endpoint compatibility
pub mod financial_taxonomy {
    tonic::include_proto!("financial_taxonomy");
}
// use financial_taxonomy::*;  // Using explicit crate::financial_taxonomy:: instead

// Import the gRPC service implementation and trait
use crate::TaxonomyServer;
use crate::financial_taxonomy::financial_taxonomy_service_server::FinancialTaxonomyService;

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


// White Truffle HTTP API structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateResourceRequest {
    pub template_id: String,
    pub onboarding_request_id: String,
    pub context: Option<String>,
    pub initial_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateResourceResponse {
    pub success: bool,
    pub message: String,
    pub instance: Option<ResourceInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInstance {
    pub instance_id: String,
    pub onboarding_request_id: String,
    pub template_id: String,
    pub status: String,
    pub instance_data: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteDslRequest {
    pub instance_id: String,
    pub execution_context: Option<String>,
    pub input_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteDslResponse {
    pub success: bool,
    pub message: String,
    pub result: Option<DslExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslExecutionResult {
    pub instance_id: String,
    pub execution_status: String,
    pub output_data: String,
    pub log_messages: Vec<String>,
    pub error_details: Option<String>,
    pub executed_at: Option<String>,
    pub execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAiSuggestionsRequest {
    pub query: String,
    pub context: Option<String>,
    pub ai_provider: Option<AiProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAiSuggestionsResponse {
    pub suggestions: Vec<AiSuggestion>,
    pub status_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub provider_type: i32, // 0=OpenAI, 1=Anthropic, 2=Offline
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub title: String,
    pub description: String,
    pub category: String,
    pub confidence: f64,
    pub applicable_contexts: Vec<String>,
}

const TEMPLATES_FILE_PATH: &str = "../resource_templates.json";

pub fn create_template_router(db_pool: PgPool, taxonomy_server: std::sync::Arc<TaxonomyServer>) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/templates", get(get_all_templates))
        .route("/api/templates/:id", get(get_template))
        // TODO: Fix axum handler issue for PUT route
        // .route("/api/templates/:id", put(upsert_template))
        // White Truffle HTTP endpoints
        .route("/api/instantiate", post(instantiate_resource))
        .route("/api/execute-dsl", post(execute_dsl))
        .route("/api/execute-cbu-dsl", post(execute_cbu_dsl_http))
        .route("/api/list-cbus", post(list_cbus))
        .route("/api/ai-suggestions", post(get_ai_suggestions))
        .route("/api/entities", post(get_entities))
        .route("/api/list-products", post(list_products))
        .with_state((db_pool, taxonomy_server))
        .layer(CorsLayer::permissive()) // Enable CORS for browser requests
}

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


async fn load_templates_from_file() -> Result<HashMap<String, ResourceTemplate>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(TEMPLATES_FILE_PATH).await?;
    let templates: HashMap<String, ResourceTemplate> = serde_json::from_str(&content)?;
    Ok(templates)
}


// White Truffle HTTP endpoint handlers
// Removed old mock HTTP functions - using gRPC delegation pattern instead

// Removed duplicate HTTP functions - using gRPC delegation pattern instead

async fn execute_cbu_dsl_http(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ExecuteCbuDsl called");

    // Extract the dsl_script from the request
    let dsl_script = request["dsl_script"].as_str()
        .unwrap_or("")
        .to_string();

    if dsl_script.is_empty() {
        let error_response = serde_json::json!({
            "success": false,
            "message": "DSL script is required",
            "cbu_id": null,
            "validation_errors": ["DSL script cannot be empty"],
            "data": null
        });
        return Ok(ResponseJson(error_response));
    }

    info!("Processing DSL script: {}", dsl_script);

    // Try LISP parser first (for S-expression syntax)
    // Check for LISP syntax: starts with '(' or contains LISP comments ';'
    let cleaned_dsl = dsl_utils::strip_comments(&dsl_script);
    let is_lisp_syntax = dsl_script.trim_start().starts_with('(') ||
                        dsl_script.contains(';') ||
                        cleaned_dsl.trim_start().starts_with('(');

    if is_lisp_syntax {
        info!("Detected LISP syntax, using LISP parser");
        let mut lisp_parser = LispCbuParser::new(Some(pool.clone()));

        match lisp_parser.parse_and_eval(&dsl_script) {
            Ok(result) => {
                let response = serde_json::json!({
                    "success": true,
                    "message": format!("LISP DSL executed successfully: {}", result.message),
                    "cbu_id": result.cbu_id,
                    "validation_errors": [],
                    "data": result.data.map(|d| serde_json::to_string(&d).unwrap_or_else(|_| "null".to_string()))
                });
                Ok(ResponseJson(response))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "success": false,
                    "message": format!("LISP Parse failed: {}", e),
                    "cbu_id": null,
                    "validation_errors": [e.to_string()],
                    "data": null
                });
                Ok(ResponseJson(error_response))
            }
        }
    } else {
        // Fallback to original EBNF parser for compatibility
        info!("Using traditional EBNF parser");
        let parser = CbuDslParser::new(Some(pool.clone()));

        match parser.parse_cbu_dsl(&dsl_script) {
            Ok(command) => {
                match parser.execute_cbu_dsl(command).await {
                    Ok(result) => {
                        let response = serde_json::json!({
                            "success": result.success,
                            "message": result.message,
                            "cbu_id": result.cbu_id,
                            "validation_errors": result.validation_errors,
                            "data": result.data.map(|d| serde_json::to_string(&d).unwrap_or_default())
                        });
                        Ok(ResponseJson(response))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "success": false,
                            "message": format!("Execution failed: {}", e),
                            "cbu_id": null,
                            "validation_errors": [e.to_string()],
                            "data": null
                        });
                        Ok(ResponseJson(error_response))
                    }
                }
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "success": false,
                    "message": format!("Parse failed: {}", e),
                    "cbu_id": null,
                    "validation_errors": [e.to_string()],
                    "data": null
                });
                Ok(ResponseJson(error_response))
            }
        }
    }
}

// Removed legacy list_cbus_http - using gRPC delegation pattern instead

// Helper modules for HTTP endpoints
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // Simple UUID generation
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            write!(f, "{:x}-{:x}-{:x}-{:x}",
                   (timestamp & 0xFFFFFFFF) as u32,
                   ((timestamp >> 32) & 0xFFFF) as u16,
                   0x4000 | (((timestamp >> 48) & 0x0FFF) as u16), // Version 4
                   0x8000 | (((timestamp >> 60) & 0x3FFF) as u16)  // Variant bits
            )
        }
    }
}

mod chrono {
    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }

    pub struct DateTime;

    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            // Simple RFC3339 timestamp
            let now = std::time::SystemTime::now();
            let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                1970 + secs / 31536000, // Rough year calculation
                1 + (secs % 31536000) / 2678400, // Rough month
                1 + (secs % 2678400) / 86400, // Rough day
                (secs % 86400) / 3600, // Hour
                (secs % 3600) / 60, // Minute
                secs % 60, // Second
                nanos / 1_000_000 // Milliseconds
            )
        }
    }
}

async fn get_entities(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP GetEntities called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let grpc_request_data = crate::financial_taxonomy::GetEntitiesRequest {
        jurisdiction: request.get("jurisdiction").and_then(|v| v.as_str()).map(|s| s.to_string()),
        entity_type: request.get("entity_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        status: request.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.get_entities(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC GetEntities succeeded, returning {} entities", response.entities.len());
            // Convert gRPC response to JSON manually
            let entities_json: Vec<serde_json::Value> = response.entities.iter().map(|entity| {
                serde_json::json!({
                    "entity_id": entity.entity_id,
                    "entity_name": entity.entity_name,
                    "jurisdiction": entity.jurisdiction,
                    "entity_type": entity.entity_type,
                    "country_code": entity.country_code,
                    "lei_code": entity.lei_code
                })
            }).collect();
            let json_response = serde_json::json!({
                "entities": entities_json
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC GetEntities failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_cbus(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ListCbus called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let grpc_request_data = crate::financial_taxonomy::ListCbusRequest {
        status_filter: request.get("status_filter").and_then(|v| v.as_str()).map(|s| s.to_string()),
        limit: request.get("limit").and_then(|v| v.as_i64()).map(|i| i as i32),
        offset: request.get("offset").and_then(|v| v.as_i64()).map(|i| i as i32),
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.list_cbus(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC ListCbus succeeded, returning {} CBUs", response.cbus.len());
            // Convert gRPC response to JSON manually
            let cbus_json: Vec<serde_json::Value> = response.cbus.iter().map(|cbu| {
                serde_json::json!({
                    "id": cbu.id,
                    "cbu_id": cbu.cbu_id,
                    "cbu_name": cbu.cbu_name,
                    "description": cbu.description,
                    "status": cbu.status
                })
            }).collect();
            let json_response = serde_json::json!({
                "cbus": cbus_json,
                "total_count": response.total_count
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC ListCbus failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_ai_suggestions(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP GetAiSuggestions called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let ai_provider = request.get("ai_provider").and_then(|v| {
        Some(crate::financial_taxonomy::AiProviderConfig {
            provider_type: v.get("provider_type").and_then(|pt| pt.as_i64()).unwrap_or(0) as i32,
            api_key: v.get("api_key").and_then(|k| k.as_str()).map(|s| s.to_string()),
        })
    });

    let grpc_request_data = crate::financial_taxonomy::GetAiSuggestionsRequest {
        query: request.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        context: request.get("context").and_then(|v| v.as_str()).map(|s| s.to_string()),
        ai_provider,
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.get_ai_suggestions(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC GetAiSuggestions succeeded, returning {} suggestions", response.suggestions.len());
            // Convert gRPC response to JSON manually
            let suggestions_json: Vec<serde_json::Value> = response.suggestions.iter().map(|suggestion| {
                serde_json::json!({
                    "title": suggestion.title,
                    "description": suggestion.description,
                    "category": suggestion.category,
                    "confidence": suggestion.confidence
                })
            }).collect();
            let json_response = serde_json::json!({
                "suggestions": suggestions_json,
                "status_message": response.status_message
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC GetAiSuggestions failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_products(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ListProducts called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let grpc_request_data = crate::financial_taxonomy::ListProductsRequest {
        status_filter: request.get("status_filter").and_then(|v| v.as_str()).map(|s| s.to_string()),
        line_of_business_filter: request.get("line_of_business_filter").and_then(|v| v.as_str()).map(|s| s.to_string()),
        limit: request.get("limit").and_then(|v| v.as_i64()).map(|i| i as i32),
        offset: request.get("offset").and_then(|v| v.as_i64()).map(|i| i as i32),
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.list_products(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC ListProducts succeeded, returning {} products", response.products.len());
            let json_response = serde_json::json!({
                "products": [],
                "total_count": response.total_count,
                "success": true,
                "message": "Products listed successfully"
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC ListProducts failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn instantiate_resource(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP InstantiateResource called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let grpc_request_data = crate::financial_taxonomy::InstantiateResourceRequest {
        template_id: request.get("template_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        onboarding_request_id: request.get("onboarding_request_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        context: request.get("context").and_then(|v| v.as_str()).map(|s| s.to_string()),
        initial_data: request.get("initial_data").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.instantiate_resource(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC InstantiateResource succeeded");
            let json_response = serde_json::json!({
                "success": response.success,
                "message": response.message,
                "instance": null
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC InstantiateResource failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn execute_dsl(
    State((_, taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ExecuteDsl called - delegating to gRPC implementation");

    // Convert JSON request to gRPC type
    let grpc_request_data = crate::financial_taxonomy::ExecuteDslRequest {
        instance_id: request.get("instance_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        execution_context: request.get("execution_context").and_then(|v| v.as_str()).map(|s| s.to_string()),
        input_data: request.get("input_data").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let grpc_request = tonic::Request::new(grpc_request_data);

    match taxonomy_server.execute_dsl(grpc_request).await {
        Ok(grpc_response) => {
            let response = grpc_response.into_inner();
            info!("gRPC ExecuteDsl succeeded");
            let json_response = serde_json::json!({
                "success": response.success,
                "message": response.message,
                "result": null
            });
            Ok(ResponseJson(json_response))
        }
        Err(status) => {
            error!("gRPC ExecuteDsl failed: {}", status);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}