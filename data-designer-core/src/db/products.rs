use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};

use super::DbOperations;

// ===== CORE DATA STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: Option<String>,
    pub status: String,
    pub pricing_model: Option<String>,
    pub target_market: Option<String>,
    pub regulatory_requirements: Option<serde_json::Value>,
    pub sla_commitments: Option<serde_json::Value>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Service {
    pub id: i32,
    pub service_id: String,
    pub service_name: String,
    pub service_category: Option<String>,
    pub description: Option<String>,
    pub is_core_service: bool,
    pub configuration_schema: Option<serde_json::Value>,
    pub dependencies: Option<Vec<String>>,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i32,
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub capacity_limits: Option<serde_json::Value>,
    pub operational_hours: Option<String>,
    pub contact_information: Option<serde_json::Value>,
    pub technical_specifications: Option<serde_json::Value>,
    pub compliance_certifications: Option<Vec<String>>,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ===== RESOURCE CAPABILITIES AND TEMPLATES =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceTemplate {
    pub id: i32,
    pub template_id: String,
    pub template_name: String,
    pub description: Option<String>,
    pub part_of_product: Option<String>,
    pub implements_service: Option<String>,
    pub resource_type: String,
    pub attributes: serde_json::Value,
    pub capabilities: serde_json::Value,
    pub dsl_template: Option<String>,
    pub version: String,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceCapability {
    pub id: i32,
    pub capability_id: String,
    pub capability_name: String,
    pub description: Option<String>,
    pub capability_type: String,
    pub required_attributes: serde_json::Value,
    pub optional_attributes: serde_json::Value,
    pub output_attributes: serde_json::Value,
    pub implementation_function: Option<String>,
    pub validation_rules: serde_json::Value,
    pub error_handling: serde_json::Value,
    pub timeout_seconds: Option<i32>,
    pub retry_attempts: Option<i32>,
    pub status: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductServiceMapping {
    pub id: i32,
    pub product_id: i32,
    pub service_id: i32,
    pub mapping_type: String,
    pub inclusion_criteria: Option<serde_json::Value>,
    pub pricing_impact: Option<f64>,
    pub delivery_sequence: Option<i32>,
    pub is_mandatory: Option<bool>,
    pub customer_configurable: Option<bool>,
    pub effective_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceResourceMapping {
    pub id: i32,
    pub service_id: i32,
    pub resource_id: i32,
    pub usage_type: String,
    pub resource_role: Option<String>,
    pub configuration_parameters: Option<serde_json::Value>,
    pub performance_requirements: Option<serde_json::Value>,
    pub usage_limits: Option<serde_json::Value>,
    pub cost_allocation_percentage: Option<f64>,
    pub dependency_level: Option<i32>,
    pub failover_resource_id: Option<i32>,
    pub monitoring_thresholds: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceTemplateCapability {
    pub id: i32,
    pub template_id: i32,
    pub capability_id: i32,
    pub capability_order: Option<i32>,
    pub is_required: Option<bool>,
    pub configuration_overrides: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ===== VIEWS AND AGGREGATE STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductServiceResourceHierarchy {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub service_id: Option<String>,
    pub service_name: Option<String>,
    pub service_category: Option<String>,
    pub is_mandatory: Option<bool>,
    pub delivery_sequence: Option<i32>,
    pub resource_id: Option<i32>,
    pub resource_name: Option<String>,
    pub resource_type: Option<String>,
    pub resource_role: Option<String>,
    pub resource_priority: Option<i32>,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub dsl_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceTemplateCapabilityView {
    pub template_id: String,
    pub template_name: String,
    pub part_of_product: Option<String>,
    pub implements_service: Option<String>,
    pub capability_id: String,
    pub capability_name: String,
    pub capability_type: String,
    pub required_attributes: serde_json::Value,
    pub optional_attributes: serde_json::Value,
    pub output_attributes: serde_json::Value,
    pub capability_order: Option<i32>,
    pub capability_required: Option<bool>,
    pub configuration_overrides: serde_json::Value,
}

// ===== ENHANCED ONBOARDING SYSTEM =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingWorkflow {
    pub id: i32,
    pub workflow_id: String,
    pub cbu_id: i32,
    pub product_ids: Vec<i32>, // Multiple products can be onboarded together
    pub workflow_status: String, // "initiated", "in_progress", "completed", "failed", "cancelled"
    pub priority: String,
    pub target_go_live_date: Option<NaiveDate>,
    pub business_requirements: Option<serde_json::Value>,
    pub compliance_requirements: Option<serde_json::Value>,
    pub resource_requirements: serde_json::Value, // Calculated from product→service→resource mapping
    pub execution_plan: serde_json::Value, // Ordered list of resource templates and capabilities
    pub current_stage: Option<String>,
    pub completion_percentage: Option<i32>,
    pub requested_by: Option<String>,
    pub assigned_to: Option<String>,
    pub approval_chain: Option<serde_json::Value>,
    pub estimated_duration_days: Option<i32>,
    pub actual_duration_days: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingResourceTask {
    pub id: i32,
    pub workflow_id: i32,
    pub resource_template_id: i32,
    pub capability_id: i32,
    pub task_order: i32,
    pub task_status: String, // "pending", "in_progress", "completed", "failed", "blocked"
    pub input_attributes: serde_json::Value,
    pub output_attributes: Option<serde_json::Value>,
    pub validation_results: Option<serde_json::Value>,
    pub execution_log: Option<serde_json::Value>,
    pub assigned_to: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub blocking_issues: Option<String>,
    pub retry_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingDependency {
    pub id: i32,
    pub workflow_id: i32,
    pub source_task_id: i32,
    pub target_task_id: i32,
    pub dependency_type: String, // "blocking", "informational", "conditional"
    pub dependency_condition: Option<String>,
    pub is_satisfied: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingApproval {
    pub id: i32,
    pub workflow_id: i32,
    pub approval_stage: String,
    pub approver_role: String,
    pub approver_user: Option<String>,
    pub approval_status: String, // "pending", "approved", "rejected", "delegated"
    pub approval_notes: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ===== ONBOARDING WORKFLOW VIEWS =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingWorkflowDetails {
    pub workflow_id: String,
    pub cbu_name: String,
    pub product_names: Vec<String>,
    pub workflow_status: String,
    pub priority: String,
    pub current_stage: Option<String>,
    pub completion_percentage: Option<i32>,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub blocked_tasks: i64,
    pub target_go_live_date: Option<NaiveDate>,
    pub days_remaining: Option<i32>,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceProvisioningStatus {
    pub workflow_id: String,
    pub resource_template_name: String,
    pub resource_type: String,
    pub total_capabilities: i64,
    pub completed_capabilities: i64,
    pub failed_capabilities: i64,
    pub current_capability: Option<String>,
    pub instance_url: Option<String>,
    pub provision_status: String,
}

// ===== CAPABILITY EXECUTION STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecution {
    pub capability_id: String,
    pub input_attributes: serde_json::Value,
    pub execution_context: ExecutionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub resource_template_id: String,
    pub workflow_id: Option<String>,
    pub client_context: Option<serde_json::Value>,
    pub execution_mode: String, // "dry_run", "production", "test"
    pub timeout_override: Option<i32>,
    pub retry_context: Option<RetryContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryContext {
    pub attempt_number: i32,
    pub max_attempts: i32,
    pub backoff_strategy: String, // "linear", "exponential", "fixed"
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecutionResult {
    pub capability_id: String,
    pub execution_status: String, // "success", "error", "timeout", "retry_needed"
    pub output_attributes: serde_json::Value,
    pub error_details: Option<String>,
    pub execution_time_ms: i64,
    pub artifacts: Option<serde_json::Value>,
    pub next_action: Option<String>, // "continue", "wait_for_approval", "manual_intervention"
}

// ===== ONBOARDING REQUEST STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingWorkflowRequest {
    pub cbu_id: String, // External CBU ID
    pub product_ids: Vec<String>, // External Product IDs
    pub priority: Option<String>,
    pub target_go_live_date: Option<NaiveDate>,
    pub business_requirements: Option<serde_json::Value>,
    pub compliance_requirements: Option<serde_json::Value>,
    pub execution_mode: Option<String>, // "auto", "manual_approval", "hybrid"
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProvisioningPlan {
    pub resource_template_id: String,
    pub resource_template_name: String,
    pub resource_type: String,
    pub service_name: String,
    pub capabilities: Vec<CapabilityExecutionPlan>,
    pub estimated_hours: f32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecutionPlan {
    pub capability_id: String,
    pub capability_name: String,
    pub capability_type: String,
    pub execution_order: i32,
    pub estimated_minutes: i32,
    pub required_attributes: Vec<String>,
    pub output_attributes: Vec<String>,
    pub approval_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingWorkflowResponse {
    pub workflow_id: String,
    pub execution_plan: Vec<ResourceProvisioningPlan>,
    pub estimated_duration_days: i32,
    pub approval_required: bool,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuProductSubscription {
    pub id: i32,
    pub cbu_id: i32,
    pub product_id: i32,
    pub subscription_status: String,
    pub subscription_date: Option<DateTime<Utc>>,
    pub activation_date: Option<DateTime<Utc>>,
    pub termination_date: Option<DateTime<Utc>>,
    pub billing_arrangement: Option<serde_json::Value>,
    pub contract_reference: Option<String>,
    pub primary_contact_role_id: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingRequest {
    pub id: i32,
    pub request_id: String,
    pub cbu_id: i32,
    pub product_id: i32,
    pub request_status: String,
    pub priority: String,
    pub target_go_live_date: Option<NaiveDate>,
    pub business_requirements: Option<serde_json::Value>,
    pub compliance_requirements: Option<serde_json::Value>,
    pub requested_by: Option<String>,
    pub assigned_to: Option<String>,
    pub approval_chain: Option<serde_json::Value>,
    pub estimated_duration_days: Option<i32>,
    pub actual_duration_days: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingTask {
    pub id: i32,
    pub onboarding_request_id: i32,
    pub task_id: String,
    pub resource_id: Option<i32>,
    pub task_type: Option<String>,
    pub task_name: String,
    pub description: Option<String>,
    pub task_status: String,
    pub assigned_to: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub due_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub blocking_issues: Option<String>,
    pub completion_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===== REQUEST/RESPONSE STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub product_name: String,
    pub line_of_business: String,
    pub description: Option<String>,
    pub pricing_model: Option<String>,
    pub target_market: Option<String>,
    pub regulatory_requirements: Option<serde_json::Value>,
    pub sla_commitments: Option<serde_json::Value>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub service_name: String,
    pub service_category: Option<String>,
    pub description: Option<String>,
    pub is_core_service: Option<bool>,
    pub configuration_schema: Option<serde_json::Value>,
    pub dependencies: Option<Vec<String>>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServiceRequest {
    pub service_name: Option<String>,
    pub service_category: Option<String>,
    pub description: Option<String>,
    pub is_core_service: Option<bool>,
    pub configuration_schema: Option<serde_json::Value>,
    pub dependencies: Option<Vec<String>>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub resource_name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub capacity_limits: Option<serde_json::Value>,
    pub operational_hours: Option<String>,
    pub contact_information: Option<serde_json::Value>,
    pub technical_specifications: Option<serde_json::Value>,
    pub compliance_certifications: Option<Vec<String>>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingRequest {
    pub cbu_id: String, // External CBU ID
    pub product_id: String, // External Product ID
    pub priority: Option<String>,
    pub target_go_live_date: Option<NaiveDate>,
    pub business_requirements: Option<serde_json::Value>,
    pub compliance_requirements: Option<serde_json::Value>,
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeCbuToProductRequest {
    pub cbu_id: String, // External CBU ID
    pub product_id: String, // External Product ID
    pub billing_arrangement: Option<serde_json::Value>,
    pub contract_reference: Option<String>,
    pub primary_contact_role_code: Option<String>,
    pub created_by: Option<String>,
}

// ===== VIEW STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductHierarchyView {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub product_status: String,
    pub service_id: String,
    pub service_name: String,
    pub service_category: Option<String>,
    pub service_required: bool,
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub resource_role: Option<String>,
    pub resource_priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CbuSubscriptionView {
    pub cbu_id: String,
    pub cbu_name: String,
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub subscription_status: String,
    pub subscription_date: Option<DateTime<Utc>>,
    pub activation_date: Option<DateTime<Utc>>,
    pub primary_contact_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingProgressView {
    pub request_id: String,
    pub cbu_name: String,
    pub product_name: String,
    pub request_status: String,
    pub target_go_live_date: Option<NaiveDate>,
    pub total_tasks: Option<i64>,
    pub completed_tasks: Option<i64>,
    pub blocked_tasks: Option<i64>,
    pub completion_percentage: Option<f64>,
}

impl DbOperations {
    // ===== PRODUCT MANAGEMENT =====

    /// Create a new custody banking product
    pub async fn create_product(request: CreateProductRequest) -> Result<Product, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Generate unique product ID
        let product_id = format!("{}-{:03}",
            request.line_of_business.to_uppercase().chars().take(4).collect::<String>(),
            chrono::Utc::now().timestamp_millis() % 1000
        );

        let query = r#"
            INSERT INTO products (
                product_id, product_name, line_of_business, description,
                pricing_model, target_market, regulatory_requirements,
                sla_commitments, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#;

        sqlx::query_as::<_, Product>(query)
            .bind(&product_id)
            .bind(&request.product_name)
            .bind(&request.line_of_business)
            .bind(&request.description)
            .bind(&request.pricing_model)
            .bind(&request.target_market)
            .bind(&request.regulatory_requirements)
            .bind(&request.sla_commitments)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create product: {}", e))
    }

    /// Get all products with optional filtering by line of business
    pub async fn list_products(line_of_business: Option<String>) -> Result<Vec<Product>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match line_of_business {
            Some(_) => "SELECT * FROM products WHERE line_of_business = $1 AND status = 'active' ORDER BY product_name",
            None => "SELECT * FROM products WHERE status = 'active' ORDER BY line_of_business, product_name",
        };

        if let Some(lob) = line_of_business {
            sqlx::query_as::<_, Product>(query)
                .bind(lob)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Product>("SELECT * FROM products WHERE status = 'active' ORDER BY line_of_business, product_name")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to list products: {}", e))
    }

    /// Get complete product hierarchy (products → services → resources)
    pub async fn get_product_hierarchy(product_id: Option<String>) -> Result<Vec<ProductHierarchyView>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match product_id {
            Some(_) => "SELECT * FROM v_product_hierarchy WHERE product_id = $1",
            None => "SELECT * FROM v_product_hierarchy ORDER BY product_name, service_name, resource_priority",
        };

        if let Some(pid) = product_id {
            sqlx::query_as::<_, ProductHierarchyView>(query)
                .bind(pid)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, ProductHierarchyView>("SELECT * FROM v_product_hierarchy ORDER BY product_name, service_name, resource_priority")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to get product hierarchy: {}", e))
    }

    // ===== SERVICE MANAGEMENT =====

    /// Create a new service
    pub async fn create_service(request: CreateServiceRequest) -> Result<Service, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Generate unique service ID
        let service_id = format!("SERV-{}",
            request.service_name.to_uppercase()
                .replace(" ", "-")
                .chars().take(10).collect::<String>()
        );

        let query = r#"
            INSERT INTO services (
                service_id, service_name, service_category, description,
                is_core_service, configuration_schema, dependencies, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;

        sqlx::query_as::<_, Service>(query)
            .bind(&service_id)
            .bind(&request.service_name)
            .bind(&request.service_category)
            .bind(&request.description)
            .bind(request.is_core_service.unwrap_or(false))
            .bind(&request.configuration_schema)
            .bind(&request.dependencies)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create service: {}", e))
    }

    /// List all services with optional filtering by category
    pub async fn list_services(category: Option<String>) -> Result<Vec<Service>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match category {
            Some(_) => "SELECT * FROM services WHERE service_category = $1 AND status = 'active' ORDER BY service_name",
            None => "SELECT * FROM services WHERE status = 'active' ORDER BY service_category, service_name",
        };

        if let Some(cat) = category {
            sqlx::query_as::<_, Service>(query)
                .bind(cat)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Service>("SELECT * FROM services WHERE status = 'active' ORDER BY service_category, service_name")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to list services: {}", e))
    }

    /// Get a service by ID
    pub async fn get_service_by_id(service_id: i32) -> Result<Option<Service>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM services WHERE id = $1";

        sqlx::query_as::<_, Service>(query)
            .bind(service_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get service: {}", e))
    }

    /// Update an existing service
    pub async fn update_service(service_id: i32, request: UpdateServiceRequest) -> Result<Service, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Build dynamic update query based on provided fields
        let mut update_fields = Vec::new();
        let mut param_count = 1;

        if request.service_name.is_some() {
            update_fields.push(format!("service_name = ${}", param_count));
            param_count += 1;
        }
        if request.service_category.is_some() {
            update_fields.push(format!("service_category = ${}", param_count));
            param_count += 1;
        }
        if request.description.is_some() {
            update_fields.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if request.is_core_service.is_some() {
            update_fields.push(format!("is_core_service = ${}", param_count));
            param_count += 1;
        }
        if request.configuration_schema.is_some() {
            update_fields.push(format!("configuration_schema = ${}", param_count));
            param_count += 1;
        }
        if request.dependencies.is_some() {
            update_fields.push(format!("dependencies = ${}", param_count));
            param_count += 1;
        }
        if request.updated_by.is_some() {
            update_fields.push(format!("updated_by = ${}", param_count));
            param_count += 1;
        }

        if update_fields.is_empty() {
            return Err("No fields to update".to_string());
        }

        // Always update the updated_at timestamp
        update_fields.push("updated_at = CURRENT_TIMESTAMP".to_string());

        let query = format!(
            "UPDATE services SET {} WHERE id = ${} RETURNING *",
            update_fields.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, Service>(&query);

        // Bind parameters in the same order as the conditions
        if let Some(name) = request.service_name {
            query_builder = query_builder.bind(name);
        }
        if let Some(category) = request.service_category {
            query_builder = query_builder.bind(category);
        }
        if let Some(desc) = request.description {
            query_builder = query_builder.bind(desc);
        }
        if let Some(is_core) = request.is_core_service {
            query_builder = query_builder.bind(is_core);
        }
        if let Some(config) = request.configuration_schema {
            query_builder = query_builder.bind(config);
        }
        if let Some(deps) = request.dependencies {
            query_builder = query_builder.bind(deps);
        }
        if let Some(updated_by) = request.updated_by {
            query_builder = query_builder.bind(updated_by);
        }

        query_builder
            .bind(service_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to update service: {}", e))
    }

    // ===== RESOURCE MANAGEMENT =====

    /// Create a new resource
    pub async fn create_resource(request: CreateResourceRequest) -> Result<Resource, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Generate unique resource ID
        let resource_id = format!("RES-{}",
            request.resource_name.to_uppercase()
                .replace(" ", "-")
                .chars().take(10).collect::<String>()
        );

        let query = r#"
            INSERT INTO resources (
                resource_id, resource_name, resource_type, description, location,
                capacity_limits, operational_hours, contact_information,
                technical_specifications, compliance_certifications, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
        "#;

        sqlx::query_as::<_, Resource>(query)
            .bind(&resource_id)
            .bind(&request.resource_name)
            .bind(&request.resource_type)
            .bind(&request.description)
            .bind(&request.location)
            .bind(&request.capacity_limits)
            .bind(&request.operational_hours)
            .bind(&request.contact_information)
            .bind(&request.technical_specifications)
            .bind(&request.compliance_certifications)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create resource: {}", e))
    }

    /// List all resources with optional filtering by type
    pub async fn list_resources(resource_type: Option<String>) -> Result<Vec<Resource>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match resource_type {
            Some(_) => "SELECT * FROM resources WHERE resource_type = $1 AND status = 'active' ORDER BY resource_name",
            None => "SELECT * FROM resources WHERE status = 'active' ORDER BY resource_type, resource_name",
        };

        if let Some(rt) = resource_type {
            sqlx::query_as::<_, Resource>(query)
                .bind(rt)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE status = 'active' ORDER BY resource_type, resource_name")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to list resources: {}", e))
    }

    // ===== CBU PRODUCT SUBSCRIPTIONS =====

    /// Subscribe a CBU to a product
    pub async fn subscribe_cbu_to_product(request: SubscribeCbuToProductRequest) -> Result<CbuProductSubscription, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Get CBU internal ID
        let cbu = Self::get_cbu_by_id(&request.cbu_id).await?
            .ok_or_else(|| format!("CBU not found: {}", request.cbu_id))?;

        // Get Product internal ID
        let product_query = "SELECT id FROM products WHERE product_id = $1";
        let product_id: (i32,) = sqlx::query_as(product_query)
            .bind(&request.product_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Product not found: {}", e))?;

        // Get role ID if specified
        let role_id = if let Some(role_code) = &request.primary_contact_role_code {
            let role_query = "SELECT id FROM cbu_roles WHERE role_code = $1";
            Some(sqlx::query_as::<_, (i32,)>(role_query)
                .bind(role_code)
                .fetch_one(&pool)
                .await
                .map_err(|e| format!("Role not found: {}", e))?
                .0)
        } else {
            None
        };

        let query = r#"
            INSERT INTO cbu_product_subscriptions (
                cbu_id, product_id, billing_arrangement, contract_reference,
                primary_contact_role_id, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#;

        sqlx::query_as::<_, CbuProductSubscription>(query)
            .bind(cbu.id)
            .bind(product_id.0)
            .bind(&request.billing_arrangement)
            .bind(&request.contract_reference)
            .bind(role_id)
            .bind(&request.created_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create subscription: {}", e))
    }

    /// Get CBU product subscriptions
    pub async fn get_cbu_subscriptions(cbu_id: Option<String>) -> Result<Vec<CbuSubscriptionView>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match cbu_id {
            Some(_) => "SELECT * FROM v_cbu_product_subscriptions WHERE cbu_id = $1 ORDER BY product_name",
            None => "SELECT * FROM v_cbu_product_subscriptions ORDER BY cbu_name, product_name",
        };

        if let Some(cbu) = cbu_id {
            sqlx::query_as::<_, CbuSubscriptionView>(query)
                .bind(cbu)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, CbuSubscriptionView>("SELECT * FROM v_cbu_product_subscriptions ORDER BY cbu_name, product_name")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to get CBU subscriptions: {}", e))
    }

    // ===== ONBOARDING WORKFLOW =====

    /// Create an onboarding request
    pub async fn create_onboarding_request(request: CreateOnboardingRequest) -> Result<OnboardingRequest, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Get CBU and Product internal IDs
        let cbu = Self::get_cbu_by_id(&request.cbu_id).await?
            .ok_or_else(|| format!("CBU not found: {}", request.cbu_id))?;

        let product_query = "SELECT id FROM products WHERE product_id = $1";
        let product_id: (i32,) = sqlx::query_as(product_query)
            .bind(&request.product_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Product not found: {}", e))?;

        // Generate unique request ID
        let request_id = format!("ONB-{}-{}-{:06}",
            cbu.cbu_id,
            request.product_id,
            chrono::Utc::now().timestamp_millis() % 1000000
        );

        let query = r#"
            INSERT INTO onboarding_requests (
                request_id, cbu_id, product_id, priority, target_go_live_date,
                business_requirements, compliance_requirements, requested_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;

        sqlx::query_as::<_, OnboardingRequest>(query)
            .bind(&request_id)
            .bind(cbu.id)
            .bind(product_id.0)
            .bind(request.priority.unwrap_or_else(|| "medium".to_string()))
            .bind(request.target_go_live_date)
            .bind(&request.business_requirements)
            .bind(&request.compliance_requirements)
            .bind(&request.requested_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create onboarding request: {}", e))
    }

    /// Get onboarding progress for all requests or specific CBU/Product
    pub async fn get_onboarding_progress(
        cbu_id: Option<String>,
        product_id: Option<String>
    ) -> Result<Vec<OnboardingProgressView>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let base_query = "SELECT * FROM v_onboarding_progress";

        let query = match (cbu_id.as_ref(), product_id.as_ref()) {
            (Some(_), Some(_)) => format!("{} WHERE cbu_name ILIKE $1 AND product_name ILIKE $2", base_query),
            (Some(_), None) => format!("{} WHERE cbu_name ILIKE $1", base_query),
            (None, Some(_)) => format!("{} WHERE product_name ILIKE $1", base_query),
            (None, None) => format!("{} ORDER BY created_at DESC", base_query),
        };

        match (cbu_id, product_id) {
            (Some(cbu), Some(prod)) => {
                sqlx::query_as::<_, OnboardingProgressView>(&query)
                    .bind(format!("%{}%", cbu))
                    .bind(format!("%{}%", prod))
                    .fetch_all(&pool)
                    .await
            },
            (Some(cbu), None) => {
                sqlx::query_as::<_, OnboardingProgressView>(&query)
                    .bind(format!("%{}%", cbu))
                    .fetch_all(&pool)
                    .await
            },
            (None, Some(prod)) => {
                sqlx::query_as::<_, OnboardingProgressView>(&query)
                    .bind(format!("%{}%", prod))
                    .fetch_all(&pool)
                    .await
            },
            (None, None) => {
                sqlx::query_as::<_, OnboardingProgressView>(&query)
                    .fetch_all(&pool)
                    .await
            },
        }
        .map_err(|e| format!("Failed to get onboarding progress: {}", e))
    }

    /// Get lines of business for product categorization
    pub async fn get_lines_of_business() -> Result<Vec<String>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT DISTINCT line_of_business FROM products WHERE status = 'active' ORDER BY line_of_business";

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get lines of business: {}", e))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Get service categories for service organization
    pub async fn get_service_categories() -> Result<Vec<String>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT DISTINCT service_category FROM services WHERE status = 'active' AND service_category IS NOT NULL ORDER BY service_category";

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get service categories: {}", e))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Get resource types for resource classification
    pub async fn get_resource_types() -> Result<Vec<String>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT DISTINCT resource_type FROM resources WHERE status = 'active' ORDER BY resource_type";

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get resource types: {}", e))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // ===== RESOURCE TEMPLATE OPERATIONS =====

    /// Get all resource templates
    pub async fn list_resource_templates() -> Result<Vec<ResourceTemplate>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM resource_templates WHERE status = 'active' ORDER BY template_name";

        sqlx::query_as::<_, ResourceTemplate>(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to list resource templates: {}", e))
    }

    /// Get resource template by ID
    pub async fn get_resource_template_by_id(template_id: &str) -> Result<Option<ResourceTemplate>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM resource_templates WHERE template_id = $1 AND status = 'active'";

        sqlx::query_as::<_, ResourceTemplate>(query)
            .bind(template_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get resource template: {}", e))
    }

    /// Get resource template with capabilities
    pub async fn get_resource_template_capabilities(template_id: &str) -> Result<Vec<ResourceTemplateCapabilityView>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM v_resource_template_capabilities WHERE template_id = $1 ORDER BY capability_order";

        sqlx::query_as::<_, ResourceTemplateCapabilityView>(query)
            .bind(template_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get resource template capabilities: {}", e))
    }

    // ===== RESOURCE CAPABILITY OPERATIONS =====

    /// Get all resource capabilities
    pub async fn list_resource_capabilities() -> Result<Vec<ResourceCapability>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM resource_capabilities WHERE status = 'active' ORDER BY capability_type, capability_name";

        sqlx::query_as::<_, ResourceCapability>(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to list resource capabilities: {}", e))
    }

    /// Get resource capability by ID
    pub async fn get_resource_capability_by_id(capability_id: &str) -> Result<Option<ResourceCapability>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM resource_capabilities WHERE capability_id = $1 AND status = 'active'";

        sqlx::query_as::<_, ResourceCapability>(query)
            .bind(capability_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get resource capability: {}", e))
    }

    /// Get capabilities by type
    pub async fn get_capabilities_by_type(capability_type: &str) -> Result<Vec<ResourceCapability>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = "SELECT * FROM resource_capabilities WHERE capability_type = $1 AND status = 'active' ORDER BY capability_name";

        sqlx::query_as::<_, ResourceCapability>(query)
            .bind(capability_type)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get capabilities by type: {}", e))
    }

    // ===== PRODUCT-SERVICE-RESOURCE HIERARCHY =====

    /// Get complete product-service-resource hierarchy
    pub async fn get_product_service_resource_hierarchy(product_id: Option<String>) -> Result<Vec<ProductServiceResourceHierarchy>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = match product_id {
            Some(_) => "SELECT * FROM v_product_service_resource_hierarchy WHERE product_id = $1",
            None => "SELECT * FROM v_product_service_resource_hierarchy ORDER BY product_name, delivery_sequence, resource_priority",
        };

        if let Some(pid) = product_id {
            sqlx::query_as::<_, ProductServiceResourceHierarchy>(query)
                .bind(pid)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, ProductServiceResourceHierarchy>("SELECT * FROM v_product_service_resource_hierarchy ORDER BY product_name, delivery_sequence, resource_priority")
                .fetch_all(&pool)
                .await
        }
        .map_err(|e| format!("Failed to get product service resource hierarchy: {}", e))
    }

    /// Get services for a product
    pub async fn get_product_services(product_id: &str) -> Result<Vec<Service>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT s.* FROM services s
            JOIN product_service_mappings psm ON s.id = psm.service_id
            JOIN products p ON psm.product_id = p.id
            WHERE p.product_id = $1 AND s.status = 'active'
            ORDER BY psm.delivery_sequence, s.service_name
        "#;

        sqlx::query_as::<_, Service>(query)
            .bind(product_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get product services: {}", e))
    }

    /// Get resources for a service
    pub async fn get_service_resources(service_id: &str) -> Result<Vec<ResourceTemplate>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT DISTINCT rt.* FROM resource_templates rt
            JOIN service_resource_mappings srm ON rt.resource_type = (
                SELECT ro.resource_type FROM resource_objects ro WHERE ro.id = srm.resource_id
            )
            JOIN services s ON srm.service_id = s.id
            WHERE s.service_id = $1 AND rt.status = 'active'
            ORDER BY rt.template_name
        "#;

        sqlx::query_as::<_, ResourceTemplate>(query)
            .bind(service_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get service resources: {}", e))
    }

    // ===== CAPABILITY EXECUTION OPERATIONS =====

    /// Validate capability execution request
    pub async fn validate_capability_execution(
        capability_id: &str,
        input_attributes: &serde_json::Value,
    ) -> Result<bool, String> {
        let capability = Self::get_resource_capability_by_id(capability_id).await?
            .ok_or_else(|| format!("Capability not found: {}", capability_id))?;

        // Validate required attributes are present
        if let Some(required_attrs) = capability.required_attributes.as_array() {
            for attr in required_attrs {
                if let Some(attr_name) = attr.as_str() {
                    if input_attributes.get(attr_name).is_none() {
                        return Err(format!("Required attribute missing: {}", attr_name));
                    }
                }
            }
        }

        // Additional validation logic can be added here
        // - Check attribute types
        // - Check validation rules
        // - Check business constraints

        Ok(true)
    }

    /// Get fund accounting template
    pub async fn get_fund_accounting_template() -> Result<Option<ResourceTemplate>, String> {
        Self::get_resource_template_by_id("CoreFAApp_v1").await
    }

    /// Get fund accounting capabilities
    pub async fn get_fund_accounting_capabilities() -> Result<Vec<ResourceTemplateCapabilityView>, String> {
        Self::get_resource_template_capabilities("CoreFAApp_v1").await
    }

    // ===== ENHANCED ONBOARDING WORKFLOW OPERATIONS =====

    /// Create a new capability-driven onboarding workflow
    pub async fn create_onboarding_workflow(request: CreateOnboardingWorkflowRequest) -> Result<OnboardingWorkflowResponse, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        // Get CBU internal ID
        let cbu = Self::get_cbu_by_id(&request.cbu_id).await?
            .ok_or_else(|| format!("CBU not found: {}", request.cbu_id))?;

        // Get Product internal IDs and validate they exist
        let mut product_internal_ids = Vec::new();
        for product_id in &request.product_ids {
            let product_query = "SELECT id FROM products WHERE product_id = $1 AND status = 'active'";
            let product_result: Result<(i32,), _> = sqlx::query_as(product_query)
                .bind(product_id)
                .fetch_one(&pool)
                .await;

            match product_result {
                Ok((id,)) => product_internal_ids.push(id),
                Err(_) => return Err(format!("Product not found: {}", product_id)),
            }
        }

        // Generate unique workflow ID
        let workflow_id = format!("WF-{}-{:06}",
            cbu.cbu_id,
            chrono::Utc::now().timestamp_millis() % 1000000
        );

        // Generate execution plan by traversing product→service→resource mappings
        let execution_plan = Self::generate_execution_plan(&request.product_ids).await?;

        // Calculate estimated duration
        let estimated_duration_days = execution_plan.iter()
            .map(|plan| (plan.estimated_hours / 8.0).ceil() as i32)
            .max()
            .unwrap_or(1);

        // Determine if approval is required
        let approval_required = execution_plan.iter()
            .any(|plan| plan.capabilities.iter().any(|cap| cap.approval_required));

        // Create workflow record
        let query = r#"
            INSERT INTO onboarding_workflows (
                workflow_id, cbu_id, product_ids, priority, target_go_live_date,
                business_requirements, compliance_requirements, resource_requirements,
                execution_plan, estimated_duration_days, requested_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
        "#;

        let workflow_db_id: (i32,) = sqlx::query_as(query)
            .bind(&workflow_id)
            .bind(cbu.id)
            .bind(&product_internal_ids)
            .bind(request.priority.unwrap_or_else(|| "medium".to_string()))
            .bind(request.target_go_live_date)
            .bind(&request.business_requirements)
            .bind(&request.compliance_requirements)
            .bind(serde_json::json!({})) // Resource requirements calculated from execution plan
            .bind(serde_json::to_value(&execution_plan).unwrap())
            .bind(estimated_duration_days)
            .bind(&request.requested_by)
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to create onboarding workflow: {}", e))?;

        // Create individual tasks for each capability
        Self::create_workflow_tasks(workflow_db_id.0, &execution_plan).await?;

        let next_steps = if approval_required {
            vec!["Awaiting approval from compliance team".to_string()]
        } else {
            vec!["Ready to begin resource provisioning".to_string()]
        };

        Ok(OnboardingWorkflowResponse {
            workflow_id,
            execution_plan,
            estimated_duration_days,
            approval_required,
            next_steps,
        })
    }

    /// Generate execution plan from product-service-resource mapping
    async fn generate_execution_plan(product_ids: &[String]) -> Result<Vec<ResourceProvisioningPlan>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;
        let mut execution_plan = Vec::new();

        for product_id in product_ids {
            // Get services for this product
            let services = Self::get_product_services(product_id).await?;

            for service in services {
                // Get resource templates for this service
                let resource_templates = Self::get_service_resources(&service.service_id).await?;

                for template in resource_templates {
                    // Get capabilities for this template
                    let capabilities_view = Self::get_resource_template_capabilities(&template.template_id).await?;

                    let capabilities: Vec<CapabilityExecutionPlan> = capabilities_view.into_iter().map(|cap| {
                        let capability_type = cap.capability_type.clone();
                        let approval_required = capability_type == "activation"; // Business rule

                        CapabilityExecutionPlan {
                            capability_id: cap.capability_id,
                            capability_name: cap.capability_name,
                            capability_type,
                            execution_order: cap.capability_order.unwrap_or(1),
                            estimated_minutes: 30, // Default estimate, could be enhanced
                            required_attributes: if let Some(attrs) = cap.required_attributes.as_array() {
                                attrs.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                            } else {
                                Vec::new()
                            },
                            output_attributes: if let Some(attrs) = cap.output_attributes.as_array() {
                                attrs.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                            } else {
                                Vec::new()
                            },
                            approval_required,
                        }
                    }).collect();

                    let estimated_hours = capabilities.len() as f32 * 0.5; // 30 min per capability

                    execution_plan.push(ResourceProvisioningPlan {
                        resource_template_id: template.template_id,
                        resource_template_name: template.template_name,
                        resource_type: template.resource_type,
                        service_name: service.service_name.clone(),
                        capabilities,
                        estimated_hours,
                        dependencies: Vec::new(), // Could be enhanced with dependency analysis
                    });
                }
            }
        }

        Ok(execution_plan)
    }

    /// Create workflow tasks for each capability
    async fn create_workflow_tasks(workflow_id: i32, execution_plan: &[ResourceProvisioningPlan]) -> Result<(), String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;
        let mut task_order = 1;

        for plan in execution_plan {
            // Get template internal ID
            let template_query = "SELECT id FROM resource_templates WHERE template_id = $1";
            let template_id: (i32,) = sqlx::query_as(template_query)
                .bind(&plan.resource_template_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| format!("Template not found: {}", e))?;

            for capability in &plan.capabilities {
                // Get capability internal ID
                let capability_query = "SELECT id FROM resource_capabilities WHERE capability_id = $1";
                let capability_id: (i32,) = sqlx::query_as(capability_query)
                    .bind(&capability.capability_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| format!("Capability not found: {}", e))?;

                // Create task
                let task_query = r#"
                    INSERT INTO onboarding_resource_tasks (
                        workflow_id, resource_template_id, capability_id, task_order,
                        task_status, input_attributes, estimated_hours
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#;

                sqlx::query(task_query)
                    .bind(workflow_id)
                    .bind(template_id.0)
                    .bind(capability_id.0)
                    .bind(task_order)
                    .bind("pending")
                    .bind(serde_json::json!({}))
                    .bind(capability.estimated_minutes as f32 / 60.0)
                    .execute(&pool)
                    .await
                    .map_err(|e| format!("Failed to create task: {}", e))?;

                task_order += 1;
            }
        }

        Ok(())
    }

    /// Get onboarding workflow details
    pub async fn get_onboarding_workflow_details(workflow_id: &str) -> Result<Option<OnboardingWorkflowDetails>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT
                ow.workflow_id,
                cbu.cbu_name,
                ARRAY_AGG(p.product_name) as product_names,
                ow.workflow_status,
                ow.priority,
                ow.current_stage,
                ow.completion_percentage,
                COUNT(ort.*) as total_tasks,
                COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) as completed_tasks,
                COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) as failed_tasks,
                COUNT(CASE WHEN ort.task_status = 'blocked' THEN 1 END) as blocked_tasks,
                ow.target_go_live_date,
                CASE
                    WHEN ow.target_go_live_date IS NOT NULL
                    THEN EXTRACT(DAY FROM ow.target_go_live_date - CURRENT_DATE)::int
                    ELSE NULL
                END as days_remaining,
                ow.assigned_to,
                ow.created_at
            FROM onboarding_workflows ow
            JOIN client_business_units cbu ON ow.cbu_id = cbu.id
            LEFT JOIN unnest(ow.product_ids) AS pid(id) ON true
            LEFT JOIN products p ON pid.id = p.id
            LEFT JOIN onboarding_resource_tasks ort ON ow.id = ort.workflow_id
            WHERE ow.workflow_id = $1
            GROUP BY ow.workflow_id, cbu.cbu_name, ow.workflow_status, ow.priority,
                     ow.current_stage, ow.completion_percentage, ow.target_go_live_date,
                     ow.assigned_to, ow.created_at
        "#;

        sqlx::query_as::<_, OnboardingWorkflowDetails>(query)
            .bind(workflow_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to get workflow details: {}", e))
    }

    /// Get resource provisioning status for a workflow
    pub async fn get_resource_provisioning_status(workflow_id: &str) -> Result<Vec<ResourceProvisioningStatus>, String> {
        let pool = Self::get_pool().await.map_err(|e| e.to_string())?;

        let query = r#"
            SELECT
                ow.workflow_id,
                rt.template_name as resource_template_name,
                rt.resource_type,
                COUNT(ort.*) as total_capabilities,
                COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) as completed_capabilities,
                COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) as failed_capabilities,
                (SELECT rc.capability_name FROM resource_capabilities rc
                 JOIN onboarding_resource_tasks ort2 ON rc.id = ort2.capability_id
                 WHERE ort2.workflow_id = ow.id AND ort2.task_status = 'in_progress'
                 LIMIT 1) as current_capability,
                NULL::text as instance_url, -- Would be populated from task output_attributes
                CASE
                    WHEN COUNT(CASE WHEN ort.task_status = 'failed' THEN 1 END) > 0 THEN 'failed'
                    WHEN COUNT(CASE WHEN ort.task_status = 'completed' THEN 1 END) = COUNT(ort.*) THEN 'completed'
                    WHEN COUNT(CASE WHEN ort.task_status IN ('in_progress', 'completed') THEN 1 END) > 0 THEN 'in_progress'
                    ELSE 'pending'
                END as provision_status
            FROM onboarding_workflows ow
            JOIN onboarding_resource_tasks ort ON ow.id = ort.workflow_id
            JOIN resource_templates rt ON ort.resource_template_id = rt.id
            WHERE ow.workflow_id = $1
            GROUP BY ow.workflow_id, rt.template_name, rt.resource_type, ow.id
            ORDER BY rt.template_name
        "#;

        sqlx::query_as::<_, ResourceProvisioningStatus>(query)
            .bind(workflow_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("Failed to get provisioning status: {}", e))
    }

    /// Execute a specific capability within a workflow
    pub async fn execute_workflow_capability(
        workflow_id: &str,
        capability_id: &str,
        input_attributes: serde_json::Value,
    ) -> Result<CapabilityExecutionResult, String> {
        // This would integrate with the actual resource provisioning systems
        // For now, we'll return a simulated result
        let execution_start = std::time::Instant::now();

        // Validate the capability execution
        Self::validate_capability_execution(capability_id, &input_attributes).await?;

        // Simulate the capability execution
        // In a real implementation, this would:
        // 1. Load the capability implementation function
        // 2. Execute it with the provided input attributes
        // 3. Handle any errors or retries
        // 4. Store the results

        let execution_time_ms = execution_start.elapsed().as_millis() as i64;

        Ok(CapabilityExecutionResult {
            capability_id: capability_id.to_string(),
            execution_status: "success".to_string(),
            output_attributes: serde_json::json!({
                "capability_executed": capability_id,
                "timestamp": chrono::Utc::now()
            }),
            error_details: None,
            execution_time_ms,
            artifacts: Some(serde_json::json!({
                "workflow_id": workflow_id,
                "execution_mode": "production"
            })),
            next_action: Some("continue".to_string()),
        })
    }
}