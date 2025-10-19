use tonic::{transport::Server, Request, Response, Status};
use sqlx::{PgPool, Row};
use std::env;
use std::process::Command;
use tracing::{info, error};
use std::collections::HashMap;

mod template_api;

// Generated protobuf code
pub mod financial_taxonomy {
    tonic::include_proto!("financial_taxonomy");
}

use financial_taxonomy::{
    financial_taxonomy_service_server::{FinancialTaxonomyService, FinancialTaxonomyServiceServer},
    *,
};

// AI Assistant types (simplified for gRPC server)
#[derive(Debug, Clone, PartialEq)]
pub enum AiProvider {
    OpenAI { api_key: Option<String> },
    Anthropic { api_key: Option<String> },
    Offline,
}

#[derive(Debug, Clone)]
pub struct LocalAiSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub code_snippet: Option<String>,
    pub confidence: f32,
    pub context_relevance: f32,
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    CodeCompletion,
    ErrorFix,
    Optimization,
    FunctionHelp,
    SyntaxHelp,
    BestPractice,
    DataIntegration,
    QuickFix,
}

pub struct SimpleAiAssistant {
    pub provider: AiProvider,
    pub suggestions_cache: HashMap<String, Vec<LocalAiSuggestion>>,
    pub pool: Option<PgPool>,
}

// Helper function to retrieve API key using macOS security command as fallback
fn get_api_key_via_security(service: &str, account: &str) -> Result<String, String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", service, "-a", account, "-w"])
        .output()
        .map_err(|e| format!("Failed to execute security command: {}", e))?;

    if output.status.success() {
        let key = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in security output: {}", e))?
            .trim()
            .to_string();

        if key.is_empty() {
            Err("Security command returned empty key".to_string())
        } else {
            Ok(key)
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Security command failed: {}", error))
    }
}

// Database connection and service implementation
pub struct TaxonomyServer {
    db_pool: PgPool,
    pool: PgPool, // For AI assistant
}

impl TaxonomyServer {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool: db_pool.clone(),
            pool: db_pool
        }
    }

    fn simulate_dsl_execution(&self, dsl_code: &str, input_data: &Option<String>) -> Result<serde_json::Value, String> {
        // Simulate DSL execution for now
        // TODO: Replace with actual transpiler integration

        let mut result = serde_json::json!({
            "dsl_executed": dsl_code,
            "simulation": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Some(input) = input_data {
            if let Ok(input_json) = serde_json::from_str::<serde_json::Value>(input) {
                result["input"] = input_json;
            }
        }

        // Simple DSL simulation based on content
        if dsl_code.contains("validate") {
            result["validation_result"] = serde_json::json!({
                "status": "passed",
                "checks": ["syntax", "semantics", "runtime"]
            });
        }

        if dsl_code.contains("calculate") || dsl_code.contains("compute") {
            result["computation_result"] = serde_json::json!({
                "value": 42.0,
                "formula": dsl_code
            });
        }

        if dsl_code.contains("kyc") || dsl_code.contains("onboarding") {
            result["kyc_result"] = serde_json::json!({
                "status": "approved",
                "score": 0.85,
                "requirements_met": true
            });
        }

        Ok(result)
    }
}

#[tonic::async_trait]
impl FinancialTaxonomyService for TaxonomyServer {
    async fn get_products(
        &self,
        request: Request<GetProductsRequest>,
    ) -> Result<Response<GetProductsResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetProducts request with status filter: {:?}", req.status_filter);

        let status_filter = req.status_filter.unwrap_or_else(|| "active".to_string());
        let limit = req.limit.unwrap_or(100) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let query = r#"
            SELECT id, product_id, product_name, line_of_business, description, status,
                   contract_type, commercial_status, pricing_model, target_market
            FROM products
            WHERE status = $1
            ORDER BY product_name
            LIMIT $2 OFFSET $3
        "#;

        match sqlx::query(query)
            .bind(&status_filter)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
        {
            Ok(rows) => {
                let products: Vec<Product> = rows
                    .into_iter()
                    .map(|row| Product {
                        id: row.get::<i32, _>("id"),
                        product_id: row.get::<String, _>("product_id"),
                        product_name: row.get::<String, _>("product_name"),
                        line_of_business: row.get::<String, _>("line_of_business"),
                        description: row.get::<Option<String>, _>("description"),
                        status: row.get::<String, _>("status"),
                        contract_type: row.get::<Option<String>, _>("contract_type"),
                        commercial_status: row.get::<Option<String>, _>("commercial_status"),
                        pricing_model: row.get::<Option<String>, _>("pricing_model"),
                        target_market: row.get::<Option<String>, _>("target_market"),
                    })
                    .collect();

                let total_count = products.len() as i32;
                let response = GetProductsResponse {
                    products,
                    total_count,
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error in get_products: {}", e);
                Err(Status::internal(format!("Database error: {}", e)))
            }
        }
    }

    async fn get_product_options(
        &self,
        request: Request<GetProductOptionsRequest>,
    ) -> Result<Response<GetProductOptionsResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetProductOptions request");

        let status_filter = req.status_filter.unwrap_or_else(|| "active".to_string());
        let limit = req.limit.unwrap_or(100) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let mut query = "SELECT id, option_id, product_id, option_name, option_category, option_type, option_value, display_name, description, pricing_impact, status FROM product_options WHERE status = $1".to_string();
        let mut param_count = 1;

        if let Some(_product_id) = &req.product_id {
            param_count += 1;
            query.push_str(&format!(" AND product_id = ${}", param_count));
        }

        query.push_str(&format!(" ORDER BY option_name LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

        let mut query_builder = sqlx::query(&query).bind(&status_filter);

        // Look up the integer ID from products table using product_id string
        if let Some(product_id_str) = &req.product_id {
            // First, get the integer ID from the products table
            let product_lookup_query = "SELECT id FROM products WHERE product_id = $1";
            match sqlx::query(product_lookup_query)
                .bind(product_id_str)
                .fetch_optional(&self.db_pool)
                .await {
                Ok(Some(row)) => {
                    let product_id_int: i32 = row.get("id");
                    query_builder = query_builder.bind(product_id_int);
                },
                Ok(None) => {
                    return Err(Status::not_found(format!("Product not found with product_id: {}", product_id_str)));
                },
                Err(e) => {
                    error!("Database error looking up product: {}", e);
                    return Err(Status::internal(format!("Database error: {}", e)));
                }
            }
        }

        query_builder = query_builder.bind(limit).bind(offset);

        match query_builder.fetch_all(&self.db_pool).await {
            Ok(rows) => {
                let product_options: Vec<ProductOption> = rows
                    .into_iter()
                    .map(|row| ProductOption {
                        id: row.get::<i32, _>("id"),
                        option_id: row.get::<String, _>("option_id"),
                        product_id: row.get::<i32, _>("product_id").to_string(),
                        option_name: row.get::<String, _>("option_name"),
                        option_category: row.get::<String, _>("option_category"),
                        option_type: row.get::<String, _>("option_type"),
                        option_value: row.get::<Option<String>, _>("option_value"),
                        display_name: row.get::<Option<String>, _>("display_name"),
                        description: row.get::<Option<String>, _>("description"),
                        pricing_impact: row.get::<Option<f64>, _>("pricing_impact"),
                        status: row.get::<String, _>("status"),
                    })
                    .collect();

                let total_count = product_options.len() as i32;
                let response = GetProductOptionsResponse {
                    product_options,
                    total_count,
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error in get_product_options: {}", e);
                Err(Status::internal(format!("Database error: {}", e)))
            }
        }
    }

    async fn get_services(
        &self,
        request: Request<GetServicesRequest>,
    ) -> Result<Response<GetServicesResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetServices request");

        let status_filter = req.status_filter.unwrap_or_else(|| "active".to_string());
        let limit = req.limit.unwrap_or(100) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let query = r#"
            SELECT id, service_id, service_name, service_category, description,
                   service_type, delivery_model, billable, status
            FROM services
            WHERE status = $1
            ORDER BY service_name
            LIMIT $2 OFFSET $3
        "#;

        match sqlx::query(query)
            .bind(&status_filter)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
        {
            Ok(rows) => {
                let services: Vec<Service> = rows
                    .into_iter()
                    .map(|row| Service {
                        id: row.get::<String, _>("service_id"),
                        name: row.get::<String, _>("service_name"),
                        description: row.get::<Option<String>, _>("description").unwrap_or_default(),
                        r#type: row.get::<Option<String>, _>("service_category").unwrap_or_default(),
                        service_type: row.get::<Option<String>, _>("service_type").unwrap_or_default(),
                        delivery_model: row.get::<Option<String>, _>("delivery_model").unwrap_or_default(),
                        billable: row.get::<Option<bool>, _>("billable").unwrap_or_default(),
                        status: row.get::<String, _>("status"),
                    })
                    .collect();

                let total_count = services.len() as i32;
                let response = GetServicesResponse {
                    services,
                    total_count,
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error in get_services: {}", e);
                Err(Status::internal(format!("Database error: {}", e)))
            }
        }
    }

    async fn get_cbu_mandate_structure(
        &self,
        request: Request<GetCbuMandateStructureRequest>,
    ) -> Result<Response<GetCbuMandateStructureResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetCbuMandateStructure request");

        let limit = req.limit.unwrap_or(100) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let query = r#"
            SELECT cbu_id, cbu_name, mandate_id, asset_owner_name, investment_manager_name,
                   base_currency, total_instruments, families, total_exposure_pct
            FROM cbu_investment_mandate_structure
            ORDER BY cbu_id
            LIMIT $1 OFFSET $2
        "#;

        match sqlx::query(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
        {
            Ok(rows) => {
                let structures: Vec<CbuInvestmentMandateStructure> = rows
                    .into_iter()
                    .map(|row| CbuInvestmentMandateStructure {
                        cbu_id: row.get::<String, _>("cbu_id"),
                        cbu_name: row.get::<String, _>("cbu_name"),
                        mandate_id: row.get::<Option<String>, _>("mandate_id"),
                        asset_owner_name: row.get::<Option<String>, _>("asset_owner_name"),
                        investment_manager_name: row.get::<Option<String>, _>("investment_manager_name"),
                        base_currency: row.get::<Option<String>, _>("base_currency"),
                        total_instruments: row.get::<Option<i32>, _>("total_instruments"),
                        families: row.get::<Option<String>, _>("families"),
                        total_exposure_pct: row.get::<Option<f64>, _>("total_exposure_pct"),
                    })
                    .collect();

                let total_count = structures.len() as i32;
                let response = GetCbuMandateStructureResponse {
                    structures,
                    total_count,
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error in get_cbu_mandate_structure: {}", e);
                Err(Status::internal(format!("Database error: {}", e)))
            }
        }
    }

    async fn get_cbu_member_roles(
        &self,
        request: Request<GetCbuMemberRolesRequest>,
    ) -> Result<Response<GetCbuMemberRolesResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetCbuMemberRoles request");

        let limit = req.limit.unwrap_or(100) as i64;
        let offset = req.offset.unwrap_or(0) as i64;

        let query = r#"
            SELECT cbu_id, cbu_name, entity_name, entity_lei, role_name, role_code,
                   investment_responsibility, mandate_id, has_trading_authority, has_settlement_authority
            FROM cbu_member_investment_roles
            ORDER BY cbu_id, role_code
            LIMIT $1 OFFSET $2
        "#;

        match sqlx::query(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
        {
            Ok(rows) => {
                let roles: Vec<CbuMemberInvestmentRole> = rows
                    .into_iter()
                    .map(|row| CbuMemberInvestmentRole {
                        cbu_id: row.get::<String, _>("cbu_id"),
                        cbu_name: row.get::<String, _>("cbu_name"),
                        entity_name: row.get::<String, _>("entity_name"),
                        entity_lei: row.get::<Option<String>, _>("entity_lei"),
                        role_name: row.get::<String, _>("role_name"),
                        role_code: row.get::<String, _>("role_code"),
                        investment_responsibility: row.get::<String, _>("investment_responsibility"),
                        mandate_id: row.get::<Option<String>, _>("mandate_id"),
                        has_trading_authority: row.get::<Option<bool>, _>("has_trading_authority"),
                        has_settlement_authority: row.get::<Option<bool>, _>("has_settlement_authority"),
                    })
                    .collect();

                let total_count = roles.len() as i32;
                let response = GetCbuMemberRolesResponse {
                    roles,
                    total_count,
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error in get_cbu_member_roles: {}", e);
                Err(Status::internal(format!("Database error: {}", e)))
            }
        }
    }

    async fn get_taxonomy_hierarchy(
        &self,
        request: Request<GetTaxonomyHierarchyRequest>,
    ) -> Result<Response<GetTaxonomyHierarchyResponse>, Status> {
        let _req = request.into_inner();
        info!("Received GetTaxonomyHierarchy request");

        // For now, return sample data
        let items = vec![
            TaxonomyHierarchyItem {
                level: 1,
                item_type: "product".to_string(),
                item_id: 1,
                item_name: "Institutional Custody Plus".to_string(),
                item_description: Some("Comprehensive custody services".to_string()),
                parent_id: None,
                configuration: None,
                metadata: None,
            },
            TaxonomyHierarchyItem {
                level: 2,
                item_type: "product_option".to_string(),
                item_id: 2,
                item_name: "US Market Settlement".to_string(),
                item_description: Some("Settlement in US markets".to_string()),
                parent_id: Some(1),
                configuration: None,
                metadata: None,
            },
        ];

        let response = GetTaxonomyHierarchyResponse { items };
        Ok(Response::new(response))
    }

    async fn get_ai_suggestions(
        &self,
        request: Request<GetAiSuggestionsRequest>,
    ) -> Result<Response<GetAiSuggestionsResponse>, Status> {
        let req = request.into_inner();
        info!("Received GetAiSuggestions request for query: {}", req.query);

        // Create AI context from request
        let context = req.context.unwrap_or_else(|| "general".to_string());

        // Initialize AI assistant based on provider
        let ai_provider = match req.ai_provider {
            Some(provider) => {
                match provider.provider_type {
                    1 => {
                        // Anthropic provider
                        AiProvider::Anthropic {
                            api_key: provider.api_key
                        }
                    },
                    0 | _ => {
                        // OpenAI provider (default)
                        AiProvider::OpenAI {
                            api_key: provider.api_key
                        }
                    }
                }
            },
            None => AiProvider::Offline
        };

        // Create AI assistant instance
        let mut ai_assistant = create_ai_assistant(ai_provider, self.pool.clone()).await;

        // Get suggestions based on query and context
        let local_suggestions = if context == "error_analysis" {
            ai_assistant.analyze_error(&req.query, "")
        } else if context == "code_completion" {
            ai_assistant.get_code_completions(&req.query, 0)
        } else if context.starts_with("rag_") {
            ai_assistant.get_rag_suggestions(&req.query, 10).await
        } else {
            ai_assistant.get_suggestions(&req.query).await
        };

        // Convert local AI suggestions to gRPC format
        let suggestions: Vec<AiSuggestion> = local_suggestions
            .into_iter()
            .map(|local_suggestion| AiSuggestion {
                title: local_suggestion.title,
                description: local_suggestion.description,
                category: format!("{:?}", local_suggestion.suggestion_type).to_lowercase(),
                confidence: local_suggestion.confidence as f64,
                applicable_contexts: vec![context.clone()],
            })
            .collect();

        let status_message = if suggestions.is_empty() {
            "No AI suggestions available for this query".to_string()
        } else {
            format!("Generated {} AI suggestions successfully", suggestions.len())
        };

        let response = GetAiSuggestionsResponse {
            suggestions,
            status_message,
        };

        Ok(Response::new(response))
    }

    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let req = request.into_inner();
        info!("Health check for service: {}", req.service);

        let response = HealthCheckResponse {
            status: 1, // 1 = serving
            message: format!("Service {} is healthy", req.service),
        };

        Ok(Response::new(response))
    }

    async fn get_database_status(
        &self,
        _request: Request<DatabaseStatusRequest>,
    ) -> Result<Response<DatabaseStatusResponse>, Status> {
        info!("Received database status request");

        // Test database connection
        let (connected, error_message) = match sqlx::query("SELECT 1").fetch_one(&self.db_pool).await {
            Ok(_) => (true, None),
            Err(e) => (false, Some(format!("Database connection error: {}", e))),
        };

        // Get database info
        let database_name = "data_designer".to_string();
        let host = "localhost".to_string();
        let port = 5432;

        // Get table counts (simplified)
        let (total_products, total_services, total_mandates) = if connected {
            let products = sqlx::query("SELECT COUNT(*) as count FROM products")
                .fetch_one(&self.db_pool).await
                .map(|row| row.get::<i64, _>("count") as i32)
                .unwrap_or(0);

            let services = sqlx::query("SELECT COUNT(*) as count FROM services")
                .fetch_one(&self.db_pool).await
                .map(|row| row.get::<i64, _>("count") as i32)
                .unwrap_or(0);

            (products, services, 0) // Mandates count placeholder
        } else {
            (0, 0, 0)
        };

        let response = DatabaseStatusResponse {
            connected,
            database_name,
            host,
            port,
            status_message: if connected { "Database connection healthy".to_string() } else { "Database connection failed".to_string() },
            last_check: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            error_message,
            total_tables: if connected { 10 } else { 0 }, // Placeholder
            total_products,
            total_services,
            total_mandates,
        };

        Ok(Response::new(response))
    }

    async fn store_api_key(
        &self,
        request: Request<StoreApiKeyRequest>,
    ) -> Result<Response<StoreApiKeyResponse>, Status> {
        let req = request.into_inner();
        info!("Storing API key for provider: {}", req.provider);

        // Use keyring to securely store the API key
        match keyring::Entry::new("data-designer", &req.provider) {
            Ok(entry) => {
                match entry.set_password(&req.api_key) {
                    Ok(_) => {
                        let response = StoreApiKeyResponse {
                            success: true,
                            message: format!("API key stored successfully for provider: {}", req.provider),
                            key_id: format!("key_{}_{}", req.provider, uuid::Uuid::new_v4()),
                        };
                        Ok(Response::new(response))
                    },
                    Err(e) => {
                        error!("Failed to store API key for provider {}: {}", req.provider, e);
                        let response = StoreApiKeyResponse {
                            success: false,
                            message: format!("Failed to store API key: {}", e),
                            key_id: String::new(),
                        };
                        Ok(Response::new(response))
                    }
                }
            },
            Err(e) => {
                error!("Failed to create keyring entry for provider {}: {}", req.provider, e);
                Err(Status::internal(format!("Keyring error: {}", e)))
            }
        }
    }

    async fn get_api_key(
        &self,
        request: Request<GetApiKeyRequest>,
    ) -> Result<Response<GetApiKeyResponse>, Status> {
        let req = request.into_inner();
        info!("Retrieving API key for provider: {}", req.provider);

        // Use keyring to securely retrieve the API key
        match keyring::Entry::new("data-designer", &req.provider) {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(api_key) => {
                        let response = GetApiKeyResponse {
                            success: true,
                            api_key,
                            message: format!("API key retrieved successfully for provider: {}", req.provider),
                            key_exists: true,
                        };
                        Ok(Response::new(response))
                    },
                    Err(keyring::Error::NoEntry) => {
                        let response = GetApiKeyResponse {
                            success: false,
                            api_key: String::new(),
                            message: format!("No API key found for provider: {}", req.provider),
                            key_exists: false,
                        };
                        Ok(Response::new(response))
                    },
                    Err(e) => {
                        error!("Failed to retrieve API key for provider {}: {}", req.provider, e);

                        // Try fallback using security command
                        info!("Attempting fallback with security command for provider: {}", req.provider);
                        match get_api_key_via_security("data-designer", &req.provider) {
                            Ok(api_key) => {
                                info!("Successfully retrieved API key via security command for provider: {}", req.provider);
                                let response = GetApiKeyResponse {
                                    success: true,
                                    api_key,
                                    message: format!("API key retrieved via security command for provider: {}", req.provider),
                                    key_exists: true,
                                };
                                Ok(Response::new(response))
                            },
                            Err(security_error) => {
                                error!("Security command also failed for provider {}: {}", req.provider, security_error);
                                let response = GetApiKeyResponse {
                                    success: false,
                                    api_key: String::new(),
                                    message: format!("Failed to retrieve API key: keyring error: {}, security error: {}", e, security_error),
                                    key_exists: false,
                                };
                                Ok(Response::new(response))
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to create keyring entry for provider {}: {}", req.provider, e);
                Err(Status::internal(format!("Keyring error: {}", e)))
            }
        }
    }

    async fn delete_api_key(
        &self,
        request: Request<DeleteApiKeyRequest>,
    ) -> Result<Response<DeleteApiKeyResponse>, Status> {
        let req = request.into_inner();
        info!("Deleting API key for provider: {}", req.provider);

        // Use keyring to securely delete the API key
        match keyring::Entry::new("data-designer", &req.provider) {
            Ok(entry) => {
                match entry.delete_password() {
                    Ok(_) => {
                        let response = DeleteApiKeyResponse {
                            success: true,
                            message: format!("API key deleted successfully for provider: {}", req.provider),
                        };
                        Ok(Response::new(response))
                    },
                    Err(keyring::Error::NoEntry) => {
                        let response = DeleteApiKeyResponse {
                            success: true, // Consider it success if key didn't exist
                            message: format!("No API key found for provider: {} (already deleted)", req.provider),
                        };
                        Ok(Response::new(response))
                    },
                    Err(e) => {
                        error!("Failed to delete API key for provider {}: {}", req.provider, e);
                        let response = DeleteApiKeyResponse {
                            success: false,
                            message: format!("Failed to delete API key: {}", e),
                        };
                        Ok(Response::new(response))
                    }
                }
            },
            Err(e) => {
                error!("Failed to create keyring entry for provider {}: {}", req.provider, e);
                Err(Status::internal(format!("Keyring error: {}", e)))
            }
        }
    }

    async fn list_api_keys(
        &self,
        _request: Request<ListApiKeysRequest>,
    ) -> Result<Response<ListApiKeysResponse>, Status> {
        info!("Listing stored API keys");

        // Check for common AI providers in keyring
        let known_providers = vec!["openai", "anthropic", "claude"];
        let mut found_providers = Vec::new();

        for provider in known_providers {
            match keyring::Entry::new("data-designer", provider) {
                Ok(entry) => {
                    if entry.get_password().is_ok() {
                        found_providers.push(provider.to_string());
                    }
                },
                Err(e) => {
                    error!("Failed to check keyring for provider {}: {}", provider, e);
                }
            }
        }

        let message = if found_providers.is_empty() {
            "No API keys currently stored".to_string()
        } else {
            format!("Found {} stored API key(s)", found_providers.len())
        };

        let response = ListApiKeysResponse {
            providers: found_providers,
            message,
        };

        Ok(Response::new(response))
    }

    async fn instantiate_resource(
        &self,
        request: Request<InstantiateResourceRequest>,
    ) -> Result<Response<InstantiateResourceResponse>, Status> {
        let req = request.into_inner();
        info!("Instantiating resource template: {} for onboarding request: {}", req.template_id, req.onboarding_request_id);

        // Generate unique instance ID
        let instance_id = uuid::Uuid::new_v4().to_string();

        // Load the template from resource_sheets table
        let template_query = "SELECT * FROM resource_sheets WHERE resource_id = $1";
        let template_row = match sqlx::query(template_query)
            .bind(&req.template_id)
            .fetch_optional(&self.db_pool)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return Err(Status::not_found(format!("Template not found: {}", req.template_id)));
            }
            Err(e) => {
                error!("Database error loading template: {}", e);
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        };

        // Extract template data
        let template_json: serde_json::Value = template_row.get("json_data");
        let mut instance_data = template_json.clone();

        // Merge initial data if provided
        if let Some(initial_data_str) = req.initial_data {
            if let Ok(initial_data) = serde_json::from_str::<serde_json::Value>(&initial_data_str) {
                if let (Some(instance_obj), Some(initial_obj)) = (instance_data.as_object_mut(), initial_data.as_object()) {
                    for (key, value) in initial_obj {
                        instance_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Insert into resource_instances table
        let insert_query = r#"
            INSERT INTO resource_instances (instance_id, onboarding_request_id, template_id, status, instance_data, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING created_at, updated_at
        "#;

        match sqlx::query(insert_query)
            .bind(&instance_id)
            .bind(&req.onboarding_request_id)
            .bind(&req.template_id)
            .bind("pending")
            .bind(&instance_data)
            .bind("system")
            .fetch_one(&self.db_pool)
            .await
        {
            Ok(row) => {
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

                let instance = ResourceInstance {
                    instance_id: instance_id.clone(),
                    onboarding_request_id: req.onboarding_request_id.clone(),
                    template_id: req.template_id.clone(),
                    status: "pending".to_string(),
                    instance_data: instance_data.to_string(),
                    created_at: Some(prost_types::Timestamp {
                        seconds: created_at.timestamp(),
                        nanos: created_at.timestamp_subsec_nanos() as i32,
                    }),
                    updated_at: Some(prost_types::Timestamp {
                        seconds: updated_at.timestamp(),
                        nanos: updated_at.timestamp_subsec_nanos() as i32,
                    }),
                    error_message: None,
                };

                let response = InstantiateResourceResponse {
                    success: true,
                    message: format!("Resource instance created successfully with ID: {}", instance_id),
                    instance: Some(instance),
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                error!("Database error creating resource instance: {}", e);
                Err(Status::internal(format!("Failed to create resource instance: {}", e)))
            }
        }
    }

    async fn execute_dsl(
        &self,
        request: Request<ExecuteDslRequest>,
    ) -> Result<Response<ExecuteDslResponse>, Status> {
        let req = request.into_inner();
        info!("Executing DSL for instance: {}", req.instance_id);

        let execution_start = std::time::Instant::now();

        // Load the resource instance
        let instance_query = "SELECT * FROM resource_instances WHERE instance_id = $1";
        let instance_row = match sqlx::query(instance_query)
            .bind(&req.instance_id)
            .fetch_optional(&self.db_pool)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return Err(Status::not_found(format!("Resource instance not found: {}", req.instance_id)));
            }
            Err(e) => {
                error!("Database error loading instance: {}", e);
                return Err(Status::internal(format!("Database error: {}", e)));
            }
        };

        let instance_data: serde_json::Value = instance_row.get("instance_data");
        let mut log_messages = Vec::new();
        let mut execution_status = "success".to_string();
        let mut error_details = None;
        let mut output_data = serde_json::json!({});

        // Extract DSL from instance data (try multiple field names)
        let dsl_code = instance_data
            .get("business_logic_dsl")
            .or_else(|| instance_data.get("dsl"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if dsl_code.is_empty() {
            execution_status = "failed".to_string();
            error_details = Some("No DSL code found in resource instance".to_string());
        } else {
            log_messages.push(format!("Executing DSL: {}", dsl_code));

            // TODO: Integrate with actual DSL transpiler and execution engine
            // For now, simulate execution
            match self.simulate_dsl_execution(dsl_code, &req.input_data) {
                Ok(result) => {
                    output_data = result;
                    log_messages.push("DSL execution completed successfully".to_string());
                }
                Err(e) => {
                    execution_status = "failed".to_string();
                    error_details = Some(e);
                    log_messages.push("DSL execution failed".to_string());
                }
            }
        }

        let execution_time = execution_start.elapsed().as_millis() as f64;

        // Update instance status
        let update_query = "UPDATE resource_instances SET status = $1, updated_at = now() WHERE instance_id = $2";
        let _ = sqlx::query(update_query)
            .bind(if execution_status == "success" { "completed" } else { "failed" })
            .bind(&req.instance_id)
            .execute(&self.db_pool)
            .await;

        // Log execution results
        let log_query = r#"
            INSERT INTO dsl_execution_logs (instance_id, execution_status, input_data, output_data, log_messages, error_details, execution_time_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        let input_data_json = req.input_data
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap_or_default();

        let _ = sqlx::query(log_query)
            .bind(&req.instance_id)
            .bind(&execution_status)
            .bind(&input_data_json)
            .bind(&output_data)
            .bind(&log_messages)
            .bind(&error_details)
            .bind(execution_time)
            .execute(&self.db_pool)
            .await;

        let result = DslExecutionResult {
            instance_id: req.instance_id.clone(),
            execution_status: execution_status.clone(),
            output_data: output_data.to_string(),
            log_messages,
            error_details,
            executed_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            execution_time_ms: execution_time,
        };

        let response = ExecuteDslResponse {
            success: execution_status == "success",
            message: if execution_status == "success" {
                "DSL execution completed successfully".to_string()
            } else {
                "DSL execution failed".to_string()
            },
            result: Some(result),
        };

        Ok(Response::new(response))
    }
}

// AI Assistant implementation
impl SimpleAiAssistant {
    pub async fn new(provider: AiProvider, pool: Option<PgPool>) -> Self {
        Self {
            provider,
            suggestions_cache: HashMap::new(),
            pool,
        }
    }

    pub async fn get_suggestions(&mut self, query: &str) -> Vec<LocalAiSuggestion> {
        // Check cache first
        if let Some(cached) = self.suggestions_cache.get(query) {
            return cached.clone();
        }

        let suggestions = match &self.provider {
            AiProvider::OpenAI { api_key } => {
                if api_key.is_some() {
                    self.get_openai_suggestions(query).await
                } else {
                    self.get_offline_suggestions(query)
                }
            },
            AiProvider::Anthropic { api_key } => {
                if api_key.is_some() {
                    self.get_anthropic_suggestions(query).await
                } else {
                    self.get_offline_suggestions(query)
                }
            },
            AiProvider::Offline => self.get_offline_suggestions(query),
        };

        // Cache suggestions
        self.suggestions_cache.insert(query.to_string(), suggestions.clone());
        suggestions
    }

    pub fn analyze_error(&self, error_message: &str, _context: &str) -> Vec<LocalAiSuggestion> {
        let mut suggestions = Vec::new();

        if error_message.contains("syntax") || error_message.contains("parse") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::SyntaxHelp,
                title: "Syntax Error Fix".to_string(),
                description: "Check for missing brackets, quotes, or semicolons".to_string(),
                code_snippet: None,
                confidence: 0.8,
                context_relevance: 0.9,
            });
        }

        if error_message.contains("type") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::ErrorFix,
                title: "Type Error Resolution".to_string(),
                description: "Ensure data types match expected values".to_string(),
                code_snippet: None,
                confidence: 0.7,
                context_relevance: 0.8,
            });
        }

        if suggestions.is_empty() {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::ErrorFix,
                title: "General Error Help".to_string(),
                description: "Review the error message and check documentation".to_string(),
                code_snippet: None,
                confidence: 0.5,
                context_relevance: 0.6,
            });
        }

        suggestions
    }

    pub fn get_code_completions(&self, input: &str, _cursor_pos: usize) -> Vec<LocalAiSuggestion> {
        let mut suggestions = Vec::new();

        if input.contains("if") || input.contains("condition") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::CodeCompletion,
                title: "Conditional Expression".to_string(),
                description: "Complete conditional logic with comparison operators".to_string(),
                code_snippet: Some("if (condition) { result } else { alternative }".to_string()),
                confidence: 0.9,
                context_relevance: 0.8,
            });
        }

        if input.contains("function") || input.contains("func") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::FunctionHelp,
                title: "Function Definition".to_string(),
                description: "Available functions: min, max, sum, count, avg".to_string(),
                code_snippet: Some("function_name(parameter1, parameter2)".to_string()),
                confidence: 0.85,
                context_relevance: 0.9,
            });
        }

        suggestions
    }

    pub async fn get_rag_suggestions(&self, query: &str, _limit: i32) -> Vec<LocalAiSuggestion> {
        // Simplified RAG implementation for gRPC server
        vec![
            LocalAiSuggestion {
                suggestion_type: SuggestionType::DataIntegration,
                title: "RAG-based Suggestion".to_string(),
                description: format!("Context-aware suggestion for: {}", query),
                code_snippet: None,
                confidence: 0.7,
                context_relevance: 0.8,
            }
        ]
    }

    async fn get_openai_suggestions(&self, query: &str) -> Vec<LocalAiSuggestion> {
        // For now, return enhanced offline suggestions
        // TODO: Implement actual OpenAI API call
        self.get_enhanced_offline_suggestions(query)
    }

    async fn get_anthropic_suggestions(&self, query: &str) -> Vec<LocalAiSuggestion> {
        // For now, return enhanced offline suggestions
        // TODO: Implement actual Anthropic API call
        self.get_enhanced_offline_suggestions(query)
    }

    fn get_offline_suggestions(&self, query: &str) -> Vec<LocalAiSuggestion> {
        let mut suggestions = Vec::new();
        let query_lower = query.to_lowercase();

        if query_lower.contains("product") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::DataIntegration,
                title: "Product Data Query".to_string(),
                description: "Access product information from the financial taxonomy".to_string(),
                code_snippet: Some("product.name == 'target_product'".to_string()),
                confidence: 0.8,
                context_relevance: 0.9,
            });
        }

        if query_lower.contains("validation") || query_lower.contains("validate") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::BestPractice,
                title: "Data Validation".to_string(),
                description: "Implement validation rules for data integrity".to_string(),
                code_snippet: Some("validate(field) && field.length > 0".to_string()),
                confidence: 0.9,
                context_relevance: 0.8,
            });
        }

        if query_lower.contains("optimize") || query_lower.contains("performance") {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::Optimization,
                title: "Performance Optimization".to_string(),
                description: "Optimize expressions for better performance".to_string(),
                code_snippet: None,
                confidence: 0.7,
                context_relevance: 0.6,
            });
        }

        if suggestions.is_empty() {
            suggestions.push(LocalAiSuggestion {
                suggestion_type: SuggestionType::FunctionHelp,
                title: "General DSL Help".to_string(),
                description: "Available operators: ==, !=, <, >, &&, ||, +, -, *, /".to_string(),
                code_snippet: None,
                confidence: 0.6,
                context_relevance: 0.5,
            });
        }

        suggestions
    }

    fn get_enhanced_offline_suggestions(&self, query: &str) -> Vec<LocalAiSuggestion> {
        let mut suggestions = self.get_offline_suggestions(query);

        // Add context-specific suggestions
        suggestions.push(LocalAiSuggestion {
            suggestion_type: SuggestionType::QuickFix,
            title: "Enhanced Context".to_string(),
            description: format!("Enhanced suggestion based on query: {}", query),
            code_snippet: None,
            confidence: 0.75,
            context_relevance: 0.7,
        });

        suggestions
    }
}

// Helper function to create AI assistant
async fn create_ai_assistant(provider: AiProvider, pool: PgPool) -> SimpleAiAssistant {
    SimpleAiAssistant::new(provider, Some(pool)).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://adamtc007@localhost/data_designer".to_string());

    info!("Connecting to database: {}", database_url);
    let db_pool = PgPool::connect(&database_url).await?;
    info!("Database connection established");

    // Create gRPC service
    let taxonomy_service = TaxonomyServer::new(db_pool);

    // Create HTTP template API router
    let template_router = template_api::create_template_router();

    // Server addresses
    let grpc_addr = "0.0.0.0:50051".parse::<std::net::SocketAddr>()?;
    let http_addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>()?;

    info!("Starting gRPC server on {}", grpc_addr);
    info!("Starting HTTP template API on {}", http_addr);

    // Run both servers concurrently
    let grpc_server = Server::builder()
        .add_service(FinancialTaxonomyServiceServer::new(taxonomy_service))
        .serve(grpc_addr);

    let http_server = axum::serve(
        tokio::net::TcpListener::bind(http_addr).await?,
        template_router
    );

    // Start both servers in parallel
    tokio::select! {
        result = grpc_server => {
            error!("gRPC server exited: {:?}", result);
            result?;
        }
        result = http_server => {
            error!("HTTP server exited: {:?}", result);
            result?;
        }
    }

    Ok(())
}