use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::wasm_utils;

// Message types matching the gRPC proto definitions
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
pub struct ExecuteCbuDslRequest {
    pub dsl_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCbuDslResponse {
    pub success: bool,
    pub message: String,
    pub cbu_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<String>, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCbusRequest {
    pub status_filter: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCbusResponse {
    pub cbus: Vec<CbuRecord>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuRecord {
    pub id: i32,
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: Option<String>,
    pub legal_entity_name: Option<String>,
    pub jurisdiction: Option<String>,
    pub business_model: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntitiesRequest {
    pub jurisdiction: Option<String>,
    pub entity_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntitiesResponse {
    pub entities: Vec<ClientEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEntity {
    pub entity_id: String,
    pub entity_name: String,
    pub entity_type: String,
    pub jurisdiction: String,
    pub country_code: String,
    pub lei_code: Option<String>,
    pub status: String,
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

// Product CRUD types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: Option<String>,
    pub contract_type: Option<String>,
    pub commercial_status: Option<String>,
    pub pricing_model: Option<String>,
    pub target_market: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductResponse {
    pub success: bool,
    pub message: String,
    pub product: Option<ProductDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProductRequest {
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProductResponse {
    pub success: bool,
    pub message: String,
    pub product: Option<ProductDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: Option<String>,
    pub contract_type: Option<String>,
    pub commercial_status: Option<String>,
    pub pricing_model: Option<String>,
    pub target_market: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductResponse {
    pub success: bool,
    pub message: String,
    pub product: Option<ProductDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProductRequest {
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProductResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProductsRequest {
    pub status_filter: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProductsResponse {
    pub products: Vec<ProductDetails>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetails {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: String,
    pub contract_type: String,
    pub commercial_status: String,
    pub pricing_model: String,
    pub target_market: String,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Clone)]
pub struct GrpcClient {
    base_url: String,
    client: reqwest::Client,
}

impl GrpcClient {
    pub fn new(grpc_endpoint: &str) -> Self {
        Self {
            base_url: grpc_endpoint.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Make a raw gRPC call using HTTP JSON requests
    async fn grpc_call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        // Use HTTP endpoint directly - no mock fallback
        self.try_http_call(service_method, request).await
    }

    /// Try to make an HTTP call to gRPC server
    async fn try_http_call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        wasm_utils::console_log(&format!("🔍 Trying HTTP call for service method: '{}'", service_method));
        let endpoint = match service_method {
            "financial_taxonomy.FinancialTaxonomyService/InstantiateResource" => "/api/instantiate",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteDsl" => "/api/execute-dsl",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteCbuDsl" => "/api/execute-cbu-dsl",
            "financial_taxonomy.FinancialTaxonomyService/ListCbus" => "/api/list-cbus",
            "financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions" => "/api/ai-suggestions",
            "financial_taxonomy.FinancialTaxonomyService/GetEntities" => "/api/entities",
            "financial_taxonomy.FinancialTaxonomyService/ListProducts" => "/api/list-products",
            _ => {
                wasm_utils::console_log(&format!("❌ Unknown service method: '{}'", service_method));
                return Err(anyhow::anyhow!("Unknown service method: {}", service_method));
            }
        };

        wasm_utils::console_log(&format!("✅ Matched endpoint: {}", endpoint));
        let url = format!("http://localhost:8080{}", endpoint);
        wasm_utils::console_log(&format!("Making HTTP request to: {}", url));

        let response = self.client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP request failed with status: {}", response.status()));
        }

        let response_body = response
            .json::<R>()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        Ok(response_body)
    }


    pub async fn instantiate_resource(
        &self,
        request: InstantiateResourceRequest,
    ) -> Result<InstantiateResourceResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/InstantiateResource", &request)
            .await
    }

    pub async fn execute_dsl(
        &self,
        request: ExecuteDslRequest,
    ) -> Result<ExecuteDslResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ExecuteDsl", &request)
            .await
    }

    pub async fn get_ai_suggestions(
        &self,
        request: GetAiSuggestionsRequest,
    ) -> Result<GetAiSuggestionsResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions", &request)
            .await
    }

    pub async fn get_entities(
        &self,
        request: GetEntitiesRequest,
    ) -> Result<GetEntitiesResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/GetEntities", &request)
            .await
    }

    // Product CRUD operations
    pub async fn create_product(
        &self,
        request: CreateProductRequest,
    ) -> Result<CreateProductResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/CreateProduct", &request)
            .await
    }

    pub async fn get_product(
        &self,
        request: GetProductRequest,
    ) -> Result<GetProductResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/GetProduct", &request)
            .await
    }

    pub async fn update_product(
        &self,
        request: UpdateProductRequest,
    ) -> Result<UpdateProductResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/UpdateProduct", &request)
            .await
    }

    pub async fn delete_product(
        &self,
        request: DeleteProductRequest,
    ) -> Result<DeleteProductResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/DeleteProduct", &request)
            .await
    }

    pub async fn list_products(
        &self,
        request: ListProductsRequest,
    ) -> Result<ListProductsResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ListProducts", &request)
            .await
    }

    pub async fn execute_cbu_dsl(
        &self,
        request: ExecuteCbuDslRequest,
    ) -> Result<ExecuteCbuDslResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ExecuteCbuDsl", &request)
            .await
    }

    pub async fn list_cbus(
        &self,
        request: ListCbusRequest,
    ) -> Result<ListCbusResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ListCbus", &request)
            .await
    }
}

// Helper function to create a UUID (WASM-compatible)
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // Simple UUID generation for WASM
            let timestamp = js_sys::Date::now() as u64;
            write!(f, "{:x}-{:x}-{:x}-{:x}",
                   timestamp & 0xFFFFFFFF,
                   (timestamp >> 32) & 0xFFFF,
                   0x4000 | ((timestamp >> 48) & 0x0FFF), // Version 4
                   0x8000 | ((timestamp >> 60) & 0x3FFF)  // Variant bits
            )
        }
    }
}

// Helper for getting current time in WASM
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
            let date = js_sys::Date::new_0();
            date.to_iso_string().into()
        }
    }
}