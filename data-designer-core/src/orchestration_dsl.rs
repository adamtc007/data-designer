use crate::models::{Expression, Value};
use crate::resource_sheets::*;
use crate::evaluator::FunctionLibrary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, bail};
use chrono::Utc;

/// Extended expression types for resource orchestration
/// These extend the base Expression enum for orchestration-specific operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrchestrationExpression {
    /// Base expression from the core system
    Base(Expression),

    /// DISCOVER_DEPENDENCIES FOR_PRODUCTS ["Product1", "Product2"]
    DiscoverDependencies {
        products: Vec<String>,
        client_profile: Option<Box<Expression>>,
    },

    /// INSTANTIATE_RESOURCE "ResourceType" WITH_DATA { key: value }
    InstantiateResource {
        resource_type: DomainType,
        resource_name: String,
        input_data: HashMap<String, Expression>,
        priority: Priority,
    },

    /// BUILD_MASTER_DICTIONARY FROM_RESOURCES ["Resource1", "Resource2"]
    BuildMasterDictionary {
        from_resources: Vec<String>,
        merge_strategy: MergeStrategy,
    },

    /// EXECUTE_RESOURCE_DSL "ResourceName"
    ExecuteResourceDSL {
        resource_name: String,
        timeout_minutes: Option<u32>,
        failure_strategy: FailureStrategy,
    },

    /// AWAIT_RESOURCES ["Resource1", "Resource2"] TO_BE "Complete"
    AwaitResources {
        resource_names: Vec<String>,
        target_status: ResourceStatus,
        timeout_minutes: Option<u32>,
        polling_interval_seconds: Option<u32>,
    },

    /// COORDINATE_PARALLEL ["Resource1", "Resource2"] WITH_SYNC_POINTS ["Point1"]
    CoordinateParallel {
        resource_names: Vec<String>,
        sync_points: Vec<String>,
        max_concurrent: Option<u32>,
    },

    /// HANDLE_FAILURE "ResourceName" WITH_STRATEGY "Retry"
    HandleFailure {
        resource_name: String,
        strategy: FailureHandlingStrategy,
        max_retries: Option<u32>,
    },

    /// ESCALATE_TO_HUMAN "ResourceName" WITH_REASON "Manual Review Required"
    EscalateToHuman {
        resource_name: String,
        reason: String,
        priority: Priority,
        required_roles: Vec<String>,
    },

    /// VALIDATE_ORCHESTRATION_STATE USING "ValidationRuleName"
    ValidateOrchestrationState {
        validation_rules: Vec<String>,
        fail_on_validation_error: bool,
    },

    /// DERIVE_GLOBAL_STATE FROM_RESOURCES ["Resource1", "Resource2"]
    DeriveGlobalState {
        from_resources: Vec<String>,
        state_expression: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    Override,      // Later values override earlier ones
    Combine,       // Combine/merge compatible values
    Validate,      // Ensure all sources agree on values
    FirstWins,     // First source wins for conflicts
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureHandlingStrategy {
    Retry,
    Skip,
    Abort,
    ManualReview,
    Fallback(String), // Fallback to alternative resource
}

/// Orchestration-specific function library
/// Extends the base FunctionLibrary with orchestration verbs
pub struct OrchestrationFunctionLibrary {
    pub base: FunctionLibrary,
    pub resource_registry: ResourceRegistry,
}

impl Default for OrchestrationFunctionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestrationFunctionLibrary {
    pub fn new() -> Self {
        Self {
            base: FunctionLibrary::new(),
            resource_registry: ResourceRegistry::new(),
        }
    }

    /// Execute orchestration function calls
    pub fn call_orchestration_function(
        &mut self,
        name: &str,
        args: &[Value],
        context: &mut ExecutionContext
    ) -> Result<Value> {
        match name.to_uppercase().as_str() {
            // Resource Discovery Functions
            "DISCOVER_DEPENDENCIES" => self.discover_dependencies(args, context),
            "ANALYZE_PRODUCT_REQUIREMENTS" => self.analyze_product_requirements(args, context),
            "GET_REGULATORY_REQUIREMENTS" => self.get_regulatory_requirements(args, context),

            // Resource Management Functions
            "INSTANTIATE_RESOURCE" => self.instantiate_resource(args, context),
            "GET_RESOURCE_STATUS" => self.get_resource_status(args, context),
            "LIST_ACTIVE_RESOURCES" => self.list_active_resources(args, context),
            "TERMINATE_RESOURCE" => self.terminate_resource(args, context),

            // Data Management Functions
            "BUILD_MASTER_DICTIONARY" => self.build_master_dictionary(args, context),
            "MERGE_RESOURCE_DATA" => self.merge_resource_data(args, context),
            "EXTRACT_SHARED_DATA" => self.extract_shared_data(args, context),

            // Execution Control Functions
            "EXECUTE_RESOURCE_DSL" => self.execute_resource_dsl(args, context),
            "AWAIT_RESOURCES" => self.await_resources(args, context),
            "CHECK_DEPENDENCIES" => self.check_dependencies(args, context),

            // Coordination Functions
            "COORDINATE_PARALLEL" => self.coordinate_parallel(args, context),
            "SYNCHRONIZE_AT_POINT" => self.synchronize_at_point(args, context),
            "SET_SYNC_BARRIER" => self.set_sync_barrier(args, context),

            // Error Handling Functions
            "HANDLE_FAILURE" => self.handle_failure(args, context),
            "RETRY_RESOURCE" => self.retry_resource(args, context),
            "ESCALATE_TO_HUMAN" => self.escalate_to_human(args, context),

            // State Management Functions
            "VALIDATE_ORCHESTRATION_STATE" => self.validate_orchestration_state(args, context),
            "DERIVE_GLOBAL_STATE" => self.derive_global_state(args, context),
            "GET_EXECUTION_METRICS" => self.get_execution_metrics(args, context),

            // Workflow Functions
            "TRANSITION_TO_PHASE" => self.transition_to_phase(args, context),
            "EVALUATE_COMPLETION_CRITERIA" => self.evaluate_completion_criteria(args, context),
            "CALCULATE_ESTIMATED_COMPLETION" => self.calculate_estimated_completion(args, context),

            _ => bail!("Unknown orchestration function '{}'", name),
        }
    }

    // ===== RESOURCE DISCOVERY FUNCTIONS =====

    fn discover_dependencies(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.is_empty() {
            bail!("DISCOVER_DEPENDENCIES requires at least 1 argument (products list)");
        }

        let products = match &args[0] {
            Value::List(items) => items.iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect::<Vec<_>>(),
            Value::String(single) => vec![single.clone()],
            _ => bail!("Products must be a list or string"),
        };

        // Discover required domain resources based on products
        let mut required_resources = Vec::new();

        for product in &products {
            if let Some(product_def) = self.resource_registry.get_product_definition(product) {
                // KYC is always required
                required_resources.push("KYC".to_string());

                // Add product-specific requirements
                match product_def.risk_category {
                    RiskLevel::High | RiskLevel::Prohibited => {
                        required_resources.push("RiskAssessment".to_string());
                        required_resources.push("ComplianceReview".to_string());
                    },
                    _ => {}
                }

                // Add based on product type
                if product.contains("Custody") {
                    required_resources.push("CustodySetup".to_string());
                }
                if product.contains("Trading") {
                    required_resources.push("TradingPermissions".to_string());
                }
            }
        }

        // Log discovery
        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "discover_dependencies".to_string(),
            message: format!("Discovered {} required resources for products: {:?}",
                           required_resources.len(), products),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::List(required_resources.into_iter().map(Value::String).collect()))
    }

    fn analyze_product_requirements(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Analyze specific requirements for products
        // Returns a structured analysis of what's needed
        Ok(Value::String("Product analysis complete".to_string()))
    }

    fn get_regulatory_requirements(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Get regulatory requirements for jurisdiction/products
        Ok(Value::List(vec![
            Value::String("AML_SCREENING".to_string()),
            Value::String("KYC_VERIFICATION".to_string()),
        ]))
    }

    // ===== RESOURCE MANAGEMENT FUNCTIONS =====

    fn instantiate_resource(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.len() < 2 {
            bail!("INSTANTIATE_RESOURCE requires at least 2 arguments (resource_type, resource_name)");
        }

        let resource_type = match &args[0] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource type must be a string"),
        };

        let resource_name = match &args[1] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource name must be a string"),
        };

        // Generate new resource ID
        let resource_id = format!("{}_{}", resource_type, uuid::Uuid::new_v4());

        // Create resource based on type
        let domain_type = match resource_type.as_str() {
            "KYC" => DomainType::KYC,
            "AccountSetup" => DomainType::AccountSetup,
            "RiskAssessment" => DomainType::RiskAssessment,
            "ComplianceReview" => DomainType::ComplianceReview,
            _ => bail!("Unknown resource type: {}", resource_type),
        };

        // Register the new resource
        let sub_resource = SubResourceReference {
            resource_id: resource_id.clone(),
            domain_type,
            status: ResourceStatus::Pending,
            dependencies: vec![],
            data_requirements: vec![],
            created_at: Utc::now(),
            completed_at: None,
        };

        self.resource_registry.register_resource(resource_name.clone(), sub_resource);

        // Log instantiation
        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "instantiate_resource".to_string(),
            message: format!("Instantiated {} resource '{}' with ID: {}",
                           resource_type, resource_name, resource_id),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::String(resource_id))
    }

    fn get_resource_status(&mut self, args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        if args.is_empty() {
            bail!("GET_RESOURCE_STATUS requires 1 argument (resource_name)");
        }

        let resource_name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource name must be a string"),
        };

        if let Some(resource) = self.resource_registry.get_resource(&resource_name) {
            let status_str = match resource.status {
                ResourceStatus::Pending => "Pending",
                ResourceStatus::Discovering => "Discovering",
                ResourceStatus::Instantiating => "Instantiating",
                ResourceStatus::Executing => "Executing",
                ResourceStatus::Waiting => "Waiting",
                ResourceStatus::Complete => "Complete",
                ResourceStatus::Review => "Review",
                ResourceStatus::Failed(_) => "Failed",
            };
            Ok(Value::String(status_str.to_string()))
        } else {
            bail!("Resource not found: {}", resource_name);
        }
    }

    fn list_active_resources(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        let active_resources = self.resource_registry.list_active_resources();
        Ok(Value::List(active_resources.into_iter().map(Value::String).collect()))
    }

    fn terminate_resource(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.is_empty() {
            bail!("TERMINATE_RESOURCE requires 1 argument (resource_name)");
        }

        let resource_name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource name must be a string"),
        };

        self.resource_registry.terminate_resource(&resource_name);

        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "terminate_resource".to_string(),
            message: format!("Terminated resource: {}", resource_name),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::Boolean(true))
    }

    // ===== DATA MANAGEMENT FUNCTIONS =====

    fn build_master_dictionary(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        // Combine data dictionaries from multiple resources
        let _combined_requirements: Vec<String> = Vec::new();
        let mut field_count = 0;

        if !args.is_empty() {
            if let Value::List(resource_names) = &args[0] {
                for resource_name in resource_names {
                    if let Value::String(_name) = resource_name {
                        // Get resource dictionary and merge
                        field_count += 10; // Simulated for now
                    }
                }
            }
        }

        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "build_master_dictionary".to_string(),
            message: format!("Built master dictionary with {} fields", field_count),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::Integer(field_count))
    }

    fn merge_resource_data(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Merge data from multiple resources using specified strategy
        Ok(Value::Boolean(true))
    }

    fn extract_shared_data(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Extract data that's shared across resources
        Ok(Value::String("Shared data extracted".to_string()))
    }

    // ===== EXECUTION CONTROL FUNCTIONS =====

    fn execute_resource_dsl(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.is_empty() {
            bail!("EXECUTE_RESOURCE_DSL requires 1 argument (resource_name)");
        }

        let resource_name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource name must be a string"),
        };

        // Update resource status to executing
        if let Some(resource) = self.resource_registry.get_resource_mut(&resource_name) {
            resource.status = ResourceStatus::Executing;
        }

        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "execute_resource_dsl".to_string(),
            message: format!("Started execution of resource: {}", resource_name),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::Boolean(true))
    }

    fn await_resources(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.len() < 2 {
            bail!("AWAIT_RESOURCES requires 2 arguments (resource_names, target_status)");
        }

        let resource_names = match &args[0] {
            Value::List(names) => names.iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => bail!("Resource names must be a list"),
        };

        let target_status = match &args[1] {
            Value::String(s) => s.clone(),
            _ => bail!("Target status must be a string"),
        };

        // Check if all resources have reached target status
        let mut all_ready = true;
        for resource_name in &resource_names {
            if let Some(resource) = self.resource_registry.get_resource(resource_name) {
                let current_status = match resource.status {
                    ResourceStatus::Complete => "Complete",
                    ResourceStatus::Failed(_) => "Failed",
                    _ => "InProgress",
                };
                if current_status != target_status {
                    all_ready = false;
                    break;
                }
            }
        }

        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "await_resources".to_string(),
            message: format!("Awaiting {} resources to reach status: {}",
                           resource_names.len(), target_status),
            level: LogLevel::Info,
            data: HashMap::new(),
        });

        Ok(Value::Boolean(all_ready))
    }

    fn check_dependencies(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Check if all dependencies are satisfied
        Ok(Value::Boolean(true))
    }

    // ===== COORDINATION FUNCTIONS =====

    fn coordinate_parallel(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Set up parallel execution coordination
        Ok(Value::Boolean(true))
    }

    fn synchronize_at_point(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Synchronize resources at a specific point
        Ok(Value::Boolean(true))
    }

    fn set_sync_barrier(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Set a synchronization barrier
        Ok(Value::Boolean(true))
    }

    // ===== ERROR HANDLING FUNCTIONS =====

    fn handle_failure(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Handle resource failure according to strategy
        Ok(Value::Boolean(true))
    }

    fn retry_resource(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Retry a failed resource
        Ok(Value::Boolean(true))
    }

    fn escalate_to_human(&mut self, args: &[Value], context: &mut ExecutionContext) -> Result<Value> {
        if args.len() < 2 {
            bail!("ESCALATE_TO_HUMAN requires 2 arguments (resource_name, reason)");
        }

        let resource_name = match &args[0] {
            Value::String(s) => s.clone(),
            _ => bail!("Resource name must be a string"),
        };

        let reason = match &args[1] {
            Value::String(s) => s.clone(),
            _ => bail!("Reason must be a string"),
        };

        // Update resource status to review
        if let Some(resource) = self.resource_registry.get_resource_mut(&resource_name) {
            resource.status = ResourceStatus::Review;
        }

        context.execution_log.push(ExecutionLogEntry {
            timestamp: Utc::now(),
            resource_id: "orchestrator".to_string(),
            step: "escalate_to_human".to_string(),
            message: format!("Escalated {} to human review: {}", resource_name, reason),
            level: LogLevel::Warning,
            data: HashMap::new(),
        });

        Ok(Value::Boolean(true))
    }

    // ===== STATE MANAGEMENT FUNCTIONS =====

    fn validate_orchestration_state(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Validate current orchestration state
        Ok(Value::Boolean(true))
    }

    fn derive_global_state(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Derive global state from resource states
        Ok(Value::String("GlobalState".to_string()))
    }

    fn get_execution_metrics(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Get execution metrics and performance data
        let active_count = self.resource_registry.list_active_resources().len();
        Ok(Value::Integer(active_count as i64))
    }

    // ===== WORKFLOW FUNCTIONS =====

    fn transition_to_phase(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Transition orchestration to next phase
        Ok(Value::Boolean(true))
    }

    fn evaluate_completion_criteria(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Evaluate if completion criteria are met
        Ok(Value::Boolean(true))
    }

    fn calculate_estimated_completion(&mut self, _args: &[Value], _context: &mut ExecutionContext) -> Result<Value> {
        // Calculate estimated completion time
        Ok(Value::Integer(30)) // 30 minutes
    }
}

/// Registry for managing active resources
#[derive(Debug, Clone)]
pub struct ResourceRegistry {
    pub active_resources: HashMap<String, SubResourceReference>,
    pub product_definitions: HashMap<String, ProductDefinition>,
    pub terminated_resources: Vec<String>,
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self {
            active_resources: HashMap::new(),
            product_definitions: HashMap::new(),
            terminated_resources: Vec::new(),
        }
    }

    pub fn register_resource(&mut self, name: String, resource: SubResourceReference) {
        self.active_resources.insert(name, resource);
    }

    pub fn get_resource(&self, name: &str) -> Option<&SubResourceReference> {
        self.active_resources.get(name)
    }

    pub fn get_resource_mut(&mut self, name: &str) -> Option<&mut SubResourceReference> {
        self.active_resources.get_mut(name)
    }

    pub fn list_active_resources(&self) -> Vec<String> {
        self.active_resources.keys().cloned().collect()
    }

    pub fn terminate_resource(&mut self, name: &str) {
        if self.active_resources.remove(name).is_some() {
            self.terminated_resources.push(name.to_string());
        }
    }

    pub fn get_product_definition(&self, product_name: &str) -> Option<&ProductDefinition> {
        self.product_definitions.get(product_name)
    }

    pub fn add_product_definition(&mut self, product: ProductDefinition) {
        self.product_definitions.insert(product.product_name.clone(), product);
    }
}

impl Value {
    /// Convert a Value to a string representation
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Regex(r) => r.clone(),
            Value::List(items) => format!("[{}]", items.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")),
        }
    }
}