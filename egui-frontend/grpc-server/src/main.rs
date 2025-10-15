use tonic::{transport::Server, Request, Response, Status};
use sqlx::{PgPool, Row};
use std::env;
use tracing::{info, error};

// Generated protobuf code
pub mod financial_taxonomy {
    tonic::include_proto!("financial_taxonomy");
}

use financial_taxonomy::{
    financial_taxonomy_service_server::{FinancialTaxonomyService, FinancialTaxonomyServiceServer},
    *,
};

// Database connection and service implementation
pub struct TaxonomyServer {
    db_pool: PgPool,
}

impl TaxonomyServer {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
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

        if let Some(product_id) = &req.product_id {
            query_builder = query_builder.bind(product_id);
        }

        query_builder = query_builder.bind(limit).bind(offset);

        match query_builder.fetch_all(&self.db_pool).await {
            Ok(rows) => {
                let product_options: Vec<ProductOption> = rows
                    .into_iter()
                    .map(|row| ProductOption {
                        id: row.get::<i32, _>("id"),
                        option_id: row.get::<String, _>("option_id"),
                        product_id: row.get::<String, _>("product_id"),
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

        // For now, return sample suggestions
        let suggestions = vec![
            AiSuggestion {
                title: "Sample AI Suggestion".to_string(),
                description: "This is a sample AI-generated suggestion".to_string(),
                category: "general".to_string(),
                confidence: 0.8,
                applicable_contexts: vec!["products".to_string(), "services".to_string()],
            },
        ];

        let response = GetAiSuggestionsResponse {
            suggestions,
            status_message: "AI suggestions generated successfully".to_string(),
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