use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{Expression, DataDictionary, Value};

/// Core resource sheet architecture - agent-based orchestration system
///
/// This module implements the two-level resource sheet concept:
/// 1. Onboarding Resource Sheet - The orchestrator/general contractor
/// 2. Domain-Specific Resource Sheets - Specialized agents (KYC, Account Setup, etc.)

// ===== CORE RESOURCE SHEET FRAMEWORK =====

/// Base trait for all resource sheet types
pub trait ResourceSheet {
    fn id(&self) -> &str;
    fn resource_type(&self) -> ResourceType;
    fn status(&self) -> ResourceStatus;
    fn metadata(&self) -> &ResourceMetadata;
    fn dictionary(&self) -> &ResourceDictionary;
    fn dsl_code(&self) -> &str;
    fn execute_step(&mut self, step: &str, context: &mut ExecutionContext) -> Result<StepResult, ResourceError>;
}

/// Resource types in the orchestration hierarchy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Top-level orchestrator - manages entire client onboarding
    Orchestrator,
    /// Domain-specific resource sheets
    Domain(DomainType),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainType {
    KYC,
    AccountSetup,
    ProductOnboarding,
    ComplianceReview,
    DocumentCollection,
    RiskAssessment,
    CustodySetup,
    TradingPermissions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceStatus {
    /// Not yet started
    Pending,
    /// Dependencies being discovered
    Discovering,
    /// Instantiating required sub-resources
    Instantiating,
    /// Actively executing DSL workflow
    Executing,
    /// Waiting for sub-resources to complete
    Waiting,
    /// Completed successfully
    Complete,
    /// Paused for manual review
    Review,
    /// Failed with error
    Failed(String),
}

// ===== ONBOARDING ORCHESTRATOR SHEET =====

/// The master onboarding resource sheet - coordinates all domain resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResourceSheet {
    pub id: String,
    pub client_id: String,
    pub products: Vec<String>,
    pub metadata: ResourceMetadata,
    pub dictionary: ResourceDictionary,
    pub orchestration_dsl: String,
    pub status: ResourceStatus,
    pub sub_resources: HashMap<String, SubResourceReference>,
    pub execution_plan: ExecutionPlan,
    pub master_data: MasterDataRegistry,
}

/// Reference to a sub-resource (domain-specific sheet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResourceReference {
    pub resource_id: String,
    pub domain_type: DomainType,
    pub status: ResourceStatus,
    pub dependencies: Vec<String>,
    pub data_requirements: Vec<DataRequirement>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Execution plan for orchestrating sub-resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub phases: Vec<ExecutionPhase>,
    pub current_phase: usize,
    pub parallel_execution: bool,
    pub failure_strategy: FailureStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPhase {
    pub name: String,
    pub description: String,
    pub resources: Vec<String>,
    pub blocking: bool, // If true, phase must complete before next starts
    pub timeout_minutes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureStrategy {
    FailFast,
    ContinueOnFailure,
    RequireManualReview,
}

// ===== DOMAIN-SPECIFIC RESOURCE SHEETS =====

/// KYC domain resource sheet - specialized for compliance clearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KYCResourceSheet {
    pub id: String,
    pub case_id: String,
    pub client_id: String,
    pub product_id: String,
    pub metadata: ResourceMetadata,
    pub dictionary: ResourceDictionary,
    pub business_logic_dsl: String,
    pub status: ResourceStatus,
    pub risk_profile: RiskProfile,
    pub documents: Vec<DocumentReference>,
    pub screenings: Vec<ScreeningResult>,
    pub regulatory_context: RegulatoryContext,
    pub clearance_decision: Option<ClearanceDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub jurisdiction_risk: RiskLevel,
    pub product_risk: RiskLevel,
    pub client_risk: RiskLevel,
    pub combined_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub category: String,
    pub description: String,
    pub score: u8, // 1-10
    pub mitigated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Prohibited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReference {
    pub document_type: String,
    pub required: bool,
    pub collected: bool,
    pub verified: bool,
    pub file_path: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningResult {
    pub source: String, // e.g., "SanctionsList", "PEPList", "WatchList"
    pub entity: String,
    pub matches: Vec<ScreeningMatch>,
    pub cleared: bool,
    pub review_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningMatch {
    pub match_score: f64,
    pub matched_entity: String,
    pub match_reason: String,
    pub false_positive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryContext {
    pub applicable_regulations: Vec<String>,
    pub jurisdiction: String,
    pub policy_overrides: HashMap<String, String>,
    pub exemptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearanceDecision {
    pub approved: bool,
    pub decision_date: DateTime<Utc>,
    pub decision_maker: String,
    pub conditions: Vec<String>,
    pub review_date: Option<DateTime<Utc>>,
    pub rationale: String,
}

// ===== RESOURCE METADATA & DICTIONARY =====

/// Enhanced metadata for resource sheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub tags: Vec<String>,
    pub priority: Priority,
    pub estimated_duration_minutes: Option<u32>,
    pub business_context: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource-specific data dictionary - extends base DataDictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDictionary {
    /// Base data dictionary
    pub base: DataDictionary,
    /// Resource-specific data requirements
    pub data_requirements: Vec<DataRequirement>,
    /// Input/output data mappings
    pub data_mappings: HashMap<String, DataMapping>,
    /// Validation rules for this resource
    pub validation_rules: Vec<ValidationRule>,
    /// Derived fields that are calculated
    pub derived_fields: HashMap<String, Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRequirement {
    pub field_name: String,
    pub data_type: String,
    pub required: bool,
    pub source: DataSource,
    pub validation_expression: Option<Expression>,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// Data provided by parent orchestrator
    Parent,
    /// Data from external system
    External(String),
    /// Data collected during this resource execution
    Internal,
    /// Data from another sub-resource
    SubResource(String),
    /// Derived from calculations
    Derived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMapping {
    pub source_field: String,
    pub target_field: String,
    pub transformation: Option<Expression>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_name: String,
    pub expression: Expression,
    pub error_message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Warning,
    Error,
    Critical,
}

// ===== EXECUTION FRAMEWORK =====

/// Execution context shared across resource sheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub client_data: HashMap<String, Value>,
    pub shared_variables: HashMap<String, Value>,
    pub execution_log: Vec<ExecutionLogEntry>,
    pub error_context: Option<ErrorContext>,
    pub timeout_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: DateTime<Utc>,
    pub resource_id: String,
    pub step: String,
    pub message: String,
    pub level: LogLevel,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_code: String,
    pub error_message: String,
    pub stack_trace: Vec<String>,
    pub recovery_suggestions: Vec<String>,
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub output_data: HashMap<String, Value>,
    pub next_step: Option<String>,
    pub sub_resource_requests: Vec<SubResourceRequest>,
    pub human_review_required: bool,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResourceRequest {
    pub domain_type: DomainType,
    pub input_data: HashMap<String, Value>,
    pub priority: Priority,
    pub dependencies: Vec<String>,
}

/// Centralized registry for master data shared across resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataRegistry {
    pub client_profile: ClientProfile,
    pub product_catalog: HashMap<String, ProductDefinition>,
    pub regulatory_requirements: HashMap<String, RegulatoryRequirement>,
    pub business_rules: HashMap<String, Expression>,
    pub lookup_tables: HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfile {
    pub client_id: String,
    pub basic_info: HashMap<String, Value>,
    pub computed_attributes: HashMap<String, Value>,
    pub risk_indicators: Vec<RiskIndicator>,
    pub relationships: Vec<ClientRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicator {
    pub indicator_type: String,
    pub value: Value,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRelationship {
    pub relationship_type: String,
    pub related_client_id: String,
    pub relationship_data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDefinition {
    pub product_id: String,
    pub product_name: String,
    pub risk_category: RiskLevel,
    pub regulatory_requirements: Vec<String>,
    pub required_documents: Vec<String>,
    pub onboarding_workflow: String, // DSL for product-specific workflow
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryRequirement {
    pub requirement_id: String,
    pub jurisdiction: String,
    pub regulation_name: String,
    pub applicability_rules: Expression,
    pub compliance_checks: Vec<ComplianceCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub check_name: String,
    pub check_expression: Expression,
    pub required: bool,
    pub frequency: CheckFrequency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckFrequency {
    OnBoarding,
    Annual,
    Continuous,
    Triggered,
}

// ===== ERROR HANDLING =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceError {
    ParseError(String),
    ValidationError(String),
    ExecutionError(String),
    DependencyError(String),
    TimeoutError,
    DataMissing(String),
    ExternalSystemError(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ResourceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ResourceError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            ResourceError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            ResourceError::TimeoutError => write!(f, "Timeout error"),
            ResourceError::DataMissing(field) => write!(f, "Missing required data: {}", field),
            ResourceError::ExternalSystemError(msg) => write!(f, "External system error: {}", msg),
        }
    }
}

impl std::error::Error for ResourceError {}