//! HTTP Client for onboarding-ui
//! Adapted from grpc_client.rs - many methods unused but kept for compatibility

#![allow(dead_code)] // Legacy code from grpc_client.rs

use serde::{Deserialize, Serialize};
use crate::wasm_utils;

// Cross-platform error handling
#[cfg(target_arch = "wasm32")]
pub type Result<T> = std::result::Result<T, String>;

#[cfg(not(target_arch = "wasm32"))]
pub type Result<T> = anyhow::Result<T>;

// Helper function for error creation that works in both contexts
#[cfg(target_arch = "wasm32")]
fn make_error(msg: &str) -> String {
    msg.to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn make_error(msg: &str) -> anyhow::Error {
    anyhow::anyhow!("{}", msg)
}

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
    pub business_model: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub dsl_content: Option<String>,      // DSL content from dsl_metadata table
    pub dsl_metadata: Option<String>,     // Metadata JSON from dsl_metadata table
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCbuRequest {
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: Option<String>,
    pub legal_entity_name: Option<String>,
    pub business_model: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCbuResponse {
    pub success: bool,
    pub message: String,
    pub cbu: Option<CbuRecord>,
}

// Resource DSL types (parallel to CBU types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub id: i32,
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub dsl_content: Option<String>,   // DSL content from dsl_metadata table (dsl_domain='Resource')
    pub dsl_metadata: Option<String>,  // Metadata JSON from dsl_metadata table
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    pub status_filter: Option<String>,
    pub resource_type_filter: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    pub resources: Vec<ResourceRecord>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourceDslRequest {
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourceDslResponse {
    pub success: bool,
    pub resource: Option<ResourceRecord>,
    pub dsl_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceDslRequest {
    pub resource_id: String,
    pub dsl_content: String,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceDslResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResourceDslRequest {
    pub resource_id: String,
    pub dsl_script: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResourceDslResponse {
    pub success: bool,
    pub message: String,
    pub validation_errors: Vec<String>,
    pub execution_result: Option<String>, // JSON string with execution details
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
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "active".to_string()
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

// CBU Update types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCbuRequest {
    pub cbu_id: String,
    pub cbu_name: Option<String>,
    pub description: Option<String>,
    pub legal_entity_name: Option<String>,
    pub business_model: Option<String>,
    pub status: Option<String>,
    pub entities: Vec<CbuEntityAssociation>,
}

// CBU Get types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCbuRequest {
    pub cbu_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCbuResponse {
    pub success: bool,
    pub message: String,
    pub cbu: Option<CbuRecord>,
    pub entities: Vec<CbuEntityAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCbuResponse {
    pub success: bool,
    pub message: String,
    pub cbu: Option<CbuRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuEntityAssociation {
    pub entity_id: String,
    pub entity_name: String,
    pub role_in_cbu: String,
    pub entity_type: Option<String>,
    pub active_in_cbu: bool,
}

#[derive(Clone)]
// Unified HTTP client for both platforms
pub struct GrpcClient {
    base_url: String,
    client: reqwest::Client,
}

impl GrpcClient {
    pub fn new(grpc_endpoint: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url: grpc_endpoint.to_string(),
            client,
        }
    }

    /// Make unified HTTP call to gRPC server (both platforms)
    async fn grpc_call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        wasm_utils::console_log(&format!("🔍 [TRACE-START] gRPC call: service_method='{}', base_url='{}'", service_method, self.base_url));

        // Serialize request for logging
        let request_json = match serde_json::to_string_pretty(request) {
            Ok(json) => json,
            Err(e) => format!("[Failed to serialize: {}]", e),
        };
        wasm_utils::console_log(&format!("📤 [TRACE-REQUEST] {}", request_json));

        let result = self.try_http_call(service_method, request).await;

        match &result {
            Ok(_) => {
                wasm_utils::console_log(&format!("✅ [TRACE-SUCCESS] gRPC call completed: '{}'", service_method));
            }
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                let error_msg = e;
                #[cfg(not(target_arch = "wasm32"))]
                let error_msg = &e.to_string();
                wasm_utils::console_log(&format!("❌ [TRACE-ERROR] gRPC call failed: '{}' - Error: {}", service_method, error_msg));
            }
        }

        result
    }

    /// Make an HTTP call to gRPC server (both platforms)
    async fn try_http_call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        wasm_utils::console_log(&format!("🔗 [TRACE-HTTP] Trying HTTP call for service method: '{}'", service_method));
        let endpoint = match service_method {
            "financial_taxonomy.FinancialTaxonomyService/InstantiateResource" => "/api/instantiate",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteDsl" => "/api/execute-dsl",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteCbuDsl" => "/api/execute-cbu-dsl",
            "financial_taxonomy.FinancialTaxonomyService/ListCbus" => "/api/list-cbus",
            "financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions" => "/api/ai-suggestions",
            "financial_taxonomy.FinancialTaxonomyService/GetEntities" => "/api/entities",
            "financial_taxonomy.FinancialTaxonomyService/ListProducts" => "/api/list-products",
            "financial_taxonomy.FinancialTaxonomyService/UpdateCbu" => "/api/update-cbu",
            "financial_taxonomy.FinancialTaxonomyService/GetCbu" => "/api/get-cbu",
            "financial_taxonomy.FinancialTaxonomyService/CreateCbu" => "/api/create-cbu",
            "financial_taxonomy.FinancialTaxonomyService/DeleteCbu" => "/api/delete-cbu",
            // Product CRUD operations
            "financial_taxonomy.FinancialTaxonomyService/CreateProduct" => "/api/create-product",
            "financial_taxonomy.FinancialTaxonomyService/GetProduct" => "/api/get-product",
            "financial_taxonomy.FinancialTaxonomyService/UpdateProduct" => "/api/update-product",
            "financial_taxonomy.FinancialTaxonomyService/DeleteProduct" => "/api/delete-product",
            // Service CRUD operations
            "financial_taxonomy.FinancialTaxonomyService/CreateService" => "/api/create-service",
            "financial_taxonomy.FinancialTaxonomyService/GetService" => "/api/get-service",
            "financial_taxonomy.FinancialTaxonomyService/UpdateService" => "/api/update-service",
            "financial_taxonomy.FinancialTaxonomyService/DeleteService" => "/api/delete-service",
            "financial_taxonomy.FinancialTaxonomyService/ListServices" => "/api/list-services",
            // Resource CRUD operations
            "financial_taxonomy.FinancialTaxonomyService/CreateResource" => "/api/create-resource",
            "financial_taxonomy.FinancialTaxonomyService/GetResource" => "/api/get-resource",
            "financial_taxonomy.FinancialTaxonomyService/UpdateResource" => "/api/update-resource",
            "financial_taxonomy.FinancialTaxonomyService/DeleteResource" => "/api/delete-resource",
            "financial_taxonomy.FinancialTaxonomyService/ListResources" => "/api/list-resources",
            // Resource DSL operations
            "financial_taxonomy.FinancialTaxonomyService/GetResourceDsl" => "/api/get-resource-dsl",
            "financial_taxonomy.FinancialTaxonomyService/UpdateResourceDsl" => "/api/update-resource-dsl",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteResourceDsl" => "/api/execute-resource-dsl",
            // Onboarding operations - Mirror gRPC proto exactly
            "financial_taxonomy.FinancialTaxonomyService/CreateOnboardingRequest" => "/api/onboarding/CreateOnboardingRequest",
            "financial_taxonomy.FinancialTaxonomyService/GetOnboardingRequest" => "/api/onboarding/GetOnboardingRequest",
            "financial_taxonomy.FinancialTaxonomyService/ListOnboardingRequests" => "/api/onboarding/ListOnboardingRequests",
            "financial_taxonomy.FinancialTaxonomyService/UpdateOnboardingRequestStatus" => "/api/onboarding/UpdateOnboardingRequestStatus",
            "financial_taxonomy.FinancialTaxonomyService/CompileOnboardingWorkflow" => "/api/onboarding/CompileOnboardingWorkflow",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteOnboardingWorkflow" => "/api/onboarding/ExecuteOnboardingWorkflow",
            _ => {
                wasm_utils::console_log(&format!("❌ Unknown service method: '{}'", service_method));
                return Err(make_error(&format!("Unknown service method: {}", service_method)));
            }
        };

        wasm_utils::console_log(&format!("🎯 [TRACE-ENDPOINT] Matched endpoint: {}", endpoint));
        let url = format!("{}{}", self.base_url, endpoint);
        wasm_utils::console_log(&format!("🌐 [TRACE-URL] Making HTTP POST to: {}", url));

        let response = self.client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("HTTP request failed: {}", e);
                wasm_utils::console_log(&format!("💥 [TRACE-HTTP-ERROR] {}", error_msg));
                make_error(&error_msg)
            })?;

        let status = response.status();
        wasm_utils::console_log(&format!("📊 [TRACE-STATUS] HTTP response status: {}", status));

        if !status.is_success() {
            let error_msg = format!("HTTP request failed with status: {}", status);
            wasm_utils::console_log(&format!("❌ [TRACE-STATUS-ERROR] {}", error_msg));
            return Err(make_error(&error_msg));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to read response text: {}", e);
                wasm_utils::console_log(&format!("💥 [TRACE-PARSE-ERROR] {}", error_msg));
                make_error(&error_msg)
            })?;

        wasm_utils::console_log(&format!("📥 [TRACE-RESPONSE] Raw response body: {}", response_text));

        let response_body: R = serde_json::from_str(&response_text)
            .map_err(|e| {
                let error_msg = format!("Failed to parse JSON response: {} - Response was: {}", e, response_text);
                wasm_utils::console_log(&format!("💥 [TRACE-JSON-ERROR] {}", error_msg));
                make_error(&error_msg)
            })?;

        wasm_utils::console_log("🎉 [TRACE-HTTP-SUCCESS] HTTP call completed successfully");
        Ok(response_body)
    }


    // Generic GET request method for custom endpoints
    pub async fn get_request<R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        wasm_utils::console_log(&format!("Making GET request to: {}", url));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| make_error(&format!("HTTP GET request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(make_error(&format!("HTTP request failed with status: {}", response.status())));
        }

        let response_body = response
            .json::<R>()
            .await
            .map_err(|e| make_error(&format!("Failed to parse response: {}", e)))?;

        Ok(response_body)
    }

    // Generic POST request method for custom endpoints
    pub async fn post_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        request: &T,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        wasm_utils::console_log(&format!("Making POST request to: {}", url));

        let response = self.client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| make_error(&format!("HTTP POST request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(make_error(&format!("HTTP request failed with status: {}", response.status())));
        }

        let response_body = response
            .json::<R>()
            .await
            .map_err(|e| make_error(&format!("Failed to parse response: {}", e)))?;

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

    pub async fn update_cbu(
        &self,
        request: UpdateCbuRequest,
    ) -> Result<UpdateCbuResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/UpdateCbu", &request)
            .await
    }

    pub async fn create_cbu(
        &self,
        request: CreateCbuRequest,
    ) -> Result<CreateCbuResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/CreateCbu", &request)
            .await
    }

    pub async fn get_cbu(
        &self,
        request: GetCbuRequest,
    ) -> Result<GetCbuResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/GetCbu", &request)
            .await
    }

    // ============================================
    // Resource DSL Methods
    // ============================================

    pub async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> Result<ListResourcesResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ListResources", &request)
            .await
    }

    pub async fn get_resource_dsl(
        &self,
        request: GetResourceDslRequest,
    ) -> Result<GetResourceDslResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/GetResourceDsl", &request)
            .await
    }

    pub async fn update_resource_dsl(
        &self,
        request: UpdateResourceDslRequest,
    ) -> Result<UpdateResourceDslResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/UpdateResourceDsl", &request)
            .await
    }

    pub async fn execute_resource_dsl(
        &self,
        request: ExecuteResourceDslRequest,
    ) -> Result<ExecuteResourceDslResponse> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ExecuteResourceDsl", &request)
            .await
    }

    // ============================================
    // Onboarding Methods - Route through gRPC API
    // ============================================

    pub async fn create_onboarding_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: T,
    ) -> Result<R> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/CreateOnboardingRequest", &request)
            .await
    }

    pub async fn list_onboarding_requests<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: T,
    ) -> Result<R> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ListOnboardingRequests", &request)
            .await
    }

    pub async fn compile_onboarding_workflow<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: T,
    ) -> Result<R> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/CompileOnboardingWorkflow", &request)
            .await
    }

    pub async fn execute_onboarding_workflow<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: T,
    ) -> Result<R> {
        self.grpc_call("financial_taxonomy.FinancialTaxonomyService/ExecuteOnboardingWorkflow", &request)
            .await
    }

    pub async fn get_onboarding_db_records<R: for<'de> Deserialize<'de>>(
        &self,
        onboarding_id: String,
    ) -> Result<R> {
        let endpoint = format!("/api/onboarding/requests/{}/db-records", onboarding_id);
        self.get_request(&endpoint).await
    }
}

// Helper function to create a UUID (cross-platform)
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            #[cfg(target_arch = "wasm32")]
            {
                // Simple UUID generation for WASM
                let timestamp = crate::wasm_utils::now_timestamp();
                write!(f, "{:x}-{:x}-{:x}-{:x}",
                       timestamp & 0xFFFFFFFF,
                       (timestamp >> 32) & 0xFFFF,
                       0x4000 | ((timestamp >> 48) & 0x0FFF), // Version 4
                       0x8000 | ((timestamp >> 60) & 0x3FFF)  // Variant bits
                )
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Use proper UUID for native
                let uuid_val = uuid::Uuid::new_v4();
                write!(f, "{}", uuid_val)
            }
        }
    }
}

// Helper for getting current time (cross-platform)
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
            #[cfg(target_arch = "wasm32")]
            {
                crate::wasm_utils::now_iso_string()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                chrono::Utc::now().to_rfc3339()
            }
        }
    }
}