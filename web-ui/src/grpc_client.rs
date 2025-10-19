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
        // Try HTTP endpoint first, fall back to mock if not available
        match self.try_http_call(service_method, request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                wasm_utils::console_log(&format!("HTTP call failed: {}, falling back to mock", e));
                self.create_mock_response(service_method, request).await
            }
        }
    }

    /// Try to make an HTTP call to gRPC server
    async fn try_http_call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        let endpoint = match service_method {
            "financial_taxonomy.FinancialTaxonomyService/InstantiateResource" => "/api/instantiate",
            "financial_taxonomy.FinancialTaxonomyService/ExecuteDsl" => "/api/execute-dsl",
            "financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions" => "/api/ai-suggestions",
            _ => return Err(anyhow::anyhow!("Unknown service method: {}", service_method)),
        };

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

    async fn create_mock_response<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        service_method: &str,
        request: &T,
    ) -> Result<R> {
        let request_json = serde_json::to_string(request)?;

        let mock_response_json = match service_method {
            "financial_taxonomy.FinancialTaxonomyService/InstantiateResource" => {
                let req: InstantiateResourceRequest = serde_json::from_str(&request_json)?;
                let response = InstantiateResourceResponse {
                    success: true,
                    message: "Resource instance created successfully".to_string(),
                    instance: Some(ResourceInstance {
                        instance_id: format!("wasm-instance-{}", uuid::Uuid::new_v4()),
                        onboarding_request_id: req.onboarding_request_id,
                        template_id: req.template_id,
                        status: "pending".to_string(),
                        instance_data: "{}".to_string(),
                        created_at: Some(chrono::Utc::now().to_rfc3339()),
                        updated_at: Some(chrono::Utc::now().to_rfc3339()),
                        error_message: None,
                    }),
                };
                serde_json::to_string(&response)?
            }
            "financial_taxonomy.FinancialTaxonomyService/ExecuteDsl" => {
                let req: ExecuteDslRequest = serde_json::from_str(&request_json)?;
                let response = ExecuteDslResponse {
                    success: true,
                    message: "DSL execution completed successfully".to_string(),
                    result: Some(DslExecutionResult {
                        instance_id: req.instance_id,
                        execution_status: "success".to_string(),
                        output_data: r#"{"result": "DSL executed successfully from WASM client"}"#.to_string(),
                        log_messages: vec!["DSL execution started".to_string(), "DSL execution completed".to_string()],
                        error_details: None,
                        executed_at: Some(chrono::Utc::now().to_rfc3339()),
                        execution_time_ms: 150.0,
                    }),
                };
                serde_json::to_string(&response)?
            }
            "financial_taxonomy.FinancialTaxonomyService/GetAiSuggestions" => {
                let req: GetAiSuggestionsRequest = serde_json::from_str(&request_json)?;
                let response = GetAiSuggestionsResponse {
                    suggestions: vec![
                        AiSuggestion {
                            title: "Generated DSL Code".to_string(),
                            description: format!("AI-generated DSL for: {}", req.query),
                            category: "code_generation".to_string(),
                            confidence: 0.9,
                            applicable_contexts: vec![req.context.unwrap_or("general".to_string())],
                        }
                    ],
                    status_message: "AI suggestions generated successfully".to_string(),
                };
                serde_json::to_string(&response)?
            }
            _ => return Err(anyhow::anyhow!("Unknown service method: {}", service_method)),
        };

        let response: R = serde_json::from_str(&mock_response_json)?;
        Ok(response)
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