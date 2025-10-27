use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::grpc_client::{GrpcClient, CompileWorkflowRequest, CompileWorkflowResponse, ExecuteWorkflowRequest, ExecuteWorkflowResponse};
use crate::wasm_utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingMetadata {
    pub product_catalog: String,
    pub cbu_templates: String,
    pub resource_dicts: HashMap<String, String>,
}


pub struct OnboardingStateManager {
    client: Option<GrpcClient>,

    // Current metadata state
    pub metadata: Option<OnboardingMetadata>,
    pub metadata_loading: bool,
    pub metadata_error: Option<String>,

    // Selected file for editing
    pub selected_file: String, // "product_catalog", "cbu_templates", or resource dict name
    pub current_content: String,
    pub content_modified: bool,

    // Simple onboarding inputs
    pub cbu_name: String,
    pub cbu_description: String,
    pub product_name: String,

    // Legacy workflow compilation inputs (keep for compatibility)
    pub instance_id: String,
    pub cbu_id: String,
    pub products_input: String, // comma-separated
    pub team_users_input: String, // JSON array
    pub cbu_profile_input: String, // JSON object

    // Compilation results
    pub compile_result: Option<CompileWorkflowResponse>,
    pub compiling: bool,

    // Execution state
    pub execute_result: Option<ExecuteWorkflowResponse>,
    pub executing: bool,

    // Async state bridges (Arc<Mutex<>> for thread-safe async updates)
    metadata_state: Option<Arc<Mutex<Option<OnboardingMetadata>>>>,
    compile_state: Option<Arc<Mutex<Option<CompileWorkflowResponse>>>>,
    execute_state: Option<Arc<Mutex<Option<ExecuteWorkflowResponse>>>>,
    error_state: Option<Arc<Mutex<Option<String>>>>,
}

impl OnboardingStateManager {
    pub fn new(client: Option<GrpcClient>) -> Self {
        Self {
            client,
            metadata: None,
            metadata_loading: false,
            metadata_error: None,
            selected_file: "product_catalog".to_string(),
            current_content: String::new(),
            content_modified: false,
            cbu_name: String::new(),
            cbu_description: String::new(),
            product_name: String::new(),
            instance_id: "OR-2025-00042".to_string(),
            cbu_id: "CBU-12345".to_string(),
            products_input: "GlobalCustody@v3".to_string(),
            team_users_input: r#"[
  {"email": "ops.admin@client.com", "role": "Administrator"},
  {"email": "ops.approver@client.com", "role": "Approver"}
]"#.to_string(),
            cbu_profile_input: r#"{"region": "EU"}"#.to_string(),
            compile_result: None,
            compiling: false,
            execute_result: None,
            executing: false,
            metadata_state: None,
            compile_state: None,
            execute_state: None,
            error_state: None,
        }
    }

    // === Metadata Operations ===

    pub fn load_metadata(&mut self) {
        if self.metadata_loading {
            return;
        }

        self.metadata_loading = true;
        self.metadata_error = None;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.metadata_error = Some("No client available".to_string());
                self.metadata_loading = false;
                return;
            }
        };

        let metadata_state = Arc::new(Mutex::new(None));
        self.metadata_state = Some(metadata_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        wasm_utils::spawn_async(async move {
            match client.get_request::<OnboardingMetadata>("/api/onboarding/get-metadata").await {
                Ok(metadata) => {
                    if let Ok(mut state) = metadata_state.lock() {
                        *state = Some(metadata);
                    }
                }
                Err(e) => {
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Failed to load metadata: {}", e));
                    }
                }
            }
        });
    }

    pub fn save_current_file(&mut self) {
        if !self.content_modified {
            return;
        }

        let client = match &self.client {
            Some(c) => c.clone(),
            None => return,
        };

        let file_type = self.selected_file.clone();
        let content = self.current_content.clone();

        #[derive(Serialize)]
        struct UpdateRequest {
            file_type: String,
            content: String,
        }

        wasm_utils::spawn_async(async move {
            let request = UpdateRequest { file_type, content };
            let _ = client.post_request::<_, serde_json::Value>("/api/onboarding/update-metadata", &request).await;
        });

        self.content_modified = false;
    }

    pub fn select_file(&mut self, file_name: &str) {
        if self.selected_file == file_name {
            return;
        }

        self.selected_file = file_name.to_string();

        // Load content from metadata
        if let Some(metadata) = &self.metadata {
            self.current_content = match file_name {
                "product_catalog" => metadata.product_catalog.clone(),
                "cbu_templates" => metadata.cbu_templates.clone(),
                name => metadata.resource_dicts.get(name).cloned().unwrap_or_default(),
            };
            self.content_modified = false;
        }
    }

    // === Compilation Operations ===

    pub fn compile_workflow(&mut self) {
        if self.compiling {
            return;
        }

        // Parse inputs
        let products: Vec<String> = self.products_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let team_users: Vec<serde_json::Value> = match serde_json::from_str(&self.team_users_input) {
            Ok(users) => users,
            Err(e) => {
                self.compile_result = Some(CompileWorkflowResponse {
                    success: false,
                    message: format!("Invalid team users JSON: {}", e),
                    plan: None,
                    idd: None,
                    bindings: None,
                });
                return;
            }
        };

        let cbu_profile: serde_json::Value = match serde_json::from_str(&self.cbu_profile_input) {
            Ok(profile) => profile,
            Err(e) => {
                self.compile_result = Some(CompileWorkflowResponse {
                    success: false,
                    message: format!("Invalid CBU profile JSON: {}", e),
                    plan: None,
                    idd: None,
                    bindings: None,
                });
                return;
            }
        };

        let request = CompileWorkflowRequest {
            instance_id: self.instance_id.clone(),
            cbu_id: self.cbu_id.clone(),
            products,
            team_users,
            cbu_profile,
        };

        self.compiling = true;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.compiling = false;
                return;
            }
        };

        let compile_state = Arc::new(Mutex::new(None));
        self.compile_state = Some(compile_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        wasm_utils::spawn_async(async move {
            match client.compile_onboarding_workflow(request).await {
                Ok(response) => {
                    if let Ok(mut state) = compile_state.lock() {
                        *state = Some(response);
                    }
                }
                Err(e) => {
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Compilation error: {}", e));
                    }
                }
            }
        });
    }

    pub fn execute_workflow(&mut self) {
        if self.executing {
            return;
        }

        let plan = match &self.compile_result {
            Some(result) if result.success => {
                match &result.plan {
                    Some(p) => p.clone(),
                    None => return,
                }
            }
            _ => return,
        };

        let request = ExecuteWorkflowRequest { plan };

        self.executing = true;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.executing = false;
                return;
            }
        };

        let execute_state = Arc::new(Mutex::new(None));
        self.execute_state = Some(execute_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        wasm_utils::spawn_async(async move {
            match client.execute_onboarding_workflow(request).await {
                Ok(response) => {
                    if let Ok(mut state) = execute_state.lock() {
                        *state = Some(response);
                    }
                }
                Err(e) => {
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Execution error: {}", e));
                    }
                }
            }
        });
    }

    // === Async Result Processing ===

    pub fn update_from_async(&mut self) {
        // Check metadata loading state
        if let Some(state) = &self.metadata_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(metadata) = guard.take() {
                    self.metadata = Some(metadata.clone());
                    self.metadata_loading = false;
                    // Load initial content
                    self.current_content = metadata.product_catalog.clone();
                }
            };
            self.metadata_state = None; // Clear after scope ends
        }

        // Check compilation state
        if let Some(state) = &self.compile_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(response) = guard.take() {
                    self.compile_result = Some(response);
                    self.compiling = false;
                }
            };
            self.compile_state = None; // Clear after scope ends
        }

        // Check execution state
        if let Some(state) = &self.execute_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(response) = guard.take() {
                    self.execute_result = Some(response);
                    self.executing = false;
                }
            };
            self.execute_state = None; // Clear after scope ends
        }

        // Check error state
        if let Some(state) = &self.error_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(error) = guard.take() {
                    self.metadata_error = Some(error);
                    self.metadata_loading = false;
                    self.compiling = false;
                    self.executing = false;
                }
            };
            self.error_state = None; // Clear after scope ends
        }
    }
}
