use crate::models::{Expression, Value};
use crate::db::{
    DbPool, DbOperations,
    ResourceCapability, CapabilityExecution,
    CapabilityExecutionResult, ExecutionContext
};
use std::collections::HashMap;
use serde_json;
use chrono::Utc;

/// Capability Execution Engine - Orchestrates resource capability execution
pub struct CapabilityEngine {
    db_pool: DbPool,
    capability_registry: HashMap<String, CapabilityImplementation>,
}

/// Capability implementation function type
pub type CapabilityImplementationFn = fn(&CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError>;

/// Registry entry for capability implementations
pub struct CapabilityImplementation {
    pub function: CapabilityImplementationFn,
    pub timeout_seconds: u64,
    pub retry_strategy: RetryStrategy,
}

/// Retry strategy configuration
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    None,
    Fixed { attempts: u32, delay_ms: u64 },
    Exponential { attempts: u32, base_delay_ms: u64, max_delay_ms: u64 },
    Linear { attempts: u32, delay_increment_ms: u64 },
}

/// Capability execution errors
#[derive(Debug, Clone)]
pub enum CapabilityError {
    NotFound(String),
    ValidationFailed(String),
    ExecutionFailed(String),
    Timeout(String),
    AttributesMissing(Vec<String>),
    InvalidInput(String),
    SystemError(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::NotFound(msg) => write!(f, "Capability not found: {}", msg),
            CapabilityError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            CapabilityError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CapabilityError::Timeout(msg) => write!(f, "Execution timeout: {}", msg),
            CapabilityError::AttributesMissing(attrs) => write!(f, "Missing required attributes: {:?}", attrs),
            CapabilityError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CapabilityError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for CapabilityError {}

impl CapabilityEngine {
    /// Create new capability engine with database connection
    pub async fn new(db_pool: DbPool) -> Result<Self, String> {
        let mut engine = Self {
            db_pool,
            capability_registry: HashMap::new(),
        };

        // Register built-in fund accounting capabilities
        engine.register_fund_accounting_capabilities()?;

        Ok(engine)
    }

    /// Register a capability implementation
    pub fn register_capability(
        &mut self,
        capability_id: String,
        implementation: CapabilityImplementation,
    ) {
        self.capability_registry.insert(capability_id, implementation);
    }

    /// Execute a DSL expression that contains capability calls
    pub async fn execute_expression(&self, expr: &Expression, context: &ExecutionContext) -> Result<Value, CapabilityError> {
        match expr {
            Expression::ConfigureSystem { capability_name, arguments } => {
                self.execute_configure_system(capability_name, arguments, context).await
            },
            Expression::Activate { target, arguments } => {
                self.execute_activate(target.as_deref(), arguments, context).await
            },
            Expression::RunHealthCheck { check_type, arguments } => {
                self.execute_health_check(check_type, arguments, context).await
            },
            Expression::SetStatus { status, target } => {
                self.execute_set_status(status, target.as_deref(), context).await
            },
            Expression::Workflow { name, steps: _ } => {
                // Simplified workflow execution for now to avoid recursion complexity
                println!("🔄 Executing WORKFLOW: {}", name);
                Ok(Value::String(format!("Workflow '{}' executed successfully", name)))
            },
            _ => {
                // For non-capability expressions, we'd delegate to the regular evaluator
                // For now, return a placeholder
                Ok(Value::String("Non-capability expression".to_string()))
            }
        }
    }

    /// Execute CONFIGURE_SYSTEM capability
    async fn execute_configure_system(
        &self,
        capability_name: &str,
        arguments: &[Expression],
        context: &ExecutionContext,
    ) -> Result<Value, CapabilityError> {
        println!("🔧 Executing CONFIGURE_SYSTEM: {}", capability_name);

        // Get capability definition from database
        let capability = DbOperations::get_resource_capability_by_id(capability_name)
            .await
            .map_err(|e| CapabilityError::SystemError(e))?
            .ok_or_else(|| CapabilityError::NotFound(capability_name.to_string()))?;

        // Validate required attributes
        let input_attributes = self.evaluate_arguments(arguments).await?;
        self.validate_capability_execution(&capability, &input_attributes)?;

        // Create execution request
        let execution = CapabilityExecution {
            capability_id: capability_name.to_string(),
            input_attributes,
            execution_context: context.clone(),
        };

        // Execute the capability
        let result = self.execute_capability_with_retry(&execution).await?;

        // Store execution result if needed
        if let Some(workflow_id) = &context.workflow_id {
            self.store_execution_result(workflow_id, &result).await?;
        }

        // Return output as Value
        Ok(Value::String(serde_json::to_string(&result.output_attributes).unwrap_or_default()))
    }

    /// Execute ACTIVATE capability
    async fn execute_activate(
        &self,
        _target: Option<&str>,
        arguments: &[Expression],
        context: &ExecutionContext,
    ) -> Result<Value, CapabilityError> {
        println!("🚀 Executing ACTIVATE");

        // For activate, we look for an "activate" capability on the current resource template
        let capability_id = "activate";

        let capability = DbOperations::get_resource_capability_by_id(capability_id)
            .await
            .map_err(|e| CapabilityError::SystemError(e))?
            .ok_or_else(|| CapabilityError::NotFound(capability_id.to_string()))?;

        let input_attributes = self.evaluate_arguments(arguments).await?;
        self.validate_capability_execution(&capability, &input_attributes)?;

        let execution = CapabilityExecution {
            capability_id: capability_id.to_string(),
            input_attributes,
            execution_context: context.clone(),
        };

        let result = self.execute_capability_with_retry(&execution).await?;

        if let Some(workflow_id) = &context.workflow_id {
            self.store_execution_result(workflow_id, &result).await?;
        }

        Ok(Value::String(serde_json::to_string(&result.output_attributes).unwrap_or_default()))
    }

    /// Execute RUN_HEALTH_CHECK capability
    async fn execute_health_check(
        &self,
        check_type: &str,
        arguments: &[Expression],
        context: &ExecutionContext,
    ) -> Result<Value, CapabilityError> {
        println!("🏥 Executing RUN_HEALTH_CHECK: {}", check_type);

        // Map check_type to capability_id
        let capability_id = match check_type {
            "health_check" => "health_check",
            "connectivity" => "health_check",
            _ => "health_check", // Default fallback
        };

        let capability = DbOperations::get_resource_capability_by_id(capability_id)
            .await
            .map_err(|e| CapabilityError::SystemError(e))?
            .ok_or_else(|| CapabilityError::NotFound(capability_id.to_string()))?;

        let input_attributes = self.evaluate_arguments(arguments).await?;
        self.validate_capability_execution(&capability, &input_attributes)?;

        let execution = CapabilityExecution {
            capability_id: capability_id.to_string(),
            input_attributes,
            execution_context: context.clone(),
        };

        let result = self.execute_capability_with_retry(&execution).await?;

        if let Some(workflow_id) = &context.workflow_id {
            self.store_execution_result(workflow_id, &result).await?;
        }

        Ok(Value::Boolean(result.execution_status == "success"))
    }

    /// Execute SET_STATUS
    async fn execute_set_status(
        &self,
        status: &str,
        target: Option<&str>,
        context: &ExecutionContext,
    ) -> Result<Value, CapabilityError> {
        println!("📊 Executing SET_STATUS: {} -> {:?}", status, target);

        // This is more of a state management operation than a capability
        // We could store this in the workflow state or resource state

        // For now, simulate success
        let result = serde_json::json!({
            "status": status,
            "target": target,
            "timestamp": Utc::now(),
            "execution_mode": context.execution_mode
        });

        Ok(Value::String(result.to_string()))
    }


    /// Evaluate expression arguments to JSON values
    async fn evaluate_arguments(&self, arguments: &[Expression]) -> Result<serde_json::Value, CapabilityError> {
        let mut arg_map = serde_json::Map::new();

        for (i, arg) in arguments.iter().enumerate() {
            let value = match arg {
                Expression::Literal(val) => self.value_to_json(val),
                Expression::Identifier(name) => {
                    // In a real implementation, we'd look up the identifier value
                    serde_json::Value::String(name.clone())
                },
                _ => serde_json::Value::String(format!("Complex expression: {:?}", arg)),
            };

            arg_map.insert(format!("arg_{}", i), value);
        }

        Ok(serde_json::Value::Object(arg_map))
    }

    /// Convert Value to JSON
    fn value_to_json(&self, value: &Value) -> serde_json::Value {
        match value {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Number(n) => serde_json::Number::from_f64(*n).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null),
            Value::Integer(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => serde_json::Number::from_f64(*f).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null),
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::Null => serde_json::Value::Null,
            Value::Regex(pattern) => serde_json::Value::String(format!("regex:{}", pattern)),
            Value::List(items) => {
                let json_items: Vec<serde_json::Value> = items.iter().map(|item| self.value_to_json(item)).collect();
                serde_json::Value::Array(json_items)
            }
        }
    }

    /// Validate capability execution
    fn validate_capability_execution(
        &self,
        capability: &ResourceCapability,
        input_attributes: &serde_json::Value,
    ) -> Result<(), CapabilityError> {
        // Check required attributes
        if let Some(required_attrs) = capability.required_attributes.as_array() {
            let mut missing_attrs = Vec::new();

            for attr in required_attrs {
                if let Some(attr_name) = attr.as_str() {
                    if !input_attributes.get(attr_name).is_some() {
                        missing_attrs.push(attr_name.to_string());
                    }
                }
            }

            if !missing_attrs.is_empty() {
                return Err(CapabilityError::AttributesMissing(missing_attrs));
            }
        }

        // Additional validation rules could be applied here
        // - Type checking
        // - Business rule validation
        // - Security checks

        Ok(())
    }

    /// Execute capability with retry logic
    async fn execute_capability_with_retry(
        &self,
        execution: &CapabilityExecution,
    ) -> Result<CapabilityExecutionResult, CapabilityError> {
        let implementation = self.capability_registry.get(&execution.capability_id)
            .ok_or_else(|| CapabilityError::NotFound(execution.capability_id.clone()))?;

        let mut attempts = 0;
        let max_attempts = match &implementation.retry_strategy {
            RetryStrategy::None => 1,
            RetryStrategy::Fixed { attempts, .. } => *attempts,
            RetryStrategy::Exponential { attempts, .. } => *attempts,
            RetryStrategy::Linear { attempts, .. } => *attempts,
        };

        loop {
            attempts += 1;
            let execution_start = std::time::Instant::now();

            // Execute the capability implementation
            let result = (implementation.function)(execution);

            let execution_time_ms = execution_start.elapsed().as_millis() as i64;

            match result {
                Ok(mut success_result) => {
                    success_result.execution_time_ms = execution_time_ms;
                    return Ok(success_result);
                },
                Err(error) => {
                    if attempts >= max_attempts {
                        return Ok(CapabilityExecutionResult {
                            capability_id: execution.capability_id.clone(),
                            execution_status: "error".to_string(),
                            output_attributes: serde_json::json!({}),
                            error_details: Some(error.to_string()),
                            execution_time_ms,
                            artifacts: None,
                            next_action: Some("manual_intervention".to_string()),
                        });
                    }

                    // Apply retry delay
                    let delay_ms = match &implementation.retry_strategy {
                        RetryStrategy::None => break,
                        RetryStrategy::Fixed { delay_ms, .. } => *delay_ms,
                        RetryStrategy::Exponential { base_delay_ms, max_delay_ms, .. } => {
                            let exponential_delay = base_delay_ms * 2_u64.pow(attempts - 1);
                            exponential_delay.min(*max_delay_ms)
                        },
                        RetryStrategy::Linear { delay_increment_ms, .. } => {
                            delay_increment_ms * attempts as u64
                        },
                    };

                    if delay_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        // This should never be reached due to the logic above
        Err(CapabilityError::SystemError("Unexpected retry loop exit".to_string()))
    }

    /// Store execution result in database
    async fn store_execution_result(
        &self,
        workflow_id: &str,
        result: &CapabilityExecutionResult,
    ) -> Result<(), CapabilityError> {
        // In a real implementation, we'd store the execution result
        // in the onboarding_resource_tasks table
        println!("📝 Storing execution result for workflow {}: {}", workflow_id, result.execution_status);
        Ok(())
    }

    /// Register built-in fund accounting capabilities
    fn register_fund_accounting_capabilities(&mut self) -> Result<(), String> {
        // Account Setup capability
        self.register_capability(
            "account_setup".to_string(),
            CapabilityImplementation {
                function: fund_accounting_account_setup,
                timeout_seconds: 300,
                retry_strategy: RetryStrategy::Exponential {
                    attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 10000,
                },
            },
        );

        // Trade Feed Setup capability
        self.register_capability(
            "trade_feed_setup".to_string(),
            CapabilityImplementation {
                function: fund_accounting_trade_feed_setup,
                timeout_seconds: 180,
                retry_strategy: RetryStrategy::Fixed {
                    attempts: 2,
                    delay_ms: 5000,
                },
            },
        );

        // NAV Calculation Setup capability
        self.register_capability(
            "nav_calculation_setup".to_string(),
            CapabilityImplementation {
                function: fund_accounting_nav_calculation_setup,
                timeout_seconds: 120,
                retry_strategy: RetryStrategy::Fixed {
                    attempts: 2,
                    delay_ms: 3000,
                },
            },
        );

        // Activate capability
        self.register_capability(
            "activate".to_string(),
            CapabilityImplementation {
                function: fund_accounting_activate,
                timeout_seconds: 60,
                retry_strategy: RetryStrategy::Fixed {
                    attempts: 3,
                    delay_ms: 2000,
                },
            },
        );

        // Health Check capability
        self.register_capability(
            "health_check".to_string(),
            CapabilityImplementation {
                function: fund_accounting_health_check,
                timeout_seconds: 30,
                retry_strategy: RetryStrategy::Fixed {
                    attempts: 2,
                    delay_ms: 1000,
                },
            },
        );

        Ok(())
    }
}

// ===== CAPABILITY IMPLEMENTATIONS =====

/// Fund Accounting Account Setup Implementation
fn fund_accounting_account_setup(execution: &CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError> {
    println!("🏦 Executing Fund Accounting Account Setup");

    // Simulate account creation logic
    // In a real implementation, this would:
    // 1. Connect to the fund accounting system
    // 2. Create the fund account structure
    // 3. Set up the base currency
    // 4. Configure initial parameters

    let fund_legal_name = execution.input_attributes.get("fund_legal_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Fund");

    let base_currency = execution.input_attributes.get("base_currency")
        .and_then(|v| v.as_str())
        .unwrap_or("USD");

    // Simulate processing time
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(CapabilityExecutionResult {
        capability_id: execution.capability_id.clone(),
        execution_status: "success".to_string(),
        output_attributes: serde_json::json!({
            "account_id": format!("ACCT_{}", chrono::Utc::now().timestamp()),
            "fund_legal_name": fund_legal_name,
            "base_currency": base_currency,
            "account_status": "active"
        }),
        error_details: None,
        execution_time_ms: 0, // Will be set by caller
        artifacts: Some(serde_json::json!({
            "configuration": {
                "fund_structure": "created",
                "currency_setup": "completed"
            }
        })),
        next_action: Some("continue".to_string()),
    })
}

/// Fund Accounting Trade Feed Setup Implementation
fn fund_accounting_trade_feed_setup(execution: &CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError> {
    println!("📊 Executing Fund Accounting Trade Feed Setup");

    let trade_feed_source = execution.input_attributes.get("trade_feed_source_system_id")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN_SOURCE");

    std::thread::sleep(std::time::Duration::from_millis(150));

    Ok(CapabilityExecutionResult {
        capability_id: execution.capability_id.clone(),
        execution_status: "success".to_string(),
        output_attributes: serde_json::json!({
            "feed_id": format!("FEED_{}", chrono::Utc::now().timestamp()),
            "source_system": trade_feed_source,
            "feed_status": "active",
            "last_update": chrono::Utc::now()
        }),
        error_details: None,
        execution_time_ms: 0,
        artifacts: Some(serde_json::json!({
            "feed_configuration": {
                "source_validated": true,
                "connection_established": true
            }
        })),
        next_action: Some("continue".to_string()),
    })
}

/// Fund Accounting NAV Calculation Setup Implementation
fn fund_accounting_nav_calculation_setup(execution: &CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError> {
    println!("📈 Executing Fund Accounting NAV Calculation Setup");

    let pricing_source = execution.input_attributes.get("pricing_source")
        .and_then(|v| v.as_str())
        .unwrap_or("BLOOMBERG");

    let calculation_frequency = execution.input_attributes.get("calculation_frequency")
        .and_then(|v| v.as_str())
        .unwrap_or("DAILY");

    std::thread::sleep(std::time::Duration::from_millis(120));

    Ok(CapabilityExecutionResult {
        capability_id: execution.capability_id.clone(),
        execution_status: "success".to_string(),
        output_attributes: serde_json::json!({
            "nav_config_id": format!("NAV_{}", chrono::Utc::now().timestamp()),
            "pricing_source": pricing_source,
            "calculation_frequency": calculation_frequency,
            "nav_calculation_status": "configured"
        }),
        error_details: None,
        execution_time_ms: 0,
        artifacts: Some(serde_json::json!({
            "nav_setup": {
                "pricing_source_validated": true,
                "calculation_rules_applied": true
            }
        })),
        next_action: Some("continue".to_string()),
    })
}

/// Fund Accounting Activate Implementation
fn fund_accounting_activate(execution: &CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError> {
    println!("🚀 Executing Fund Accounting Activation");

    // Generate a simulated instance URL
    let instance_id = chrono::Utc::now().timestamp();
    let instance_url = format!("https://fundaccounting.example.com/instance/{}", instance_id);

    std::thread::sleep(std::time::Duration::from_millis(200));

    Ok(CapabilityExecutionResult {
        capability_id: execution.capability_id.clone(),
        execution_status: "success".to_string(),
        output_attributes: serde_json::json!({
            "core_fa_instance_url": instance_url,
            "instance_id": instance_id,
            "activation_status": "live",
            "go_live_date": chrono::Utc::now()
        }),
        error_details: None,
        execution_time_ms: 0,
        artifacts: Some(serde_json::json!({
            "activation": {
                "instance_created": true,
                "health_check_passed": true,
                "ready_for_trading": true
            }
        })),
        next_action: Some("continue".to_string()),
    })
}

/// Fund Accounting Health Check Implementation
fn fund_accounting_health_check(execution: &CapabilityExecution) -> Result<CapabilityExecutionResult, CapabilityError> {
    println!("🏥 Executing Fund Accounting Health Check");

    // Simulate health check
    std::thread::sleep(std::time::Duration::from_millis(50));

    Ok(CapabilityExecutionResult {
        capability_id: execution.capability_id.clone(),
        execution_status: "success".to_string(),
        output_attributes: serde_json::json!({
            "health_status": "healthy",
            "last_check_time": chrono::Utc::now(),
            "connectivity_status": "connected",
            "system_status": "operational"
        }),
        error_details: None,
        execution_time_ms: 0,
        artifacts: Some(serde_json::json!({
            "health_check": {
                "database_connection": "ok",
                "external_services": "ok",
                "system_resources": "ok"
            }
        })),
        next_action: Some("continue".to_string()),
    })
}

// Implement Debug trait for CapabilityEngine to satisfy the compiler
impl std::fmt::Debug for CapabilityEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityEngine")
            .field("db_pool", &"DbPool")
            .field("capability_registry", &self.capability_registry.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Debug for CapabilityImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityImplementation")
            .field("function", &"<function pointer>")
            .field("timeout_seconds", &self.timeout_seconds)
            .field("retry_strategy", &self.retry_strategy)
            .finish()
    }
}