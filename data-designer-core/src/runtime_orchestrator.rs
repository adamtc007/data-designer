use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::db::DbPool;

/// Helper structs for template loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDictionary {
    pub template_name: String,
    pub domain_id: i32,
    pub ebnf_template_id: i32,
    pub public_attributes: HashMap<String, AttributeDefinition>,
    pub private_attributes: HashMap<String, DerivedAttributeDefinition>,
    pub required_attributes: Vec<String>,
    pub produced_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateJson {
    pub id: Option<String>,
    pub description: Option<String>,
    pub dsl: Option<String>,
    pub attributes: Option<Vec<TemplateAttributeJson>>,
    pub required_attributes: Option<Vec<String>>,
    pub produced_attributes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAttributeJson {
    pub name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub visibility_scope: Option<String>,
    pub derivation_rule: Option<String>,
    pub ebnf_pattern: Option<String>,
    pub source_attributes: Option<Vec<String>>,
    pub materialization_strategy: Option<String>,
}

/// Core runtime orchestrator for managing onboarding instances and template execution
#[derive(Debug, Clone)]
pub struct RuntimeOrchestrator {
    /// Unique instance identifier (e.g., "ONBOARD_001")
    pub instance_id: String,
    /// Ordered list of template dependencies for this instance
    pub template_dependencies: Vec<String>,
    /// Aggregated master dictionary from all templates
    pub master_dictionary: InstanceDictionary,
    /// Current execution context and state
    pub execution_context: ExecutionContext,
    /// Command executor for DSL commands
    pub command_executor: CommandExecutor,
    /// Database connection pool
    pub db_pool: Arc<DbPool>,
}

/// Aggregated data dictionary for a specific onboarding instance
#[derive(Debug, Clone, Default)]
pub struct InstanceDictionary {
    /// Public attributes (client-recognizable, standardized)
    pub public_attributes: HashMap<String, AttributeDefinition>,
    /// Private attributes (internal, computed via rules)
    pub private_attributes: HashMap<String, DerivedAttributeDefinition>,
    /// Per-template context mappings
    pub template_mappings: HashMap<String, TemplateContext>,
    /// Execution dependency graph
    pub dependency_graph: DependencyGraph,
}

/// Execution context for a running onboarding instance
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Unique instance identifier
    pub instance_id: String,
    /// Current workflow state
    pub workflow_state: WorkflowState,
    /// Current execution step
    pub current_step: String,
    /// Collected data values
    pub collected_data: HashMap<String, serde_json::Value>,
    /// Pending data solicitation requests
    pub pending_solicitations: Vec<DataSolicitationRequest>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Complete audit trail
    pub audit_trail: Vec<AuditEvent>,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Command executor for processing DSL commands
#[derive(Debug, Clone)]
pub struct CommandExecutor {
    /// Reference to instance dictionary
    pub dictionary: Arc<Mutex<InstanceDictionary>>,
    /// Data resolution strategies
    pub data_resolvers: HashMap<String, DataResolverType>,
    /// Validation engine
    pub validators: HashMap<String, ValidatorType>,
    /// Rule execution engine
    pub rule_executor: EbnfRuleExecutor,
}

/// Attribute definition in the instance dictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub validation_rules: Vec<String>,
    pub ui_hints: UIHints,
    pub source_template: String,
}

/// Derived attribute definition with computation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedAttributeDefinition {
    pub attribute_name: String,
    pub full_path: String,
    pub data_type: String,
    pub derivation_rule: String,
    pub ebnf_pattern: String,
    pub source_attributes: Vec<String>,
    pub materialization_strategy: MaterializationStrategy,
    pub source_template: String,
}

/// Template context for a specific template within an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    pub template_id: String,
    pub template_name: String,
    pub domain_id: i32,
    pub ebnf_template_id: i32,
    pub execution_order: i32,
    pub status: TemplateExecutionStatus,
    pub required_attributes: Vec<String>,
    pub produced_attributes: Vec<String>,
}

/// Dependency graph for template execution ordering
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>, // (from_template, to_template)
    pub execution_order: Vec<String>,
}

/// Data solicitation request for client data collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSolicitationRequest {
    pub instance_id: String,
    pub attribute_path: String,
    pub data_type: String,
    pub validation_rules: Vec<String>,
    pub ui_hints: UIHints,
    pub required: bool,
    pub description: String,
    pub template_source: String,
    pub created_at: DateTime<Utc>,
}

/// UI hints for data collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIHints {
    pub input_type: String,        // "text", "dropdown", "date", "file_upload"
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub allowed_values: Option<Vec<String>>,
    pub validation_pattern: Option<String>,
    pub max_length: Option<i32>,
    pub min_length: Option<i32>,
}

/// Validation result for data quality checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub attribute_path: String,
    pub validation_rule: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub validated_at: DateTime<Utc>,
}

/// Audit event for execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: String,
    pub attribute: String,
    pub rule_used: Option<String>,
    pub input_values: HashMap<String, serde_json::Value>,
    pub output_value: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub template_source: String,
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetadata {
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_attributes: i32,
    pub collected_attributes: i32,
    pub derived_attributes: i32,
    pub validation_errors: i32,
}

/// Workflow execution states
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WorkflowState {
    #[default]
    Initialized,
    CollectingData,
    ProcessingDerivations,
    ValidatingData,
    Completed,
    Failed(String),
    Suspended,
}

/// Template execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
    Skipped,
}

/// Materialization strategies for derived attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterializationStrategy {
    Computed,    // Calculate on-demand
    Cached,      // Compute once, cache result
    Persisted,   // Store in database
    Hybrid,      // Smart caching with fallback
}

/// Data resolver types
#[derive(Debug, Clone)]
pub enum DataResolverType {
    UserInput,
    DatabaseLookup,
    ApiCall,
    ReferenceData,
    Derivation,
}

/// Validator types
#[derive(Debug, Clone)]
pub enum ValidatorType {
    RegexPattern,
    BusinessRule,
    DataQuality,
    ReferenceCheck,
}

/// EBNF rule executor for derived attribute computation
#[derive(Debug, Clone, Default)]
pub struct EbnfRuleExecutor {
    pub rule_cache: HashMap<String, CompiledRule>,
}

/// Compiled EBNF rule for efficient execution
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub pattern: String,
    pub dependencies: Vec<String>,
    pub computation_logic: String,
}

/// Populate-data command structure for DSL parsing
#[derive(Debug, Clone)]
pub struct PopulateDataCommand {
    pub instance_id: String,
    pub target_attribute: String,
    pub command_type: PopulateDataCommandType,
}

/// Types of populate-data commands
#[derive(Debug, Clone)]
pub enum PopulateDataCommandType {
    GetData {
        attribute_path: String,
        data_source: String,
        condition: Option<String>,
    },
    SolicitData {
        attribute_path: String,
        ui_config: UIHints,
    },
    ValidateData {
        attribute_path: String,
        validation_rules: Vec<String>,
    },
    DeriveData {
        target_attribute: String,
        derivation_rule: String,
        source_attributes: Vec<String>,
    },
}

/// Result of executing a populate-data command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulateDataResult {
    pub instance_id: String,
    pub executed_at: DateTime<Utc>,
    pub collected_values: HashMap<String, serde_json::Value>,
    pub pending_solicitations: Vec<DataSolicitationRequest>,
    pub validation_results: Vec<ValidationResult>,
    pub errors: Vec<String>,
}

impl PopulateDataResult {
    pub fn new(instance_id: &str) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            executed_at: Utc::now(),
            collected_values: HashMap::new(),
            pending_solicitations: Vec::new(),
            validation_results: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Main execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResult {
    pub instance_id: String,
    pub status: WorkflowState,
    pub collected_data: HashMap<String, serde_json::Value>,
    pub pending_solicitations: Vec<DataSolicitationRequest>,
    pub validation_results: Vec<ValidationResult>,
    pub audit_trail: Vec<AuditEvent>,
    pub next_actions: Vec<String>,
    pub execution_summary: ExecutionSummary,
}

/// Execution summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_templates: i32,
    pub executed_templates: i32,
    pub total_attributes: i32,
    pub collected_attributes: i32,
    pub derived_attributes: i32,
    pub pending_solicitations: i32,
    pub validation_errors: i32,
    pub execution_time_ms: i64,
}

/// Runtime execution errors
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Attribute not found: {0}")]
    AttributeNotFound(String),
    #[error("Dependency cycle detected: {0:?}")]
    DependencyCycle(Vec<String>),
    #[error("Rule execution failed: {0}")]
    RuleExecutionFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Invalid DSL: {0}")]
    InvalidDsl(String),
}

impl RuntimeOrchestrator {
    /// Create a new runtime orchestrator for an onboarding instance
    pub fn new(instance_id: String, db_pool: Arc<DbPool>) -> Self {
        Self {
            instance_id: instance_id.clone(),
            template_dependencies: Vec::new(),
            master_dictionary: InstanceDictionary::default(),
            execution_context: ExecutionContext {
                instance_id,
                ..Default::default()
            },
            command_executor: CommandExecutor::new(),
            db_pool,
        }
    }

    /// Build master dictionary by aggregating all template dictionaries
    pub async fn build_instance_master_dictionary(&mut self) -> Result<(), RuntimeError> {
        let mut aggregated_public = HashMap::new();
        let mut aggregated_private = HashMap::new();
        let mut template_mappings = HashMap::new();

        for (index, template_id) in self.template_dependencies.iter().enumerate() {
            // Load template definition and its dictionary requirements
            let template_dict = self.load_template_dictionary(template_id).await?;

            // Create template context
            let template_context = TemplateContext {
                template_id: template_id.clone(),
                template_name: template_dict.template_name,
                domain_id: template_dict.domain_id,
                ebnf_template_id: template_dict.ebnf_template_id,
                execution_order: index as i32,
                status: TemplateExecutionStatus::Pending,
                required_attributes: template_dict.required_attributes,
                produced_attributes: template_dict.produced_attributes,
            };

            template_mappings.insert(template_id.clone(), template_context);

            // Merge public attributes (standardized - no conflicts expected)
            for (path, attr) in template_dict.public_attributes {
                if aggregated_public.contains_key(&path) {
                    // Verify consistency but don't conflict - public attributes should be standardized
                    tracing::debug!("Public attribute {} already exists - verifying consistency", path);
                } else {
                    aggregated_public.insert(path, attr);
                }
            }

            // Merge private attributes with template scoping
            for (path, attr) in template_dict.private_attributes {
                let scoped_path = format!("{}::{}", template_id, path);
                aggregated_private.insert(scoped_path, attr);

                // Also add unscoped for easy lookup (last template wins)
                aggregated_private.insert(path, attr.clone());
            }
        }

        // Update master dictionary
        self.master_dictionary.public_attributes = aggregated_public;
        self.master_dictionary.private_attributes = aggregated_private;
        self.master_dictionary.template_mappings = template_mappings;

        tracing::info!(
            "Built master dictionary: {} public attributes, {} private attributes from {} templates",
            self.master_dictionary.public_attributes.len(),
            self.master_dictionary.private_attributes.len(),
            self.template_dependencies.len()
        );

        Ok(())
    }

    /// Load template dictionary from database
    async fn load_template_dictionary(&self, template_id: &str) -> Result<TemplateDictionary, RuntimeError> {
        let query = r#"
            SELECT resource_id, name, domain_id, ebnf_template_id, json_data
            FROM resource_sheets
            WHERE resource_id = $1 AND resource_type = 'template'
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| RuntimeError::TemplateNotFound(format!("{}: {}", template_id, e)))?;

        let template_name: String = row.try_get("name")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let domain_id: i32 = row.try_get("domain_id")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let ebnf_template_id: i32 = row.try_get("ebnf_template_id")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let json_data: serde_json::Value = row.try_get("json_data")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        // Parse template attributes from JSON
        let template_json: TemplateJson = serde_json::from_value(json_data)
            .map_err(|e| RuntimeError::DatabaseError(format!("Invalid template JSON: {}", e)))?;

        // Load public attributes (from data dictionary)
        let public_attributes = self.load_public_attributes_for_template(&template_json).await?;

        // Load private attributes (from template definition)
        let private_attributes = self.load_private_attributes_for_template(&template_json).await?;

        Ok(TemplateDictionary {
            template_name,
            domain_id,
            ebnf_template_id,
            public_attributes,
            private_attributes,
            required_attributes: template_json.required_attributes.unwrap_or_default(),
            produced_attributes: template_json.produced_attributes.unwrap_or_default(),
        })
    }

    /// Load public attributes referenced by template
    async fn load_public_attributes_for_template(
        &self,
        template_json: &TemplateJson,
    ) -> Result<HashMap<String, AttributeDefinition>, RuntimeError> {
        let mut public_attributes = HashMap::new();

        // Extract attribute references from template DSL
        if let Some(ref dsl) = template_json.dsl {
            let referenced_attributes = self.extract_attribute_references(dsl);

            for attr_path in referenced_attributes {
                if self.is_public_attribute(&attr_path).await? {
                    let attr_def = self.load_public_attribute_definition(&attr_path).await?;
                    public_attributes.insert(attr_path, attr_def);
                }
            }
        }

        // Also include explicitly declared attributes
        if let Some(ref attributes) = template_json.attributes {
            for attr in attributes {
                if attr.visibility_scope.as_deref() == Some("public") {
                    let attr_def = AttributeDefinition {
                        attribute_name: attr.name.clone(),
                        full_path: format!("Client.{}", attr.name), // Assume Client entity for now
                        data_type: attr.data_type.clone(),
                        description: attr.description.clone(),
                        required: attr.required.unwrap_or(false),
                        validation_rules: Vec::new(),
                        ui_hints: UIHints::default(),
                        source_template: template_json.id.clone().unwrap_or_default(),
                    };
                    public_attributes.insert(attr_def.full_path.clone(), attr_def);
                }
            }
        }

        Ok(public_attributes)
    }

    /// Load private attributes defined by template
    async fn load_private_attributes_for_template(
        &self,
        template_json: &TemplateJson,
    ) -> Result<HashMap<String, DerivedAttributeDefinition>, RuntimeError> {
        let mut private_attributes = HashMap::new();

        if let Some(ref attributes) = template_json.attributes {
            for attr in attributes {
                if attr.visibility_scope.as_deref() == Some("private") {
                    let derived_attr = DerivedAttributeDefinition {
                        attribute_name: attr.name.clone(),
                        full_path: format!("Internal.{}", attr.name),
                        data_type: attr.data_type.clone(),
                        derivation_rule: attr.derivation_rule.clone().unwrap_or_default(),
                        ebnf_pattern: attr.ebnf_pattern.clone().unwrap_or_default(),
                        source_attributes: attr.source_attributes.clone().unwrap_or_default(),
                        materialization_strategy: match attr.materialization_strategy.as_deref() {
                            Some("cached") => MaterializationStrategy::Cached,
                            Some("persisted") => MaterializationStrategy::Persisted,
                            Some("hybrid") => MaterializationStrategy::Hybrid,
                            _ => MaterializationStrategy::Computed,
                        },
                        source_template: template_json.id.clone().unwrap_or_default(),
                    };
                    private_attributes.insert(derived_attr.full_path.clone(), derived_attr);
                }
            }
        }

        Ok(private_attributes)
    }

    /// Extract attribute references from DSL code
    fn extract_attribute_references(&self, dsl: &str) -> Vec<String> {
        let mut attributes = Vec::new();

        // Simple regex-based extraction for now - in production would use proper AST parsing
        let lines: Vec<&str> = dsl.lines().collect();
        for line in lines {
            let trimmed = line.trim();

            // Look for patterns like "Client.attribute_name" or "Internal.attribute_name"
            if let Some(start) = trimmed.find("Client.") {
                if let Some(end) = trimmed[start..].find(' ') {
                    let attr_ref = &trimmed[start..start + end];
                    attributes.push(attr_ref.to_string());
                }
            }

            // Look for GET-DATA commands
            if trimmed.contains("GET-DATA") {
                // Extract attribute path from GET-DATA commands
                // GET-DATA Client.legal_entity_name FROM customer_data
                if let Some(start) = trimmed.find("GET-DATA ") {
                    let rest = &trimmed[start + 9..];
                    if let Some(end) = rest.find(" FROM") {
                        let attr_ref = rest[..end].trim();
                        attributes.push(attr_ref.to_string());
                    }
                }
            }
        }

        // Remove duplicates
        attributes.sort();
        attributes.dedup();
        attributes
    }

    /// Check if an attribute path refers to a public attribute
    async fn is_public_attribute(&self, attr_path: &str) -> Result<bool, RuntimeError> {
        // Check in data dictionary for public attributes
        let query = r#"
            SELECT COUNT(*) as count
            FROM mv_data_dictionary
            WHERE full_path = $1
            AND attribute_type IN ('business', 'system')
            AND visibility_scope = 'public'
        "#;

        let row = sqlx::query(query)
            .bind(attr_path)
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("count")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        Ok(count > 0)
    }

    /// Load public attribute definition from data dictionary
    async fn load_public_attribute_definition(&self, attr_path: &str) -> Result<AttributeDefinition, RuntimeError> {
        let query = r#"
            SELECT attribute_name, full_path, data_type, description, required
            FROM mv_data_dictionary
            WHERE full_path = $1
            AND attribute_type IN ('business', 'system')
        "#;

        let row = sqlx::query(query)
            .bind(attr_path)
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| RuntimeError::AttributeNotFound(format!("{}: {}", attr_path, e)))?;

        let attribute_name: String = row.try_get("attribute_name")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let full_path: String = row.try_get("full_path")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let data_type: String = row.try_get("data_type")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let description: Option<String> = row.try_get("description")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
        let required: bool = row.try_get("required")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        Ok(AttributeDefinition {
            attribute_name,
            full_path,
            data_type,
            description,
            required,
            validation_rules: Vec::new(),
            ui_hints: UIHints::default(),
            source_template: "data_dictionary".to_string(),
        })
    }

    /// Resolve template dependencies for a workflow type and jurisdiction
    pub async fn resolve_template_dependencies(
        &mut self,
        workflow_type: &str,
        jurisdiction: &str,
    ) -> Result<Vec<String>, RuntimeError> {
        // Query database for matching templates
        let query = r#"
            SELECT resource_id, json_data->>'execution_order' as order_hint
            FROM resource_sheets
            WHERE resource_type = 'template'
            AND (json_data->>'workflow_type' = $1 OR json_data->>'domain' = $1)
            AND (json_data->>'jurisdiction' = $2 OR json_data->>'jurisdiction' IS NULL)
            AND status = 'Active'
            ORDER BY COALESCE((json_data->>'execution_order')::int, 999), created_at
        "#;

        let rows = sqlx::query(query)
            .bind(workflow_type)
            .bind(jurisdiction)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        let mut templates = Vec::new();
        for row in rows {
            let template_id: String = row.try_get("resource_id")
                .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;
            templates.push(template_id);
        }

        // Build dependency graph and determine execution order
        let dependency_graph = self.build_dependency_graph(&templates).await?;
        let execution_order = dependency_graph.topological_sort()?;

        self.template_dependencies = execution_order.clone();
        self.master_dictionary.dependency_graph = dependency_graph;

        Ok(execution_order)
    }

    /// Build dependency graph from template list
    async fn build_dependency_graph(&self, templates: &[String]) -> Result<DependencyGraph, RuntimeError> {
        let mut graph = DependencyGraph {
            nodes: templates.to_vec(),
            edges: Vec::new(),
            execution_order: Vec::new(),
        };

        // For now, use simple ordering - in production would analyze actual dependencies
        graph.execution_order = templates.to_vec();

        Ok(graph)
    }

    /// Main workflow execution entry point - orchestrates the entire onboarding process
    pub async fn execute_onboarding_workflow(
        &mut self,
        workflow_type: &str,
        jurisdiction: &str,
        initial_data: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<OnboardingResult, RuntimeError> {
        tracing::info!("🚀 Starting onboarding workflow: {} for jurisdiction: {}", workflow_type, jurisdiction);

        // Initialize execution context
        self.execution_context = ExecutionContext::new(
            self.instance_id.clone(),
            initial_data.unwrap_or_default(),
        );
        self.execution_context.workflow_state = WorkflowState::Initialized;

        // Step 1: Resolve template dependencies
        tracing::info!("📋 Resolving template dependencies...");
        self.execution_context.current_step = "template_resolution".to_string();
        let dependencies = self.resolve_template_dependencies(workflow_type, jurisdiction).await?;
        tracing::info!("✅ Resolved {} template dependencies", dependencies.len());

        // Step 2: Build master dictionary
        tracing::info!("📚 Building instance master dictionary...");
        self.execution_context.current_step = "dictionary_build".to_string();
        self.build_instance_master_dictionary().await?;
        tracing::info!("✅ Built master dictionary with {} public and {} private attributes",
            self.master_dictionary.public_attributes.len(),
            self.master_dictionary.private_attributes.len()
        );

        // Step 3: Execute templates in dependency order
        self.execution_context.workflow_state = WorkflowState::CollectingData;
        let mut execution_results = Vec::new();

        for template_id in &self.template_dependencies {
            tracing::info!("🎯 Executing template: {}", template_id);
            self.execution_context.current_step = format!("template_execution:{}", template_id);

            match self.execute_template(template_id).await {
                Ok(result) => {
                    execution_results.push(result);

                    // Update template status
                    if let Some(template_context) = self.master_dictionary.template_mappings.get_mut(template_id) {
                        template_context.status = TemplateExecutionStatus::Completed;
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Template execution failed for {}: {}", template_id, e);

                    // Update template status
                    if let Some(template_context) = self.master_dictionary.template_mappings.get_mut(template_id) {
                        template_context.status = TemplateExecutionStatus::Failed(e.to_string());
                    }

                    // Decide whether to continue or fail the entire workflow
                    // For now, continue with other templates
                    execution_results.push(PopulateDataResult {
                        instance_id: self.instance_id.clone(),
                        executed_at: Utc::now(),
                        collected_values: HashMap::new(),
                        pending_solicitations: Vec::new(),
                        validation_results: Vec::new(),
                        errors: vec![e.to_string()],
                    });
                }
            }
        }

        // Step 4: Process derived attributes
        tracing::info!("🧮 Processing derived attributes...");
        self.execution_context.workflow_state = WorkflowState::ProcessingDerivations;
        self.execution_context.current_step = "derivation_processing".to_string();
        self.execute_derived_attributes().await?;

        // Step 5: Final validation
        tracing::info!("✅ Running final validation...");
        self.execution_context.workflow_state = WorkflowState::ValidatingData;
        self.execution_context.current_step = "final_validation".to_string();
        self.execute_final_validation().await?;

        // Step 6: Complete workflow
        tracing::info!("🏁 Completing workflow...");
        self.execution_context.workflow_state = WorkflowState::Completed;
        self.execution_context.current_step = "workflow_completion".to_string();
        self.execution_context.metadata.completed_at = Some(Utc::now());

        // Calculate execution summary
        let execution_summary = self.calculate_execution_summary(&execution_results);

        // Build final result
        let next_actions = self.determine_next_actions();

        let result = OnboardingResult {
            instance_id: self.instance_id.clone(),
            status: self.execution_context.workflow_state.clone(),
            collected_data: self.execution_context.collected_data.clone(),
            pending_solicitations: self.execution_context.pending_solicitations.clone(),
            validation_results: self.execution_context.validation_results.clone(),
            audit_trail: self.execution_context.audit_trail.clone(),
            next_actions,
            execution_summary,
        };

        tracing::info!("✨ Workflow completed successfully! Collected {} data points with {} pending solicitations",
            result.collected_data.len(),
            result.pending_solicitations.len()
        );

        Ok(result)
    }

    /// Execute a single template and its DSL commands
    async fn execute_template(&mut self, template_id: &str) -> Result<PopulateDataResult, RuntimeError> {
        tracing::info!("📄 Executing template: {}", template_id);

        // Load template DSL
        let dsl_code = self.load_template_dsl(template_id).await?;
        let commands = self.parse_dsl_commands(&dsl_code)?;

        let mut combined_result = PopulateDataResult::new(&self.instance_id);

        // Execute each DSL command
        for command in commands {
            tracing::info!("⚡ Executing command: {}", command);

            let command_result = self.command_executor.execute_populate_data_command(
                &command,
                &mut self.execution_context,
                self.db_pool.as_ref(),
            ).await?;

            // Merge results
            combined_result.collected_values.extend(command_result.collected_values);
            combined_result.pending_solicitations.extend(command_result.pending_solicitations);
            combined_result.validation_results.extend(command_result.validation_results);
            combined_result.errors.extend(command_result.errors);
        }

        Ok(combined_result)
    }

    /// Load template DSL code from database
    async fn load_template_dsl(&self, template_id: &str) -> Result<String, RuntimeError> {
        let query = r#"
            SELECT json_data->>'dsl' as dsl_code
            FROM resource_sheets
            WHERE resource_id = $1 AND resource_type = 'template'
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| RuntimeError::TemplateNotFound(format!("{}: {}", template_id, e)))?;

        let dsl_code: Option<String> = row.try_get("dsl_code")
            .map_err(|e| RuntimeError::DatabaseError(e.to_string()))?;

        dsl_code.ok_or_else(|| RuntimeError::InvalidDsl(format!("No DSL code found for template: {}", template_id)))
    }

    /// Parse DSL code into individual commands
    fn parse_dsl_commands(&self, dsl_code: &str) -> Result<Vec<String>, RuntimeError> {
        let mut commands = Vec::new();
        let lines: Vec<&str> = dsl_code.lines().collect();
        let mut current_command = String::new();

        for line in lines {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
                continue;
            }

            // Check if this is the start of a new command
            if trimmed.starts_with("GET-DATA") ||
               trimmed.starts_with("SOLICIT-DATA") ||
               trimmed.starts_with("VALIDATE") ||
               trimmed.starts_with("DERIVE") {

                // Save previous command if exists
                if !current_command.is_empty() {
                    commands.push(current_command.trim().to_string());
                }

                // Start new command
                current_command = trimmed.to_string();
            } else {
                // Continue building current command
                if !current_command.is_empty() {
                    current_command.push('\n');
                    current_command.push_str(trimmed);
                }
            }
        }

        // Add the last command
        if !current_command.is_empty() {
            commands.push(current_command.trim().to_string());
        }

        Ok(commands)
    }

    /// Execute all derived attributes in the master dictionary
    async fn execute_derived_attributes(&mut self) -> Result<(), RuntimeError> {
        tracing::info!("🧮 Processing {} derived attributes", self.master_dictionary.private_attributes.len());

        // Sort derived attributes by dependency order
        let sorted_attributes = self.sort_derived_attributes_by_dependencies()?;

        for attr_path in sorted_attributes {
            if let Some(derived_attr) = self.master_dictionary.private_attributes.get(&attr_path) {
                tracing::info!("🔧 Computing derived attribute: {}", attr_path);

                // Collect source values
                let mut source_values = HashMap::new();
                for source_attr in &derived_attr.source_attributes {
                    if let Some(value) = self.execution_context.collected_data.get(source_attr) {
                        source_values.insert(source_attr.clone(), value.clone());
                    } else {
                        tracing::warn!("⚠️ Missing source attribute for derivation: {}", source_attr);
                    }
                }

                // Execute derivation rule
                if !source_values.is_empty() {
                    match self.command_executor.rule_executor.execute_derivation_rule(
                        &derived_attr.derivation_rule,
                        &source_values,
                    ).await {
                        Ok(derived_value) => {
                            self.execution_context.collected_data.insert(attr_path.clone(), derived_value.clone());

                            // Add audit event
                            let audit_event = AuditEvent {
                                event_type: "derived_attribute".to_string(),
                                attribute: attr_path.clone(),
                                rule_used: Some(derived_attr.derivation_rule.clone()),
                                input_values: source_values,
                                output_value: Some(derived_value),
                                timestamp: Utc::now(),
                                template_source: derived_attr.source_template.clone(),
                            };
                            self.execution_context.audit_trail.push(audit_event);

                            tracing::info!("✅ Computed derived attribute: {}", attr_path);
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to compute derived attribute {}: {}", attr_path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Sort derived attributes by their dependency order
    fn sort_derived_attributes_by_dependencies(&self) -> Result<Vec<String>, RuntimeError> {
        // Simple implementation - return attributes in alphabetical order
        // In production, would implement proper topological sorting based on dependencies
        let mut attributes: Vec<String> = self.master_dictionary.private_attributes.keys().cloned().collect();
        attributes.sort();
        Ok(attributes)
    }

    /// Execute final validation of all collected data
    async fn execute_final_validation(&mut self) -> Result<(), RuntimeError> {
        tracing::info!("🔍 Running final validation on {} attributes", self.execution_context.collected_data.len());

        // Validate all required public attributes are present
        for (attr_path, attr_def) in &self.master_dictionary.public_attributes {
            if attr_def.required && !self.execution_context.collected_data.contains_key(attr_path) {
                let validation_result = ValidationResult {
                    attribute_path: attr_path.clone(),
                    validation_rule: "required_attribute".to_string(),
                    passed: false,
                    error_message: Some(format!("Required attribute {} is missing", attr_path)),
                    validated_at: Utc::now(),
                };
                self.execution_context.validation_results.push(validation_result);
            }
        }

        // Run business rule validations
        for (attr_path, value) in &self.execution_context.collected_data {
            if let Some(attr_def) = self.master_dictionary.public_attributes.get(attr_path) {
                for validation_rule in &attr_def.validation_rules {
                    let validation_result = self.command_executor.validate_value(
                        value,
                        validation_rule,
                        attr_path,
                    ).await?;
                    self.execution_context.validation_results.push(validation_result);
                }
            }
        }

        let validation_errors = self.execution_context.validation_results.iter()
            .filter(|v| !v.passed)
            .count();

        if validation_errors > 0 {
            tracing::warn!("⚠️ Final validation completed with {} errors", validation_errors);
        } else {
            tracing::info!("✅ Final validation passed with no errors");
        }

        Ok(())
    }

    /// Calculate execution summary statistics
    fn calculate_execution_summary(&self, execution_results: &[PopulateDataResult]) -> ExecutionSummary {
        let total_templates = self.template_dependencies.len() as i32;
        let executed_templates = execution_results.len() as i32;
        let total_attributes = self.master_dictionary.public_attributes.len() + self.master_dictionary.private_attributes.len();
        let collected_attributes = self.execution_context.collected_data.len() as i32;
        let derived_attributes = self.master_dictionary.private_attributes.len() as i32;
        let pending_solicitations = self.execution_context.pending_solicitations.len() as i32;
        let validation_errors = self.execution_context.validation_results.iter()
            .filter(|v| !v.passed)
            .count() as i32;

        let execution_time_ms = if let (Some(start), Some(end)) = (
            self.execution_context.metadata.started_at,
            self.execution_context.metadata.completed_at,
        ) {
            (end - start).num_milliseconds()
        } else {
            0
        };

        ExecutionSummary {
            total_templates,
            executed_templates,
            total_attributes: total_attributes as i32,
            collected_attributes,
            derived_attributes,
            pending_solicitations,
            validation_errors,
            execution_time_ms,
        }
    }

    /// Determine next actions based on current state
    fn determine_next_actions(&self) -> Vec<String> {
        let mut next_actions = Vec::new();

        // Check for pending solicitations
        if !self.execution_context.pending_solicitations.is_empty() {
            next_actions.push(format!("Complete {} pending data solicitations",
                self.execution_context.pending_solicitations.len()));
        }

        // Check for validation errors
        let validation_errors = self.execution_context.validation_results.iter()
            .filter(|v| !v.passed)
            .count();
        if validation_errors > 0 {
            next_actions.push(format!("Resolve {} validation errors", validation_errors));
        }

        // Check for missing required attributes
        let missing_required = self.master_dictionary.public_attributes.iter()
            .filter(|(path, attr)| attr.required && !self.execution_context.collected_data.contains_key(*path))
            .count();
        if missing_required > 0 {
            next_actions.push(format!("Provide {} missing required attributes", missing_required));
        }

        // If no pending actions, workflow is complete
        if next_actions.is_empty() {
            next_actions.push("Workflow complete - ready for submission".to_string());
        }

        next_actions
    }
}

impl DependencyGraph {
    /// Perform topological sort to determine execution order
    pub fn topological_sort(&self) -> Result<Vec<String>, RuntimeError> {
        // Simplified implementation - return nodes in current order
        // In production, would implement proper topological sorting
        Ok(self.nodes.clone())
    }
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            dictionary: Arc::new(Mutex::new(InstanceDictionary::default())),
            data_resolvers: HashMap::new(),
            validators: HashMap::new(),
            rule_executor: EbnfRuleExecutor::default(),
        }
    }

    /// Execute a populate-data command from the onboarding DSL
    pub async fn execute_populate_data_command(
        &mut self,
        command: &str,
        execution_context: &mut ExecutionContext,
        db_pool: &DbPool,
    ) -> Result<PopulateDataResult, RuntimeError> {
        tracing::info!("🚀 Executing populate-data command: {}", command);

        let parsed_command = self.parse_populate_data_command(command)?;
        let mut result = PopulateDataResult::new(&parsed_command.instance_id);

        match parsed_command.command_type {
            PopulateDataCommandType::GetData { attribute_path, data_source, condition } => {
                self.execute_get_data(&attribute_path, &data_source, condition.as_deref(), execution_context, db_pool, &mut result).await?;
            }
            PopulateDataCommandType::SolicitData { attribute_path, ui_config } => {
                self.execute_solicit_data(&attribute_path, ui_config, execution_context, &mut result).await?;
            }
            PopulateDataCommandType::ValidateData { attribute_path, validation_rules } => {
                self.execute_validate_data(&attribute_path, &validation_rules, execution_context, &mut result).await?;
            }
            PopulateDataCommandType::DeriveData { target_attribute, derivation_rule, source_attributes } => {
                self.execute_derive_data(&target_attribute, &derivation_rule, &source_attributes, execution_context, &mut result).await?;
            }
        }

        // Add audit event
        let audit_event = AuditEvent {
            event_type: "populate_data_command".to_string(),
            attribute: parsed_command.target_attribute.clone(),
            rule_used: Some(command.to_string()),
            input_values: HashMap::new(),
            output_value: result.collected_values.values().next().cloned(),
            timestamp: Utc::now(),
            template_source: execution_context.current_step.clone(),
        };
        execution_context.audit_trail.push(audit_event);

        tracing::info!("✅ Command executed successfully: {} data points collected", result.collected_values.len());
        Ok(result)
    }

    /// Parse a populate-data command into structured components
    fn parse_populate_data_command(&self, command: &str) -> Result<PopulateDataCommand, RuntimeError> {
        let trimmed = command.trim();
        let lines: Vec<&str> = trimmed.lines().collect();

        if lines.is_empty() {
            return Err(RuntimeError::InvalidDsl("Empty command".to_string()));
        }

        let first_line = lines[0].trim();

        // Parse GET-DATA commands
        if first_line.starts_with("GET-DATA") {
            return self.parse_get_data_command(trimmed);
        }

        // Parse SOLICIT-DATA commands
        if first_line.starts_with("SOLICIT-DATA") {
            return self.parse_solicit_data_command(trimmed);
        }

        // Parse VALIDATE commands
        if first_line.starts_with("VALIDATE") {
            return self.parse_validate_command(trimmed);
        }

        // Parse DERIVE commands
        if first_line.starts_with("DERIVE") {
            return self.parse_derive_command(trimmed);
        }

        Err(RuntimeError::InvalidDsl(format!("Unknown command type: {}", first_line)))
    }

    /// Parse GET-DATA command: GET-DATA Client.legal_entity_name FROM customer_data WHERE client_id = instance.client_id
    fn parse_get_data_command(&self, command: &str) -> Result<PopulateDataCommand, RuntimeError> {
        let get_data_pattern = regex::Regex::new(r"GET-DATA\s+(\S+)\s+FROM\s+(\S+)(?:\s+WHERE\s+(.+))?").unwrap();

        if let Some(captures) = get_data_pattern.captures(command) {
            let attribute_path = captures.get(1).unwrap().as_str().to_string();
            let data_source = captures.get(2).unwrap().as_str().to_string();
            let condition = captures.get(3).map(|m| m.as_str().to_string());

            Ok(PopulateDataCommand {
                instance_id: "current".to_string(), // Will be filled in by context
                target_attribute: attribute_path.clone(),
                command_type: PopulateDataCommandType::GetData {
                    attribute_path,
                    data_source,
                    condition,
                },
            })
        } else {
            Err(RuntimeError::InvalidDsl(format!("Invalid GET-DATA syntax: {}", command)))
        }
    }

    /// Parse SOLICIT-DATA command with UI configuration
    fn parse_solicit_data_command(&self, command: &str) -> Result<PopulateDataCommand, RuntimeError> {
        let lines: Vec<&str> = command.lines().collect();
        let first_line = lines[0].trim();

        let solicit_pattern = regex::Regex::new(r"SOLICIT-DATA\s+(\S+)").unwrap();

        if let Some(captures) = solicit_pattern.captures(first_line) {
            let attribute_path = captures.get(1).unwrap().as_str().to_string();

            // Parse UI configuration from subsequent lines
            let mut ui_config = UIHints::default();
            for line in &lines[1..] {
                let trimmed = line.trim();
                if trimmed.starts_with("INPUT_TYPE:") {
                    ui_config.input_type = trimmed.replace("INPUT_TYPE:", "").trim().to_string();
                } else if trimmed.starts_with("PLACEHOLDER:") {
                    ui_config.placeholder = Some(trimmed.replace("PLACEHOLDER:", "").trim().to_string());
                } else if trimmed.starts_with("HELP_TEXT:") {
                    ui_config.help_text = Some(trimmed.replace("HELP_TEXT:", "").trim().to_string());
                }
            }

            Ok(PopulateDataCommand {
                instance_id: "current".to_string(),
                target_attribute: attribute_path.clone(),
                command_type: PopulateDataCommandType::SolicitData {
                    attribute_path,
                    ui_config,
                },
            })
        } else {
            Err(RuntimeError::InvalidDsl(format!("Invalid SOLICIT-DATA syntax: {}", command)))
        }
    }

    /// Parse VALIDATE command
    fn parse_validate_command(&self, command: &str) -> Result<PopulateDataCommand, RuntimeError> {
        let validate_pattern = regex::Regex::new(r"VALIDATE\s+(\S+)\s+(.+)").unwrap();

        if let Some(captures) = validate_pattern.captures(command) {
            let attribute_path = captures.get(1).unwrap().as_str().to_string();
            let validation_rules = captures.get(2).unwrap().as_str().split(',')
                .map(|s| s.trim().to_string())
                .collect();

            Ok(PopulateDataCommand {
                instance_id: "current".to_string(),
                target_attribute: attribute_path.clone(),
                command_type: PopulateDataCommandType::ValidateData {
                    attribute_path,
                    validation_rules,
                },
            })
        } else {
            Err(RuntimeError::InvalidDsl(format!("Invalid VALIDATE syntax: {}", command)))
        }
    }

    /// Parse DERIVE command
    fn parse_derive_command(&self, command: &str) -> Result<PopulateDataCommand, RuntimeError> {
        let derive_pattern = regex::Regex::new(r"DERIVE\s+(\S+)\s+FROM\s+(.+)").unwrap();

        if let Some(captures) = derive_pattern.captures(command) {
            let target_attribute = captures.get(1).unwrap().as_str().to_string();
            let derivation_spec = captures.get(2).unwrap().as_str();

            // Extract source attributes from derivation specification
            let source_attributes = self.extract_source_attributes_from_derivation(derivation_spec);

            Ok(PopulateDataCommand {
                instance_id: "current".to_string(),
                target_attribute: target_attribute.clone(),
                command_type: PopulateDataCommandType::DeriveData {
                    target_attribute,
                    derivation_rule: derivation_spec.to_string(),
                    source_attributes,
                },
            })
        } else {
            Err(RuntimeError::InvalidDsl(format!("Invalid DERIVE syntax: {}", command)))
        }
    }

    /// Extract source attributes from derivation rule
    fn extract_source_attributes_from_derivation(&self, derivation_rule: &str) -> Vec<String> {
        let mut source_attrs = Vec::new();

        // Look for attribute references in the form "Client.attr" or "Internal.attr"
        let attr_pattern = regex::Regex::new(r"\b(Client|Internal)\.\w+").unwrap();
        for capture in attr_pattern.captures_iter(derivation_rule) {
            if let Some(attr_ref) = capture.get(0) {
                source_attrs.push(attr_ref.as_str().to_string());
            }
        }

        // Remove duplicates
        source_attrs.sort();
        source_attrs.dedup();
        source_attrs
    }

    /// Execute GET-DATA command to retrieve data from a data source
    async fn execute_get_data(
        &mut self,
        attribute_path: &str,
        data_source: &str,
        condition: Option<&str>,
        execution_context: &mut ExecutionContext,
        db_pool: &DbPool,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("📊 Getting data for {} from {}", attribute_path, data_source);

        match data_source {
            "customer_data" => {
                self.get_data_from_customer_database(attribute_path, condition, execution_context, db_pool, result).await?;
            }
            "reference_data" => {
                self.get_data_from_reference_database(attribute_path, condition, execution_context, db_pool, result).await?;
            }
            "external_api" => {
                self.get_data_from_external_api(attribute_path, condition, execution_context, result).await?;
            }
            _ => {
                return Err(RuntimeError::InvalidDsl(format!("Unknown data source: {}", data_source)));
            }
        }

        Ok(())
    }

    /// Execute SOLICIT-DATA command to request data from client
    async fn execute_solicit_data(
        &mut self,
        attribute_path: &str,
        ui_config: UIHints,
        execution_context: &mut ExecutionContext,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("📝 Soliciting data for {} with UI type: {}", attribute_path, ui_config.input_type);

        // Create data solicitation request
        let solicitation_request = DataSolicitationRequest {
            instance_id: execution_context.instance_id.clone(),
            attribute_path: attribute_path.to_string(),
            data_type: self.get_attribute_data_type(attribute_path).await?,
            validation_rules: self.get_attribute_validation_rules(attribute_path).await?,
            ui_hints: ui_config,
            required: self.is_attribute_required(attribute_path).await?,
            description: self.get_attribute_description(attribute_path).await?,
            template_source: execution_context.current_step.clone(),
            created_at: Utc::now(),
        };

        execution_context.pending_solicitations.push(solicitation_request.clone());
        result.pending_solicitations.push(solicitation_request);

        Ok(())
    }

    /// Execute VALIDATE command to validate data quality
    async fn execute_validate_data(
        &mut self,
        attribute_path: &str,
        validation_rules: &[String],
        execution_context: &mut ExecutionContext,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("✅ Validating {} with rules: {:?}", attribute_path, validation_rules);

        if let Some(value) = execution_context.collected_data.get(attribute_path) {
            for rule in validation_rules {
                let validation_result = self.validate_value(value, rule, attribute_path).await?;
                execution_context.validation_results.push(validation_result.clone());
                result.validation_results.push(validation_result);
            }
        } else {
            let validation_result = ValidationResult {
                attribute_path: attribute_path.to_string(),
                validation_rule: "required_value".to_string(),
                passed: false,
                error_message: Some("Value not found for validation".to_string()),
                validated_at: Utc::now(),
            };
            execution_context.validation_results.push(validation_result.clone());
            result.validation_results.push(validation_result);
        }

        Ok(())
    }

    /// Execute DERIVE command to compute derived attributes
    async fn execute_derive_data(
        &mut self,
        target_attribute: &str,
        derivation_rule: &str,
        source_attributes: &[String],
        execution_context: &mut ExecutionContext,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("🧮 Deriving {} from {} using rule: {}", target_attribute, source_attributes.join(", "), derivation_rule);

        // Collect source values
        let mut source_values = HashMap::new();
        for source_attr in source_attributes {
            if let Some(value) = execution_context.collected_data.get(source_attr) {
                source_values.insert(source_attr.clone(), value.clone());
            } else {
                return Err(RuntimeError::ValidationFailed(format!("Missing source attribute: {}", source_attr)));
            }
        }

        // Execute derivation rule using EBNF rule executor
        let derived_value = self.rule_executor.execute_derivation_rule(
            derivation_rule,
            &source_values,
        ).await?;

        // Store derived value
        execution_context.collected_data.insert(target_attribute.to_string(), derived_value.clone());
        result.collected_values.insert(target_attribute.to_string(), derived_value);

        Ok(())
    }

    // Helper methods for data retrieval

    async fn get_data_from_customer_database(
        &self,
        attribute_path: &str,
        condition: Option<&str>,
        execution_context: &ExecutionContext,
        db_pool: &DbPool,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        // Mock implementation - in production would query actual customer database
        tracing::info!("📚 Querying customer database for {}", attribute_path);

        let mock_value = match attribute_path {
            "Client.legal_entity_name" => serde_json::Value::String("Example Corp Ltd".to_string()),
            "Client.client_id" => serde_json::Value::String(format!("CLIENT_{}", execution_context.instance_id)),
            "Client.incorporation_date" => serde_json::Value::String("2020-01-15".to_string()),
            _ => serde_json::Value::String(format!("mock_value_for_{}", attribute_path)),
        };

        result.collected_values.insert(attribute_path.to_string(), mock_value);
        Ok(())
    }

    async fn get_data_from_reference_database(
        &self,
        attribute_path: &str,
        condition: Option<&str>,
        execution_context: &ExecutionContext,
        db_pool: &DbPool,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("📖 Querying reference database for {}", attribute_path);

        // Mock reference data
        let mock_value = match attribute_path {
            "Reference.jurisdiction_code" => serde_json::Value::String("US".to_string()),
            "Reference.currency_code" => serde_json::Value::String("USD".to_string()),
            _ => serde_json::Value::String(format!("ref_value_for_{}", attribute_path)),
        };

        result.collected_values.insert(attribute_path.to_string(), mock_value);
        Ok(())
    }

    async fn get_data_from_external_api(
        &self,
        attribute_path: &str,
        condition: Option<&str>,
        execution_context: &ExecutionContext,
        result: &mut PopulateDataResult,
    ) -> Result<(), RuntimeError> {
        tracing::info!("🌐 Calling external API for {}", attribute_path);

        // Mock external API response
        let mock_value = match attribute_path {
            "External.credit_rating" => serde_json::Value::String("AAA".to_string()),
            "External.market_data" => serde_json::Value::Object(serde_json::Map::new()),
            _ => serde_json::Value::String(format!("api_value_for_{}", attribute_path)),
        };

        result.collected_values.insert(attribute_path.to_string(), mock_value);
        Ok(())
    }

    async fn get_attribute_data_type(&self, attribute_path: &str) -> Result<String, RuntimeError> {
        // Mock implementation - would query data dictionary
        Ok("string".to_string())
    }

    async fn get_attribute_validation_rules(&self, attribute_path: &str) -> Result<Vec<String>, RuntimeError> {
        // Mock implementation - would query validation rules
        Ok(vec!["required".to_string()])
    }

    async fn is_attribute_required(&self, attribute_path: &str) -> Result<bool, RuntimeError> {
        // Mock implementation - would check attribute definition
        Ok(true)
    }

    async fn get_attribute_description(&self, attribute_path: &str) -> Result<String, RuntimeError> {
        // Mock implementation - would get from data dictionary
        Ok(format!("Description for {}", attribute_path))
    }

    async fn validate_value(
        &self,
        value: &serde_json::Value,
        rule: &str,
        attribute_path: &str,
    ) -> Result<ValidationResult, RuntimeError> {
        let passed = match rule {
            "required" => !value.is_null(),
            "non_empty" => !value.as_str().unwrap_or("").is_empty(),
            _ => true, // Mock validation
        };

        Ok(ValidationResult {
            attribute_path: attribute_path.to_string(),
            validation_rule: rule.to_string(),
            passed,
            error_message: if passed { None } else { Some(format!("Validation failed for rule: {}", rule)) },
            validated_at: Utc::now(),
        })
    }
}

impl EbnfRuleExecutor {
    /// Execute a derivation rule with source values
    pub async fn execute_derivation_rule(
        &mut self,
        derivation_rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        tracing::info!("🧮 Executing derivation rule: {}", derivation_rule);

        // Check if rule is already compiled and cached
        if let Some(compiled_rule) = self.rule_cache.get(derivation_rule) {
            return self.execute_compiled_rule(compiled_rule, source_values).await;
        }

        // Compile and cache the rule
        let compiled_rule = self.compile_derivation_rule(derivation_rule)?;
        self.rule_cache.insert(derivation_rule.to_string(), compiled_rule.clone());

        self.execute_compiled_rule(&compiled_rule, source_values).await
    }

    /// Compile a derivation rule into an executable form
    fn compile_derivation_rule(&self, derivation_rule: &str) -> Result<CompiledRule, RuntimeError> {
        // Parse the derivation rule to extract pattern and dependencies
        let dependencies = self.extract_dependencies(derivation_rule);

        // For now, store the rule as computation logic
        // In production, this would compile to AST or bytecode
        let compiled_rule = CompiledRule {
            pattern: derivation_rule.to_string(),
            dependencies,
            computation_logic: derivation_rule.to_string(),
        };

        Ok(compiled_rule)
    }

    /// Execute a compiled rule with source values
    async fn execute_compiled_rule(
        &self,
        compiled_rule: &CompiledRule,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        let computation_logic = &compiled_rule.computation_logic;

        // Simple rule execution - in production would use proper expression evaluator
        match computation_logic {
            rule if rule.contains("CONCATENATE") => {
                self.execute_concatenation_rule(rule, source_values)
            }
            rule if rule.contains("ARITHMETIC") => {
                self.execute_arithmetic_rule(rule, source_values)
            }
            rule if rule.contains("CONDITIONAL") => {
                self.execute_conditional_rule(rule, source_values)
            }
            rule if rule.contains("LOOKUP") => {
                self.execute_lookup_rule(rule, source_values)
            }
            rule if rule.contains("VALIDATE") => {
                self.execute_validation_rule(rule, source_values)
            }
            _ => {
                // Default: return first available source value or null
                Ok(source_values.values().next().cloned().unwrap_or(serde_json::Value::Null))
            }
        }
    }

    /// Extract dependencies from derivation rule
    fn extract_dependencies(&self, derivation_rule: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Extract attribute references
        let attr_pattern = regex::Regex::new(r"\b(Client|Internal|Reference|External)\.\w+").unwrap();
        for capture in attr_pattern.captures_iter(derivation_rule) {
            if let Some(attr_ref) = capture.get(0) {
                dependencies.push(attr_ref.as_str().to_string());
            }
        }

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Execute concatenation rule
    fn execute_concatenation_rule(
        &self,
        rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        let mut result = String::new();

        // Simple concatenation of all string values
        for (_, value) in source_values {
            if let Some(s) = value.as_str() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(s);
            }
        }

        Ok(serde_json::Value::String(result))
    }

    /// Execute arithmetic rule
    fn execute_arithmetic_rule(
        &self,
        rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        // Simple arithmetic - sum all numeric values
        let mut sum = 0.0;
        let mut count = 0;

        for (_, value) in source_values {
            if let Some(num) = value.as_f64() {
                sum += num;
                count += 1;
            }
        }

        if count > 0 {
            Ok(serde_json::Value::Number(serde_json::Number::from_f64(sum).unwrap()))
        } else {
            Ok(serde_json::Value::Number(serde_json::Number::from(0)))
        }
    }

    /// Execute conditional rule
    fn execute_conditional_rule(
        &self,
        rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        // Simple conditional: if any value is true-ish, return "true", else "false"
        for (_, value) in source_values {
            match value {
                serde_json::Value::Bool(true) => return Ok(serde_json::Value::String("true".to_string())),
                serde_json::Value::String(s) if !s.is_empty() => return Ok(serde_json::Value::String("true".to_string())),
                serde_json::Value::Number(n) if n.as_f64().unwrap_or(0.0) > 0.0 => return Ok(serde_json::Value::String("true".to_string())),
                _ => continue,
            }
        }

        Ok(serde_json::Value::String("false".to_string()))
    }

    /// Execute lookup rule
    fn execute_lookup_rule(
        &self,
        rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        // Mock lookup table
        let lookup_table: HashMap<&str, &str> = [
            ("US", "United States"),
            ("UK", "United Kingdom"),
            ("CA", "Canada"),
            ("AAA", "Highest Credit Rating"),
            ("USD", "US Dollar"),
        ].iter().cloned().collect();

        // Look up the first string value
        for (_, value) in source_values {
            if let Some(key) = value.as_str() {
                if let Some(&result) = lookup_table.get(key) {
                    return Ok(serde_json::Value::String(result.to_string()));
                }
            }
        }

        Ok(serde_json::Value::String("Unknown".to_string()))
    }

    /// Execute validation rule
    fn execute_validation_rule(
        &self,
        rule: &str,
        source_values: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, RuntimeError> {
        // Simple validation: check if all values are non-null and non-empty
        for (_, value) in source_values {
            match value {
                serde_json::Value::Null => return Ok(serde_json::Value::String("INVALID: null value".to_string())),
                serde_json::Value::String(s) if s.is_empty() => return Ok(serde_json::Value::String("INVALID: empty string".to_string())),
                _ => continue,
            }
        }

        Ok(serde_json::Value::String("VALID".to_string()))
    }
}

impl ExecutionContext {
    pub fn new(instance_id: String, initial_data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            instance_id,
            collected_data: initial_data,
            workflow_state: WorkflowState::Initialized,
            metadata: ExecutionMetadata {
                started_at: Some(Utc::now()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}