// CBU State Manager - Central state management for all CBU/Entity data
// Separates state from UI rendering for clean architecture

use crate::grpc_client::{GrpcClient, CbuRecord};
use crate::wasm_utils;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslResponse {
    pub success: bool,
    pub message: String,
    pub cbu_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CbuContext {
    None,        // Initial state - user needs to choose
    CreateNew,   // Creating a new CBU
    EditExisting, // Editing an existing CBU
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub entity_id: String,
    pub entity_name: String,
    pub entity_type: String,
    pub jurisdiction: String,
    pub country_code: String,
    pub lei_code: Option<String>,
    pub status: String,
}

/// Central state manager for all CBU/Entity operations
/// Pure state - no UI logic, just data and business operations
pub struct CbuStateManager {
    // ---- Core CBU State ----
    pub available_cbus: Vec<CbuRecord>,
    pub active_cbu_id: Option<String>,
    pub active_cbu_name: String,
    pub cbu_context: CbuContext,
    pub selected_cbu_id: Option<String>,

    // ---- Entity State ----
    pub available_entities: Vec<EntityInfo>,
    pub selected_entities: Vec<(String, String)>, // (entity_id, role)

    // ---- DSL Editor State ----
    pub dsl_script: String,
    pub syntax_errors: Vec<String>,

    // ---- Loading States (simple flags) ----
    pub loading_cbus: bool,
    pub loading_entities: bool,
    pub executing_dsl: bool,
    pub creating_cbu: bool,

    // ---- Results ----
    pub execution_result: Option<CbuDslResponse>,
    pub last_error: Option<String>,

    // ---- Internal (hidden from UI) ----
    grpc_client: Option<GrpcClient>,

    // Async state bridges (only used internally for async->sync updates)
    entities_loading_state: Option<Arc<Mutex<Vec<EntityInfo>>>>,
    cbus_loading_state: Option<Arc<Mutex<Vec<CbuRecord>>>>,
}

impl CbuStateManager {
    pub fn new(grpc_client: Option<GrpcClient>) -> Self {
        Self {
            available_cbus: Vec::new(),
            active_cbu_id: None,
            active_cbu_name: String::new(),
            cbu_context: CbuContext::None,
            selected_cbu_id: None,

            available_entities: Vec::new(),
            selected_entities: Vec::new(),

            dsl_script: String::new(),
            syntax_errors: Vec::new(),

            loading_cbus: false,
            loading_entities: false,
            executing_dsl: false,
            creating_cbu: false,

            execution_result: None,
            last_error: None,

            grpc_client,
            entities_loading_state: None,
            cbus_loading_state: None,
        }
    }

    // ============================================
    // PUBLIC API - UI calls these methods
    // ============================================

    /// Trigger async load of all CBUs from backend
    pub fn load_cbus(&mut self) {
        if self.loading_cbus || self.grpc_client.is_none() {
            return;
        }

        self.loading_cbus = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let cbus_state = Arc::new(Mutex::new(Vec::<CbuRecord>::new()));
        let cbus_clone = cbus_state.clone();

        // Store reference for polling (will be removed in next iteration)
        self.cbus_loading_state = Some(cbus_state);

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log("🔄 State Manager: Loading CBUs from gRPC API...");

            match client_clone.list_cbus(crate::grpc_client::ListCbusRequest {
                status_filter: None,
                limit: None,
                offset: None,
            }).await {
                Ok(response) => {
                    let mut cbus = cbus_clone.lock().unwrap();
                    *cbus = response.cbus;
                    wasm_utils::console_log(&format!("✅ State Manager: Loaded {} CBUs", cbus.len()));
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ State Manager: Failed to load CBUs: {}", e));
                }
            }
        });
    }

    /// Trigger async load of all entities from backend
    pub fn load_entities(&mut self) {
        if self.loading_entities || self.grpc_client.is_none() {
            return;
        }

        self.loading_entities = true;
        self.last_error = None;

        let client_clone = self.grpc_client.as_ref().unwrap().clone();
        let entities_state = Arc::new(Mutex::new(Vec::<EntityInfo>::new()));
        let entities_clone = entities_state.clone();

        // Store reference for polling (will be removed in next iteration)
        self.entities_loading_state = Some(entities_state);

        wasm_utils::spawn_async(async move {
            wasm_utils::console_log("🔄 State Manager: Loading entities from gRPC API...");

            let request = crate::grpc_client::GetEntitiesRequest {
                jurisdiction: None,
                entity_type: None,
                status: None,
            };

            match client_clone.get_entities(request).await {
                Ok(response) => {
                    let mut entities = entities_clone.lock().unwrap();

                    for entity in response.entities {
                        entities.push(EntityInfo {
                            entity_id: entity.entity_id,
                            entity_name: entity.entity_name,
                            jurisdiction: entity.jurisdiction,
                            entity_type: entity.entity_type,
                            country_code: entity.country_code,
                            lei_code: entity.lei_code,
                            status: entity.status,
                        });
                    }

                    wasm_utils::console_log(&format!("✅ State Manager: Loaded {} entities", entities.len()));
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ State Manager: Failed to load entities: {}", e));
                }
            }
        });
    }

    /// Set the active CBU context
    pub fn set_active_cbu(&mut self, cbu_id: String, cbu_name: String) {
        self.active_cbu_id = Some(cbu_id);
        self.active_cbu_name = cbu_name;
    }

    /// Update the DSL script content
    pub fn update_dsl_script(&mut self, content: String) {
        self.dsl_script = content;
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.dsl_script.clear();
        self.selected_entities.clear();
        self.syntax_errors.clear();
        self.execution_result = None;
        self.last_error = None;
    }

    /// Set the CBU context (CreateNew vs EditExisting)
    pub fn set_cbu_context(&mut self, context: CbuContext) {
        self.cbu_context = context;
    }

    // ============================================
    // INTERNAL - Async state polling (temporary)
    // ============================================

    /// Poll async state and update synchronous state
    /// Called every frame by UI to pick up async results
    /// TODO: Replace with proper callback mechanism
    pub fn update_from_async(&mut self) {
        // Update CBUs if loaded
        let should_clear_cbus = if let Some(loading_state) = &self.cbus_loading_state {
            if let Ok(cbus) = loading_state.try_lock() {
                if !cbus.is_empty() {
                    self.available_cbus = cbus.clone();
                    self.loading_cbus = false;
                    wasm_utils::console_log(&format!("🔄 State Manager: Updated {} CBUs from async", cbus.len()));
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
        if should_clear_cbus {
            self.cbus_loading_state = None;
        }

        // Update entities if loaded
        let should_clear_entities = if let Some(loading_state) = &self.entities_loading_state {
            if let Ok(entities) = loading_state.try_lock() {
                if !entities.is_empty() {
                    self.available_entities = entities.clone();
                    self.loading_entities = false;
                    wasm_utils::console_log(&format!("🔄 State Manager: Updated {} entities from async", entities.len()));
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
        if should_clear_entities {
            self.entities_loading_state = None;
        }
    }

    // ============================================
    // READ-ONLY GETTERS
    // ============================================

    pub fn is_loading(&self) -> bool {
        self.loading_cbus || self.loading_entities || self.executing_dsl || self.creating_cbu
    }

    pub fn get_available_cbus(&self) -> &[CbuRecord] {
        &self.available_cbus
    }

    pub fn get_available_entities(&self) -> &[EntityInfo] {
        &self.available_entities
    }

    pub fn get_grpc_client(&self) -> Option<&GrpcClient> {
        self.grpc_client.as_ref()
    }
}

impl Default for CbuStateManager {
    fn default() -> Self {
        Self::new(None)
    }
}
