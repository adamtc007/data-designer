// Resource State Manager - Central state management for all Resource/DSL data
// Separates state from UI rendering for clean architecture
// Parallel structure to CbuStateManager for Resource DSL operations

use crate::grpc_client::{GrpcClient, ResourceRecord};
use crate::wasm_utils;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDslResponse {
    pub success: bool,
    pub message: String,
    pub resource_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceContext {
    None,          // Initial state - user needs to choose
    CreateNew,     // Creating a new Resource DSL
    EditExisting,  // Editing an existing Resource DSL
}

/// Central state manager for all Resource/DSL operations
/// Pure state - no UI logic, just data and business operations
pub struct ResourceStateManager {
    // ---- Core Resource State ----
    pub available_resources: Vec<ResourceRecord>,
    pub active_resource_id: Option<String>,
    pub active_resource_name: String,
    pub resource_context: ResourceContext,
    pub selected_resource_id: Option<String>,

    // ---- DSL Editor State ----
    pub dsl_script: String,
    pub syntax_errors: Vec<String>,

    // ---- Loading States (simple flags) ----
    pub loading_resources: bool,
    pub executing_dsl: bool,
    pub creating_resource: bool,
    pub updating_dsl: bool,

    // ---- Results ----
    pub execution_result: Option<ResourceDslResponse>,
    pub last_error: Option<String>,

    // ---- Internal (hidden from UI) ----
    grpc_client: Option<GrpcClient>,

    // Async state bridges (only used internally for async->sync updates)
    resources_loading_state: Option<Arc<Mutex<Vec<ResourceRecord>>>>,
}

impl ResourceStateManager {
    pub fn new(grpc_client: Option<GrpcClient>) -> Self {
        Self {
            available_resources: Vec::new(),
            active_resource_id: None,
            active_resource_name: String::new(),
            resource_context: ResourceContext::None,
            selected_resource_id: None,

            dsl_script: String::new(),
            syntax_errors: Vec::new(),

            loading_resources: false,
            executing_dsl: false,
            creating_resource: false,
            updating_dsl: false,

            execution_result: None,
            last_error: None,

            grpc_client,
            resources_loading_state: None,
        }
    }

    // ============================================
    // PUBLIC API - UI calls these methods
    // ============================================

    /// Trigger async load of all Resources from backend
    pub fn load_resources(&mut self) {
        if self.loading_resources || self.grpc_client.is_none() {
            return;
        }

        self.loading_resources = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let resources_state = Arc::new(Mutex::new(Vec::<ResourceRecord>::new()));
        let resources_clone = resources_state.clone();

        // Store reference for polling (will be removed in next iteration)
        self.resources_loading_state = Some(resources_state);

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log("🔄 Resource State Manager: Loading Resources from gRPC API...");

            match client_clone.list_resources(crate::grpc_client::ListResourcesRequest {
                status_filter: Some("active".to_string()),
                resource_type_filter: None,
                limit: None,
                offset: None,
            }).await {
                Ok(response) => {
                    let mut resources = resources_clone.lock().unwrap();
                    *resources = response.resources;
                    wasm_utils::console_log(&format!("✅ Resource State Manager: Loaded {} Resources", resources.len()));
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Resource State Manager: Failed to load Resources: {}", e));
                }
            }
        });
    }

    /// Load DSL content for a specific Resource
    pub fn load_resource_dsl(&mut self, resource_id: String) {
        if self.grpc_client.is_none() {
            self.last_error = Some("No gRPC client available".to_string());
            return;
        }

        self.loading_resources = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let resource_id_clone = resource_id.clone();

        // For now, we'll just look it up from available_resources
        // In the future, this should make an API call to get_resource_dsl
        if let Some(resource) = self.available_resources.iter().find(|r| r.resource_id == resource_id) {
            self.active_resource_id = Some(resource_id.clone());
            self.active_resource_name = resource.resource_name.clone();
            self.dsl_script = resource.dsl_content.clone().unwrap_or_else(|| {
                // Generate default S-expression DSL template
                format!(
                    "(resource\n  (kind {})\n  (version 1)\n  (attributes\n    ;; Add attributes here\n  )\n  (provisioning\n    ;; Add provisioning endpoint\n  ))",
                    resource.resource_type
                )
            });
            self.resource_context = ResourceContext::EditExisting;
            self.loading_resources = false;

            wasm_utils::console_log(&format!("📝 Resource State Manager: Loaded DSL for Resource: {} ({})",
                                            resource.resource_name, resource.resource_id));
        } else {
            self.last_error = Some(format!("Resource not found: {}", resource_id));
            self.loading_resources = false;
        }
    }

    /// Save/Update DSL content for the active Resource
    pub fn save_resource_dsl(&mut self) {
        if self.grpc_client.is_none() {
            self.last_error = Some("No gRPC client available".to_string());
            return;
        }

        let Some(resource_id) = self.active_resource_id.clone() else {
            self.last_error = Some("No active resource selected".to_string());
            return;
        };

        self.updating_dsl = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let dsl_content = self.dsl_script.clone();

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log(&format!("💾 Resource State Manager: Saving DSL for Resource: {}", resource_id));

            match client_clone.update_resource_dsl(crate::grpc_client::UpdateResourceDslRequest {
                resource_id: resource_id.clone(),
                dsl_content,
                metadata: None,
            }).await {
                Ok(response) => {
                    if response.success {
                        wasm_utils::console_log(&format!("✅ Resource State Manager: Saved DSL for Resource: {}", resource_id));
                    } else {
                        wasm_utils::console_log(&format!("❌ Resource State Manager: Failed to save DSL: {}", response.message));
                    }
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Resource State Manager: Save failed: {}", e));
                }
            }
        });
    }

    /// Execute the current Resource DSL
    pub fn execute_dsl(&mut self) {
        if self.grpc_client.is_none() {
            self.last_error = Some("No gRPC client available".to_string());
            return;
        }

        let Some(resource_id) = self.active_resource_id.clone() else {
            self.last_error = Some("No active resource selected".to_string());
            return;
        };

        self.executing_dsl = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let dsl_script = self.dsl_script.clone();

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log(&format!("⚙️ Resource State Manager: Executing DSL for Resource: {}", resource_id));

            match client_clone.execute_resource_dsl(crate::grpc_client::ExecuteResourceDslRequest {
                resource_id: resource_id.clone(),
                dsl_script,
                context: None,
            }).await {
                Ok(response) => {
                    if response.success {
                        wasm_utils::console_log(&format!("✅ Resource State Manager: DSL execution succeeded for Resource: {}", resource_id));
                    } else {
                        wasm_utils::console_log(&format!("⚠️ Resource State Manager: DSL execution completed with validation errors"));
                    }
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Resource State Manager: DSL execution failed: {}", e));
                }
            }
        });
    }

    /// Set the active Resource context
    pub fn set_active_resource(&mut self, resource_id: String, resource_name: String) {
        self.active_resource_id = Some(resource_id);
        self.active_resource_name = resource_name;
    }

    /// Update the DSL script content
    pub fn update_dsl_script(&mut self, content: String) {
        self.dsl_script = content;
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.dsl_script.clear();
        self.syntax_errors.clear();
        self.execution_result = None;
        self.last_error = None;
    }

    /// Set the Resource context (CreateNew vs EditExisting)
    pub fn set_resource_context(&mut self, context: ResourceContext) {
        self.resource_context = context;
    }

    // ============================================
    // INTERNAL - Async state polling (temporary)
    // ============================================

    /// Poll async state and update synchronous state
    /// Called every frame by UI to pick up async results
    /// TODO: Replace with proper callback mechanism
    pub fn update_from_async(&mut self) {
        // Update Resources if loaded
        let should_clear_resources = if let Some(loading_state) = &self.resources_loading_state {
            if let Ok(resources) = loading_state.try_lock() {
                if !resources.is_empty() {
                    self.available_resources = resources.clone();
                    self.loading_resources = false;
                    wasm_utils::console_log(&format!("🔄 Resource State Manager: Updated {} Resources from async", resources.len()));
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        if should_clear_resources {
            self.resources_loading_state = None;
        }

        // Check async operations completion
        if self.updating_dsl || self.executing_dsl || self.creating_resource {
            // These would be tracked by additional async state bridges
            // For now, just reset flags after a reasonable delay
            // TODO: Implement proper async result tracking
        }
    }

    // ============================================
    // READ-ONLY GETTERS
    // ============================================

    pub fn is_loading(&self) -> bool {
        self.loading_resources || self.executing_dsl || self.creating_resource || self.updating_dsl
    }

    pub fn get_available_resources(&self) -> &[ResourceRecord] {
        &self.available_resources
    }

    pub fn get_grpc_client(&self) -> Option<&GrpcClient> {
        self.grpc_client.as_ref()
    }
}

impl Default for ResourceStateManager {
    fn default() -> Self {
        Self::new(None)
    }
}
