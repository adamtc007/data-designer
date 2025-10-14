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
}