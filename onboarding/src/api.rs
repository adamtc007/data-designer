use serde::{Deserialize, Serialize};
use crate::{Plan, Idd, Bindings, MetaBundle};
use crate::ast::oodl::OnboardIntent;
use crate::planner::compile::{compile_onboard, CompileInputs};
use chrono::{DateTime, Utc};

/// Commands for onboarding workflow management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboarding {
    pub instance_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachCBU {
    pub instance_id: String,
    pub cbu_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachProducts {
    pub instance_id: String,
    pub products: Vec<String>, // ["GlobalCustody@v3", ...]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compile {
    pub instance_id: String,
}

/// Instance lifecycle states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceState {
    Draft,
    ReadyToCompile,
    Compiled,
    Executing,
    Completed,
    Failed,
}

/// Onboarding instance entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingInstance {
    pub id: String,
    pub state: InstanceState,
    pub cbu_id: Option<String>,
    pub products: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain events for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingEvent {
    OnboardingCreated { instance_id: String },
    CBUAttached { instance_id: String, cbu_id: String },
    ProductsAttached { instance_id: String, product_ids: Vec<String> },
    PlanCompiled { instance_id: String, steps: Vec<String>, idd_gaps: Vec<String> },
    TaskStarted { instance_id: String, task_id: String },
    TaskSucceeded { instance_id: String, task_id: String },
    TaskFailed { instance_id: String, task_id: String, error: String },
}

/// Command handlers (library-first)
pub fn handle_create(cmd: CreateOnboarding) -> Result<OnboardingInstance, String> {
    let instance = OnboardingInstance {
        id: cmd.instance_id.clone(),
        state: InstanceState::Draft,
        cbu_id: None,
        products: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // TODO: Persist to onboarding_instance table
    // INSERT INTO onboarding_instance (instance_id, state, cbu_id, products, created_at, updated_at)
    // VALUES ($1, $2, $3, $4, $5, $6)

    // emit_event(OnboardingEvent::OnboardingCreated { instance_id: cmd.instance_id });

    Ok(instance)
}

pub fn handle_attach_cbu(_cmd: AttachCBU) -> Result<OnboardingInstance, String> {
    // TODO: Load instance from storage
    // SELECT * FROM onboarding_instance WHERE instance_id = $1

    // TODO: Validate CBU exists in client_business_units table
    // SELECT 1 FROM client_business_units WHERE cbu_id = $1

    // TODO: Update instance with cbu_id
    // UPDATE onboarding_instance SET cbu_id = $2, updated_at = $3 WHERE instance_id = $1

    // TODO: Check if products are also set, transition to ReadyToCompile if so
    // UPDATE onboarding_instance SET state = 'ReadyToCompile' WHERE instance_id = $1 AND products IS NOT NULL

    // emit_event(OnboardingEvent::CBUAttached { instance_id: cmd.instance_id, cbu_id: cmd.cbu_id });

    Err("Not implemented yet".to_string())
}

pub fn handle_attach_products(_cmd: AttachProducts) -> Result<OnboardingInstance, String> {
    // TODO: Load instance from storage
    // SELECT * FROM onboarding_instance WHERE instance_id = $1

    // TODO: Validate products exist in metadata catalog
    // Check against onboarding/metadata/product_catalog.yaml

    // TODO: Update instance with products
    // UPDATE onboarding_instance SET products = $2, updated_at = $3 WHERE instance_id = $1

    // TODO: Check if cbu_id is also set, transition to ReadyToCompile if so
    // UPDATE onboarding_instance SET state = 'ReadyToCompile' WHERE instance_id = $1 AND cbu_id IS NOT NULL

    // emit_event(OnboardingEvent::ProductsAttached { instance_id: cmd.instance_id, product_ids: cmd.products });

    Err("Not implemented yet".to_string())
}

pub fn handle_compile(
    cmd: Compile,
    meta: &MetaBundle,
    cbu_profile: serde_json::Value,
    team_users: Vec<serde_json::Value>
) -> Result<(Plan, Idd, Bindings), String> {
    // TODO: Load instance from storage
    // SELECT * FROM onboarding_instance WHERE instance_id = $1

    // TODO: Validate instance is in ReadyToCompile state
    // Ensure state = 'ReadyToCompile' and both cbu_id and products are set

    // For now, create a mock instance until persistence is implemented
    let mock_instance = OnboardingInstance {
        id: cmd.instance_id.clone(),
        state: InstanceState::ReadyToCompile,
        cbu_id: Some("CBU-12345".to_string()), // Mock CBU ID
        products: vec!["GlobalCustody@v3".to_string()], // Mock product
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Convert instance to OnboardIntent for compiler
    let intent = OnboardIntent {
        instance_id: mock_instance.id,
        cbu_id: mock_instance.cbu_id.unwrap(),
        products: mock_instance.products,
    };

    // Call tree-walk compiler using existing compile_onboard function
    let compile_inputs = CompileInputs { intent: &intent, meta, team_users, cbu_profile };
    let outputs = compile_onboard(compile_inputs).map_err(|e| e.to_string())?;

    // TODO: Persist plan to instance_plan table
    // INSERT INTO instance_plan (instance_id, plan_json, idd_json, bindings_json, created_at)
    // VALUES ($1, $2, $3, $4, $5)

    // TODO: Transition instance to Compiled state
    // UPDATE onboarding_instance SET state = 'Compiled', updated_at = $2 WHERE instance_id = $1

    // emit_event(OnboardingEvent::PlanCompiled {
    //     instance_id: cmd.instance_id,
    //     steps: outputs.plan.steps.iter().map(|s| s.id.clone()).collect(),
    //     idd_gaps: outputs.idd.gaps.clone()
    // });

    Ok((outputs.plan, outputs.idd, outputs.bindings))
}

/// Idempotency support
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct IdempotencyKey {
    pub instance_id: String,
    pub command_type: String,
}

impl IdempotencyKey {
    pub fn for_create(instance_id: &str) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            command_type: "Create".to_string(),
        }
    }

    pub fn for_attach_cbu(instance_id: &str) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            command_type: "AttachCBU".to_string(),
        }
    }

    pub fn for_attach_products(instance_id: &str) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            command_type: "AttachProducts".to_string(),
        }
    }

    pub fn for_compile(instance_id: &str) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            command_type: "Compile".to_string(),
        }
    }
}