use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::http_client::GrpcClient;
use crate::wasm_utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct OnboardingMetadata {
    pub product_catalog: String,
    pub cbu_templates: String,
    pub resource_dicts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used internally
pub struct CompileWorkflowRequest {
    pub instance_id: String,
    pub cbu_id: String,
    pub products: Vec<String>,
    pub team_users: Vec<serde_json::Value>,
    pub cbu_profile: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct CompileWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub plan: Option<serde_json::Value>,
    pub idd: Option<serde_json::Value>,
    pub bindings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used internally
pub struct ExecuteWorkflowRequest {
    pub plan: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct ExecuteWorkflowResponse {
    pub success: bool,
    pub message: String,
    pub execution_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used internally
pub struct CreateRequestInput {
    pub name: String,
    pub description: Option<String>,
    pub cbu_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct CreateRequestResponse {
    pub success: bool,
    pub message: String,
    pub onboarding_id: Option<String>,
    pub request_id: Option<i32>,
}

#[allow(dead_code)] // Used by app.rs
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

    // Create Request form state
    pub request_name: String,
    pub request_description: String,
    pub request_cbu_id: String,
    pub creating_request: bool,
    pub create_request_result: Option<CreateRequestResponse>,

    // Current onboarding request (if loaded)
    pub current_onboarding_id: Option<String>,

    // Workflow compilation inputs
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
    create_request_state: Option<Arc<Mutex<Option<CreateRequestResponse>>>>,
    error_state: Option<Arc<Mutex<Option<String>>>>,
}

#[allow(dead_code)] // All methods used by app.rs
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
            request_name: String::new(),
            request_description: String::new(),
            request_cbu_id: String::new(),
            creating_request: false,
            create_request_result: None,
            current_onboarding_id: None,
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
            create_request_state: None,
            error_state: None,
        }
    }

    // === Metadata Operations ===

    pub fn load_metadata(&mut self) {
        if self.metadata_loading {
            log::warn!("⚠️ [STATE] Load metadata called but already loading - ignoring");
            return;
        }

        log::info!("🔄 [STATE] Starting metadata load");
        self.metadata_loading = true;
        self.metadata_error = None;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                log::error!("❌ [STATE] No client available for metadata load");
                self.metadata_error = Some("No client available".to_string());
                self.metadata_loading = false;
                return;
            }
        };

        let metadata_state = Arc::new(Mutex::new(None));
        self.metadata_state = Some(metadata_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        log::info!("📡 [STATE] Sending GET request to /api/onboarding/get-metadata");
        wasm_utils::spawn_async(async move {
            let result = client.get_request::<OnboardingMetadata>("/api/onboarding/get-metadata").await;

            match result {
                Ok(metadata) => {
                    log::info!("✅ [STATE] Metadata loaded from server");
                    log::info!("  Product catalog: {} bytes", metadata.product_catalog.len());
                    log::info!("  CBU templates: {} bytes", metadata.cbu_templates.len());
                    log::info!("  Resource dicts: {} files", metadata.resource_dicts.len());

                    if let Ok(mut state) = metadata_state.lock() {
                        *state = Some(metadata);
                        log::info!("✅ [STATE] Metadata stored in state successfully");
                    } else {
                        log::error!("❌ [STATE] Failed to acquire metadata lock");
                    }
                }
                Err(e) => {
                    log::error!("❌ [STATE] Failed to load metadata: {}", e);
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
            log::warn!("⚠️ [STATE] Compile called but already compiling - ignoring");
            return;
        }

        log::info!("⚙️ [STATE] Starting workflow compilation");
        log::info!("  Instance ID: {}", self.instance_id);
        log::info!("  CBU ID: {}", self.cbu_id);

        // Parse inputs
        let products: Vec<String> = self.products_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        log::info!("  Products: {:?}", products);

        let team_users: Vec<serde_json::Value> = match serde_json::from_str(&self.team_users_input) {
            Ok(users) => users,
            Err(e) => {
                log::error!("❌ [STATE] Invalid team users JSON: {}", e);
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

        log::info!("  Team users: {} entries", team_users.len());

        let cbu_profile: serde_json::Value = match serde_json::from_str(&self.cbu_profile_input) {
            Ok(profile) => profile,
            Err(e) => {
                log::error!("❌ [STATE] Invalid CBU profile JSON: {}", e);
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

        log::info!("  CBU profile: {}", cbu_profile);

        let request = CompileWorkflowRequest {
            instance_id: self.instance_id.clone(),
            cbu_id: self.cbu_id.clone(),
            products,
            team_users,
            cbu_profile,
        };

        self.compiling = true;
        log::info!("📡 [STATE] Sending POST request to /api/onboarding/compile");

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
            let result = client.post_request::<_, CompileWorkflowResponse>("/api/onboarding/compile", &request).await;

            match result {
                Ok(response) => {
                    if response.success {
                        log::info!("✅ [STATE] Compilation successful");
                        if let Some(ref plan) = response.plan {
                            log::info!("  Plan received: {} bytes", serde_json::to_string(plan).map(|s| s.len()).unwrap_or(0));
                        }
                    } else {
                        log::error!("❌ [STATE] Compilation failed: {}", response.message);
                    }

                    if let Ok(mut state) = compile_state.lock() {
                        *state = Some(response);
                        log::info!("✅ [STATE] Compile result stored in state");
                    } else {
                        log::error!("❌ [STATE] Failed to acquire compile state lock");
                    }
                }
                Err(e) => {
                    log::error!("❌ [STATE] Compilation request failed: {}", e);
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Compilation error: {}", e));
                    }
                }
            }
        });
    }

    pub fn execute_workflow(&mut self) {
        if self.executing {
            log::warn!("⚠️ [STATE] Execute called but already executing - ignoring");
            return;
        }

        log::info!("▶️ [STATE] Starting workflow execution");

        let plan = match &self.compile_result {
            Some(result) if result.success => {
                match &result.plan {
                    Some(p) => {
                        log::info!("  Using compiled plan from previous compilation");
                        p.clone()
                    }
                    None => {
                        log::error!("❌ [STATE] No plan available in compile result");
                        return;
                    }
                }
            }
            _ => {
                log::error!("❌ [STATE] Cannot execute - no successful compilation result");
                return;
            }
        };

        let request = ExecuteWorkflowRequest { plan };

        self.executing = true;
        log::info!("📡 [STATE] Sending POST request to /api/onboarding/execute");

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
            let result = client.post_request::<_, ExecuteWorkflowResponse>("/api/onboarding/execute", &request).await;

            match result {
                Ok(response) => {
                    if response.success {
                        log::info!("✅ [STATE] Execution successful");
                        log::info!("  Execution log entries: {}", response.execution_log.len());
                    } else {
                        log::error!("❌ [STATE] Execution failed: {}", response.message);
                    }

                    if let Ok(mut state) = execute_state.lock() {
                        *state = Some(response);
                        log::info!("✅ [STATE] Execute result stored in state");
                    } else {
                        log::error!("❌ [STATE] Failed to acquire execute state lock");
                    }
                }
                Err(e) => {
                    log::error!("❌ [STATE] Execution request failed: {}", e);
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Execution error: {}", e));
                    }
                }
            }
        });
    }

    // === Create Onboarding Request ===

    pub fn create_onboarding_request(&mut self) {
        if self.creating_request {
            log::warn!("⚠️ [STATE] Create request called but already creating - ignoring");
            return;
        }

        log::info!("📝 [STATE] Creating new onboarding request");
        log::info!("  Name: {}", self.request_name);
        log::info!("  Description: {}", self.request_description);
        log::info!("  CBU ID: {}", self.request_cbu_id);

        self.creating_request = true;
        self.create_request_result = None;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                log::error!("❌ [STATE] No client available for create request");
                self.creating_request = false;
                return;
            }
        };

        let request = CreateRequestInput {
            name: self.request_name.clone(),
            description: if self.request_description.is_empty() {
                None
            } else {
                Some(self.request_description.clone())
            },
            cbu_id: if self.request_cbu_id.is_empty() {
                None
            } else {
                Some(self.request_cbu_id.clone())
            },
        };

        let create_request_state = Arc::new(Mutex::new(None));
        self.create_request_state = Some(create_request_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        log::info!("📡 [STATE] Sending POST request to /api/onboarding/requests");
        wasm_utils::spawn_async(async move {
            let result = client.post_request::<_, CreateRequestResponse>("/api/onboarding/requests", &request).await;

            match result {
                Ok(response) => {
                    if response.success {
                        log::info!("✅ [STATE] Onboarding request created successfully");
                        log::info!("  Onboarding ID: {:?}", response.onboarding_id);
                        log::info!("  Request ID: {:?}", response.request_id);
                    } else {
                        log::error!("❌ [STATE] Create request failed: {}", response.message);
                    }

                    if let Ok(mut state) = create_request_state.lock() {
                        *state = Some(response);
                        log::info!("✅ [STATE] Create result stored in state");
                    } else {
                        log::error!("❌ [STATE] Failed to acquire create request state lock");
                    }
                }
                Err(e) => {
                    log::error!("❌ [STATE] Create request failed: {}", e);
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Create request error: {}", e));
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
                    log::info!("🎉 [STATE] Metadata received from async task!");
                    self.metadata = Some(metadata.clone());
                    self.metadata_loading = false;
                    // Load initial content
                    self.current_content = metadata.product_catalog.clone();
                    log::info!("✅ [STATE] Metadata loaded into UI state");
                } else {
                    log::debug!("⏳ [STATE] Checking metadata state - not ready yet");
                }
            } else {
                log::warn!("⚠️ [STATE] Failed to lock metadata state in update_from_async");
            };
            // Don't clear metadata_state until we actually got data
            if self.metadata.is_some() {
                self.metadata_state = None; // Clear after successfully loading
            }
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

        // Check create request state
        if let Some(state) = &self.create_request_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(response) = guard.take() {
                    self.create_request_result = Some(response);
                    self.creating_request = false;
                }
            };
            self.create_request_state = None; // Clear after scope ends
        }

        // Check error state
        if let Some(state) = &self.error_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(error) = guard.take() {
                    self.metadata_error = Some(error);
                    self.metadata_loading = false;
                    self.compiling = false;
                    self.executing = false;
                    self.creating_request = false;
                }
            };
            self.error_state = None; // Clear after scope ends
        }
    }
}
