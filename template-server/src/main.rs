use axum::{
    extract::{Path, Json, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, put, post},
    Router,
    middleware,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use tracing::{info, error};
use tower_http::cors::CorsLayer;

mod logging;
use logging::api_logging_middleware;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbnfTemplate {
    pub id: i32,
    pub template_name: String,
    pub description: String,
    pub ebnf_pattern: String,
    pub complexity_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub template_name: String,
    pub description: String,
    pub domain_id: i32,
    pub ebnf_template_id: i32,
    pub dsl_code: String,
    pub attributes: Vec<TemplateAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateResponse {
    pub success: bool,
    pub resource_id: String,
    pub message: String,
}

// Runtime execution data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowRequest {
    pub workflow_type: String,
    pub jurisdiction: String,
    pub initial_data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflowResponse {
    pub success: bool,
    pub instance_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusQuery {
    pub instance_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub instance_id: String,
    pub status: String,
    pub collected_data: HashMap<String, serde_json::Value>,
    pub pending_solicitations: Vec<DataSolicitationResponse>,
    pub validation_results: Vec<ValidationResponse>,
    pub next_actions: Vec<String>,
    pub execution_summary: ExecutionSummaryResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSolicitationResponse {
    pub instance_id: String,
    pub attribute_path: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub ui_hints: UIHintsResponse,
    pub template_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIHintsResponse {
    pub input_type: String,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub allowed_values: Option<Vec<String>>,
    pub validation_pattern: Option<String>,
    pub max_length: Option<i32>,
    pub min_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub attribute_path: String,
    pub validation_rule: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub validated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummaryResponse {
    pub total_templates: i32,
    pub executed_templates: i32,
    pub total_attributes: i32,
    pub collected_attributes: i32,
    pub derived_attributes: i32,
    pub pending_solicitations: i32,
    pub validation_errors: i32,
    pub execution_time_ms: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDataRequest {
    pub instance_id: String,
    pub attribute_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDataResponse {
    pub success: bool,
    pub message: String,
    pub updated_solicitations: Vec<String>,
    pub validation_results: Vec<ValidationResponse>,
}

const TEMPLATES_FILE_PATH: &str = "../resource_templates.json";

fn create_template_router() -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/templates", get(get_all_templates))
        .route("/api/templates/:id", get(get_template))
        .route("/api/templates/:id", put(upsert_template))
        .route("/api/templates/create", put(create_template))
        .route("/api/ebnf-templates", get(get_ebnf_templates))
        // Private attributes endpoints
        .route("/api/private-attributes", get(get_private_attributes))
        .route("/api/private-attributes", post(create_private_attribute))
        .route("/api/private-attributes/:id", get(get_private_attribute))
        .route("/api/private-attributes/:id", put(update_private_attribute))
        .route("/api/private-attributes/:id", axum::routing::delete(delete_private_attribute))
        // Runtime execution endpoints
        .route("/api/runtime/workflows/start", post(start_workflow))
        .route("/api/runtime/workflows/status", get(get_workflow_status))
        .route("/api/runtime/workflows/submit-data", post(submit_workflow_data))
        .route("/api/runtime/workflows/:instance_id/stop", post(stop_workflow))
        .layer(middleware::from_fn(api_logging_middleware))
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

#[axum::debug_handler]
async fn get_ebnf_templates() -> Result<ResponseJson<Vec<EbnfTemplate>>, StatusCode> {
    info!("EBNF templates endpoint called");

    // In production, this would query the database
    // For now, return hardcoded data that matches the database
    let ebnf_templates = vec![
        EbnfTemplate {
            id: 1,
            template_name: "simple_concatenation".to_string(),
            description: "Concatenate two or more string attributes".to_string(),
            ebnf_pattern: "result ::= {source_attr} (\" \" {source_attr})*".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 2,
            template_name: "conditional_assignment".to_string(),
            description: "Assign value based on condition".to_string(),
            ebnf_pattern: "result ::= IF {condition} THEN {true_value} ELSE {false_value}".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 3,
            template_name: "lookup_transformation".to_string(),
            description: "Transform value using lookup table".to_string(),
            ebnf_pattern: "result ::= LOOKUP({source_attr}, {lookup_table})".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 4,
            template_name: "arithmetic_calculation".to_string(),
            description: "Perform arithmetic operations".to_string(),
            ebnf_pattern: "result ::= {operand1} {operator} {operand2}".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 5,
            template_name: "validation_rule".to_string(),
            description: "Validate data against business rules".to_string(),
            ebnf_pattern: "result ::= VALIDATE({source_attr}, {rule_expr})".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 6,
            template_name: "aggregation_rule".to_string(),
            description: "Aggregate data using functions".to_string(),
            ebnf_pattern: "result ::= {agg_function}({source_attrs})".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 7,
            template_name: "data_dictionary_lookup".to_string(),
            description: "Retrieve data from the data dictionary using GET-DATA verb".to_string(),
            ebnf_pattern: "result ::= GET-DATA {attribute_path} FROM {data_source} [WHERE {condition}]".to_string(),
            complexity_level: "simple".to_string(),
        },
        EbnfTemplate {
            id: 8,
            template_name: "data_attribute_derivation".to_string(),
            description: "Define private data attributes through ETL pipeline with sources, filters, and tests".to_string(),
            ebnf_pattern: "derived_attr ::= DERIVE {target_name} FROM {source_spec} [WHERE {filter_expr}] [WITH {transformation_spec}] [EXTRACT REGEX {pattern}] [TEST {validation_expr}] [MATERIALIZE {strategy}]".to_string(),
            complexity_level: "moderate".to_string(),
        },
    ];

    info!("📝 Returning {} EBNF templates", ebnf_templates.len());

    Ok(ResponseJson(ebnf_templates))
}

#[axum::debug_handler]
async fn create_template(Json(request): Json<CreateTemplateRequest>) -> Result<ResponseJson<CreateTemplateResponse>, StatusCode> {
    info!("Create template endpoint called for: {}", request.template_name);

    // In production, this would insert into the database with proper FK relationships
    // For now, simulate the database insert and return success

    let resource_id = format!("{}_{}", request.template_name.to_lowercase().replace(" ", "_"), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());

    // Simulate database insert
    info!("💾 Creating template in database:");
    info!("  - Resource ID: {}", resource_id);
    info!("  - Template Name: {}", request.template_name);
    info!("  - Description: {}", request.description);
    info!("  - Domain ID: {}", request.domain_id);
    info!("  - EBNF Template ID: {}", request.ebnf_template_id);
    info!("  - DSL Code Length: {} characters", request.dsl_code.len());
    info!("  - Attributes Count: {}", request.attributes.len());

    // Build the full template data structure for storage
    let _template_data = serde_json::json!({
        "resource_id": resource_id,
        "resource_type": "template",
        "name": request.template_name,
        "description": request.description,
        "version": "1.0.0",
        "status": "Active",
        "domain_id": request.domain_id,
        "ebnf_template_id": request.ebnf_template_id,
        "dsl_code": request.dsl_code,
        "json_data": {
            "id": resource_id,
            "description": request.description,
            "attributes": request.attributes,
            "dsl": request.dsl_code
        },
        "metadata": {
            "created_via": "template_designer",
            "domain_id": request.domain_id,
            "ebnf_template_id": request.ebnf_template_id
        },
        "created_by": "template_designer",
        "created_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "tags": ["user_created", "template_designer"]
    });

    info!("📊 Template data structure prepared for database insertion");

    // TODO: Insert into database with proper SQL
    // INSERT INTO resource_sheets (resource_id, resource_type, name, description, domain_id, ebnf_template_id, dsl_code, json_data, metadata, created_by)
    // VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)

    let response = CreateTemplateResponse {
        success: true,
        resource_id: resource_id.clone(),
        message: format!("Template '{}' created successfully with domain linkage", request.template_name),
    };

    info!("✅ Template creation completed: {}", resource_id);

    Ok(ResponseJson(response))
}

// Private Attributes API endpoints

// In-memory store for private attributes (in production, this would be in database)
static mut PRIVATE_ATTRIBUTES: std::sync::OnceLock<std::sync::Mutex<Vec<PrivateAttributeDefinition>>> = std::sync::OnceLock::new();
static mut NEXT_ATTRIBUTE_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

fn get_private_attributes_store() -> &'static std::sync::Mutex<Vec<PrivateAttributeDefinition>> {
    unsafe {
        PRIVATE_ATTRIBUTES.get_or_init(|| {
            // Initialize with some sample data
            let sample_attributes = vec![
                PrivateAttributeDefinition {
                    id: Some(1),
                    attribute_name: "internal_risk_tier".to_string(),
                    description: "Internal risk classification tier derived from multiple risk factors".to_string(),
                    data_type: "Enum".to_string(),
                    visibility_scope: "private".to_string(),
                    attribute_class: "derived".to_string(),
                    source_attributes: vec!["Client.credit_score".to_string(), "Client.income_level".to_string()],
                    filter_expression: Some("credit_score > 600".to_string()),
                    transformation_logic: Some("risk calculation based on multiple factors".to_string()),
                    regex_pattern: None,
                    validation_tests: Some("value in ['Low', 'Medium', 'High']".to_string()),
                    materialization_strategy: "lazy".to_string(),
                    derivation_rule_ebnf: "risk_tier ::= DERIVE internal_risk_tier FROM Client.credit_score, Client.income_level WHERE credit_score > 600".to_string(),
                    created_at: Some(chrono::Utc::now().to_rfc3339()),
                    updated_at: Some(chrono::Utc::now().to_rfc3339()),
                },
                PrivateAttributeDefinition {
                    id: Some(2),
                    attribute_name: "composite_score".to_string(),
                    description: "Composite scoring algorithm result".to_string(),
                    data_type: "Decimal".to_string(),
                    visibility_scope: "private".to_string(),
                    attribute_class: "derived".to_string(),
                    source_attributes: vec!["Client.age".to_string(), "Client.annual_income".to_string()],
                    filter_expression: None,
                    transformation_logic: Some("weighted average calculation".to_string()),
                    regex_pattern: None,
                    validation_tests: Some("value >= 0 and value <= 100".to_string()),
                    materialization_strategy: "eager".to_string(),
                    derivation_rule_ebnf: "composite_score ::= DERIVE composite_score FROM Client.age, Client.annual_income WITH weighted_avg".to_string(),
                    created_at: Some(chrono::Utc::now().to_rfc3339()),
                    updated_at: Some(chrono::Utc::now().to_rfc3339()),
                },
            ];
            unsafe { NEXT_ATTRIBUTE_ID.store(3, std::sync::atomic::Ordering::SeqCst); }
            std::sync::Mutex::new(sample_attributes)
        })
    }
}

#[axum::debug_handler]
async fn get_private_attributes() -> Result<ResponseJson<GetPrivateAttributesResponse>, StatusCode> {
    info!("📊 Getting all private attributes");

    let attributes_store = get_private_attributes_store();
    match attributes_store.lock() {
        Ok(attributes) => {
            let response = GetPrivateAttributesResponse {
                attributes: attributes.clone(),
            };
            info!("✅ Retrieved {} private attributes", attributes.len());
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("❌ Failed to lock private attributes store: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn get_private_attribute(Path(id): Path<i32>) -> Result<ResponseJson<PrivateAttributeDefinition>, StatusCode> {
    info!("📊 Getting private attribute with id: {}", id);

    let attributes_store = get_private_attributes_store();
    match attributes_store.lock() {
        Ok(attributes) => {
            match attributes.iter().find(|attr| attr.id == Some(id)) {
                Some(attribute) => {
                    info!("✅ Found private attribute: {}", attribute.attribute_name);
                    Ok(ResponseJson(attribute.clone()))
                }
                None => {
                    error!("❌ Private attribute not found: {}", id);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to lock private attributes store: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn create_private_attribute(Json(request): Json<CreatePrivateAttributeRequest>) -> Result<ResponseJson<CreatePrivateAttributeResponse>, StatusCode> {
    info!("➕ Creating private attribute: {}", request.attribute_name);

    let attributes_store = get_private_attributes_store();
    match attributes_store.lock() {
        Ok(mut attributes) => {
            // Generate new ID
            let new_id = unsafe { NEXT_ATTRIBUTE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) };

            // Create new attribute
            let new_attribute = PrivateAttributeDefinition {
                id: Some(new_id),
                attribute_name: request.attribute_name.clone(),
                description: request.description,
                data_type: request.data_type,
                visibility_scope: "private".to_string(),
                attribute_class: "derived".to_string(),
                source_attributes: request.source_attributes,
                filter_expression: request.filter_expression,
                transformation_logic: request.transformation_logic,
                regex_pattern: request.regex_pattern,
                validation_tests: request.validation_tests,
                materialization_strategy: request.materialization_strategy,
                derivation_rule_ebnf: request.derivation_rule_ebnf,
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                updated_at: Some(chrono::Utc::now().to_rfc3339()),
            };

            // Add to store
            attributes.push(new_attribute);

            let response = CreatePrivateAttributeResponse {
                success: true,
                attribute_id: new_id,
                message: format!("Private attribute '{}' created successfully", request.attribute_name),
            };

            info!("✅ Private attribute created with ID: {}", new_id);
            Ok(ResponseJson(response))
        }
        Err(e) => {
            error!("❌ Failed to lock private attributes store: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn update_private_attribute(
    Path(id): Path<i32>,
    Json(request): Json<CreatePrivateAttributeRequest>,
) -> Result<ResponseJson<CreatePrivateAttributeResponse>, StatusCode> {
    info!("✏️ Updating private attribute: {}", id);

    let attributes_store = get_private_attributes_store();
    match attributes_store.lock() {
        Ok(mut attributes) => {
            match attributes.iter_mut().find(|attr| attr.id == Some(id)) {
                Some(attribute) => {
                    // Update the attribute
                    attribute.attribute_name = request.attribute_name.clone();
                    attribute.description = request.description;
                    attribute.data_type = request.data_type;
                    attribute.source_attributes = request.source_attributes;
                    attribute.filter_expression = request.filter_expression;
                    attribute.transformation_logic = request.transformation_logic;
                    attribute.regex_pattern = request.regex_pattern;
                    attribute.validation_tests = request.validation_tests;
                    attribute.materialization_strategy = request.materialization_strategy;
                    attribute.derivation_rule_ebnf = request.derivation_rule_ebnf;
                    attribute.updated_at = Some(chrono::Utc::now().to_rfc3339());

                    let response = CreatePrivateAttributeResponse {
                        success: true,
                        attribute_id: id,
                        message: format!("Private attribute '{}' updated successfully", request.attribute_name),
                    };

                    info!("✅ Private attribute {} updated successfully", id);
                    Ok(ResponseJson(response))
                }
                None => {
                    error!("❌ Private attribute not found: {}", id);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to lock private attributes store: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
async fn delete_private_attribute(Path(id): Path<i32>) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("🗑️ Deleting private attribute: {}", id);

    let attributes_store = get_private_attributes_store();
    match attributes_store.lock() {
        Ok(mut attributes) => {
            let original_len = attributes.len();
            attributes.retain(|attr| attr.id != Some(id));

            if attributes.len() < original_len {
                let response = serde_json::json!({
                    "success": true,
                    "message": format!("Private attribute {} deleted successfully", id)
                });
                info!("✅ Private attribute {} deleted successfully", id);
                Ok(ResponseJson(response))
            } else {
                error!("❌ Private attribute not found: {}", id);
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("❌ Failed to lock private attributes store: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Runtime execution API endpoints

#[axum::debug_handler]
async fn start_workflow(Json(request): Json<StartWorkflowRequest>) -> Result<ResponseJson<StartWorkflowResponse>, StatusCode> {
    info!("🚀 Starting workflow: {} for jurisdiction: {}", request.workflow_type, request.jurisdiction);

    // Generate unique instance ID
    let instance_id = format!("ONBOARD_{}_{}",
        request.workflow_type.to_uppercase(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
    );

    // In production, this would:
    // 1. Create RuntimeOrchestrator instance
    // 2. Initialize with database connection
    // 3. Start workflow execution asynchronously
    // 4. Store instance state in database/memory

    info!("💾 Created workflow instance: {}", instance_id);
    info!("📊 Workflow configuration:");
    info!("  - Type: {}", request.workflow_type);
    info!("  - Jurisdiction: {}", request.jurisdiction);
    info!("  - Initial Data Points: {}", request.initial_data.as_ref().map_or(0, |d| d.len()));

    // Simulate workflow initialization
    let response = StartWorkflowResponse {
        success: true,
        instance_id: instance_id.clone(),
        message: format!("Workflow instance {} started successfully", instance_id),
    };

    info!("✅ Workflow startup completed: {}", instance_id);

    Ok(ResponseJson(response))
}

#[axum::debug_handler]
async fn get_workflow_status(Query(params): Query<WorkflowStatusQuery>) -> Result<ResponseJson<WorkflowStatusResponse>, StatusCode> {
    info!("📊 Getting workflow status for instance: {}", params.instance_id);

    // In production, this would:
    // 1. Load RuntimeOrchestrator instance from storage
    // 2. Get current execution state
    // 3. Return comprehensive status

    // Mock workflow status
    let status_response = WorkflowStatusResponse {
        instance_id: params.instance_id.clone(),
        status: "CollectingData".to_string(),
        collected_data: {
            let mut data = HashMap::new();
            data.insert("Client.legal_entity_name".to_string(), serde_json::Value::String("Example Corp Ltd".to_string()));
            data.insert("Client.client_id".to_string(), serde_json::Value::String(format!("CLIENT_{}", params.instance_id)));
            data
        },
        pending_solicitations: vec![
            DataSolicitationResponse {
                instance_id: params.instance_id.clone(),
                attribute_path: "Client.incorporation_date".to_string(),
                data_type: "date".to_string(),
                required: true,
                description: "Date of incorporation for the legal entity".to_string(),
                ui_hints: UIHintsResponse {
                    input_type: "date".to_string(),
                    placeholder: Some("YYYY-MM-DD".to_string()),
                    help_text: Some("Enter the official incorporation date".to_string()),
                    allowed_values: None,
                    validation_pattern: Some(r"^\d{4}-\d{2}-\d{2}$".to_string()),
                    max_length: None,
                    min_length: None,
                },
                template_source: "legal_entity_template".to_string(),
            },
            DataSolicitationResponse {
                instance_id: params.instance_id.clone(),
                attribute_path: "Client.business_type".to_string(),
                data_type: "string".to_string(),
                required: true,
                description: "Type of business entity".to_string(),
                ui_hints: UIHintsResponse {
                    input_type: "dropdown".to_string(),
                    placeholder: Some("Select business type".to_string()),
                    help_text: Some("Choose the appropriate business entity type".to_string()),
                    allowed_values: Some(vec![
                        "Corporation".to_string(),
                        "LLC".to_string(),
                        "Partnership".to_string(),
                        "Sole Proprietorship".to_string(),
                    ]),
                    validation_pattern: None,
                    max_length: None,
                    min_length: None,
                },
                template_source: "business_classification_template".to_string(),
            },
        ],
        validation_results: vec![
            ValidationResponse {
                attribute_path: "Client.legal_entity_name".to_string(),
                validation_rule: "required".to_string(),
                passed: true,
                error_message: None,
                validated_at: chrono::Utc::now().to_rfc3339(),
            },
        ],
        next_actions: vec![
            "Complete 2 pending data solicitations".to_string(),
            "Provide incorporation date".to_string(),
            "Select business type".to_string(),
        ],
        execution_summary: ExecutionSummaryResponse {
            total_templates: 5,
            executed_templates: 2,
            total_attributes: 15,
            collected_attributes: 2,
            derived_attributes: 3,
            pending_solicitations: 2,
            validation_errors: 0,
            execution_time_ms: 1250,
        },
    };

    info!("📈 Status retrieved for {}: {} collected, {} pending solicitations",
        params.instance_id,
        status_response.collected_data.len(),
        status_response.pending_solicitations.len()
    );

    Ok(ResponseJson(status_response))
}

#[axum::debug_handler]
async fn submit_workflow_data(Json(request): Json<SubmitDataRequest>) -> Result<ResponseJson<SubmitDataResponse>, StatusCode> {
    info!("📝 Submitting data for workflow instance: {}", request.instance_id);
    info!("📊 Received {} data attributes", request.attribute_data.len());

    // Log submitted data attributes
    for (attr_path, value) in &request.attribute_data {
        info!("  - {}: {:?}", attr_path, value);
    }

    // In production, this would:
    // 1. Load RuntimeOrchestrator instance
    // 2. Update collected data
    // 3. Re-run validation
    // 4. Update solicitation status
    // 5. Continue workflow execution if possible

    // Mock validation results
    let validation_results = request.attribute_data.iter().map(|(attr_path, value)| {
        let passed = !value.is_null() && !value.as_str().unwrap_or("").is_empty();
        ValidationResponse {
            attribute_path: attr_path.clone(),
            validation_rule: "required".to_string(),
            passed,
            error_message: if passed { None } else { Some("Value is required".to_string()) },
            validated_at: chrono::Utc::now().to_rfc3339(),
        }
    }).collect();

    let updated_solicitations: Vec<String> = request.attribute_data.keys().cloned().collect();

    let response = SubmitDataResponse {
        success: true,
        message: format!("Successfully updated {} attributes for instance {}",
            request.attribute_data.len(),
            request.instance_id
        ),
        updated_solicitations,
        validation_results,
    };

    info!("✅ Data submission completed for {}: {} attributes processed",
        request.instance_id,
        request.attribute_data.len()
    );

    Ok(ResponseJson(response))
}

#[axum::debug_handler]
async fn stop_workflow(Path(instance_id): Path<String>) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    info!("🛑 Stopping workflow instance: {}", instance_id);

    // In production, this would:
    // 1. Load RuntimeOrchestrator instance
    // 2. Set workflow state to stopped/suspended
    // 3. Clean up resources
    // 4. Save final state

    let response = serde_json::json!({
        "success": true,
        "instance_id": instance_id,
        "message": format!("Workflow instance {} stopped successfully", instance_id),
        "final_status": "Suspended"
    });

    info!("✅ Workflow {} stopped successfully", instance_id);

    Ok(ResponseJson(response))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create HTTP template API router with logging
    let template_router = create_template_router();

    // Server address
    let http_addr = "0.0.0.0:3030".parse::<std::net::SocketAddr>()?;

    info!("🚀 Starting Template API server on {}", http_addr);
    info!("📁 Template file: {}", TEMPLATES_FILE_PATH);
    info!("📊 API logging enabled - sending to Elasticsearch at http://localhost:9200");

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    info!("✅ Template API server ready with comprehensive logging!");

    axum::serve(listener, template_router).await?;

    Ok(())
}