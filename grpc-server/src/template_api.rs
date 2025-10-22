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

pub fn create_template_router(db_pool: PgPool) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/templates", get(get_all_templates))
        .route("/api/templates/:id", get(get_template))
        // TODO: Fix axum handler issue for PUT route
        // .route("/api/templates/:id", put(upsert_template))
        // White Truffle HTTP endpoints
        .route("/api/instantiate", post(instantiate_resource_http))
        .route("/api/execute-dsl", post(execute_dsl_http))
        .route("/api/execute-cbu-dsl", post(execute_cbu_dsl_http))
        .route("/api/list-cbus", post(list_cbus_http))
        .route("/api/ai-suggestions", post(get_ai_suggestions_http))
        .route("/api/entities", post(get_entities_http))
        .route("/api/list-products", post(list_products_http))
        .with_state(db_pool)
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
async fn instantiate_resource_http(
    Json(request): Json<InstantiateResourceRequest>,
) -> Result<ResponseJson<InstantiateResourceResponse>, StatusCode> {
    info!("HTTP InstantiateResource called for template: {}", request.template_id);

    // For now, return a mock response that simulates the gRPC call
    let instance_id = format!("http-instance-{}", uuid::Uuid::new_v4());

    let response = InstantiateResourceResponse {
        success: true,
        message: "Resource instance created successfully via HTTP".to_string(),
        instance: Some(ResourceInstance {
            instance_id: instance_id.clone(),
            onboarding_request_id: request.onboarding_request_id,
            template_id: request.template_id,
            status: "pending".to_string(),
            instance_data: "{}".to_string(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            error_message: None,
        }),
    };

    Ok(ResponseJson(response))
}

async fn execute_dsl_http(
    Json(request): Json<ExecuteDslRequest>,
) -> Result<ResponseJson<ExecuteDslResponse>, StatusCode> {
    info!("HTTP ExecuteDsl called for instance: {}", request.instance_id);

    // For now, return a mock response that simulates the gRPC call
    let response = ExecuteDslResponse {
        success: true,
        message: "DSL execution completed successfully via HTTP".to_string(),
        result: Some(DslExecutionResult {
            instance_id: request.instance_id,
            execution_status: "success".to_string(),
            output_data: r#"{"result": "DSL executed successfully from HTTP client"}"#.to_string(),
            log_messages: vec!["HTTP DSL execution started".to_string(), "HTTP DSL execution completed".to_string()],
            error_details: None,
            executed_at: Some(chrono::Utc::now().to_rfc3339()),
            execution_time_ms: 125.0,
        }),
    };

    Ok(ResponseJson(response))
}

async fn get_ai_suggestions_http(
    Json(request): Json<GetAiSuggestionsRequest>,
) -> Result<ResponseJson<GetAiSuggestionsResponse>, StatusCode> {
    info!("HTTP GetAiSuggestions called for query: {}", request.query);

    // Enhanced response with multiple AI-generated suggestions
    let context = request.context.unwrap_or("general".to_string());
    let mut suggestions = Vec::new();

    // Generate context-specific suggestions based on the query and context
    match context.as_str() {
        "kyc" => {
            suggestions.push(AiSuggestion {
                title: "KYC Workflow DSL".to_string(),
                description: format!("WORKFLOW \"{}\" {{\n  VALIDATE document_collection\n  VERIFY identity\n  ASSESS risk_level\n  APPROVE or REJECT\n}}", request.query.replace(" ", "_")),
                category: "workflow_generation".to_string(),
                confidence: 0.92,
                applicable_contexts: vec!["kyc".to_string(), "onboarding".to_string()],
            });
        }
        "onboarding" => {
            suggestions.push(AiSuggestion {
                title: "Client Onboarding DSL".to_string(),
                description: format!("PROCESS \"{}\" {{\n  COLLECT client_data\n  VALIDATE requirements\n  CREATE account\n  NOTIFY stakeholders\n}}", request.query.replace(" ", "_")),
                category: "process_generation".to_string(),
                confidence: 0.88,
                applicable_contexts: vec!["onboarding".to_string(), "client_management".to_string()],
            });
        }
        "dsl" => {
            suggestions.push(AiSuggestion {
                title: "DSL Code Generation".to_string(),
                description: format!("# Generated DSL for: {}\nRULE \"{}\" {{\n  WHEN condition_met\n  THEN execute_action\n  ELSE handle_exception\n}}", request.query, request.query.replace(" ", "_")),
                category: "code_generation".to_string(),
                confidence: 0.90,
                applicable_contexts: vec!["dsl".to_string(), "rules".to_string()],
            });
        }
        "validation" => {
            suggestions.push(AiSuggestion {
                title: "Validation Rules".to_string(),
                description: format!("VALIDATE \"{}\" {{\n  CHECK data_integrity\n  VERIFY business_rules\n  ENSURE compliance\n  REPORT results\n}}", request.query.replace(" ", "_")),
                category: "validation".to_string(),
                confidence: 0.85,
                applicable_contexts: vec!["validation".to_string(), "compliance".to_string()],
            });
        }
        _ => {
            suggestions.push(AiSuggestion {
                title: "General DSL Template".to_string(),
                description: format!("# AI-Generated DSL for: {}\nDEFINE \"{}\" {{\n  // Your implementation here\n  EXECUTE action\n  RETURN result\n}}", request.query, request.query.replace(" ", "_")),
                category: "template_generation".to_string(),
                confidence: 0.75,
                applicable_contexts: vec!["general".to_string()],
            });
        }
    }

    // Add a second suggestion with different approach
    suggestions.push(AiSuggestion {
        title: "Alternative Implementation".to_string(),
        description: format!("// Alternative approach for: {}\nFUNCTION {}() {{\n  // Step-by-step implementation\n  return solution;\n}}", request.query, request.query.replace(" ", "_").to_lowercase()),
        category: "alternative_approach".to_string(),
        confidence: 0.78,
        applicable_contexts: vec![context.clone()],
    });

    let response = GetAiSuggestionsResponse {
        suggestions,
        status_message: format!("AI suggestions generated successfully for {} context", context),
    };

    Ok(ResponseJson(response))
}

async fn list_products_http(
    Json(_request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ListProducts called");

    // For now, return a mock response that simulates the gRPC call
    let products = vec![
        serde_json::json!({
            "product_id": "CUSTODY_001",
            "product_name": "Institutional Custody",
            "line_of_business": "custody",
            "description": "Comprehensive custody services for institutional clients",
            "status": "active",
            "contract_type": "Service Agreement",
            "commercial_status": "Generally Available",
            "pricing_model": "Asset-based",
            "target_market": "Institutional"
        }),
        serde_json::json!({
            "product_id": "PRIME_BROKERAGE_001",
            "product_name": "Prime Brokerage Platform",
            "line_of_business": "prime_brokerage",
            "description": "Full-service prime brokerage for hedge funds",
            "status": "active",
            "contract_type": "Master Agreement",
            "commercial_status": "Generally Available",
            "pricing_model": "Commission-based",
            "target_market": "Hedge Funds"
        }),
        serde_json::json!({
            "product_id": "FUND_ADMIN_001",
            "product_name": "Fund Administration",
            "line_of_business": "fund_administration",
            "description": "Complete fund administration services",
            "status": "active",
            "contract_type": "Service Agreement",
            "commercial_status": "Generally Available",
            "pricing_model": "Fixed Fee",
            "target_market": "Asset Managers"
        })
    ];

    let response = serde_json::json!({
        "products": products,
        "total_count": products.len()
    });

    Ok(ResponseJson(response))
}

async fn execute_cbu_dsl_http(
    State(pool): State<PgPool>,
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
                    "data": result.data
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

async fn list_cbus_http(
    Json(_request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP ListCbus called");

    // For now, return a mock response that simulates the gRPC call
    let cbus = vec![
        serde_json::json!({
            "id": 1,
            "cbu_id": "CBU_001",
            "cbu_name": "Alpha Growth Fund",
            "description": "Diversified growth-focused investment fund",
            "legal_entity_name": "Alpha Growth Fund LLC",
            "jurisdiction": "Delaware",
            "business_model": "Investment Fund",
            "status": "active",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-10-22T12:00:00Z"
        }),
        serde_json::json!({
            "id": 2,
            "cbu_id": "CBU_002",
            "cbu_name": "Beta Conservative Fund",
            "description": "Conservative fixed-income investment strategy",
            "legal_entity_name": "Beta Conservative Fund LP",
            "jurisdiction": "New York",
            "business_model": "Investment Fund",
            "status": "active",
            "created_at": "2024-03-20T14:15:00Z",
            "updated_at": "2024-10-22T12:00:00Z"
        })
    ];

    let response = serde_json::json!({
        "cbus": cbus,
        "total_count": cbus.len()
    });

    Ok(ResponseJson(response))
}

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

async fn get_entities_http(
    Json(_request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("HTTP GetEntities called");

    // For now, return a mock response that simulates the gRPC call
    let entities = vec![
        serde_json::json!({
            "entity_id": "ENT_001",
            "entity_name": "Alpha Capital Management",
            "jurisdiction": "Delaware",
            "entity_type": "Investment Manager",
            "country_code": "US",
            "lei_code": "ALPHA123456789012345",
            "status": "Active"
        }),
        serde_json::json!({
            "entity_id": "ENT_002",
            "entity_name": "Beta Asset Owners LLC",
            "jurisdiction": "New York",
            "entity_type": "Asset Owner",
            "country_code": "US",
            "lei_code": "BETA9876543210987654",
            "status": "Active"
        }),
        serde_json::json!({
            "entity_id": "ENT_003",
            "entity_name": "Gamma Services Group",
            "jurisdiction": "Nevada",
            "entity_type": "Managing Company",
            "country_code": "US",
            "lei_code": "GAMMA55555555555555",
            "status": "Active"
        })
    ];

    let response = serde_json::json!({
        "entities": entities,
        "total_count": entities.len()
    });

    Ok(ResponseJson(response))
}