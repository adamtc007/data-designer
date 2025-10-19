use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::Server;
use uuid::Uuid;

/// Mock gRPC services for testing
pub struct MockGrpcServices {
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
    pub port: u16,
    pub service_mocks: HashMap<String, ServiceMock>,
}

impl MockGrpcServices {
    /// Create new mock gRPC services
    pub async fn new() -> Result<Self> {
        let port = Self::find_available_port().await?;

        Ok(Self {
            server_handle: None,
            port,
            service_mocks: HashMap::new(),
        })
    }

    /// Start the mock gRPC server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port).parse()?;

        // Create a mock financial taxonomy service
        let mock_service = MockFinancialTaxonomyService::new();

        let server_future = Server::builder()
            .add_service(mock_service.into_service())
            .serve(addr);

        let handle = tokio::spawn(async move {
            if let Err(e) = server_future.await {
                eprintln!("Mock gRPC server error: {}", e);
            }
        });

        self.server_handle = Some(handle);

        // Wait a bit for server to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        tracing::info!("Mock gRPC server started on port {}", self.port);
        Ok(())
    }

    /// Stop the mock gRPC server
    pub async fn stop(&self) -> Result<()> {
        if let Some(handle) = &self.server_handle {
            handle.abort();
        }
        tracing::info!("Mock gRPC server stopped");
        Ok(())
    }

    /// Get the server endpoint URL
    pub fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Add a service mock
    pub fn add_service_mock(&mut self, service_name: String, mock: ServiceMock) {
        self.service_mocks.insert(service_name, mock);
    }

    /// Configure method response
    pub fn expect_call(&mut self, service: &str, method: &str) -> &mut MethodExpectation {
        let service_mock = self.service_mocks.entry(service.to_string())
            .or_insert_with(ServiceMock::new);

        service_mock.expect_call(method)
    }

    /// Find an available port for the mock server
    async fn find_available_port() -> Result<u16> {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        Ok(port)
    }
}

/// Mock for a specific gRPC service
pub struct ServiceMock {
    pub method_expectations: HashMap<String, MethodExpectation>,
}

impl ServiceMock {
    pub fn new() -> Self {
        Self {
            method_expectations: HashMap::new(),
        }
    }

    pub fn expect_call(&mut self, method: &str) -> &mut MethodExpectation {
        self.method_expectations.entry(method.to_string())
            .or_insert_with(MethodExpectation::new)
    }
}

/// Expectation for a specific method call
pub struct MethodExpectation {
    pub responses: Vec<MockResponse>,
    pub delays: Vec<Duration>,
    pub failures: Vec<MockError>,
    pub call_count: usize,
}

impl MethodExpectation {
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
            delays: Vec::new(),
            failures: Vec::new(),
            call_count: 0,
        }
    }

    pub fn and_return(&mut self, response: MockResponse) -> &mut Self {
        self.responses.push(response);
        self
    }

    pub fn with_delay(&mut self, delay: Duration) -> &mut Self {
        self.delays.push(delay);
        self
    }

    pub fn and_fail(&mut self, error: MockError) -> &mut Self {
        self.failures.push(error);
        self
    }

    pub fn times(&mut self, count: usize) -> &mut Self {
        // Repeat the last response/delay/failure for the specified number of times
        if let Some(last_response) = self.responses.last() {
            for _ in 1..count {
                self.responses.push(last_response.clone());
            }
        }
        self
    }
}

/// Mock response for gRPC methods
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub data: serde_json::Value,
    pub status: tonic::Code,
}

impl MockResponse {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            data,
            status: tonic::Code::Ok,
        }
    }

    pub fn error(status: tonic::Code, message: &str) -> Self {
        Self {
            data: serde_json::json!({ "error": message }),
            status,
        }
    }
}

/// Mock error for gRPC methods
#[derive(Debug, Clone)]
pub struct MockError {
    pub code: tonic::Code,
    pub message: String,
}

impl MockError {
    pub fn new(code: tonic::Code, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self::new(tonic::Code::Internal, message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(tonic::Code::NotFound, message)
    }

    pub fn unavailable(message: &str) -> Self {
        Self::new(tonic::Code::Unavailable, message)
    }
}

/// Mock implementation of the Financial Taxonomy Service
pub struct MockFinancialTaxonomyService {
    id: String,
}

impl MockFinancialTaxonomyService {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }

    pub fn into_service(self) -> impl tonic::server::NamedService {
        // This would be the actual gRPC service implementation
        // For now, we'll create a placeholder
        MockGrpcServiceImpl { inner: self }
    }
}

/// Placeholder for the actual gRPC service implementation
pub struct MockGrpcServiceImpl {
    inner: MockFinancialTaxonomyService,
}

impl tonic::server::NamedService for MockGrpcServiceImpl {
    const NAME: &'static str = "financial_taxonomy.FinancialTaxonomyService";
}

/// Client for testing gRPC calls
pub struct TestGrpcClient {
    endpoint: String,
    client: Option<tonic::transport::Channel>,
}

impl TestGrpcClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let channel = tonic::transport::Channel::from_shared(self.endpoint.clone())?
            .connect()
            .await?;

        self.client = Some(channel);
        Ok(())
    }

    pub async fn call_method(&self, method: &str, request: serde_json::Value) -> Result<serde_json::Value> {
        // This would make actual gRPC calls
        // For now, we'll simulate the call
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(serde_json::json!({
            "method": method,
            "request": request,
            "response": "mock_response"
        }))
    }

    pub async fn health_check(&self) -> Result<bool> {
        // Simulate health check
        Ok(true)
    }
}

/// Helper for creating test scenarios
pub struct GrpcTestScenarios;

impl GrpcTestScenarios {
    /// Create a scenario where all services are healthy
    pub fn healthy_services() -> HashMap<String, ServiceMock> {
        let mut services = HashMap::new();

        let mut financial_service = ServiceMock::new();
        financial_service.expect_call("GetProducts")
            .and_return(MockResponse::success(serde_json::json!({
                "products": [
                    {"id": "1", "name": "Savings Account"},
                    {"id": "2", "name": "Checking Account"}
                ]
            })));

        services.insert("FinancialTaxonomyService".to_string(), financial_service);
        services
    }

    /// Create a scenario where services are slow
    pub fn slow_services() -> HashMap<String, ServiceMock> {
        let mut services = HashMap::new();

        let mut financial_service = ServiceMock::new();
        financial_service.expect_call("GetProducts")
            .with_delay(Duration::from_millis(2000))
            .and_return(MockResponse::success(serde_json::json!({
                "products": []
            })));

        services.insert("FinancialTaxonomyService".to_string(), financial_service);
        services
    }

    /// Create a scenario where services fail
    pub fn failing_services() -> HashMap<String, ServiceMock> {
        let mut services = HashMap::new();

        let mut financial_service = ServiceMock::new();
        financial_service.expect_call("GetProducts")
            .and_fail(MockError::internal_error("Database connection failed"));

        services.insert("FinancialTaxonomyService".to_string(), financial_service);
        services
    }
}