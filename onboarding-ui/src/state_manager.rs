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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct OnboardingRequestRecord {
    pub id: i32,
    pub onboarding_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub cbu_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub dsl_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used by app.rs
pub struct OnboardingRequestDslRecord {
    pub id: i32,
    pub onboarding_request_id: i32,
    pub instance_id: Option<String>,
    pub products: Option<Vec<String>>,
    pub team_users: Option<serde_json::Value>,
    pub cbu_profile: Option<serde_json::Value>,
    pub template_version: Option<String>,
    pub dsl_content: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used internally
pub struct DbRecordsResponse {
    pub request_record: OnboardingRequestRecord,
    pub dsl_record: Option<OnboardingRequestDslRecord>,
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
    pub create_request_timeout_counter: u32,

    // Database records (fetched after successful create)
    pub db_request_record: Option<OnboardingRequestRecord>,
    pub db_dsl_record: Option<OnboardingRequestDslRecord>,
    pub loading_db_records: bool,

    // Current onboarding request (if loaded)
    pub current_onboarding_id: Option<String>,

    // List of all onboarding requests
    pub onboarding_requests: Vec<OnboardingRequestRecord>,

    // Editing state
    pub editing_request: Option<OnboardingRequestRecord>,
    pub editing_dsl_content: String,
    pub edit_mode: bool,

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
    metadata_state: Option<Arc<Mutex<Option<Vec<OnboardingRequestRecord>>>>>,
    compile_state: Option<Arc<Mutex<Option<CompileWorkflowResponse>>>>,
    execute_state: Option<Arc<Mutex<Option<ExecuteWorkflowResponse>>>>,
    create_request_state: Option<Arc<Mutex<Option<CreateRequestResponse>>>>,
    db_records_state: Option<Arc<Mutex<Option<DbRecordsResponse>>>>,
    dsl_content_state: Option<Arc<Mutex<Option<String>>>>,
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
            create_request_timeout_counter: 0,
            db_request_record: None,
            db_dsl_record: None,
            loading_db_records: false,
            current_onboarding_id: None,
            onboarding_requests: Vec::new(),
            editing_request: None,
            editing_dsl_content: String::new(),
            edit_mode: false,
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
            db_records_state: None,
            dsl_content_state: None,
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

        let metadata_state: Arc<Mutex<Option<Vec<OnboardingRequestRecord>>>> = Arc::new(Mutex::new(None));
        self.metadata_state = Some(metadata_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        wasm_utils::console_log("🚀 [UI-TRACE] Starting metadata load operation");
        log::info!("📡 [STATE] Sending gRPC request for onboarding metadata");
        wasm_utils::spawn_async(async move {
            wasm_utils::console_log("🔄 [UI-TRACE] Calling client.list_onboarding_requests() for metadata");
            let empty_request = serde_json::json!({});
            let result = client.list_onboarding_requests::<_, Vec<OnboardingRequestRecord>>(empty_request).await;

            match result {
                Ok(requests) => {
                    log::info!("✅ [STATE] Onboarding requests loaded from server");
                    log::info!("  Found {} onboarding requests", requests.len());
                    for req in &requests {
                        log::info!("    - {} ({}): {}", req.onboarding_id, req.status, req.name.as_ref().unwrap_or(&"No name".to_string()));
                    }

                    if let Ok(mut state) = metadata_state.lock() {
                        *state = Some(requests);
                        log::info!("✅ [STATE] Onboarding requests stored in state successfully");
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
            let result = client.compile_onboarding_workflow::<_, CompileWorkflowResponse>(request).await;

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
            let result = client.execute_onboarding_workflow::<_, ExecuteWorkflowResponse>(request).await;

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
        // Clear previous database records
        self.db_request_record = None;
        self.db_dsl_record = None;

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

        let create_request_state = Arc::new(Mutex::new(None::<CreateRequestResponse>));
        let response_state_clone = create_request_state.clone();

        // Store reference for polling (will be removed in next iteration)
        self.create_request_state = Some(create_request_state);
        self.create_request_timeout_counter = 0;

        wasm_utils::console_log("🚀 [UI-TRACE] Starting create request operation");
        log::info!("📡 [STATE] Sending gRPC request for onboarding request creation");
        wasm_utils::spawn_async(async move {
            wasm_utils::console_log("🔄 [UI-TRACE] Calling client.create_onboarding_request()");

            match client.create_onboarding_request::<_, CreateRequestResponse>(request).await {
                Ok(response) => {
                    wasm_utils::console_log("✅ [UI-TRACE] Create request HTTP call succeeded");
                    if response.success {
                        log::info!("✅ [STATE] Onboarding request created successfully");
                        log::info!("  Onboarding ID: {:?}", response.onboarding_id);
                        log::info!("  Request ID: {:?}", response.request_id);
                    } else {
                        log::error!("❌ [STATE] Create request failed: {}", response.message);
                    }

                    let mut state = response_state_clone.lock().unwrap();
                    *state = Some(response);
                    wasm_utils::console_log("✅ [UI-TRACE] Create result stored in create_request_state");
                }
                Err(e) => {
                    log::error!("❌ [STATE] Create request failed: {}", e);
                    wasm_utils::console_log(&format!("❌ [UI-TRACE] Create request HTTP call failed: {}", e));

                    // Store error response
                    let mut state = response_state_clone.lock().unwrap();
                    *state = Some(CreateRequestResponse {
                        success: false,
                        message: format!("HTTP error: {}", e),
                        onboarding_id: None,
                        request_id: None,
                    });
                    wasm_utils::console_log("✅ [UI-TRACE] Error response stored in create_request_state");
                }
            }
        });
    }

    // === Database Records Loading ===

    pub fn load_database_records(&mut self, onboarding_id: String) {
        if self.loading_db_records {
            log::warn!("⚠️ [STATE] Load database records called but already loading - ignoring");
            return;
        }

        log::info!("📊 [STATE] Loading database records for onboarding_id: {}", onboarding_id);
        self.loading_db_records = true;
        self.current_onboarding_id = Some(onboarding_id.clone());

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                log::error!("❌ [STATE] No client available for database records load");
                self.loading_db_records = false;
                return;
            }
        };

        let db_records_state = Arc::new(Mutex::new(None));
        self.db_records_state = Some(db_records_state.clone());

        let error_state = Arc::new(Mutex::new(None));
        self.error_state = Some(error_state.clone());

        wasm_utils::console_log(&format!("🚀 [UI-TRACE] Starting DB records load operation for ID: {}", onboarding_id));
        log::info!("📡 [STATE] Sending gRPC request for database records");

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log(&format!("🔄 [UI-TRACE] Calling client.get_onboarding_request({})", onboarding_id));
            let request = serde_json::json!({
                "onboarding_id": onboarding_id
            });
            let result = client.get_onboarding_request::<_, serde_json::Value>(request).await;

            match result {
                Ok(response) => {
                    log::info!("✅ [STATE] Database records loaded successfully");

                    // Parse the response which contains request_record and dsl_record
                    if let Some(request_record_json) = response.get("request_record") {
                        if let Ok(request_record) = serde_json::from_value::<OnboardingRequestRecord>(request_record_json.clone()) {
                            log::info!("  Request record ID: {}", request_record.id);

                            let dsl_record = response.get("dsl_record")
                                .and_then(|dsl_json| serde_json::from_value::<OnboardingRequestDslRecord>(dsl_json.clone()).ok());

                            if let Some(ref dsl_record) = dsl_record {
                                log::info!("  DSL record ID: {}", dsl_record.id);
                            } else {
                                log::info!("  No DSL record found (this is normal for new requests)");
                            }

                            let db_response = DbRecordsResponse {
                                request_record,
                                dsl_record,
                            };

                            if let Ok(mut state) = db_records_state.lock() {
                                *state = Some(db_response);
                                log::info!("✅ [STATE] Database records stored in state bridge");
                            } else {
                                log::error!("❌ [STATE] Failed to acquire database records state lock");
                            }
                        } else {
                            log::error!("❌ [STATE] Failed to parse request record from response");
                            if let Ok(mut state) = error_state.lock() {
                                *state = Some("Failed to parse request record from response".to_string());
                            }
                        }
                    } else {
                        log::error!("❌ [STATE] No request record found in response");
                        if let Ok(mut state) = error_state.lock() {
                            *state = Some("No request record found in response".to_string());
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ [STATE] Failed to load database records: {}", e);
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Failed to load database records: {}", e));
                    }
                }
            }
        });
    }

    // === Async Result Processing ===

    pub fn update_from_async(&mut self) {
        // Check onboarding requests loading state
        if let Some(state) = &self.metadata_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(requests) = guard.take() {
                    log::info!("🎉 [STATE] Onboarding requests received from async task!");
                    log::info!("  Loaded {} onboarding requests", requests.len());
                    self.onboarding_requests = requests;
                    self.metadata_loading = false;
                    log::info!("✅ [STATE] Onboarding requests loaded into UI state");
                } else {
                    log::debug!("⏳ [STATE] Checking onboarding requests state - not ready yet");
                }
            } else {
                log::warn!("⚠️ [STATE] Failed to lock onboarding requests state in update_from_async");
            };
            // Don't clear metadata_state until we actually got data
            if !self.onboarding_requests.is_empty() {
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

        // Check create request state (using CBU designer pattern)
        let onboarding_id_to_load: Option<String> = None;
        let should_clear_create_state = if let Some(loading_state) = &self.create_request_state {
            if let Ok(response_option) = loading_state.try_lock() {
                if response_option.is_some() {
                    let response = response_option.clone().unwrap();
                    log::info!("🎉 [STATE] Create request response received from async task!");
                    wasm_utils::console_log("🎉 [UI-TRACE] Create request response received, resetting creating_request to false");

                    // If create succeeded, we could auto-load database records, but for now skip it to avoid complexity
                    if response.success && response.onboarding_id.is_some() {
                        if let Some(onboarding_id) = &response.onboarding_id {
                            log::info!("✅ [STATE] Create succeeded for onboarding_id: {} (skipping auto-load of DB records)", onboarding_id);
                            // onboarding_id_to_load = Some(onboarding_id.clone()); // Disabled for now
                        }
                    }
                    self.create_request_result = Some(response);
                    self.creating_request = false;
                    wasm_utils::console_log("🔄 State Manager: Updated create request from async");
                    true
                } else {
                    // Increment timeout counter
                    self.create_request_timeout_counter += 1;
                    if self.create_request_timeout_counter > 300 { // 5 seconds at 60fps
                        log::warn!("⚠️ [STATE] Create request timed out after 5 seconds, resetting state");
                        wasm_utils::console_log("⏳ [UI-TRACE] Create request timed out, resetting creating_request to false");
                        self.creating_request = false;
                        self.create_request_result = Some(CreateRequestResponse {
                            success: false,
                            message: "Request timed out".to_string(),
                            onboarding_id: None,
                            request_id: None,
                        });
                        true
                    } else {
                        false
                    }
                }
            } else {
                log::warn!("⚠️ [STATE] Failed to acquire create request state lock");
                false
            }
        } else {
            false
        };
        if should_clear_create_state {
            self.create_request_state = None;
            self.create_request_timeout_counter = 0;
        }

        // Load database records if needed (after releasing the borrow)
        if let Some(onboarding_id) = onboarding_id_to_load {
            self.load_database_records(onboarding_id);
        }

        // Check database records state (using CBU designer pattern)
        let should_clear_db_state = if let Some(loading_state) = &self.db_records_state {
            if let Ok(db_records_option) = loading_state.try_lock() {
                if db_records_option.is_some() {
                    let db_records = db_records_option.clone().unwrap();
                    log::info!("🎉 [STATE] Database records received from async task!");
                    self.db_request_record = Some(db_records.request_record);
                    self.db_dsl_record = db_records.dsl_record; // Handle optional DSL record
                    self.loading_db_records = false;
                    log::info!("✅ [STATE] Database records loaded into UI state");
                    if self.db_dsl_record.is_some() {
                        log::info!("  DSL record found and loaded");
                    } else {
                        log::info!("  No DSL record found (this is normal for new requests)");
                    }
                    wasm_utils::console_log("🔄 State Manager: Updated DB records from async");
                    true
                } else {
                    // No DB records yet, timeout not needed since it's disabled by default
                    false
                }
            } else {
                log::warn!("⚠️ [STATE] Failed to acquire database records state lock");
                // Reset loading flag if we can't even lock the state
                self.loading_db_records = false;
                wasm_utils::console_log("⚠️ [UI-TRACE] Failed to lock DB records state, resetting loading_db_records to false");
                true
            }
        } else {
            false
        };
        if should_clear_db_state {
            self.db_records_state = None;
        }

        // Check DSL content loading state
        if let Some(state) = &self.dsl_content_state {
            if let Ok(mut guard) = state.lock() {
                if let Some(dsl_content) = guard.take() {
                    log::info!("✅ [EDIT] DSL content loaded for editing ({} chars)", dsl_content.len());
                    self.editing_dsl_content = dsl_content;
                }
            };
            self.dsl_content_state = None; // Clear after scope ends
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

    // === Editing Operations ===

    /// Start editing an onboarding request
    pub fn start_editing(&mut self, request: &OnboardingRequestRecord) {
        self.editing_request = Some(request.clone());
        self.edit_mode = true;
        self.editing_dsl_content = String::new(); // Will be loaded from server

        // Load the DSL content for this request
        self.load_dsl_content_for_editing(&request.onboarding_id);
    }

    /// Cancel editing mode
    pub fn cancel_editing(&mut self) {
        self.editing_request = None;
        self.edit_mode = false;
        self.editing_dsl_content.clear();
    }

    /// Save the edited request
    pub fn save_editing(&mut self) {
        if let Some(request) = &self.editing_request {
            log::info!("💾 [EDIT] Saving changes for {}", request.onboarding_id);

            let client = match &self.client {
                Some(c) => c.clone(),
                None => {
                    log::error!("❌ [EDIT] No client available for save");
                    return;
                }
            };

            let onboarding_id = request.onboarding_id.clone();
            let dsl_content = self.editing_dsl_content.clone();

            log::info!("📡 [EDIT] Sending DSL update to server ({} chars)", dsl_content.len());

            // Create async task for saving
            wasm_utils::spawn_async(async move {
                let save_request = serde_json::json!({
                    "onboarding_id": onboarding_id,
                    "dsl_content": dsl_content
                });

                match client.update_onboarding_request_dsl::<_, serde_json::Value>(save_request).await {
                    Ok(response) => {
                        if let Some(success) = response.get("success").and_then(|v| v.as_bool()) {
                            if success {
                                log::info!("✅ [EDIT] DSL content saved successfully for {}", onboarding_id);
                            } else {
                                log::error!("❌ [EDIT] Server reported save failure for {}", onboarding_id);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("❌ [EDIT] Failed to save DSL content: {:?}", e);
                    }
                }
            });

            // Exit edit mode immediately (don't wait for async save)
            self.cancel_editing();

            // Refresh the onboarding requests list to show updated data
            log::info!("🔄 [EDIT] Refreshing onboarding requests list after save");
            self.load_metadata();
        }
    }

    /// Load DSL content for editing
    fn load_dsl_content_for_editing(&mut self, onboarding_id: &str) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                log::error!("❌ [EDIT] No client available for DSL content load");
                return;
            }
        };

        let onboarding_id = onboarding_id.to_string();
        log::info!("🔍 [EDIT] Loading DSL content for editing: {}", onboarding_id);

        // Create state for tracking the async result
        let dsl_content_state = Arc::new(Mutex::new(None::<String>));
        let error_state = Arc::new(Mutex::new(None::<String>));

        // Store references for update_from_async to poll
        self.dsl_content_state = Some(dsl_content_state.clone());
        self.error_state = Some(error_state.clone());

        wasm_utils::spawn_async(async move {
            log::info!("📡 [EDIT] Fetching DSL content from server for: {}", onboarding_id);

            // Use the gRPC GetOnboardingRequest endpoint
            let request = serde_json::json!({
                "onboarding_id": onboarding_id
            });

            match client.get_onboarding_request::<_, serde_json::Value>(request).await {
                Ok(response) => {
                    log::info!("✅ [EDIT] GetOnboardingRequest response received");

                    // Extract DSL content from the response
                    if let Some(dsl_record) = response.get("dsl_record") {
                        if let Some(dsl_content) = dsl_record.get("dsl_content").and_then(|v| v.as_str()) {
                            log::info!("✅ [EDIT] DSL content loaded successfully ({} chars)", dsl_content.len());
                            if let Ok(mut state) = dsl_content_state.lock() {
                                *state = Some(dsl_content.to_string());
                            }
                        } else {
                            log::warn!("⚠️ [EDIT] No DSL content found in response for: {}", onboarding_id);
                            let default_dsl = format!(
                                r#"# Onboarding DSL for {}
# No existing DSL content found - this is a new template

onboard_request {{
    name = "New Request"
    description = "Generated template"
    cbu_id = "CBU-001"
    status = "draft"

    team {{
        admin_email = "ops.admin@client.com"
        approver_email = "ops.approver@client.com"
    }}

    cbu_profile {{
        region = "EU"
        compliance_level = "standard"
    }}

    products = []

    workflow {{
        auto_approve = false
        require_compliance_check = true
        notification_enabled = true
    }}
}}
"#,
                                onboarding_id
                            );
                            if let Ok(mut state) = dsl_content_state.lock() {
                                *state = Some(default_dsl);
                            }
                        }
                    } else {
                        log::warn!("⚠️ [EDIT] No DSL record found in response for: {}", onboarding_id);
                        if let Ok(mut state) = error_state.lock() {
                            *state = Some(format!("No DSL record found for: {}", onboarding_id));
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ [EDIT] Failed to load DSL content: {:?}", e);
                    if let Ok(mut state) = error_state.lock() {
                        *state = Some(format!("Failed to load DSL content: {:?}", e));
                    }
                }
            }
        });
    }
}
