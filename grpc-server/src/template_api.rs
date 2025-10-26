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
use tracing::{info, error, warn};
use tower_http::cors::CorsLayer;
use sqlx::{PgPool, Row};
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
        // Resource DSL endpoints
        .route("/api/list-resources", post(list_resources))
        .route("/api/get-resource-dsl", post(get_resource_dsl))
        .route("/api/update-resource-dsl", post(update_resource_dsl))
        .route("/api/execute-resource-dsl", post(execute_resource_dsl))
        // Onboarding workflow endpoints
        .route("/api/onboarding/get-metadata", get(get_onboarding_metadata))
        .route("/api/onboarding/update-metadata", post(update_onboarding_metadata))
        .route("/api/onboarding/compile", post(compile_onboarding_workflow))
        .route("/api/onboarding/execute", post(execute_onboarding_workflow))
        // Onboarding request management endpoints
        .route("/api/onboarding/requests", post(create_onboarding_request))
        .route("/api/onboarding/requests", get(list_onboarding_requests))
        .route("/api/onboarding/requests/:onboarding_id", get(get_onboarding_request))
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

// ============================================
// RESOURCE DSL ENDPOINTS
// ============================================

async fn list_resources(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ListResources called");

    let status_filter = request.get("status_filter")
        .and_then(|v| v.as_str())
        .unwrap_or("active");
    let resource_type_filter = request.get("resource_type_filter")
        .and_then(|v| v.as_str());

    // Query resources table with optional join to dsl_metadata
    let query = if let Some(type_filter) = resource_type_filter {
        sqlx::query(
            "SELECT r.id, r.resource_id, r.resource_name, r.resource_type, r.description,
                    r.location, r.status, r.created_at, r.updated_at,
                    d.dsl_content, d.metadata as dsl_metadata
             FROM resources r
             LEFT JOIN dsl_metadata d ON r.resource_id = d.entity_id AND d.dsl_domain = 'Resource'
             WHERE r.status = $1 AND r.resource_type = $2
             ORDER BY r.resource_name"
        )
        .bind(status_filter)
        .bind(type_filter)
    } else {
        sqlx::query(
            "SELECT r.id, r.resource_id, r.resource_name, r.resource_type, r.description,
                    r.location, r.status, r.created_at, r.updated_at,
                    d.dsl_content, d.metadata as dsl_metadata
             FROM resources r
             LEFT JOIN dsl_metadata d ON r.resource_id = d.entity_id AND d.dsl_domain = 'Resource'
             WHERE r.status = $1
             ORDER BY r.resource_name"
        )
        .bind(status_filter)
    };

    match query.fetch_all(&pool).await {
        Ok(rows) => {
            let resources: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<i32, _>("id"),
                    "resource_id": row.get::<String, _>("resource_id"),
                    "resource_name": row.get::<String, _>("resource_name"),
                    "resource_type": row.get::<String, _>("resource_type"),
                    "description": row.get::<Option<String>, _>("description"),
                    "location": row.get::<Option<String>, _>("location"),
                    "status": row.get::<String, _>("status"),
                    "created_at": null,  // Simplified - timestamps handled separately if needed
                    "updated_at": null,
                    "dsl_content": row.get::<Option<String>, _>("dsl_content"),
                    "dsl_metadata": row.get::<Option<String>, _>("dsl_metadata"),
                })
            }).collect();

            info!("Successfully fetched {} resources", resources.len());
            let json_response = serde_json::json!({
                "resources": resources,
                "total_count": resources.len()
            });
            Ok(ResponseJson(json_response))
        }
        Err(e) => {
            error!("Failed to fetch resources: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_resource_dsl(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP GetResourceDsl called");

    let resource_id = request.get("resource_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Query dsl_metadata table for Resource DSL
    let query = sqlx::query(
        "SELECT entity_id, dsl_content, dsl_version, syntax_valid, metadata,
                created_at, updated_at
         FROM dsl_metadata
         WHERE entity_id = $1 AND dsl_domain = 'Resource'"
    )
    .bind(resource_id);

    match query.fetch_optional(&pool).await {
        Ok(Some(row)) => {
            let json_response = serde_json::json!({
                "success": true,
                "resource_id": row.get::<String, _>("entity_id"),
                "dsl_content": row.get::<String, _>("dsl_content"),
                "dsl_version": row.get::<i32, _>("dsl_version"),
                "syntax_valid": row.get::<bool, _>("syntax_valid"),
                "metadata": row.get::<Option<serde_json::Value>, _>("metadata"),
            });
            Ok(ResponseJson(json_response))
        }
        Ok(None) => {
            info!("No DSL found for resource: {}", resource_id);
            let json_response = serde_json::json!({
                "success": false,
                "message": format!("No DSL found for resource: {}", resource_id),
                "resource_id": resource_id,
            });
            Ok(ResponseJson(json_response))
        }
        Err(e) => {
            error!("Failed to fetch resource DSL: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_resource_dsl(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP UpdateResourceDsl called");

    let resource_id = request.get("resource_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let dsl_content = request.get("dsl_content")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let metadata = request.get("metadata")
        .and_then(|v| v.as_str());

    // Parse metadata as JSON if provided
    let metadata_json: Option<serde_json::Value> = metadata
        .and_then(|m| serde_json::from_str(m).ok());

    // Upsert into dsl_metadata table
    let query = sqlx::query(
        "INSERT INTO dsl_metadata (entity_id, dsl_domain, dsl_content, metadata, updated_at)
         VALUES ($1, 'Resource', $2, $3, CURRENT_TIMESTAMP)
         ON CONFLICT (entity_id, dsl_domain)
         DO UPDATE SET
            dsl_content = EXCLUDED.dsl_content,
            metadata = EXCLUDED.metadata,
            updated_at = CURRENT_TIMESTAMP,
            dsl_version = dsl_metadata.dsl_version + 1"
    )
    .bind(resource_id)
    .bind(dsl_content)
    .bind(metadata_json);

    match query.execute(&pool).await {
        Ok(_) => {
            info!("Successfully updated DSL for resource: {}", resource_id);
            let json_response = serde_json::json!({
                "success": true,
                "message": "Resource DSL updated successfully",
                "resource_id": resource_id,
            });
            Ok(ResponseJson(json_response))
        }
        Err(e) => {
            error!("Failed to update resource DSL: {}", e);
            let json_response = serde_json::json!({
                "success": false,
                "message": format!("Failed to update Resource DSL: {}", e),
                "resource_id": resource_id,
            });
            Ok(ResponseJson(json_response))
        }
    }
}

async fn execute_resource_dsl(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ExecuteResourceDsl called");

    let resource_id = request.get("resource_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let dsl_script = request.get("dsl_script")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Executing Resource DSL for resource: {}", resource_id);
    info!("DSL Script: {}", dsl_script);

    // TODO: Implement Resource DSL parser and execution engine
    // For now, return a stub success response
    let json_response = serde_json::json!({
        "success": true,
        "message": "Resource DSL execution completed (stub implementation)",
        "resource_id": resource_id,
        "validation_errors": [],
        "data": null,
    });

    Ok(ResponseJson(json_response))
}

// ========== ONBOARDING WORKFLOW ENDPOINTS ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingMetadata {
    pub product_catalog: String,
    pub cbu_templates: String,
    pub resource_dicts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadataRequest {
    pub file_type: String, // "product_catalog", "cbu_templates", or resource dict name
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileWorkflowRequest {
    pub instance_id: String,
    pub cbu_id: String,
    pub products: Vec<String>,
    pub team_users: Vec<serde_json::Value>,
    pub cbu_profile: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub plan: Option<serde_json::Value>,
    pub idd: Option<serde_json::Value>,
    pub bindings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub plan: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub execution_log: Vec<String>,
}

async fn get_onboarding_metadata() -> Result<ResponseJson<OnboardingMetadata>, StatusCode> {
    info!("🔄 [STATE] Getting onboarding metadata");

    let metadata_dir = std::path::Path::new("onboarding/metadata");

    // Read product catalog
    info!("📦 [LOAD] Reading product_catalog.yaml");
    let product_catalog = tokio::fs::read_to_string(metadata_dir.join("product_catalog.yaml"))
        .await
        .map_err(|e| {
            error!("❌ [ERROR] Failed to read product_catalog.yaml: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("✅ [LOAD] Product catalog loaded ({} bytes)", product_catalog.len());

    // Read CBU templates
    info!("📋 [LOAD] Reading cbu_templates.yaml");
    let cbu_templates = tokio::fs::read_to_string(metadata_dir.join("cbu_templates.yaml"))
        .await
        .map_err(|e| {
            error!("❌ [ERROR] Failed to read cbu_templates.yaml: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("✅ [LOAD] CBU templates loaded ({} bytes)", cbu_templates.len());

    // Read resource dictionaries
    let mut resource_dicts = HashMap::new();
    let resource_dicts_dir = metadata_dir.join("resource_dicts");

    info!("📚 [LOAD] Reading resource dictionaries from {:?}", resource_dicts_dir);
    if let Ok(mut entries) = tokio::fs::read_dir(&resource_dicts_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        info!("  ✅ Loaded resource dict: {} ({} bytes)", file_name, content.len());
                        resource_dicts.insert(file_name.to_string(), content);
                    } else {
                        warn!("  ⚠️ Failed to read resource dict: {}", file_name);
                    }
                }
            }
        }
    } else {
        warn!("⚠️ [WARN] Resource dicts directory not found or inaccessible");
    }

    info!("✅ [STATE] Metadata loaded successfully: {} resource dictionaries", resource_dicts.len());

    let response = OnboardingMetadata {
        product_catalog,
        cbu_templates,
        resource_dicts,
    };

    Ok(ResponseJson(response))
}

async fn update_onboarding_metadata(
    Json(request): Json<UpdateMetadataRequest>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("Updating onboarding metadata: {}", request.file_type);

    let metadata_dir = std::path::Path::new("onboarding/metadata");

    let file_path = match request.file_type.as_str() {
        "product_catalog" => metadata_dir.join("product_catalog.yaml"),
        "cbu_templates" => metadata_dir.join("cbu_templates.yaml"),
        name => metadata_dir.join("resource_dicts").join(format!("{}.yaml", name)),
    };

    tokio::fs::write(&file_path, request.content)
        .await
        .map_err(|e| {
            error!("Failed to write file {:?}: {}", file_path, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = serde_json::json!({
        "success": true,
        "message": format!("Successfully updated {}", request.file_type),
    });

    Ok(ResponseJson(response))
}

async fn compile_onboarding_workflow(
    Json(request): Json<CompileWorkflowRequest>,
) -> Result<ResponseJson<CompileWorkflowResponse>, StatusCode> {
    info!("⚙️ [COMPILE] Starting workflow compilation");
    info!("  Instance ID: {}", request.instance_id);
    info!("  CBU ID: {}", request.cbu_id);
    info!("  Products: {:?}", request.products);
    info!("  Team users: {} entries", request.team_users.len());

    use onboarding::{compile_onboard, CompileInputs};
    use onboarding::ast::oodl::OnboardIntent;
    use onboarding::meta::loader::load_from_dir;

    // Load metadata from disk
    info!("📚 [COMPILE] Loading metadata from disk");
    let meta = load_from_dir(std::path::Path::new("onboarding/metadata"))
        .map_err(|e| {
            error!("❌ [COMPILE] Failed to load metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("✅ [COMPILE] Metadata loaded successfully");

    // Create onboard intent
    info!("📝 [COMPILE] Creating onboard intent");
    let intent = OnboardIntent {
        instance_id: request.instance_id.clone(),
        cbu_id: request.cbu_id.clone(),
        products: request.products.clone(),
    };

    // Compile
    info!("🔧 [COMPILE] Starting compilation with {} products", request.products.len());
    let inputs = CompileInputs {
        intent: &intent,
        meta: &meta,
        team_users: request.team_users.clone(),
        cbu_profile: request.cbu_profile.clone(),
    };

    match compile_onboard(inputs) {
        Ok(outputs) => {
            info!("✅ [COMPILE] Compilation successful");
            info!("  Plan steps: {}", outputs.plan.steps.len());
            info!("  IDD gaps: {}", outputs.idd.gaps.len());
            info!("  Bindings tasks: {}", outputs.bindings.tasks.len());

            let plan_json = serde_json::to_value(&outputs.plan)
                .map_err(|e| {
                    error!("❌ [COMPILE] Failed to serialize plan: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            let idd_json = serde_json::to_value(&outputs.idd)
                .map_err(|e| {
                    error!("❌ [COMPILE] Failed to serialize IDD: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            let bindings_json = serde_json::to_value(&outputs.bindings)
                .map_err(|e| {
                    error!("❌ [COMPILE] Failed to serialize bindings: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let response = CompileWorkflowResponse {
                success: true,
                message: "Workflow compiled successfully".to_string(),
                plan: Some(plan_json),
                idd: Some(idd_json),
                bindings: Some(bindings_json),
            };

            info!("✅ [COMPILE] Response prepared and returning");
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("❌ [COMPILE] Compilation failed: {}", e);
            let response = CompileWorkflowResponse {
                success: false,
                message: format!("Compilation failed: {}", e),
                plan: None,
                idd: None,
                bindings: None,
            };
            Ok(ResponseJson(response))
        }
    }
}

async fn execute_onboarding_workflow(
    Json(request): Json<ExecuteWorkflowRequest>,
) -> Result<ResponseJson<ExecuteWorkflowResponse>, StatusCode> {
    info!("▶️ [EXECUTE] Starting workflow execution");

    use onboarding::{execute_plan, ExecutionConfig};
    use onboarding::ir::Plan;

    // Deserialize plan from JSON
    info!("📝 [EXECUTE] Deserializing execution plan");
    let plan: Plan = serde_json::from_value(request.plan)
        .map_err(|e| {
            error!("❌ [EXECUTE] Failed to deserialize plan: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    info!("✅ [EXECUTE] Plan deserialized: {} steps to execute", plan.steps.len());

    let config = ExecutionConfig {};

    info!("🚀 [EXECUTE] Executing plan with {} steps", plan.steps.len());
    match execute_plan(&plan, &config).await {
        Ok(_) => {
            info!("✅ [EXECUTE] Workflow executed successfully");
            let response = ExecuteWorkflowResponse {
                success: true,
                message: "Workflow executed successfully".to_string(),
                execution_log: vec![
                    "Execution started".to_string(),
                    format!("Executed {} tasks", plan.steps.len()),
                    "Execution completed".to_string(),
                ],
            };
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("❌ [EXECUTE] Execution failed: {}", e);
            let response = ExecuteWorkflowResponse {
                success: false,
                message: format!("Execution failed: {}", e),
                execution_log: vec![],
            };
            Ok(ResponseJson(response))
        }
    }
}

// ========== ONBOARDING REQUEST MANAGEMENT ENDPOINTS ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingRequestRequest {
    pub name: String,
    pub description: Option<String>,
    pub cbu_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingRequestResponse {
    pub success: bool,
    pub message: String,
    pub onboarding_id: Option<String>,
    pub request_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingRequestSummary {
    pub id: i32,
    pub onboarding_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub cbu_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

async fn create_onboarding_request(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Json(request): Json<CreateOnboardingRequestRequest>,
) -> Result<ResponseJson<CreateOnboardingRequestResponse>, StatusCode> {
    info!("📝 [ONBOARDING] Creating new onboarding request");
    info!("  Name: {}", request.name);
    info!("  Description: {:?}", request.description);
    info!("  CBU ID: {:?}", request.cbu_id);

    // Generate onboarding ID
    use sqlx::types::chrono::Utc;
    let year = Utc::now().format("%Y").to_string();

    // Get next sequence number for this year
    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM onboarding_request WHERE onboarding_id LIKE $1"
    )
    .bind(format!("OR-{}-%%", year))
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("❌ [ONBOARDING] Failed to count existing requests: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let next_num = count_result + 1;
    let onboarding_id = format!("OR-{}-{:05}", year, next_num);

    info!("  Generated onboarding_id: {}", onboarding_id);

    // Insert into onboarding_request table
    let insert_result = sqlx::query(
        "INSERT INTO onboarding_request (onboarding_id, name, description, status, cbu_id, created_at, updated_at)
         VALUES ($1, $2, $3, 'draft', $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id"
    )
    .bind(&onboarding_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.cbu_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        error!("❌ [ONBOARDING] Failed to insert onboarding request: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let request_id: i32 = insert_result.get("id");
    info!("✅ [ONBOARDING] Created onboarding request with ID: {}", request_id);

    // Insert initial DSL from template into onboarding_request_dsl
    let default_products: Vec<String> = vec![];
    let default_team_users = serde_json::json!([
        {"email": "ops.admin@client.com", "role": "Administrator"},
        {"email": "ops.approver@client.com", "role": "Approver"}
    ]);
    let default_cbu_profile = serde_json::json!({"region": "EU"});

    sqlx::query(
        "INSERT INTO onboarding_request_dsl (onboarding_request_id, instance_id, products, team_users, cbu_profile, template_version, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'v1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(request_id)
    .bind(&onboarding_id)
    .bind(&default_products)
    .bind(&default_team_users)
    .bind(&default_cbu_profile)
    .execute(&pool)
    .await
    .map_err(|e| {
        error!("❌ [ONBOARDING] Failed to insert DSL template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("✅ [ONBOARDING] Initialized DSL template for request {}", onboarding_id);

    let response = CreateOnboardingRequestResponse {
        success: true,
        message: format!("Onboarding request {} created successfully", onboarding_id),
        onboarding_id: Some(onboarding_id),
        request_id: Some(request_id),
    };

    Ok(ResponseJson(response))
}

async fn list_onboarding_requests(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
) -> Result<ResponseJson<Vec<OnboardingRequestSummary>>, StatusCode> {
    info!("📋 [ONBOARDING] Listing all onboarding requests");

    let rows = sqlx::query(
        "SELECT id, onboarding_id, name, description, status, cbu_id, created_at, updated_at
         FROM onboarding_request
         ORDER BY created_at DESC
         LIMIT 100"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!("❌ [ONBOARDING] Failed to fetch requests: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let requests: Vec<OnboardingRequestSummary> = rows.iter().map(|row| {
        use sqlx::types::chrono::{DateTime, Utc};
        OnboardingRequestSummary {
            id: row.get("id"),
            onboarding_id: row.get("onboarding_id"),
            name: row.get("name"),
            description: row.get("description"),
            status: row.get("status"),
            cbu_id: row.get("cbu_id"),
            created_at: row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<DateTime<Utc>, _>("updated_at").to_rfc3339(),
        }
    }).collect();

    info!("✅ [ONBOARDING] Found {} requests", requests.len());

    Ok(ResponseJson(requests))
}

async fn get_onboarding_request(
    State((pool, _taxonomy_server)): State<(PgPool, std::sync::Arc<TaxonomyServer>)>,
    Path(onboarding_id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("🔍 [ONBOARDING] Getting request: {}", onboarding_id);

    // Get request details
    let request_row = sqlx::query(
        "SELECT r.id, r.onboarding_id, r.name, r.description, r.status, r.cbu_id, r.created_at, r.updated_at,
                d.products, d.team_users, d.cbu_profile, d.workflow_config
         FROM onboarding_request r
         LEFT JOIN onboarding_request_dsl d ON r.id = d.onboarding_request_id
         WHERE r.onboarding_id = $1"
    )
    .bind(&onboarding_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        error!("❌ [ONBOARDING] Failed to fetch request: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match request_row {
        Some(row) => {
            use sqlx::types::chrono::{DateTime, Utc};
            let response = serde_json::json!({
                "success": true,
                "id": row.get::<i32, _>("id"),
                "onboarding_id": row.get::<String, _>("onboarding_id"),
                "name": row.get::<String, _>("name"),
                "description": row.get::<Option<String>, _>("description"),
                "status": row.get::<String, _>("status"),
                "cbu_id": row.get::<Option<String>, _>("cbu_id"),
                "products": row.get::<Option<Vec<String>>, _>("products").unwrap_or_default(),
                "team_users": row.get::<Option<serde_json::Value>, _>("team_users"),
                "cbu_profile": row.get::<Option<serde_json::Value>, _>("cbu_profile"),
                "workflow_config": row.get::<Option<serde_json::Value>, _>("workflow_config"),
                "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
                "updated_at": row.get::<DateTime<Utc>, _>("updated_at").to_rfc3339(),
            });

            info!("✅ [ONBOARDING] Found request {}", onboarding_id);
            Ok(ResponseJson(response))
        }
        None => {
            warn!("⚠️ [ONBOARDING] Request not found: {}", onboarding_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}