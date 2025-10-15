use crate::financial_taxonomy::{
    financial_taxonomy_service_client::FinancialTaxonomyServiceClient,
    *,
};
use anyhow::Result;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{info, error};

#[derive(Clone)]
pub struct TaxonomyGrpcClient {
    client: FinancialTaxonomyServiceClient<Channel>,
}

impl TaxonomyGrpcClient {
    pub async fn new(server_url: &str) -> Result<Self> {
        info!("Connecting to gRPC server at {}", server_url);
        let client = FinancialTaxonomyServiceClient::connect(server_url.to_string()).await?;
        info!("gRPC client connected successfully");

        Ok(Self { client })
    }

    pub async fn get_products(&mut self, status_filter: Option<String>) -> Result<Vec<Product>> {
        let request = Request::new(GetProductsRequest {
            status_filter,
            limit: None,
            offset: None,
        });

        match self.client.get_products(request).await {
            Ok(response) => {
                let products = response.into_inner().products;
                info!("Retrieved {} products", products.len());
                Ok(products)
            }
            Err(e) => {
                error!("Error getting products: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_product_options(&mut self, product_id: Option<String>) -> Result<Vec<ProductOption>> {
        let request = Request::new(GetProductOptionsRequest {
            product_id,
            status_filter: Some("active".to_string()),
            limit: None,
            offset: None,
        });

        match self.client.get_product_options(request).await {
            Ok(response) => {
                let options = response.into_inner().product_options;
                info!("Retrieved {} product options", options.len());
                Ok(options)
            }
            Err(e) => {
                error!("Error getting product options: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_services(&mut self, service_category: Option<String>) -> Result<Vec<Service>> {
        let request = Request::new(GetServicesRequest {
            status_filter: Some("active".to_string()),
            service_category,
            limit: None,
            offset: None,
        });

        match self.client.get_services(request).await {
            Ok(response) => {
                let services = response.into_inner().services;
                info!("Retrieved {} services", services.len());
                Ok(services)
            }
            Err(e) => {
                error!("Error getting services: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_cbu_mandate_structure(&mut self) -> Result<Vec<CbuInvestmentMandateStructure>> {
        let request = Request::new(GetCbuMandateStructureRequest {
            cbu_id: None,
            mandate_id: None,
            limit: None,
            offset: None,
        });

        match self.client.get_cbu_mandate_structure(request).await {
            Ok(response) => {
                let structures = response.into_inner().structures;
                info!("Retrieved {} CBU mandate structures", structures.len());
                Ok(structures)
            }
            Err(e) => {
                error!("Error getting CBU mandate structure: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_cbu_member_roles(&mut self) -> Result<Vec<CbuMemberInvestmentRole>> {
        let request = Request::new(GetCbuMemberRolesRequest {
            cbu_id: None,
            role_code: None,
            limit: None,
            offset: None,
        });

        match self.client.get_cbu_member_roles(request).await {
            Ok(response) => {
                let roles = response.into_inner().roles;
                info!("Retrieved {} CBU member roles", roles.len());
                Ok(roles)
            }
            Err(e) => {
                error!("Error getting CBU member roles: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_taxonomy_hierarchy(&mut self) -> Result<Vec<TaxonomyHierarchyItem>> {
        let request = Request::new(GetTaxonomyHierarchyRequest {
            max_levels: None,
            item_type_filter: None,
        });

        match self.client.get_taxonomy_hierarchy(request).await {
            Ok(response) => {
                let items = response.into_inner().items;
                info!("Retrieved {} taxonomy hierarchy items", items.len());
                Ok(items)
            }
            Err(e) => {
                error!("Error getting taxonomy hierarchy: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_ai_suggestions(&mut self, query: String, ai_provider: AiProvider) -> Result<Vec<AiSuggestion>> {
        let request = Request::new(GetAiSuggestionsRequest {
            query,
            ai_provider: Some(ai_provider),
            context: None,
        });

        match self.client.get_ai_suggestions(request).await {
            Ok(response) => {
                let response_inner = response.into_inner();
                info!("Retrieved {} AI suggestions: {}", response_inner.suggestions.len(), response_inner.status_message);
                Ok(response_inner.suggestions)
            }
            Err(e) => {
                error!("Error getting AI suggestions: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn health_check(&mut self, service: String) -> Result<HealthCheckResponse> {
        let request = Request::new(HealthCheckRequest { service });

        match self.client.check(request).await {
            Ok(response) => {
                let health_response = response.into_inner();
                info!("Health check status: {:?}", health_response.status);
                Ok(health_response)
            }
            Err(e) => {
                error!("Error in health check: {}", e);
                Err(e.into())
            }
        }
    }
}