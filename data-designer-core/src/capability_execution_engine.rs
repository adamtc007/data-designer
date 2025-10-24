use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::models::Value;
use anyhow::{Result, anyhow};

/// Core trait that all executable capabilities must implement
/// This is the "contract" for capability implementations
#[async_trait]
pub trait Capability: Send + Sync {
    /// Execute the capability with the provided inputs
    /// Returns the output value if successful, or an error
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>>;

    /// Get the capability name for registration
    fn name(&self) -> &str;

    /// Get the required attributes for this capability
    fn required_attributes(&self) -> &[&str];

    /// Get the optional attributes for this capability
    fn optional_attributes(&self) -> &[&str] {
        &[]
    }

    /// Get the expected output attribute name (if any)
    fn output_attribute(&self) -> Option<&str> {
        None
    }

    /// Validate inputs before execution
    fn validate_inputs(&self, inputs: &HashMap<String, Value>) -> Result<()> {
        // Check required attributes
        for attr in self.required_attributes() {
            if !inputs.contains_key(*attr) {
                return Err(anyhow!("Missing required attribute: {}", attr));
            }
        }
        Ok(())
    }
}

/// Configuration for capability dependency injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    pub name: String,
    pub implementation: String,
    pub config: HashMap<String, JsonValue>,
}

/// Registry that maps capability names to their implementations
pub struct CapabilityRegistry {
    capabilities: HashMap<String, Arc<dyn Capability>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Register a capability implementation
    pub fn register<T: Capability + 'static>(&mut self, capability: T) {
        let name = capability.name().to_string();
        self.capabilities.insert(name, Arc::new(capability));
    }

    /// Get a capability by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.capabilities.get(name).cloned()
    }

    /// List all registered capabilities
    pub fn list_capabilities(&self) -> Vec<String> {
        self.capabilities.keys().cloned().collect()
    }

    /// Execute a capability by name
    pub async fn execute_capability(
        &self,
        name: &str,
        inputs: &HashMap<String, Value>
    ) -> Result<Option<Value>> {
        let capability = self.get(name)
            .ok_or_else(|| anyhow!("Capability not found: {}", name))?;

        // Validate inputs
        capability.validate_inputs(inputs)?;

        // Execute the capability
        capability.execute(inputs).await
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution context for capabilities
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub instance_id: String,
    pub workflow_id: String,
    pub client_id: Option<String>,
    pub execution_metadata: HashMap<String, JsonValue>,
}

impl ExecutionContext {
    pub fn new(instance_id: String, workflow_id: String) -> Self {
        Self {
            instance_id,
            workflow_id,
            client_id: None,
            execution_metadata: HashMap::new(),
        }
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.execution_metadata.insert(key, value);
        self
    }
}

/// Result of capability execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExecutionResult {
    pub capability_name: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub output: Option<JsonValue>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Status of capability execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Pending => write!(f, "pending"),
            ExecutionStatus::Running => write!(f, "running"),
            ExecutionStatus::Completed => write!(f, "completed"),
            ExecutionStatus::Failed => write!(f, "failed"),
            ExecutionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Capability execution engine that manages the lifecycle of capability execution
pub struct CapabilityExecutionEngine {
    registry: CapabilityRegistry,
    execution_history: HashMap<String, CapabilityExecutionResult>,
}

impl CapabilityExecutionEngine {
    pub fn new() -> Self {
        Self {
            registry: CapabilityRegistry::new(),
            execution_history: HashMap::new(),
        }
    }

    /// Initialize the engine with default capabilities
    pub fn with_default_capabilities(mut self) -> Self {
        // Register built-in capabilities
        self.registry.register(AccountSetupCapability::new());
        self.registry.register(KycVerificationCapability::new());
        self.registry.register(CustodyOnboardingCapability::new());
        self.registry.register(TradeFeedSetupCapability::new());
        self.registry.register(ReportingConfigCapability::new());
        self.registry.register(ComplianceSetupCapability::new());
        self.registry.register(CashManagementCapability::new());
        self.registry.register(SetupValidationCapability::new());
        self.registry.register(ServiceActivationCapability::new());
        self.registry.register(HealthCheckCapability::new());

        self
    }

    /// Execute a capability with full lifecycle management
    pub async fn execute_capability_with_context(
        &mut self,
        capability_name: &str,
        inputs: &HashMap<String, Value>,
        _context: &ExecutionContext,
    ) -> Result<CapabilityExecutionResult> {
        let execution_id = uuid::Uuid::new_v4().as_string();
        let started_at = chrono::Utc::now();

        // Create initial result
        let mut result = CapabilityExecutionResult {
            capability_name: capability_name.to_string(),
            execution_id: execution_id.clone(),
            status: ExecutionStatus::Running,
            output: None,
            error_message: None,
            execution_time_ms: 0,
            started_at,
            completed_at: None,
        };

        // Store in history as running
        self.execution_history.insert(execution_id.clone(), result.clone());

        // Execute the capability
        let start_time = std::time::Instant::now();
        match self.registry.execute_capability(capability_name, inputs).await {
            Ok(output) => {
                let completed_at = chrono::Utc::now();
                let execution_time = start_time.elapsed().as_millis() as u64;

                result.status = ExecutionStatus::Completed;
                result.output = output.and_then(|v| serde_json::to_value(v).ok());
                result.execution_time_ms = execution_time;
                result.completed_at = Some(completed_at);
            }
            Err(e) => {
                let completed_at = chrono::Utc::now();
                let execution_time = start_time.elapsed().as_millis() as u64;

                result.status = ExecutionStatus::Failed;
                result.error_message = Some(e.to_string());
                result.execution_time_ms = execution_time;
                result.completed_at = Some(completed_at);
            }
        }

        // Update history
        self.execution_history.insert(execution_id, result.clone());

        Ok(result)
    }

    /// Get execution result by ID
    pub fn get_execution_result(&self, execution_id: &str) -> Option<&CapabilityExecutionResult> {
        self.execution_history.get(execution_id)
    }

    /// List all capabilities in the registry
    pub fn list_capabilities(&self) -> Vec<String> {
        self.registry.list_capabilities()
    }
}

impl Default for CapabilityExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================================================
// BUILT-IN CAPABILITY IMPLEMENTATIONS
// ========================================================================

/// Account Setup capability implementation
pub struct AccountSetupCapability;

impl Default for AccountSetupCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountSetupCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for AccountSetupCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        // Extract required inputs
        let client_id = inputs.get("client_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("client_id must be a string"))?;

        let cbu_id = inputs.get("cbu_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("cbu_id must be a string"))?;

        let account_type = inputs.get("account_type")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_type must be a string"))?;

        // Simulate account setup process
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Generate account ID
        let account_id = format!("ACC-{}-{}", cbu_id.to_uppercase(),
            chrono::Utc::now().format("%Y%m%d%H%M%S"));

        // Return account setup result
        Ok(Some(Value::String(format!(
            "Account {} successfully created for client {} with type {}",
            account_id, client_id, account_type
        ))))
    }

    fn name(&self) -> &str {
        "CAP-ACCOUNT-SETUP"
    }

    fn required_attributes(&self) -> &[&str] {
        &["client_id", "cbu_id", "account_type", "jurisdiction", "base_currency"]
    }

    fn output_attribute(&self) -> Option<&str> {
        Some("account_id")
    }
}

/// KYC Verification capability implementation
pub struct KycVerificationCapability;

impl Default for KycVerificationCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl KycVerificationCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for KycVerificationCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let client_id = inputs.get("client_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("client_id must be a string"))?;

        let entity_type = inputs.get("entity_type")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("entity_type must be a string"))?;

        // Simulate KYC verification process
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // KYC verification logic
        let verification_score = 0.95; // High confidence score
        let status = if verification_score > 0.8 { "approved" } else { "rejected" };

        Ok(Some(Value::String(format!(
            "KYC verification for {} ({}) completed with status: {} (score: {})",
            client_id, entity_type, status, verification_score
        ))))
    }

    fn name(&self) -> &str {
        "CAP-KYC-VERIFICATION"
    }

    fn required_attributes(&self) -> &[&str] {
        &["client_id", "entity_type", "jurisdiction", "risk_profile"]
    }
}

/// Custody Onboarding capability implementation
pub struct CustodyOnboardingCapability;

impl Default for CustodyOnboardingCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl CustodyOnboardingCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for CustodyOnboardingCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate custody setup
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        Ok(Some(Value::String(format!(
            "Custody services initialized for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-ONBOARD-CUSTODY"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "custody_products", "segregation_model", "reporting_frequency"]
    }
}

/// Trade Feed Setup capability implementation
pub struct TradeFeedSetupCapability;

impl Default for TradeFeedSetupCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeFeedSetupCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for TradeFeedSetupCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate trade feed configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        Ok(Some(Value::String(format!(
            "Trade feeds configured for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-TRADE-FEED-SETUP"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "settlement_instructions", "trade_feeds", "counterparties"]
    }
}

/// Reporting Configuration capability implementation
pub struct ReportingConfigCapability;

impl Default for ReportingConfigCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportingConfigCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for ReportingConfigCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate reporting setup
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        Ok(Some(Value::String(format!(
            "Reporting configuration completed for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-REPORTING-CONFIG"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "report_types", "delivery_method", "frequency", "recipients"]
    }
}

/// Compliance Setup capability implementation
pub struct ComplianceSetupCapability;

impl Default for ComplianceSetupCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceSetupCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for ComplianceSetupCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate compliance configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        Ok(Some(Value::String(format!(
            "Compliance monitoring configured for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-COMPLIANCE-SETUP"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "investment_guidelines", "risk_limits", "monitoring_frequency"]
    }
}

/// Cash Management capability implementation
pub struct CashManagementCapability;

impl Default for CashManagementCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl CashManagementCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for CashManagementCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate cash management setup
        tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;

        Ok(Some(Value::String(format!(
            "Cash management configured for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-CASH-MANAGEMENT"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "sweep_config", "cash_targets", "yield_preferences"]
    }
}

/// Setup Validation capability implementation
pub struct SetupValidationCapability;

impl Default for SetupValidationCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupValidationCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for SetupValidationCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate validation process
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(Some(Value::String(format!(
            "Setup validation completed successfully for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-VALIDATE-SETUP"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "validation_checklist", "dependencies"]
    }
}

/// Service Activation capability implementation
pub struct ServiceActivationCapability;

impl Default for ServiceActivationCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceActivationCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for ServiceActivationCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate service activation
        tokio::time::sleep(tokio::time::Duration::from_millis(750)).await;

        Ok(Some(Value::String(format!(
            "All services activated for account: {}", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-ACTIVATE-SERVICES"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "services_to_activate", "activation_schedule"]
    }
}

/// Health Check capability implementation
pub struct HealthCheckCapability;

impl Default for HealthCheckCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCheckCapability {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Capability for HealthCheckCapability {
    async fn execute(&self, inputs: &HashMap<String, Value>) -> Result<Option<Value>> {
        let account_id = inputs.get("account_id")
            .and_then(|v| if let Value::String(s) = v { Some(s) } else { None })
            .ok_or_else(|| anyhow!("account_id must be a string"))?;

        // Simulate health check
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Health check always passes for demo
        Ok(Some(Value::String(format!(
            "Health check passed for account: {} - All systems operational", account_id
        ))))
    }

    fn name(&self) -> &str {
        "CAP-HEALTH-CHECK"
    }

    fn required_attributes(&self) -> &[&str] {
        &["account_id", "check_types", "threshold_config"]
    }
}

/// UUID generation module (simplified for demo)
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }

        pub fn as_string(&self) -> String {
            format!("cap-exec-{}", chrono::Utc::now().timestamp())
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_string())
        }
    }
}