use tonic::{transport::Server, Request, Response, Status};
use sqlx::{PgPool, Row};
use std::env;
use std::process::Command;
use tracing::{info, error};
use std::collections::HashMap;

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
                        id: row.get::<i32, _>("id"),
                        service_id: row.get::<String, _>("service_id"),
                        service_name: row.get::<String, _>("service_name"),
                        service_category: row.get::<Option<String>, _>("service_category"),
                        description: row.get::<Option<String>, _>("description"),
                        service_type: row.get::<Option<String>, _>("service_type"),
                        delivery_model: row.get::<Option<String>, _>("delivery_model"),
                        billable: row.get::<Option<bool>, _>("billable"),
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
            status: health_check_response::ServingStatus::Serving as i32,
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

    // Create service
    let taxonomy_service = TaxonomyServer::new(db_pool);

    // Server address
    let addr = "0.0.0.0:50051".parse()?;
    info!("Starting gRPC server on {}", addr);

    // Start server
    Server::builder()
        .add_service(FinancialTaxonomyServiceServer::new(taxonomy_service))
        .serve(addr)
        .await?;

    Ok(())
}