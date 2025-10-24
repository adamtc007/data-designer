// CBU DSL IDE - Interactive panel for writing and executing CBU CRUD operations
use eframe::egui;
use crate::grpc_client::CbuRecord;
use crate::wasm_utils;
use crate::dsl_syntax_highlighter::{DslSyntaxHighlighter, SyntaxTheme};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

// Import types from state manager (when available for WASM)
#[cfg(not(target_arch = "wasm32"))]
use crate::cbu_state_manager::{CbuDslResponse, CbuContext, EntityInfo, CbuStateManager};

// For WASM builds, define locally until module loading is fixed
#[cfg(target_arch = "wasm32")]
type CbuStateManager = crate::cbu_state_manager::CbuStateManager;
#[cfg(target_arch = "wasm32")]
type CbuContext = crate::cbu_state_manager::CbuContext;
#[cfg(target_arch = "wasm32")]
type EntityInfo = crate::cbu_state_manager::EntityInfo;
#[cfg(target_arch = "wasm32")]
type CbuDslResponse = crate::cbu_state_manager::CbuDslResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslRequest {
    pub dsl_script: String,
}

pub struct CbuDslIDE {
    // ========================================
    // UI-ONLY STATE (no business logic here)
    // ========================================

    // DSL Editor UI state
    syntax_highlighter: DslSyntaxHighlighter,
    show_syntax_highlighting: bool,

    // Code completion UI
    show_completion_popup: bool,
    completion_suggestions: Vec<String>,
    selected_completion: usize,
    completion_trigger_pos: usize,

    // Legacy execution state (TODO: move to state manager)
    executing: Arc<AtomicBool>,
    execution_result: Arc<Mutex<Option<CbuDslResponse>>>,

    // UI toggles
    show_examples: bool,
    show_help: bool,
    selected_example: usize,

    // Entity picker UI state
    show_entity_picker: bool,
    show_floating_entity_picker: bool,
    entity_picker_window_size: egui::Vec2,
    entity_picker_first_open: bool,
    entity_filter_jurisdiction: String,
    entity_filter_type: String,
    entity_search_name: String,

    // DSL Format Mode
    lisp_mode: bool,

    // New CBU creation UI
    new_cbu_name: String,

    // ========================================
    // ALL BUSINESS STATE REMOVED - NOW IN CbuStateManager:
    // - available_entities (now state.available_entities)
    // - available_cbus (now state.available_cbus)
    // - loading_entities (now state.loading_entities)
    // - loading_cbus (now state.loading_cbus)
    // - cbu_context (now state.cbu_context)
    // - selected_cbu_id (now state.selected_cbu_id)
    // - active_cbu_id (now state.active_cbu_id)
    // - active_cbu_name (now state.active_cbu_name)
    // - dsl_script (now state.dsl_script)
    // - syntax_errors (now state.syntax_errors)
    // - selected_entities (now state.selected_entities)
    // - creating_cbu (now state.creating_cbu)
    // - entities_loading_state (now internal to state manager)
    // - cbus_loading_state (now internal to state manager)
    // ========================================
}

// CbuRecord is now imported from grpc_client module
// CbuContext and EntityInfo are imported from cbu_state_manager at top of file (lines 11-22)

impl Default for CbuDslIDE {
    fn default() -> Self {
        Self::new()
    }
}

// DSL Operation types for single management function
#[derive(Debug, Clone)]
enum DslOperation {
    LoadForCreateNew { cbu_key: String },
    UpdateWithEntities { preserve_header: bool },
    Clear,
}

impl CbuDslIDE {
    pub fn new() -> Self {
        Self {
            // DSL Editor UI state
            syntax_highlighter: DslSyntaxHighlighter::new(SyntaxTheme::dark_theme()),
            show_syntax_highlighting: true,

            // Code completion UI
            show_completion_popup: false,
            completion_suggestions: Vec::new(),
            selected_completion: 0,
            completion_trigger_pos: 0,

            // Legacy execution state
            executing: Arc::new(AtomicBool::new(false)),
            execution_result: Arc::new(Mutex::new(None)),

            // UI toggles
            show_examples: false,
            show_help: false,
            selected_example: 0,

            // Entity picker UI state
            show_entity_picker: false,
            show_floating_entity_picker: false,
            entity_picker_window_size: egui::Vec2::new(720.0, 420.0),
            entity_picker_first_open: true,
            entity_filter_jurisdiction: "All".to_string(),
            entity_filter_type: "All".to_string(),
            entity_search_name: String::new(),

            // DSL Format Mode
            lisp_mode: true, // Default to LISP mode for better parsing

            // New CBU creation UI
            new_cbu_name: String::new(),
        }
    }

    /// **CBU DSL CONTEXT MANAGER** - Handles switching between create new and edit existing states
    fn start_create_new_cbu(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        wasm_utils::console_log("🆕 Starting Create New CBU flow");
        state.cbu_context = CbuContext::CreateNew;
        self.new_cbu_name.clear();
        state.dsl_script.clear();
        state.active_cbu_id = None;
        state.active_cbu_name.clear();

        // Leave DSL editor empty - user should start fresh
    }

    fn start_edit_existing_cbu(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        wasm_utils::console_log("📝 Starting Edit Existing CBU flow");
        state.cbu_context = CbuContext::EditExisting;
        self.new_cbu_name.clear();
        state.dsl_script.clear();
        state.active_cbu_id = None;
        state.active_cbu_name.clear();

        // Always refresh CBU list when entering edit mode
        self.refresh_cbu_list(state);
    }

    fn set_active_cbu(&mut self, cbu_id: String, cbu_name: String, state: &mut crate::cbu_state_manager::CbuStateManager) {
        wasm_utils::console_log(&format!("🎯 Setting active CBU: {} ({})", cbu_name, cbu_id));
        state.active_cbu_id = Some(cbu_id.clone());
        state.active_cbu_name = cbu_name;

        // Load DSL for this CBU
        self.load_cbu_dsl(&cbu_id, state);
    }

    fn reset_context(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        wasm_utils::console_log("🔄 Resetting CBU DSL context");
        state.cbu_context = CbuContext::None;
        self.new_cbu_name.clear();
        state.creating_cbu = false;
        state.dsl_script.clear();
        state.active_cbu_id = None;
        state.active_cbu_name.clear();
        state.selected_cbu_id = None;
    }

    /// Refresh CBU list from gRPC (force reload)
    fn refresh_cbu_list(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        wasm_utils::console_log("🔄 Refreshing CBU list from gRPC");
        // Clear existing state to force fresh load and reload via state manager
        state.load_cbus();
    }

    /// Create new CBU via gRPC and set up DSL context
    fn create_new_cbu(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        let client_clone = match state.get_grpc_client() {
            Some(client) => client.clone(),
            None => {
                wasm_utils::console_log("❌ No gRPC client available for CBU creation");
                return;
            }
        };

        if self.new_cbu_name.trim().is_empty() {
            wasm_utils::console_log("❌ CBU name cannot be empty");
            return;
        }

        state.creating_cbu = true;
        let cbu_name = self.new_cbu_name.trim().to_string();
        let cbu_id = format!("cbu_{}", wasm_utils::now_timestamp()); // Generate unique ID

        wasm_utils::console_log(&format!("🔨 Creating new CBU: {}", cbu_name));

        // Build simple LISP DSL for CBU creation (no legal entities)
        let dsl_content = format!(
            "; CBU Creation LISP DSL - Generated {}\n; CBU: {} ({})\n\n(define-cbu \"{}\" \"{}\")\n\n; Ready for additional entities via Entity Picker",
            wasm_utils::now_iso_string(),
            cbu_name,
            cbu_id,
            cbu_id,
            cbu_name
        );

        wasm_utils::spawn_async(async move {
            let request = crate::grpc_client::CreateCbuRequest {
                cbu_id: cbu_id.clone(),
                cbu_name: cbu_name.clone(),
                description: Some(format!("CBU created via DSL IDE: {}", cbu_name)),
                legal_entity_name: None,
                business_model: Some("Standard".to_string()),
                status: "active".to_string(),
            };

            match client_clone.create_cbu(request).await {
                Ok(response) => {
                    if response.success {
                        if let Some(cbu) = response.cbu {
                            wasm_utils::console_log(&format!("✅ Created CBU: {} with ID: {}", cbu.cbu_name, cbu.cbu_id));

                            // Now send the DSL content to create DSL metadata
                            let dsl_request = crate::grpc_client::ExecuteCbuDslRequest {
                                dsl_script: dsl_content.clone(),
                            };

                            match client_clone.execute_cbu_dsl(dsl_request).await {
                                Ok(dsl_response) => {
                                    if dsl_response.success {
                                        wasm_utils::console_log(&format!("✅ Created DSL metadata for CBU: {}", cbu.cbu_name));

                                        // Store the completed CBU and DSL for UI to pick up (WASM only)
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            let window = web_sys::window().unwrap();
                                            let storage = window.local_storage().unwrap().unwrap();
                                            let _ = storage.set_item("data_designer_new_cbu_created", &serde_json::to_string(&cbu).unwrap_or_default());
                                            let _ = storage.set_item("data_designer_new_cbu_dsl", &dsl_content);
                                            let _ = storage.set_item("data_designer_new_cbu_name", &cbu.cbu_name);
                                            let _ = storage.set_item("data_designer_cbu_creation_complete", "true");
                                        }
                                    } else {
                                        wasm_utils::console_log(&format!("❌ DSL execution failed: {}", dsl_response.message));
                                    }
                                }
                                Err(e) => {
                                    wasm_utils::console_log(&format!("❌ Error executing DSL: {}", e));
                                }
                            }
                        } else {
                            wasm_utils::console_log("❌ CreateCbu response success but no CBU data");
                        }
                    } else {
                        wasm_utils::console_log(&format!("❌ CreateCbu failed: {}", response.message));
                    }
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Error creating CBU: {}", e));
                }
            }
        });
    }

    /// **SINGLE DSL MANAGEMENT FUNCTION** - All DSL state changes must go through here
    /// This prevents multiple injection points and maintains clean state management
    /// Includes EBNF validation for all DSL operations
    fn manage_dsl_state(&mut self, operation: DslOperation, state: &mut crate::cbu_state_manager::CbuStateManager) {
        let new_dsl = match operation {
            DslOperation::LoadForCreateNew { cbu_key } => {
                state.selected_cbu_id = None;
                state.selected_entities.clear();

                // Generate LISP format for new CBU creation - ALWAYS LISP
                let dsl = "; Create a new CBU - add entities below using Entity Picker\n(create-cbu \"New CBU Name\" \"CBU Purpose Description\")\n; Use Entity Picker to add entities".to_string();

                wasm_utils::console_log(&format!("📝 DSL initialized for CREATE NEW: {}", cbu_key));
                dsl
            },
            DslOperation::UpdateWithEntities { preserve_header } => {
                if state.selected_entities.is_empty() {
                    wasm_utils::console_log("⚠️  No entities selected for DSL update");
                    return;
                }

                let dsl = if self.lisp_mode {
                    // Generate LISP-style DSL
                    wasm_utils::console_log("🔧 Generating LISP-style DSL");
                    self.build_lisp_dsl(state)
                } else if preserve_header {
                    // Preserve existing header and update entities (EBNF format)
                    self.build_dsl_preserving_header(state)
                } else {
                    // Generate completely new DSL (EBNF format)
                    self.build_dsl_from_scratch(state)
                };
                wasm_utils::console_log(&format!("✅ DSL updated with {} entities", state.selected_entities.len()));
                dsl
            },
            DslOperation::Clear => {
                state.selected_entities.clear();
                state.selected_cbu_id = None;
                wasm_utils::console_log("🧹 DSL state cleared");
                String::new()
            },
        };

        // **EBNF VALIDATION** - Validate DSL syntax before applying changes
        if !new_dsl.is_empty() {
            match self.validate_dsl_syntax(&new_dsl) {
                Ok(_) => {
                    state.dsl_script = new_dsl;
                    wasm_utils::console_log("✅ DSL syntax validation passed");
                },
                Err(validation_error) => {
                    wasm_utils::console_log(&format!("❌ DSL syntax validation failed: {}", validation_error));
                    // Still apply the DSL but log the validation error
                    // This allows the user to see and fix syntax issues
                    state.dsl_script = new_dsl;
                    // TODO: Show validation error in UI
                }
            }
        } else {
            state.dsl_script = new_dsl;
        }
    }

    /// Validate DSL syntax against CBU EBNF grammar
    /// Returns Ok(()) if valid, Err(error_message) if invalid
    fn validate_dsl_syntax(&self, dsl: &str) -> Result<(), String> {
        // Skip validation for incomplete DSL (just comments or placeholders)
        let non_comment_lines: Vec<&str> = dsl.lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .collect();

        if non_comment_lines.is_empty() {
            return Ok(()); // Empty DSL is valid
        }

        // Check for basic s-expression structure
        let dsl_text = dsl.trim();

        // Validate s-expression syntax
        if !dsl_text.is_empty() && !self.validate_s_expression_syntax(dsl_text) {
            return Err("S-expression syntax error: Expected valid LISP format starting with '(' and ending with ')'".to_string());
        }

        Ok(())
    }

    /// Validate s-expression DSL syntax
    fn validate_s_expression_syntax(&self, dsl: &str) -> bool {
        let trimmed = dsl.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with(';') {
            return true;
        }

        // Basic s-expression validation: must start with '(' and have balanced parentheses
        if !trimmed.starts_with('(') {
            return false;
        }

        // Check for balanced parentheses
        let mut paren_count = 0;
        for ch in trimmed.chars() {
            match ch {
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        return false; // More closing than opening
                    }
                }
                _ => {}
            }
        }

        // Must have balanced parentheses
        paren_count == 0
    }

    /// Helper: Build DSL preserving existing header (used by single DSL manager)
    fn build_dsl_preserving_header(&self, state: &crate::cbu_state_manager::CbuStateManager) -> String {
        // Parse existing DSL to preserve CBU-level information
        let existing_lines: Vec<&str> = state.dsl_script.lines().collect();
        let mut cbu_header_lines = Vec::new();
        let mut found_with = false;

        // Extract CBU header lines (before WITH clause)
        for line in existing_lines {
            let trimmed = line.trim();
            if trimmed.contains("WITH") {
                found_with = true;
                cbu_header_lines.push(line.to_string());
                break;
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                cbu_header_lines.push(line.to_string());
            } else if trimmed.starts_with('#') {
                cbu_header_lines.push(line.to_string()); // Keep comments
            }
        }

        // If no CBU header found, use LISP fallback
        if cbu_header_lines.is_empty() || !found_with {
            if state.cbu_context == CbuContext::CreateNew {
                let header = "; Create new CBU with LISP DSL\n(define-cbu \"New CBU Name\" \"CBU Description\")".to_string();
                cbu_header_lines = vec![header];
            } else {
                let default_id = "CBU_ID".to_string();
                let cbu_id = state.selected_cbu_id.as_ref().unwrap_or(&default_id);
                let header = format!("; Update existing CBU with LISP DSL\n(update-cbu \"{}\" \"Updated CBU\" \"Updated Description\")", cbu_id);
                cbu_header_lines = vec![header];
            }
        }

        // Build new DSL with preserved header and new entities
        let mut new_dsl = String::new();

        // Add header lines
        for line in &cbu_header_lines {
            new_dsl.push_str(line);
            new_dsl.push('\n');
        }

        // Add entity definitions
        self.append_entities_to_dsl(&mut new_dsl, state);

        new_dsl
    }

    /// Helper: Build DSL from scratch (used by single DSL manager)
    fn build_dsl_from_scratch(&self, state: &crate::cbu_state_manager::CbuStateManager) -> String {
        let mut new_dsl = String::new();

        // Create header based on context - ALWAYS use LISP format
        if state.cbu_context == CbuContext::CreateNew {
            new_dsl.push_str("; Create new CBU with LISP DSL\n(define-cbu \"New CBU Name\" \"CBU Description\")\n\n");
        } else {
            let default_id = "CBU_ID".to_string();
            let cbu_id = state.selected_cbu_id.as_ref().unwrap_or(&default_id);
            new_dsl.push_str(&format!("; Update existing CBU with LISP DSL\n(update-cbu \"{}\" \"Updated CBU\" \"Updated Description\")\n\n", cbu_id));
        }

        // Add entity definitions
        self.append_entities_to_dsl(&mut new_dsl, state);

        new_dsl
    }

    /// Helper: Append entities to DSL (used by both DSL builders)
    fn append_entities_to_dsl(&self, dsl: &mut String, state: &crate::cbu_state_manager::CbuStateManager) {
        for (i, (entity_info, role)) in state.selected_entities.iter().enumerate() {
            let parts: Vec<&str> = entity_info.split(" (").collect();
            if parts.len() == 2 {
                let entity_name = parts[0];
                let entity_id = parts[1].trim_end_matches(')');

                if i > 0 {
                    dsl.push_str(" AND\n");
                }
                // DSL format: ENTITY entity_id AS 'role' # entity_name (for hover tooltips)
                dsl.push_str(&format!("  ENTITY {} AS '{}' # {}", entity_id, role, entity_name));
            }
        }
    }

    /// Generate LISP-style DSL for elegant list processing
    fn build_lisp_dsl(&self, state: &crate::cbu_state_manager::CbuStateManager) -> String {
        let mut lisp_dsl = String::new();

        // Start comment
        lisp_dsl.push_str("; LISP-style CBU DSL - list processing for financial entities\n");

        // Build S-expression based on context
        if state.cbu_context == CbuContext::CreateNew {
            // Extract CBU name and description from existing DSL if available
            let (cbu_name, cbu_description) = self.extract_cbu_info_from_dsl(state);

            lisp_dsl.push_str(&format!(
                "(create-cbu \"{}\" \"{}\"\n",
                cbu_name.unwrap_or_else(|| "New CBU Name".to_string()),
                cbu_description.unwrap_or_else(|| "CBU Description".to_string())
            ));
        } else {
            let cbu_id = state.selected_cbu_id.as_ref().unwrap_or(&"CBU_ID".to_string()).clone();
            lisp_dsl.push_str(&format!("(update-cbu \"{}\"\n", cbu_id));
        }

        // Add entities list if we have any
        if !state.selected_entities.is_empty() {
            lisp_dsl.push_str("  (entities\n");

            for (entity_info, role) in &state.selected_entities {
                let parts: Vec<&str> = entity_info.split(" (").collect();
                if parts.len() == 2 {
                    let entity_name = parts[0];
                    let entity_id = parts[1].trim_end_matches(')');
                    let role_symbol = role.to_lowercase().replace(" ", "-");

                    lisp_dsl.push_str(&format!(
                        "    (entity \"{}\" \"{}\" {})\n",
                        entity_id, entity_name, role_symbol
                    ));
                }
            }

            lisp_dsl.push_str("  )");
        }

        lisp_dsl.push(')');
        lisp_dsl
    }

    /// Extract CBU name and description from existing DSL
    fn extract_cbu_info_from_dsl(&self, state: &crate::cbu_state_manager::CbuStateManager) -> (Option<String>, Option<String>) {
        if state.dsl_script.is_empty() {
            return (None, None);
        }

        // Parse existing DSL to extract CBU name and description
        for line in state.dsl_script.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("CREATE CBU") || trimmed.starts_with("UPDATE CBU") {
                // Try to extract quoted strings for name and description
                let parts: Vec<&str> = trimmed.split('\'').collect();
                if parts.len() >= 4 {
                    let name = parts[1].to_string();
                    let description = parts[3].to_string();
                    return (Some(name), Some(description));
                }
            }
        }

        (None, None)
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // No more async polling! State manager handles it in app.rs:39
        // **CRITICAL** - Check for CBU DSL loads every frame to ensure immediate UI updates
        self.check_for_cbu_dsl_loaded(state);
        // **60FPS THREAD-SAFE STATE READ** - Read execution state from Arc/Mutex cache

        // Store context for floating window (outside UI constraints)
        let ctx = ui.ctx().clone();
        ui.heading("🏢 CBU DSL Management");
        ui.separator();

        // Auto-load CBU list on startup for "Edit Existing" picker
        if state.available_cbus.is_empty() && !state.loading_cbus {
            wasm_utils::console_log("🔄 Auto-loading CBU list for Edit Existing picker");
            state.load_cbus();
        }

        // CBU Context Selection - Prominent at the top
        self.render_cbu_context_selection(ui, state);

        // Only show the rest of the UI if context is selected
        if state.cbu_context == crate::cbu_state_manager::CbuContext::None {
            return;
        }

        // Auto-load entities only when a CBU context is selected
        if state.available_entities.is_empty() && !state.loading_entities {
            wasm_utils::console_log("🔄 Auto-loading entities for selected CBU context");
            state.load_entities();
        }

        // Toolbar
        self.render_toolbar(ui, state);

        // Debug info
        ui.horizontal(|ui| {
            ui.label(format!("📊 Entities loaded: {}", state.available_entities.len()));
            if state.loading_entities {
                ui.spinner();
                ui.label("Loading...");
            }
        });

        ui.add_space(10.0);

        // Main content area
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 100.0)
            .show(ui, |ui| {
                self.render_main_content(ui, state);
            });

        // Render floating entity picker if open (pass ctx directly to avoid CentralPanel constraints)
        // Note: This is called OUTSIDE the ui context to avoid layout constraints from CentralPanel
        self.render_floating_entity_picker(&ctx, state);
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        ui.horizontal(|ui| {
            // Execute button
            let execute_button = ui.add_enabled(
                !state.dsl_script.trim().is_empty() && !self.executing.load(Ordering::SeqCst),
                egui::Button::new("▶ Execute DSL")
            );

            if execute_button.clicked() {
                self.execute_dsl(state);
            }

            // Emergency cancel execution button
            let is_executing = self.executing.load(Ordering::SeqCst);
            if is_executing {
                let cancel_button = ui.add(
                    egui::Button::new("🛑 Cancel Execution")
                        .fill(egui::Color32::from_rgb(200, 100, 100))
                );

                if cancel_button.clicked() {
                    wasm_utils::console_log("🛑 Emergency cancel requested by user");
                    self.executing.store(false, Ordering::SeqCst);

                    // Clear execution result and show cancellation message
                    if let Ok(mut result) = self.execution_result.lock() {
                        *result = Some(CbuDslResponse {
                            success: false,
                            message: "🛑 Execution cancelled by user".to_string(),
                            cbu_id: None,
                            validation_errors: Vec::new(),
                            data: None,
                        });
                    }
                    wasm_utils::console_log("🔓 Execution flag force-cleared - UI unlocked");
                }
            }

            // Clear button - REMOVED DEFAULT ACTION
            // if ui.button("🗑 Clear").clicked() {
            //     state.dsl_script.clear(); // REMOVED: default action that bypassed gRPC state management
            //     if let Ok(mut result) = self.execution_result.lock() {
            //         *result = None;
            //     }
            // }

            ui.separator();

            // Examples button
            if ui.button("📝 Examples").clicked() {
                self.show_examples = !self.show_examples;
            }

            // Help button
            if ui.button("❓ Help").clicked() {
                self.show_help = !self.show_help;
            }

            ui.separator();

            // Load entities button
            let load_entities_button = ui.add_enabled(
                !state.loading_entities,
                egui::Button::new("🔄 Load Entities")
            );

            if load_entities_button.clicked() {
                state.load_entities();
            }

            // Entity picker - compact display with expand button
            ui.horizontal(|ui| {
                // Show selected entities count and expand button
                let selected_count = state.selected_entities.len();
                let entities_count = state.available_entities.len();

                if selected_count > 0 {
                    ui.label(format!("👥 Selected: {}", selected_count));
                    ui.separator();
                }

                ui.label(format!("📊 {} entities available", entities_count));

                if ui.button("🔍 Pick Entities").clicked() {
                    wasm_utils::console_log("🔍 Opening SIMPLIFIED entity picker window");
                    self.show_floating_entity_picker = true;
                }
            });

            if state.loading_entities {
                ui.spinner();
                ui.label("Loading entities...");
            }
        });
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // Two-column layout
        ui.columns(2, |columns| {
            // Left column: DSL Editor
            columns[0].group(|ui| {
                ui.heading("📝 DSL Script Editor");
                ui.separator();

                // DSL text editor with enhanced IDE features - only show hint if we have an active CBU context
                let hint_text = if state.active_cbu_id.is_some() {
                    r#"Write CBU DSL commands here. Example:

; Define a new CBU using s-expression syntax
(define-cbu "Growth Fund Alpha" "Diversified growth fund"
  (entities
    (entity "AC001" "Alpha Capital" asset-owner)
    (entity "BM002" "Beta Management" investment-manager)
    (entity "GS003" "Gamma Services" managing-company)))"#
                } else {
                    "" // No hint text when no active CBU context
                };

                // Clipboard controls above editor
                ui.horizontal(|ui| {
                    ui.label("📝 DSL Editor:");
                    ui.separator();
                    if ui.button("📋 Copy").on_hover_text("Copy DSL to clipboard").clicked() {
                        self.copy_to_clipboard(state);
                    }
                    if ui.button("📄 Paste").on_hover_text("Paste from clipboard").clicked() {
                        self.paste_from_clipboard();
                    }
                    // Clear button - REMOVED DEFAULT ACTION
                    // if ui.button("🗑️ Clear").on_hover_text("Clear DSL editor").clicked() {
                    //     state.dsl_script.clear(); // REMOVED: default action that bypassed gRPC state management
                    // }
                });

                // Enhanced DSL editor with hover support
                let editor_response = self.render_enhanced_dsl_editor(ui, hint_text, state);

                // Auto-completion suggestions
                if editor_response.has_focus() && !state.available_entities.is_empty() {
                    self.show_auto_completion_popup(ui, state);
                }

                ui.add_space(10.0);

                // Execution status
                if self.executing.load(Ordering::SeqCst) {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Executing DSL script...");
                    });
                }
            });

            // Right column: Results and Help
            columns[1].group(|ui| {
                self.render_results_panel(ui);
            });
        });

        ui.add_space(10.0);

        // Bottom panels
        if self.show_examples {
            self.render_examples_panel(ui);
        }

        if self.show_help {
            self.render_help_panel(ui);
        }

        // Inline entity picker removed - now using floating panel
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Execution Results");
        ui.separator();

        // Read execution state from thread-safe cache
        let (_is_executing, result) = self.get_execution_state();

        if let Some(result) = &result {
            // Success/Error indicator
            ui.horizontal(|ui| {
                if result.success {
                    ui.colored_label(egui::Color32::GREEN, "✅ Success");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ Error");
                }
                ui.label(&result.message);
            });

            ui.add_space(5.0);

            // CBU ID if created
            if let Some(cbu_id) = &result.cbu_id {
                ui.horizontal(|ui| {
                    ui.label("CBU ID:");
                    ui.code(cbu_id);
                    if ui.button("📋").clicked() {
                        ui.ctx().copy_text(cbu_id.clone());
                    }
                });
            }

            // Validation errors
            if !result.validation_errors.is_empty() {
                ui.separator();
                ui.heading("❌ Validation Errors:");
                for error in &result.validation_errors {
                    ui.label(format!("• {}", error));
                }
            }

            // Query results
            if let Some(data) = &result.data {
                ui.separator();
                ui.heading("📋 Query Results:");

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let json_str = serde_json::to_string_pretty(data).unwrap_or_default();
                        ui.add(
                            egui::TextEdit::multiline(&mut json_str.as_str())
                                .desired_width(f32::INFINITY)
                                .code_editor()
                        );
                    });
            }
        } else {
            ui.label("No execution results yet. Write a DSL script and click Execute.");
        }
    }

    fn render_examples_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.collapsing("📝 DSL Examples", |ui| {
            let examples = [("Create CBU", r#"CREATE CBU 'Growth Fund Alpha' ; 'A diversified growth-focused investment fund' WITH
  ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
  ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
  ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'"#),
                ("Update CBU", "UPDATE CBU 'CBU001' SET description = 'Updated description'"),
                ("Delete CBU", "DELETE CBU 'CBU001'"),
                ("Query CBUs", "QUERY CBU WHERE status = 'active'")];

            let selected_example_name = examples[self.selected_example].0;
            let selected_example_code = examples[self.selected_example].1;

            ui.horizontal(|ui| {
                ui.label("Select example:");
                egui::ComboBox::from_id_salt("example_selector")
                    .selected_text(selected_example_name)
                    .show_ui(ui, |ui| {
                        for (index, (name, _)) in examples.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_example, index, *name);
                        }
                    });

                // Examples panel is view-only - user can copy examples manually
            });

            ui.add_space(5.0);

            // Show selected example
            ui.group(|ui| {
                ui.label("Example code:");
                let mut example_display = selected_example_code.to_string();
                ui.add(
                    egui::TextEdit::multiline(&mut example_display)
                        .desired_width(f32::INFINITY)
                        .code_editor()
                );
            });
        });
    }

    fn render_help_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.collapsing("❓ CBU DSL Help", |ui| {
            ui.label("CBU DSL S-Expression Syntax Reference:");

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("CREATE CBU");
                    ui.code("(define-cbu \"CBU_ID\" \"CBU Name\"");
                    ui.code("  (description \"Purpose description\")");
                    ui.code("  (business-model \"Investment Management\")");
                    ui.code("  (jurisdiction \"US\")");
                    ui.code("  (status \"active\")");
                    ui.code("  (entities");
                    ui.code("    (entity \"ENTITY_ID\" \"Entity Name\" role)))");

                    ui.add_space(10.0);

                    ui.heading("UPDATE CBU");
                    ui.code("(update-cbu \"CBU_ID\"");
                    ui.code("  (description \"New description\")");
                    ui.code("  (business-model \"New model\"))");

                    ui.add_space(10.0);

                    ui.heading("LOAD CBU");
                    ui.code("(load-cbu \"CBU_ID\")");

                    ui.add_space(10.0);

                    ui.heading("Entity Roles:");
                    ui.label("• administrator - Entity providing administrative services");
                    ui.label("• custodian - Entity providing custodial services");
                    ui.label("• manager - Entity providing management services");

                    ui.add_space(10.0);

                    ui.heading("Notes:");
                    ui.label("• All entities must exist in the client entities table");
                    ui.label("• Strings must be quoted with single quotes");
                    ui.label("• CBU IDs are auto-generated for CREATE operations");
                });
            });
        });
    }

    fn show_auto_completion_popup(&self, ui: &mut egui::Ui, state: &crate::cbu_state_manager::CbuStateManager) {
        // Simple auto-completion based on available entities
        // Simplified implementation - just show entities in a collapsing section
        ui.collapsing("Available Entities", |ui| {
            for entity in &state.available_entities {
                if ui.button(format!("'{}' ({})", entity.entity_name, entity.entity_id)).clicked() {
                    // Insert entity into DSL script (simplified)
                    // In a real implementation, this would insert at cursor position
                }
            }
        });
    }

    fn render_enhanced_dsl_editor(&mut self, ui: &mut egui::Ui, hint_text: &str, state: &mut crate::cbu_state_manager::CbuStateManager) -> egui::Response {
        ui.vertical(|ui| {
            // Syntax highlighting controls
            ui.horizontal(|ui| {
                ui.label("🎨 Editor Options:");
                ui.checkbox(&mut self.show_syntax_highlighting, "Syntax Highlighting");

                // Theme selector
                ui.separator();
                if ui.button("Dark Theme").clicked() {
                    self.syntax_highlighter.set_theme(SyntaxTheme::dark_theme());
                }
                if ui.button("Light Theme").clicked() {
                    self.syntax_highlighter.set_theme(SyntaxTheme::light_theme());
                }

                ui.separator();
                ui.label(format!("Mode: {}", if self.lisp_mode { "LISP" } else { "EBNF" }));
            });

            ui.separator();

            // Validate syntax when content changes
            if !state.dsl_script.is_empty() {
                state.syntax_errors = self.syntax_highlighter.validate_syntax(&state.dsl_script);
            }

            // Show syntax errors if any
            if !state.syntax_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, "⚠️ Syntax Errors:");
                ui.indent("syntax_errors", |ui| {
                    for error in &state.syntax_errors {
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                });
                ui.separator();
            }

            let editor_response = if self.show_syntax_highlighting && !state.dsl_script.is_empty() {
                // Show syntax-highlighted preview alongside editor
                ui.horizontal(|ui| {
                    // Editor on the left with completion
                    let text_response = ui.vertical(|ui| {
                        let text_response = ui.add_sized(
                            [ui.available_width() * 0.5 - 10.0, 400.0],
                            egui::TextEdit::multiline(&mut state.dsl_script)
                                .code_editor()
                                .hint_text(hint_text)
                        );

                        // Handle completion trigger
                        if text_response.has_focus() {
                            self.handle_completion_input(ui, &text_response, state);
                        }

                        // Show completion popup if active
                        if self.show_completion_popup {
                            self.render_completion_popup(ui, state);
                        }

                        text_response
                    }).inner;

                    ui.separator();

                    // Syntax-highlighted preview on the right
                    ui.vertical(|ui| {
                        ui.label("🌈 Syntax Highlighted Preview:");
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .max_height(380.0)
                            .show(ui, |ui| {
                                if state.dsl_script.lines().count() > 20 {
                                    // For large files, show with line numbers
                                    self.syntax_highlighter.render_with_line_numbers(ui, &state.dsl_script);
                                } else {
                                    // For smaller files, show highlighted lines
                                    self.syntax_highlighter.render_highlighted_lines(ui, &state.dsl_script);
                                }
                            });
                    });

                    text_response
                }).inner
            } else {
                // Standard editor with completion
                ui.vertical(|ui| {
                    let text_response = ui.add(
                        egui::TextEdit::multiline(&mut state.dsl_script)
                            .desired_width(f32::INFINITY)
                            .desired_rows(15)
                            .code_editor()
                            .hint_text(hint_text)
                    );

                    // Handle completion trigger
                    if text_response.has_focus() {
                        self.handle_completion_input(ui, &text_response, state);
                    }

                    // Show completion popup if active
                    if self.show_completion_popup {
                        self.render_completion_popup(ui, state);
                    }

                    text_response
                }).inner
            };

            // Show tooltip on hover - let egui handle the hover detection
            editor_response.on_hover_ui(|ui| {
                self.show_dsl_content_tooltip(ui, state);
            })
        }).inner
    }

    /// Handle code completion input triggers
    fn handle_completion_input(&mut self, ui: &mut egui::Ui, text_response: &egui::Response, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // Check for completion triggers
        let ctx = ui.ctx();

        // Trigger completion on Ctrl+Space
        if ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) {
            self.trigger_completion(state);
        }

        // Handle completion navigation
        if self.show_completion_popup {
            let should_apply = ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowUp) {
                    if self.selected_completion > 0 {
                        self.selected_completion -= 1;
                    }
                    false
                } else if i.key_pressed(egui::Key::ArrowDown) {
                    if self.selected_completion < self.completion_suggestions.len().saturating_sub(1) {
                        self.selected_completion += 1;
                    }
                    false
                } else if i.key_pressed(egui::Key::Enter) {
                    true
                } else if i.key_pressed(egui::Key::Escape) {
                    self.show_completion_popup = false;
                    false
                } else {
                    false
                }
            });

            if should_apply {
                self.apply_completion(state);
            }
        }

        // Auto-trigger completion on typing
        if text_response.changed() {
            // Simple auto-trigger when typing certain characters
            if state.dsl_script.ends_with('(') || state.dsl_script.ends_with(' ') {
                self.trigger_completion(state);
            }
        }
    }

    /// Trigger code completion
    fn trigger_completion(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        let cursor_pos = state.dsl_script.len();
        self.completion_suggestions = self.syntax_highlighter.get_completions(&state.dsl_script, cursor_pos);

        if !self.completion_suggestions.is_empty() {
            self.show_completion_popup = true;
            self.selected_completion = 0;
            self.completion_trigger_pos = cursor_pos;
        }
    }

    /// Apply the selected completion
    fn apply_completion(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        if let Some(completion) = self.completion_suggestions.get(self.selected_completion) {
            // Find the word at cursor to replace
            let word_start = self.find_word_start(self.completion_trigger_pos, state);

            // Replace the partial word with the completion
            let before = &state.dsl_script[..word_start];
            let after = &state.dsl_script[self.completion_trigger_pos..];

            state.dsl_script = format!("{}{}{}", before, completion, after);
        }

        self.show_completion_popup = false;
    }

    /// Find the start of the current word
    fn find_word_start(&self, pos: usize, state: &crate::cbu_state_manager::CbuStateManager) -> usize {
        let chars: Vec<char> = state.dsl_script.chars().collect();
        let mut start = pos;

        while start > 0 {
            let ch = chars[start - 1];
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        start
    }

    /// Render the completion popup
    fn render_completion_popup(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        if self.completion_suggestions.is_empty() {
            return;
        }

        egui::Area::new("completion_popup".into())
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label("💡 Code Completion");
                            ui.separator();

                            let mut should_apply_completion = false;

                            for (i, suggestion) in self.completion_suggestions.iter().enumerate() {
                                let is_selected = i == self.selected_completion;

                                let response = ui.selectable_label(is_selected, suggestion);

                                if response.clicked() {
                                    self.selected_completion = i;
                                    should_apply_completion = true;
                                }

                                // Show description for known completions
                                if is_selected {
                                    let description = self.get_completion_description(suggestion);
                                    if !description.is_empty() {
                                        ui.small(description);
                                    }
                                }
                            }

                            // Apply completion after the loop to avoid borrow conflicts
                            if should_apply_completion {
                                self.apply_completion(state);
                            }

                            ui.separator();
                            ui.small("↑↓ Navigate • Enter: Apply • Esc: Cancel");
                        });
                    });
            });
    }

    /// Get description for a completion suggestion
    fn get_completion_description(&self, suggestion: &str) -> String {
        match suggestion {
            "create-cbu" => "Create a new Client Business Unit".to_string(),
            "update-cbu" => "Update an existing CBU".to_string(),
            "delete-cbu" => "Delete a CBU".to_string(),
            "query-cbu" => "Query CBUs".to_string(),
            "entity" => "Define an entity with ID, name, and role".to_string(),
            "entities" => "Group multiple entities".to_string(),
            "asset-owner" => "Entity role: Legal owner of assets".to_string(),
            "investment-manager" => "Entity role: Makes investment decisions".to_string(),
            "custodian" => "Entity role: Safekeeps assets".to_string(),
            "prime-broker" => "Entity role: Provides brokerage services".to_string(),
            _ => String::new(),
        }
    }

    fn show_dsl_content_tooltip(&self, ui: &mut egui::Ui, state: &crate::cbu_state_manager::CbuStateManager) {
        // Show contextual tooltips based on DSL content
        ui.label("📝 CBU DSL Editor");
        ui.separator();

        // Count and show CBU and entity references
        let cbu_count = state.dsl_script.lines().filter(|line|
            line.trim().starts_with("CREATE CBU ") || line.trim().starts_with("UPDATE CBU ")
        ).count();

        let entity_count = state.dsl_script.lines().filter(|line|
            line.trim().contains("ENTITY ") && line.trim().contains(" AS ")
        ).count();

        if cbu_count > 0 {
            ui.label(format!("🏢 {} CBU operation(s)", cbu_count));
        }
        if entity_count > 0 {
            ui.label(format!("👤 {} Entity reference(s)", entity_count));
        }

        if cbu_count == 0 && entity_count == 0 {
            ui.label("💡 Add CBU operations and entity references");
        }

        // Show first CBU operation details if any
        for line in state.dsl_script.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("CREATE CBU ") {
                if let Some(cbu_info) = self.parse_cbu_line(trimmed) {
                    ui.separator();
                    ui.label("🏢 Creating CBU:");
                    ui.label(format!("  Key: {}", cbu_info.0));
                    ui.label(format!("  Name: {}", cbu_info.1));
                    ui.label(format!("  Purpose: {}", cbu_info.2));
                    break;
                }
            }

            if trimmed.starts_with("UPDATE CBU ") {
                if let Some(cbu_key) = self.parse_update_cbu_line(trimmed) {
                    ui.separator();
                    ui.label("✏️ Updating CBU:");
                    ui.label(format!("  Key: {}", cbu_key));

                    // Look up real CBU data if available
                    if let Some(cbu) = state.available_cbus.iter().find(|c| c.cbu_id == cbu_key) {
                        ui.label(format!("  Current Name: {}", cbu.cbu_name));
                        ui.label(format!("  Status: {}", cbu.status));
                    }
                    break;
                }
            }
        }

        // Show entity summary
        let entities: Vec<_> = state.dsl_script.lines()
            .filter_map(|line| self.parse_entity_line(line.trim()))
            .collect();

        if !entities.is_empty() {
            ui.separator();
            ui.label("👤 Entity Roles:");
            for (entity_key, role, name) in entities.iter().take(3) {
                let display_name = name.as_ref().unwrap_or(entity_key);
                ui.label(format!("  {} → {}", display_name, role));
            }
            if entities.len() > 3 {
                ui.label(format!("  ... and {} more", entities.len() - 3));
            }
        }
    }

    fn parse_cbu_line(&self, line: &str) -> Option<(String, String, String)> {
        // Parse: "CREATE CBU CBU_12345 'CBU Name' ; 'CBU Purpose' WITH"
        if let Some(cbu_start) = line.find("CREATE CBU ") {
            let after_cbu = &line[cbu_start + 11..]; // Skip "CREATE CBU "
            let parts: Vec<&str> = after_cbu.split_whitespace().collect();
            if !parts.is_empty() {
                let cbu_key = parts[0].to_string();

                // Extract name and purpose from quoted strings
                let name = self.extract_quoted_string(line, 0).unwrap_or("Unknown Name".to_string());
                let purpose = self.extract_quoted_string(line, 1).unwrap_or("Unknown Purpose".to_string());

                return Some((cbu_key, name, purpose));
            }
        }
        None
    }

    fn parse_update_cbu_line(&self, line: &str) -> Option<String> {
        // Parse: "UPDATE CBU CBU_12345 SET ..."
        if let Some(cbu_start) = line.find("UPDATE CBU ") {
            let after_cbu = &line[cbu_start + 11..]; // Skip "UPDATE CBU "
            let parts: Vec<&str> = after_cbu.split_whitespace().collect();
            if !parts.is_empty() {
                return Some(parts[0].to_string());
            }
        }
        None
    }

    fn parse_entity_line(&self, line: &str) -> Option<(String, String, Option<String>)> {
        // Parse: "ENTITY AC001 AS 'Asset Owner' # Alpha Capital"
        if let Some(entity_start) = line.find("ENTITY ") {
            let after_entity = &line[entity_start + 7..]; // Skip "ENTITY "
            if let Some(as_pos) = after_entity.find(" AS ") {
                let entity_key = after_entity[..as_pos].trim().to_string();
                let after_as = &after_entity[as_pos + 4..];

                // Extract role from quotes
                if let Some(role_start) = after_as.find('\'') {
                    if let Some(role_end) = after_as[role_start + 1..].find('\'') {
                        let role = after_as[role_start + 1..role_start + 1 + role_end].to_string();

                        // Extract entity name from comment
                        let entity_name = line.find(" # ").map(|comment_pos| line[comment_pos + 3..].trim().to_string());

                        return Some((entity_key, role, entity_name));
                    }
                }
            }
        }
        None
    }

    fn extract_quoted_string(&self, text: &str, occurrence: usize) -> Option<String> {
        // Extract the nth quoted string from text
        let mut count = 0;
        let mut chars = text.chars();
        let mut start_pos = None;

        while let Some(ch) = chars.next() {
            if ch == '\'' {
                if count == occurrence && start_pos.is_none() {
                    start_pos = Some(chars.as_str());
                } else if count == occurrence && start_pos.is_some() {
                    // Found the end quote for our target occurrence
                    let start = start_pos.unwrap();
                    let end_pos = start.len() - chars.as_str().len() - 1;
                    return Some(start[..end_pos].to_string());
                } else if start_pos.is_none() {
                    count += 1;
                }
            }
        }
        None
    }

    fn get_editor_hint(&self) -> &str {
        r#"Write CBU DSL commands here. Examples:

; Define a new CBU
(define-cbu "Growth Fund Alpha" "Diversified growth fund"
  (entities
    (entity "AC001" "Alpha Capital" asset-owner)
    (entity "BM002" "Beta Management" investment-manager)
    (entity "GS003" "Gamma Services" managing-company)))

; Update an existing CBU
(update-cbu "CBU001" "Updated Fund Name" "Updated description")

; Delete a CBU
(delete-cbu "CBU001")

; Query CBUs
(query-cbu (where (status "active")))"#
    }

    fn get_dsl_examples(&self) -> Vec<(&str, &str)> {
        vec![
            ("Create CBU", r#"; Create new CBU with entities
(define-cbu "CBU0000001" "Growth Fund Alpha"
  (description "A diversified growth-focused investment fund")
  (business-model "Equity Fund Management")
  (jurisdiction "United States")
  (status "active")
  (entities
    (entity "AC001" "Alpha Capital" administrator)
    (entity "BM002" "Beta Management" custodian)
    (entity "GS003" "Gamma Services" manager)))"#),

            ("Update CBU", r#"; Update existing CBU
(update-cbu "CBU0000001"
  (description "Updated fund description")
  (business-model "Hedge Fund")
  (status "active"))"#),

            ("Query CBU", r#"; Load CBU for editing
(load-cbu "CBU0000001")"#),

            ("Simple CBU", r#"; Minimal CBU definition
(define-cbu "NEW_CBU_001" "Simple Fund"
  (description "Basic fund structure")
  (business-model "Investment Management")
  (jurisdiction "US")
  (status "active")
  (entities ()))"#),
        ]
    }

    /// Read execution state from thread-safe cache for 60fps UI performance
    /// Uses atomic reads and mutex locks for safe async-to-UI communication
    fn get_execution_state(&self) -> (bool, Option<CbuDslResponse>) {
        // **ATOMIC READ** - Lock-free check of execution status
        let is_executing = self.executing.load(Ordering::SeqCst);

        // **MUTEX READ** - Thread-safe access to execution result
        let result = if let Ok(guard) = self.execution_result.lock() {
            guard.clone()
        } else {
            None
        };

        (is_executing, result)
    }

    fn execute_dsl(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // **CENTRALIZED DSL EXECUTION** - All execution goes through validation first
        wasm_utils::console_log("🚀 EXECUTE DSL CALLED - Starting execution process");
        wasm_utils::console_log(&format!("📝 DSL Script: '{}'", state.dsl_script));
        wasm_utils::console_log(&format!("🔗 gRPC Client: {}", if state.get_grpc_client().is_some() { "Available" } else { "None" }));

        // Step 1: Validate DSL through central manager before execution
        match self.validate_dsl_syntax(&state.dsl_script) {
            Ok(_) => {
                wasm_utils::console_log("✅ DSL validation passed - proceeding with execution");
            },
            Err(validation_error) => {
                wasm_utils::console_log(&format!("❌ DSL validation failed: {}", validation_error));
                // Set validation error through thread-safe state
                if let Ok(mut result) = self.execution_result.lock() {
                    *result = Some(CbuDslResponse {
                        success: false,
                        message: format!("Validation Error: {}", validation_error),
                        cbu_id: None,
                        validation_errors: vec![validation_error],
                        data: None,
                    });
                }
                return; // Don't execute invalid DSL
            }
        }

        let client_clone = match state.get_grpc_client() {
            Some(client) => client.clone(),
            None => {
                wasm_utils::console_log("❌ No gRPC client available for DSL execution");
                return;
            }
        };

        // **THREAD-SAFE 60FPS EXECUTION** - Use Arc/Mutex for async-to-UI synchronization

        // Set executing state atomically (lock-free read from UI thread)
        self.executing.store(true, Ordering::SeqCst);

        // Set initial result through mutex (thread-safe write)
        if let Ok(mut result) = self.execution_result.lock() {
            *result = Some(CbuDslResponse {
                success: false,
                message: "🚀 Executing validated DSL via gRPC... Check console for progress".to_string(),
                cbu_id: None,
                validation_errors: Vec::new(),
                data: None,
            });
        }

        wasm_utils::console_log("🚀 Executing validated CBU DSL via gRPC...");

        // Import the request type
        use crate::grpc_client::ExecuteCbuDslRequest;

        // Create gRPC request
        let request = ExecuteCbuDslRequest {
            dsl_script: state.dsl_script.clone(),
        };

        // Clone for async operation - includes thread-safe state
        let executing_clone = self.executing.clone();
        let result_clone = self.execution_result.clone();

        // **PERFORMANT ASYNC WITH ROBUST ERROR HANDLING**
        // Async can now safely update UI state through Arc/Mutex
        wasm_utils::spawn_async(async move {
            let start_time = std::time::Instant::now();
            wasm_utils::console_log("🚀 Starting DSL execution with robust error handling");

            let final_result = match client_clone.execute_cbu_dsl(request).await {
                Ok(response) => {
                    wasm_utils::console_log(&format!("✅ gRPC CBU DSL execution successful: {}", response.message));
                    CbuDslResponse {
                        success: true,
                        message: response.message,
                        cbu_id: response.cbu_id,
                        validation_errors: Vec::new(),
                        data: None,
                    }
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ gRPC CBU DSL execution failed: {}", e));
                    CbuDslResponse {
                        success: false,
                        message: format!("Execution Error: {}", e),
                        cbu_id: None,
                        validation_errors: Vec::new(),
                        data: None,
                    }
                }
            };

            let elapsed = start_time.elapsed();
            wasm_utils::console_log(&format!("⏱️ Execution completed in {:.2}s", elapsed.as_secs_f64()));

            // **THREAD-SAFE UI STATE UPDATE** - Update result atomically
            if let Ok(mut result) = result_clone.lock() {
                *result = Some(final_result);
            } else {
                wasm_utils::console_log("⚠️ Failed to update execution result - mutex lock failed");
            }

            // **ATOMIC EXECUTION FLAG** - CRITICAL: Always clear executing state
            executing_clone.store(false, Ordering::SeqCst);
            wasm_utils::console_log("🔓 Execution flag cleared - UI unlocked");

            wasm_utils::console_log("💡 Execution complete - UI will refresh on next frame");
        });

        // **60FPS EGUI PATTERN** - Immediate UI feedback, async updates state cache
        // UI thread reads atomic flags and mutex state for smooth 60fps performance
    }

    fn simulate_execution(&mut self, state: &crate::cbu_state_manager::CbuStateManager) {
        // Simulate execution result
        let script = state.dsl_script.trim();

        if script.to_uppercase().starts_with("CREATE CBU") {
            if let Ok(mut result) = self.execution_result.lock() {
                *result = Some(CbuDslResponse {
                    success: true,
                    message: "CBU created successfully".to_string(),
                    cbu_id: Some(format!("CBU{:06}", 123456)), // Simplified for demo
                    validation_errors: Vec::new(),
                    data: None,
                });
            }
        } else if script.to_uppercase().starts_with("UPDATE CBU") {
            if let Ok(mut result) = self.execution_result.lock() {
                *result = Some(CbuDslResponse {
                    success: true,
                    message: "CBU updated successfully".to_string(),
                    cbu_id: None,
                    validation_errors: Vec::new(),
                    data: None,
                });
            }
        } else if script.to_uppercase().starts_with("DELETE CBU") {
            if let Ok(mut result) = self.execution_result.lock() {
                *result = Some(CbuDslResponse {
                    success: true,
                    message: "CBU deleted successfully".to_string(),
                    cbu_id: None,
                    validation_errors: Vec::new(),
                    data: None,
                });
            }
        } else if script.to_uppercase().starts_with("QUERY CBU") {
            let sample_data = serde_json::json!([
                {
                    "cbu_id": "CBU001",
                    "cbu_name": "Growth Fund Alpha",
                    "description": "A diversified growth-focused investment fund",
                    "status": "active",
                    "entities": [
                        "Alpha Capital (Asset Owner)",
                        "Beta Management (Investment Manager)",
                        "Gamma Services (Managing Company)"
                    ]
                }
            ]);

            if let Ok(mut result) = self.execution_result.lock() {
                *result = Some(CbuDslResponse {
                    success: true,
                    message: "Query executed successfully".to_string(),
                    cbu_id: None,
                    validation_errors: Vec::new(),
                    data: Some(sample_data),
                });
            }
        } else if let Ok(mut result) = self.execution_result.lock() {
            *result = Some(CbuDslResponse {
                success: false,
                message: "Invalid DSL command".to_string(),
                cbu_id: None,
                validation_errors: vec!["Command must start with CREATE CBU, UPDATE CBU, DELETE CBU, or QUERY CBU".to_string()],
                data: None,
            });
        }

        self.executing.store(false, Ordering::SeqCst);
    }

    fn render_cbu_context_selection(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        ui.group(|ui| {
            ui.heading("📋 CBU Operation Mode");
            ui.add_space(5.0);

            match state.cbu_context {
                CbuContext::None => {
                    ui.label("Choose what you want to do:");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        // Create New CBU - Prominent Blue Button
                        let create_button = ui.add_sized(
                            [180.0, 40.0],
                            egui::Button::new("🆕 Create New CBU")
                                .fill(egui::Color32::from_rgb(30, 144, 255))
                        );

                        if create_button.clicked() {
                            self.start_create_new_cbu(state);
                        }

                        ui.add_space(20.0);

                        // Edit Existing CBU
                        let edit_button = ui.add_sized(
                            [180.0, 40.0],
                            egui::Button::new("✏️ Edit Existing CBU")
                        );

                        if edit_button.clicked() {
                            self.start_edit_existing_cbu(state);
                        }
                    });
                },
                CbuContext::CreateNew => {
                    ui.horizontal(|ui| {
                        ui.label("🆕 Mode: Creating New CBU");
                        ui.separator();
                        if ui.button("🔙 Back to Selection").clicked() {
                            self.reset_context(state);
                        }
                    });

                    ui.add_space(10.0);

                    // CBU Name Input
                    ui.horizontal(|ui| {
                        ui.label("CBU Name:");
                        ui.add_space(10.0);
                        let name_input = ui.add_sized(
                            [300.0, 25.0],
                            egui::TextEdit::singleline(&mut self.new_cbu_name)
                                .hint_text("Enter CBU name (e.g., 'Investment Management Fund')")
                        );

                        ui.add_space(20.0);

                        // Create CBU Button
                        let create_enabled = !self.new_cbu_name.trim().is_empty() && !state.creating_cbu;
                        let create_button = ui.add_enabled(
                            create_enabled,
                            egui::Button::new(if state.creating_cbu { "🔄 Creating..." } else { "🔨 Create CBU" })
                                .fill(if create_enabled { egui::Color32::from_rgb(34, 139, 34) } else { egui::Color32::GRAY })
                        );

                        if create_button.clicked() && create_enabled {
                            self.create_new_cbu(state);
                        }

                        // Auto-focus the text input
                        if name_input.gained_focus() || name_input.has_focus() {
                            // Keep focus active
                        }
                    });

                    // Show creation status
                    if state.creating_cbu {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Creating new CBU and generating DSL...");
                        });
                    }

                    // Instructions
                    ui.add_space(10.0);
                    ui.label("💡 Instructions:");
                    ui.label("1. Enter a descriptive name for your CBU");
                    ui.label("2. Click 'Create CBU' to generate the record and DSL");
                    ui.label("3. The system will create the CBU in the database and return editable DSL");
                },
                CbuContext::EditExisting => {
                    ui.horizontal(|ui| {
                        ui.label("✏️ Mode: Editing Existing CBU");

                        ui.separator();

                        // Refresh button
                        if ui.button("🔄 Refresh List").clicked() {
                            self.refresh_cbu_list(state);
                        }

                        ui.separator();
                        if ui.button("🔙 Back to Selection").clicked() {
                            state.cbu_context = CbuContext::None;
                        }
                    });

                    ui.add_space(10.0);

                    // CBU Selection Section
                    if state.loading_cbus {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading CBUs...");
                        });
                    } else if state.available_cbus.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("⚠️ No CBUs found. Click 'Refresh List' to try again.");
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(format!("📋 Found {} CBUs. Select one:", state.available_cbus.len()));
                        });

                        ui.add_space(5.0);

                        let selected_name = state.selected_cbu_id.as_ref()
                            .and_then(|id| state.available_cbus.iter().find(|cbu| cbu.cbu_id == *id))
                            .map(|cbu| cbu.cbu_name.as_str())
                            .unwrap_or("Choose CBU...");

                        let mut selected_cbu_id_for_loading = None;
                        egui::ComboBox::from_id_salt("cbu_selector")
                            .selected_text(selected_name)
                            .width(400.0)
                            .show_ui(ui, |ui| {
                                for cbu in &state.available_cbus {
                                    let selected = ui.selectable_value(
                                        &mut state.selected_cbu_id,
                                        Some(cbu.cbu_id.clone()),
                                        format!("{} ({})", cbu.cbu_name, cbu.cbu_id)
                                    );

                                    if selected.clicked() {
                                        // Store the ID to load DSL after borrowing ends
                                        selected_cbu_id_for_loading = Some(cbu.cbu_id.clone());
                                        wasm_utils::console_log(&format!("🎯 CBU selected: {} ({})", cbu.cbu_name, cbu.cbu_id));

                                        // Immediately set active CBU context (don't wait for async DSL load)
                                        state.active_cbu_id = Some(cbu.cbu_id.clone());
                                        state.active_cbu_name = cbu.cbu_name.clone();
                                        wasm_utils::console_log(&format!("✅ Active CBU context set: {} ({})", cbu.cbu_name, cbu.cbu_id));
                                    }
                                }
                            });

                        // Load DSL if a CBU was selected
                        if let Some(cbu_id) = selected_cbu_id_for_loading {
                            self.load_cbu_dsl(&cbu_id, state);
                        }

                        // Show selected CBU info
                        if let Some(selected_id) = &state.selected_cbu_id {
                            if let Some(cbu) = state.available_cbus.iter().find(|c| &c.cbu_id == selected_id) {
                                ui.add_space(10.0);
                                ui.label(format!("✅ Selected: {} ({})", cbu.cbu_name, cbu.cbu_id));
                                if let Some(description) = &cbu.description {
                                    ui.label(format!("📝 Description: {}", description));
                                }
                            }
                        }
                    }
                }
            }
        });
        ui.add_space(10.0);
    }

    fn load_cbu_dsl(&mut self, cbu_id: &str, state: &mut crate::cbu_state_manager::CbuStateManager) {
        let client_clone = match state.get_grpc_client() {
            Some(client) => client.clone(),
            None => {
                wasm_utils::console_log("❌ No gRPC client available for CBU DSL loading");
                return;
            }
        };

        // Load actual DSL from database via gRPC GetCbu call
        let cbu_id_clone = cbu_id.to_string();

        wasm_utils::console_log(&format!("🔍 Loading DSL for CBU: {}", cbu_id));

        // Use async task to load CBU and DSL content from database
        wasm_utils::spawn_async(async move {
            let request = crate::grpc_client::GetCbuRequest {
                cbu_id: cbu_id_clone.clone(),
            };

            match client_clone.get_cbu(request).await {
                Ok(response) => {
                    if response.success {
                        if let Some(cbu) = response.cbu {
                            let dsl_content = cbu.dsl_content.unwrap_or_default();

                            if dsl_content.is_empty() {
                                wasm_utils::console_log(&format!("📭 No DSL found for CBU: {}", cbu.cbu_name));
                                // Store empty DSL to indicate we need to create one (WASM only)
                                #[cfg(target_arch = "wasm32")]
                                {
                                    let window = web_sys::window().unwrap();
                                    let storage = window.local_storage().unwrap().unwrap();
                                    let _ = storage.set_item("data_designer_cbu_dsl_loaded", "");
                                    let _ = storage.set_item("data_designer_cbu_dsl_cbu_id", &cbu_id_clone);
                                }
                            } else {
                                wasm_utils::console_log(&format!("✅ Loaded DSL for CBU: {} ({} chars)", cbu.cbu_name, dsl_content.len()));
                                // Store actual DSL content for UI to pick up (WASM only)
                                #[cfg(target_arch = "wasm32")]
                                {
                                    let window = web_sys::window().unwrap();
                                    let storage = window.local_storage().unwrap().unwrap();
                                    let _ = storage.set_item("data_designer_cbu_dsl_loaded", &dsl_content);
                                    let _ = storage.set_item("data_designer_cbu_dsl_cbu_id", &cbu_id_clone);
                                }
                            }
                        } else {
                            wasm_utils::console_log("❌ GetCbu response success but no CBU data");
                        }
                    } else {
                        wasm_utils::console_log(&format!("❌ GetCbu failed: {}", response.message));
                    }
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Error loading CBU DSL: {}", e));
                }
            }
        });
    }

    /// Check for completed CBU creation from async task and switch to active CBU context (WASM only)
    fn check_for_new_cbu_creation(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(window) = web_sys::window().ok_or("no window") {
            if let Ok(storage) = window.local_storage().ok().flatten().ok_or("no storage") {
                // Check if CBU creation is complete
                if let Ok(Some(complete_flag)) = storage.get_item("data_designer_cbu_creation_complete") {
                    if complete_flag == "true" {
                        // Get the created CBU data
                        if let Ok(Some(cbu_json)) = storage.get_item("data_designer_new_cbu_created") {
                            if let Ok(cbu) = serde_json::from_str::<CbuRecord>(&cbu_json) {
                                // Get the DSL content
                                if let Ok(Some(dsl_content)) = storage.get_item("data_designer_new_cbu_dsl") {
                                    wasm_utils::console_log(&format!("🎉 CBU creation completed: {} - switching to active context", cbu.cbu_name));

                                    // Switch to active CBU DSL context
                                    state.cbu_context = CbuContext::EditExisting;
                                    state.active_cbu_id = Some(cbu.cbu_id.clone());
                                    state.active_cbu_name = cbu.cbu_name.clone();
                                    state.dsl_script = dsl_content;
                                    state.creating_cbu = false;
                                    self.new_cbu_name.clear();

                                    // Refresh CBU list to include the new CBU
                                    if !state.available_cbus.iter().any(|c| c.cbu_id == cbu.cbu_id) {
                                        state.available_cbus.push(cbu);
                                        wasm_utils::console_log("📋 Added new CBU to available CBUs list");
                                    }

                                    // Clear localStorage flags
                                    let _ = storage.remove_item("data_designer_cbu_creation_complete");
                                    let _ = storage.remove_item("data_designer_new_cbu_created");
                                    let _ = storage.remove_item("data_designer_new_cbu_dsl");
                                    let _ = storage.remove_item("data_designer_new_cbu_name");

                                    wasm_utils::console_log("✅ Switched to active CBU DSL context - user can now edit the DSL");
                                }
                            }
                        }
                    }
                }
            }
        }
        }
    }

    /// Check for completed CBU DSL loading from async task and update DSL content (WASM only)
    fn check_for_cbu_dsl_loaded(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(window) = web_sys::window().ok_or("no window") {
            if let Ok(storage) = window.local_storage().ok().flatten().ok_or("no storage") {
                // Check if DSL has been loaded from the database
                if let Ok(Some(dsl_content)) = storage.get_item("data_designer_cbu_dsl_loaded") {
                    if let Ok(Some(cbu_id)) = storage.get_item("data_designer_cbu_dsl_cbu_id") {
                        // Find the CBU name from available CBUs
                        let cbu_name = state.available_cbus.iter()
                            .find(|cbu| cbu.cbu_id == cbu_id)
                            .map(|cbu| cbu.cbu_name.clone())
                            .unwrap_or_else(|| format!("CBU {}", cbu_id));

                        if dsl_content.is_empty() {
                            wasm_utils::console_log(&format!("📭 No DSL found for CBU: {} - leaving editor empty", cbu_name));
                            // Leave editor empty - user should create their own DSL
                            state.dsl_script.clear();
                        } else {
                            wasm_utils::console_log(&format!("✅ Loaded DSL for CBU: {} ({} chars)", cbu_name, dsl_content.len()));
                            state.dsl_script = dsl_content;
                        }

                        // Set active CBU context - this is the key fix
                        state.active_cbu_id = Some(cbu_id.clone());
                        state.active_cbu_name = cbu_name.clone();
                        state.selected_cbu_id = Some(cbu_id.clone());

                        wasm_utils::console_log(&format!("✅ CBU context switched to: {} ({})", cbu_name, cbu_id));

                        // Clear localStorage flags
                        let _ = storage.remove_item("data_designer_cbu_dsl_loaded");
                        let _ = storage.remove_item("data_designer_cbu_dsl_cbu_id");

                        wasm_utils::console_log("✅ DSL loaded and UI updated - user can now edit the DSL");
                    }
                }
            }
            }
        }
    }

    fn render_entity_picker_panel(&mut self, ui: &mut egui::Ui, state: &mut crate::cbu_state_manager::CbuStateManager) {
        ui.separator();
        ui.group(|ui| {
            ui.heading("👥 Smart Entity Picker - Client Entity Table");
            wasm_utils::console_log(&format!("🎯 Rendering entity picker panel with {} entities available", state.available_entities.len()));

            // Track entity selections to avoid borrowing issues
            let mut entity_selections: Vec<(String, String, String)> = Vec::new(); // (entity_id, entity_name, role)
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut self.entity_search_name);

                ui.separator();

                ui.label("🌍 Region:");
                egui::ComboBox::from_id_salt("region_filter")
                    .selected_text(&self.entity_filter_jurisdiction)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.entity_filter_jurisdiction, "All".to_string(), "All Regions");
                        ui.selectable_value(&mut self.entity_filter_jurisdiction, "US".to_string(), "🇺🇸 United States");
                        ui.selectable_value(&mut self.entity_filter_jurisdiction, "EU".to_string(), "🇪🇺 Europe");
                        ui.selectable_value(&mut self.entity_filter_jurisdiction, "APAC".to_string(), "🌏 Asia Pacific");
                    });

                ui.separator();

                ui.label("🏢 Type:");
                egui::ComboBox::from_id_salt("type_filter")
                    .selected_text(&self.entity_filter_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.entity_filter_type, "All".to_string(), "All Types");
                        ui.selectable_value(&mut self.entity_filter_type, "Investment Manager".to_string(), "Investment Manager");
                        ui.selectable_value(&mut self.entity_filter_type, "Asset Owner".to_string(), "Asset Owner");
                        ui.selectable_value(&mut self.entity_filter_type, "Service Provider".to_string(), "Service Provider");
                    });
            });

            ui.add_space(10.0);

            // Filter entities based on search criteria
            let filtered_entities: Vec<&EntityInfo> = state.available_entities.iter()
                .filter(|entity| {
                    // Region filter
                    let region_match = self.entity_filter_jurisdiction == "All" ||
                        (self.entity_filter_jurisdiction == "US" && entity.country_code == "US") ||
                        (self.entity_filter_jurisdiction == "EU" && ["DE", "FR", "CH", "GB", "NL"].contains(&entity.country_code.as_str())) ||
                        (self.entity_filter_jurisdiction == "APAC" && ["JP", "CN", "SG", "NZ", "AU", "KR", "HK", "MY", "TH"].contains(&entity.country_code.as_str()));

                    // Type filter
                    let type_match = self.entity_filter_type == "All" || entity.entity_type == self.entity_filter_type;

                    // Name search (filter-as-you-type)
                    let name_match = self.entity_search_name.is_empty() ||
                        entity.entity_name.to_lowercase().contains(&self.entity_search_name.to_lowercase()) ||
                        entity.entity_id.to_lowercase().contains(&self.entity_search_name.to_lowercase());

                    let passes_filter = region_match && type_match && name_match;
                    if !passes_filter {
                        wasm_utils::console_log(&format!("❌ Entity {} filtered out - region:{}, type:{}, name:{}",
                            entity.entity_id, region_match, type_match, name_match));
                    }
                    passes_filter
                })
                .collect();

            // Log first few entities for debugging
            if !filtered_entities.is_empty() {
                wasm_utils::console_log(&format!("📝 First filtered entity: {} ({})",
                    filtered_entities[0].entity_name, filtered_entities[0].entity_id));
            }

            ui.label(format!("📋 Found {} entities:", filtered_entities.len()));
            wasm_utils::console_log(&format!("🔍 Filtering {} entities -> {} results", state.available_entities.len(), filtered_entities.len()));
            wasm_utils::console_log(&format!("🎯 Current filters - Jurisdiction: '{}', Type: '{}', Search: '{}'",
                self.entity_filter_jurisdiction, self.entity_filter_type, self.entity_search_name));
            ui.separator();

            // Scrollable list of filtered entities
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for entity in &filtered_entities {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Entity info
                                ui.vertical(|ui| {
                                    ui.label(format!("🏢 {}", entity.entity_name));
                                    ui.horizontal(|ui| {
                                        ui.label(format!("🆔 {}", entity.entity_id));
                                        ui.label("•");
                                        ui.label(format!("📍 {}", entity.jurisdiction));
                                        ui.label("•");
                                        ui.label(format!("🏷️ {}", entity.entity_type));
                                    });
                                    if let Some(lei) = &entity.lei_code {
                                        ui.label(format!("🔢 LEI: {}", lei));
                                    }
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Role selection buttons
                                    if ui.button("👤 Asset Owner").clicked() {
                                        entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Asset Owner".to_string()));
                                    }
                                    if ui.button("💼 Investment Manager").clicked() {
                                        entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Investment Manager".to_string()));
                                    }
                                    if ui.button("🔧 Managing Company").clicked() {
                                        entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Managing Company".to_string()));
                                    }
                                });
                            });
                        });
                        ui.add_space(5.0);
                    }

                    if filtered_entities.is_empty() && !state.available_entities.is_empty() {
                        ui.label("🔍 No entities match your search criteria. Try adjusting the filters.");
                    } else if state.available_entities.is_empty() && !state.loading_entities {
                        ui.vertical_centered(|ui| {
                            ui.label("📭 No entities loaded");
                            ui.label("Click 'Load Entities' to fetch from the client entity table");
                        });
                    }
                });

            ui.add_space(10.0);

            // Selected entities preview
            let mut entities_to_remove = Vec::new();
            let mut generate_dsl = false;

            if !state.selected_entities.is_empty() {
                ui.separator();
                ui.label("✅ Selected Entities for CBU:");
                for (i, (entity_id, role)) in state.selected_entities.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {} as {}", entity_id, role));
                        if ui.button("❌").clicked() {
                            entities_to_remove.push(i);
                        }
                    });
                }

                ui.add_space(5.0);
                if ui.button("📝 Generate CBU DSL").clicked() {
                    generate_dsl = true;
                }
            }

            // Process entity selections after UI rendering
            for (entity_id, entity_name, role) in entity_selections {
                self.add_entity_to_dsl(&entity_id, &entity_name, &role, state);
            }

            // Remove entities (in reverse order to maintain indices)
            let mut _entities_removed = false;
            for &i in entities_to_remove.iter().rev() {
                if i < state.selected_entities.len() {
                    let removed_entity = state.selected_entities.remove(i);
                    wasm_utils::console_log(&format!("🗑️ Removed entity: {}", removed_entity.0));
                    _entities_removed = true;
                }
            }

            // REMOVED: Auto-update DSL when entities are removed - this was overriding user edits
            // if entities_removed && self.cbu_context == CbuContext::CreateNew {
            //     self.update_dsl_with_current_entities();
            // }

            // Generate DSL if requested (manual button click)
            if generate_dsl {
                self.generate_cbu_dsl_from_selection(state);
            }
        });
    }

    fn render_floating_entity_picker(&mut self, ctx: &egui::Context, state: &mut crate::cbu_state_manager::CbuStateManager) {
        if !self.show_floating_entity_picker {
            return;
        }

        wasm_utils::console_log("🎯 Rendering SIMPLIFIED floating entity picker");

        let mut open = self.show_floating_entity_picker;

        // FIXED: Removed default_size to allow user resizing without reset
        egui::Window::new("👥 Smart Entity Picker - Client Entity Table")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                // Track entity selections to avoid borrowing issues
                let mut entity_selections: Vec<(String, String, String)> = Vec::new(); // (entity_id, entity_name, role)
                let mut entities_to_remove = Vec::new();
                let mut generate_dsl = false;

                // Filter controls (outside ScrollArea - fixed height)
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    ui.text_edit_singleline(&mut self.entity_search_name);

                    ui.separator();

                    ui.label("🌍 Region:");
                    egui::ComboBox::from_id_salt("floating_region_filter")
                        .selected_text(&self.entity_filter_jurisdiction)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.entity_filter_jurisdiction, "All".to_string(), "All Regions");
                            ui.selectable_value(&mut self.entity_filter_jurisdiction, "US".to_string(), "🇺🇸 United States");
                            ui.selectable_value(&mut self.entity_filter_jurisdiction, "EU".to_string(), "🇪🇺 Europe");
                            ui.selectable_value(&mut self.entity_filter_jurisdiction, "APAC".to_string(), "🌏 Asia Pacific");
                        });

                    ui.separator();

                    ui.label("🏢 Type:");
                    egui::ComboBox::from_id_salt("floating_type_filter")
                        .selected_text(&self.entity_filter_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.entity_filter_type, "All".to_string(), "All Types");
                            ui.selectable_value(&mut self.entity_filter_type, "Investment Manager".to_string(), "Investment Manager");
                            ui.selectable_value(&mut self.entity_filter_type, "Asset Owner".to_string(), "Asset Owner");
                            ui.selectable_value(&mut self.entity_filter_type, "Service Provider".to_string(), "Service Provider");
                        });
                });
                ui.separator();

                // Main content in ScrollArea with FIXED height to prevent window auto-sizing
                let available_height = ui.available_height();
                let scroll_area_height = (available_height - 60.0).max(200.0); // Reserve 60px for bottom buttons

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false]) // Don't shrink to content
                    .max_height(scroll_area_height) // CRITICAL: Fixed maximum height
                    .show(ui, |ui| {
                        // Filter entities based on search criteria
                        let filtered_entities: Vec<&EntityInfo> = state.available_entities.iter()
                            .filter(|entity| {
                                // Region filter
                                let region_match = self.entity_filter_jurisdiction == "All" ||
                                    (self.entity_filter_jurisdiction == "US" && entity.country_code == "US") ||
                                    (self.entity_filter_jurisdiction == "EU" && ["DE", "FR", "CH", "GB", "NL"].contains(&entity.country_code.as_str())) ||
                                    (self.entity_filter_jurisdiction == "APAC" && ["JP", "CN", "SG", "NZ", "AU", "KR", "HK", "MY", "TH"].contains(&entity.country_code.as_str()));

                                // Type filter
                                let type_match = self.entity_filter_type == "All" || entity.entity_type == self.entity_filter_type;

                                // Name search (filter-as-you-type)
                                let name_match = self.entity_search_name.is_empty() ||
                                    entity.entity_name.to_lowercase().contains(&self.entity_search_name.to_lowercase());

                                region_match && type_match && name_match
                            })
                            .collect();

                        ui.label(format!("📋 Found {} entities:", filtered_entities.len()));
                        ui.separator();

                        // Available entities list
                        for entity in &filtered_entities {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    // Entity info
                                    ui.vertical(|ui| {
                                        ui.heading(format!("🏢 {}", entity.entity_name));
                                        ui.horizontal(|ui| {
                                            ui.label(format!("🆔 {}", entity.entity_id));
                                            ui.label("•");
                                            ui.label(format!("📍 {}", entity.jurisdiction));
                                            ui.label("•");
                                            ui.label(format!("🏷️ {}", entity.entity_type));
                                        });
                                        if let Some(lei) = &entity.lei_code {
                                            ui.label(format!("🔢 LEI: {}", lei));
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Role selection buttons
                                        if ui.add_sized([120.0, 30.0], egui::Button::new("👤 Asset Owner")).clicked() {
                                            entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Asset Owner".to_string()));
                                        }
                                        if ui.add_sized([140.0, 30.0], egui::Button::new("💼 Investment Mgr")).clicked() {
                                            entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Investment Manager".to_string()));
                                        }
                                        if ui.add_sized([130.0, 30.0], egui::Button::new("🔧 Managing Co")).clicked() {
                                            entity_selections.push((entity.entity_id.clone(), entity.entity_name.clone(), "Managing Company".to_string()));
                                        }
                                    });
                                });
                            });
                            ui.add_space(8.0);
                        }

                        // Handle empty states
                        if filtered_entities.is_empty() && !state.available_entities.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label("🔍 No entities match your search criteria.");
                                ui.label("Try adjusting the filters above.");
                            });
                        } else if state.available_entities.is_empty() && !state.loading_entities {
                            ui.vertical_centered(|ui| {
                                ui.label("📭 No entities loaded");
                                ui.label("Click 'Load Entities' to fetch from the client entity table");
                            });
                        }

                        // Selected entities list (INSIDE ScrollArea to prevent springing)
                        if !state.selected_entities.is_empty() {
                            ui.separator();
                            ui.label("✅ Selected Entities for CBU:");

                            for (i, (entity_info, role)) in state.selected_entities.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("🏢 {} - 🎭 {}", entity_info, role));
                                    if ui.button("❌").clicked() {
                                        entities_to_remove.push(i);
                                    }
                                });
                            }

                            ui.add_space(5.0);

                            ui.horizontal(|ui| {
                                if ui.button("🚀 Generate CBU DSL").clicked() {
                                    generate_dsl = true;
                                }
                            });
                        }
                    });

                // Bottom buttons (outside ScrollArea - fixed height)
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("✅ Done").min_size(egui::Vec2::new(80.0, 30.0))).clicked() {
                        wasm_utils::console_log("🔄 Done button clicked - generating DSL and closing picker");

                        // Generate DSL first using centralized manager
                        if !state.selected_entities.is_empty() {
                            self.manage_dsl_state(DslOperation::UpdateWithEntities { preserve_header: true }, state);
                        }

                        // Force close the picker - reset both flags
                        self.show_floating_entity_picker = false;
                        self.show_entity_picker = false;
                        wasm_utils::console_log("✅ Entity picker window closed");
                    }

                    if ui.add(egui::Button::new("❌ Cancel").min_size(egui::Vec2::new(80.0, 30.0))).clicked() {
                        // Just close without updating DSL - reset both flags
                        self.show_floating_entity_picker = false;
                        self.show_entity_picker = false;
                        wasm_utils::console_log("❌ Entity picker cancelled");
                    }

                    ui.label("Select entities and roles, then click 'Done'");
                });

                // Process entity selections after UI to avoid borrowing issues
                for (entity_id, entity_name, role) in entity_selections {
                    self.add_entity_to_dsl(&entity_id, &entity_name, &role, state);
                }

                // Remove entities (in reverse order to maintain indices)
                for &i in entities_to_remove.iter().rev() {
                    if i < state.selected_entities.len() {
                        state.selected_entities.remove(i);
                    }
                }

                // Generate DSL if requested and auto-close panel
                if generate_dsl {
                    self.generate_cbu_dsl_from_selection(state);
                    self.show_floating_entity_picker = false; // Auto-close after generating DSL
                }
            });

        // Update state if window was closed via X button
        self.show_floating_entity_picker = open;
    }

    fn add_entity_to_dsl(&mut self, entity_id: &str, entity_name: &str, role: &str, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // Check if this entity+role combination already exists
        let entity_info = format!("{} ({})", entity_name, entity_id);
        if !state.selected_entities.iter().any(|(id, r)| id == &entity_info && r == role) {
            state.selected_entities.push((entity_info, role.to_string()));
            wasm_utils::console_log(&format!("➕ Added entity: {} as {}", entity_name, role));

            // REMOVED: Auto-update DSL in real-time - this was overriding user edits
            // if state.cbu_context == CbuContext::CreateNew {
            //     self.update_dsl_with_current_entities(state);
            // }
        } else {
            wasm_utils::console_log(&format!("⚠️  Entity {} with role {} already selected", entity_name, role));
        }
    }

    fn update_dsl_with_current_entities(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        if state.selected_entities.is_empty() {
            // Don't add any template content - leave editor as user left it
            return;
        }

        // Generate DSL with current entities
        self.generate_cbu_dsl_from_selection(state);
        wasm_utils::console_log("🔄 Auto-updated DSL with current entities");
    }

    fn generate_cbu_dsl_from_selection(&mut self, state: &mut crate::cbu_state_manager::CbuStateManager) {
        // Use single DSL management function with header preservation
        self.manage_dsl_state(DslOperation::UpdateWithEntities { preserve_header: true }, state);
    }

    /// Copy DSL script to clipboard (simplified for egui/gaming context)
    fn copy_to_clipboard(&self, state: &crate::cbu_state_manager::CbuStateManager) {
        // egui games typically don't focus on text editing - simplified implementation
        if !state.dsl_script.is_empty() {
            wasm_utils::console_log(&format!("📋 DSL Content:\n{}", state.dsl_script));
        }
    }

    /// Paste from clipboard - limited in gaming context
    fn paste_from_clipboard(&mut self) {
        // egui doesn't prioritize clipboard access (gaming-focused)
        wasm_utils::console_log("📄 Use browser's paste (Ctrl+V) directly in the text editor");
    }
}

// Syntax highlighting for CBU DSL (simplified)
pub fn highlight_cbu_dsl(ui: &mut egui::Ui, text: &str) {
    // Detect format: LISP or EBNF
    let is_lisp = text.trim_start().starts_with('(') || text.trim_start().starts_with(';');

    if is_lisp {
        highlight_lisp_syntax(ui, text);
    } else {
        highlight_ebnf_syntax(ui, text);
    }
}

fn highlight_lisp_syntax(ui: &mut egui::Ui, text: &str) {
    let lisp_functions = [
        "create-cbu", "update-cbu", "delete-cbu", "query-cbu",
        "entities", "entity", "list", "quote"
    ];
    let role_symbols = [
        "asset-owner", "investment-manager", "managing-company",
        "general-partner", "limited-partner", "prime-broker",
        "administrator", "custodian"
    ];
    let lisp_keywords = ["nil", "true", "false"];

    for line in text.lines() {
        ui.horizontal(|ui| {
            let mut chars = line.chars().peekable();
            let mut current_word = String::new();
            let mut in_string = false;
            let mut paren_depth = 0;

            while let Some(ch) = chars.next() {
                match ch {
                    ';' if !in_string => {
                        // LISP comment - rest of line is comment
                        if !current_word.is_empty() {
                            highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
                            current_word.clear();
                        }
                        let comment = ch.to_string() + &chars.collect::<String>();
                        ui.colored_label(egui::Color32::from_rgb(128, 128, 128), comment);
                        break;
                    }
                    '(' if !in_string => {
                        // Opening parenthesis - highlight as structure
                        if !current_word.is_empty() {
                            highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
                            current_word.clear();
                        }
                        paren_depth += 1;
                        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "(");
                    }
                    ')' if !in_string => {
                        // Closing parenthesis - highlight as structure
                        if !current_word.is_empty() {
                            highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
                            current_word.clear();
                        }
                        paren_depth = paren_depth.saturating_sub(1);
                        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), ")");
                    }
                    '"' => {
                        // String literal handling
                        if !current_word.is_empty() {
                            highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
                            current_word.clear();
                        }

                        if !in_string {
                            // Starting a string
                            in_string = true;
                            let mut string_literal = "\"".to_string();
                            while let Some(str_ch) = chars.next() {
                                string_literal.push(str_ch);
                                if str_ch == '"' && !string_literal.ends_with("\\\"") {
                                    in_string = false;
                                    break;
                                }
                            }
                            ui.colored_label(egui::Color32::from_rgb(255, 255, 150), string_literal);
                        }
                    }
                    ' ' | '\t' | '\n' if !in_string => {
                        // Whitespace - end current word
                        if !current_word.is_empty() {
                            highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
                            current_word.clear();
                        }
                        ui.label(" ");
                    }
                    _ => {
                        current_word.push(ch);
                    }
                }
            }

            // Handle final word
            if !current_word.is_empty() {
                highlight_lisp_word(ui, &current_word, &lisp_functions, &role_symbols, &lisp_keywords, paren_depth);
            }
        });
    }
}

fn highlight_lisp_word(ui: &mut egui::Ui, word: &str, functions: &[&str], roles: &[&str], keywords: &[&str], paren_depth: usize) {
    // Check if it's a number
    if word.parse::<f64>().is_ok() {
        ui.colored_label(egui::Color32::from_rgb(200, 150, 255), word);
    }
    // Check if it's a function (first element in a list gets special treatment)
    else if functions.contains(&word) {
        if paren_depth > 0 {
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), word); // Functions in bright green
        } else {
            ui.colored_label(egui::Color32::from_rgb(150, 150, 255), word); // Functions outside lists
        }
    }
    // Check if it's a role symbol
    else if roles.contains(&word) {
        ui.colored_label(egui::Color32::from_rgb(255, 180, 100), word); // Roles in orange
    }
    // Check if it's a keyword
    else if keywords.contains(&word) {
        ui.colored_label(egui::Color32::from_rgb(200, 100, 200), word); // Keywords in purple
    }
    // Special highlighting for symbols that look like identifiers
    else if word.contains('-') && !word.starts_with('-') {
        ui.colored_label(egui::Color32::from_rgb(150, 200, 255), word); // Hyphenated symbols in light blue
    }
    // Default symbol
    else {
        ui.colored_label(egui::Color32::WHITE, word);
    }
}

fn highlight_ebnf_syntax(ui: &mut egui::Ui, text: &str) {
    let keywords = ["CREATE", "UPDATE", "DELETE", "QUERY", "CBU", "WITH", "ENTITY", "AS", "AND", "SET", "WHERE"];
    let roles = ["Asset Owner", "Investment Manager", "Managing Company"];

    for line in text.lines() {
        ui.horizontal(|ui| {
            for word in line.split_whitespace() {
                if keywords.contains(&word.to_uppercase().as_str()) {
                    ui.colored_label(egui::Color32::BLUE, word);
                } else if roles.iter().any(|role| word.contains(role)) {
                    ui.colored_label(egui::Color32::GREEN, word);
                } else if word.starts_with('\'') && word.ends_with('\'') {
                    ui.colored_label(egui::Color32::YELLOW, word);
                } else if word.starts_with('#') {
                    ui.colored_label(egui::Color32::GRAY, word);
                } else {
                    ui.label(word);
                }
                ui.label(" ");
            }
        });
    }
}