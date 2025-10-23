// CBU DSL IDE - Interactive panel for writing and executing CBU CRUD operations
use eframe::egui;
use crate::grpc_client::{GrpcClient, CbuRecord};
use crate::wasm_utils;
use crate::dsl_syntax_highlighter::{DslSyntaxHighlighter, SyntaxTheme};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslRequest {
    pub dsl_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbuDslResponse {
    pub success: bool,
    pub message: String,
    pub cbu_id: Option<String>,
    pub validation_errors: Vec<String>,
    pub data: Option<serde_json::Value>,
}

pub struct CbuDslIDE {
    // DSL Editor state
    dsl_script: String,
    syntax_highlighter: DslSyntaxHighlighter,
    show_syntax_highlighting: bool,
    syntax_errors: Vec<String>,

    // Code completion state
    show_completion_popup: bool,
    completion_suggestions: Vec<String>,
    selected_completion: usize,
    completion_trigger_pos: usize,

    // Execution state - Thread-safe for async/egui synchronization
    executing: Arc<AtomicBool>, // Atomic for lock-free reads from UI thread
    execution_result: Arc<Mutex<Option<CbuDslResponse>>>, // Mutex for safe updates from async

    // Entity loading state - For async entity loading from gRPC
    entities_loading_state: Option<Arc<Mutex<Vec<EntityInfo>>>>,

    // CBU loading state - For async CBU loading from gRPC
    cbus_loading_state: Option<Arc<Mutex<Vec<CbuRecord>>>>,

    // UI state
    show_examples: bool,
    show_help: bool,
    selected_example: usize,

    // Entity lookup for auto-completion
    available_entities: Vec<EntityInfo>,
    loading_entities: bool,

    // CBU Context Selection
    cbu_context: CbuContext,
    selected_cbu_id: Option<String>,
    available_cbus: Vec<CbuRecord>,
    loading_cbus: bool,

    // Create New CBU state
    new_cbu_name: String,
    creating_cbu: bool,

    // Active CBU DSL context
    active_cbu_id: Option<String>,
    active_cbu_name: String,

    // Entity picker state
    show_entity_picker: bool,
    show_floating_entity_picker: bool, // New floating panel state
    entity_picker_window_size: egui::Vec2, // Track window size
    entity_picker_first_open: bool, // Track first open to apply default size only once
    entity_filter_jurisdiction: String,
    entity_filter_type: String,
    entity_search_name: String,
    selected_entities: Vec<(String, String)>, // (entity_id, role)

    // DSL Format Mode
    lisp_mode: bool, // true = LISP S-expressions, false = EBNF format
}

#[derive(Debug, Clone, PartialEq)]
pub enum CbuContext {
    None,        // Initial state - user needs to choose
    CreateNew,   // Creating a new CBU
    EditExisting, // Editing an existing CBU
}

// CbuRecord is now imported from grpc_client module

#[derive(Debug, Clone)]
struct EntityInfo {
    entity_id: String,
    entity_name: String,
    entity_type: String,
    jurisdiction: String,
    country_code: String,
    lei_code: Option<String>,
    status: String,
}

impl Default for CbuDslIDE {
    fn default() -> Self {
        Self::new()
    }
}

// DSL Operation types for single management function
#[derive(Debug, Clone)]
enum DslOperation {
    LoadForCreateNew { cbu_key: String },
    LoadForEdit { cbu_id: String, cbu_name: String, cbu_purpose: String },
    UpdateWithEntities { preserve_header: bool },
    Clear,
}

impl CbuDslIDE {
    pub fn new() -> Self {
        Self {
            dsl_script: String::new(),
            syntax_highlighter: DslSyntaxHighlighter::new(SyntaxTheme::dark_theme()),
            show_syntax_highlighting: true,
            syntax_errors: Vec::new(),

            // Code completion
            show_completion_popup: false,
            completion_suggestions: Vec::new(),
            selected_completion: 0,
            completion_trigger_pos: 0,
            executing: Arc::new(AtomicBool::new(false)),
            execution_result: Arc::new(Mutex::new(None)),
            entities_loading_state: None,
            cbus_loading_state: None,
            show_examples: false,
            show_help: false,
            selected_example: 0,
            available_entities: Vec::new(),
            loading_entities: false,
            cbu_context: CbuContext::None,
            selected_cbu_id: None,
            available_cbus: Vec::new(),
            loading_cbus: false,
            new_cbu_name: String::new(),
            creating_cbu: false,
            active_cbu_id: None,
            active_cbu_name: String::new(),
            show_entity_picker: false,
            show_floating_entity_picker: false,
            entity_picker_window_size: egui::Vec2::new(720.0, 420.0),
            entity_picker_first_open: true,
            entity_filter_jurisdiction: "All".to_string(),
            entity_filter_type: "All".to_string(),
            entity_search_name: String::new(),
            selected_entities: Vec::new(),
            lisp_mode: true, // Default to LISP mode for better parsing
        }
    }

    /// **CBU DSL CONTEXT MANAGER** - Handles switching between create new and edit existing states
    fn start_create_new_cbu(&mut self) {
        wasm_utils::console_log("🆕 Starting Create New CBU flow");
        self.cbu_context = CbuContext::CreateNew;
        self.new_cbu_name.clear();
        self.dsl_script.clear();
        self.active_cbu_id = None;
        self.active_cbu_name.clear();

        // Load empty DSL template for new CBU
        self.dsl_script = "; New CBU DSL Template\n; Enter CBU name below, then add entities using Entity Picker\n\n; CBU Definition:\n; (define-cbu \"[CBU_NAME]\" \"[PURPOSE]\")\n\n; Entities will be added here after using Entity Picker".to_string();
    }

    fn start_edit_existing_cbu(&mut self, grpc_client: Option<&GrpcClient>) {
        wasm_utils::console_log("📝 Starting Edit Existing CBU flow");
        self.cbu_context = CbuContext::EditExisting;
        self.new_cbu_name.clear();
        self.dsl_script.clear();
        self.active_cbu_id = None;
        self.active_cbu_name.clear();

        // Always refresh CBU list when entering edit mode
        self.refresh_cbu_list(grpc_client);
    }

    fn set_active_cbu(&mut self, cbu_id: String, cbu_name: String, grpc_client: Option<&GrpcClient>) {
        wasm_utils::console_log(&format!("🎯 Setting active CBU: {} ({})", cbu_name, cbu_id));
        self.active_cbu_id = Some(cbu_id.clone());
        self.active_cbu_name = cbu_name;

        // Load DSL for this CBU
        self.load_cbu_dsl(&cbu_id, grpc_client);
    }

    fn reset_context(&mut self) {
        wasm_utils::console_log("🔄 Resetting CBU DSL context");
        self.cbu_context = CbuContext::None;
        self.new_cbu_name.clear();
        self.creating_cbu = false;
        self.dsl_script.clear();
        self.active_cbu_id = None;
        self.active_cbu_name.clear();
        self.selected_cbu_id = None;
    }

    /// Refresh CBU list from gRPC (force reload)
    fn refresh_cbu_list(&mut self, grpc_client: Option<&GrpcClient>) {
        wasm_utils::console_log("🔄 Refreshing CBU list from gRPC");
        // Clear existing state to force fresh load
        self.available_cbus.clear();
        self.cbus_loading_state = None;
        // Force load
        self.load_available_cbus(grpc_client);
    }

    /// Create new CBU via gRPC and set up DSL context
    fn create_new_cbu(&mut self, grpc_client: Option<&GrpcClient>) {
        let Some(client) = grpc_client else {
            wasm_utils::console_log("❌ No gRPC client available for CBU creation");
            return;
        };

        if self.new_cbu_name.trim().is_empty() {
            wasm_utils::console_log("❌ CBU name cannot be empty");
            return;
        }

        self.creating_cbu = true;
        let client_clone = client.clone();
        let cbu_name = self.new_cbu_name.trim().to_string();
        let cbu_id = format!("cbu_{}", js_sys::Date::now() as u64); // Generate unique ID

        wasm_utils::console_log(&format!("🔨 Creating new CBU: {}", cbu_name));

        // Build simple DSL for CBU creation (no legal entities)
        let dsl_content = format!(
            "# CBU Creation DSL - Generated {}\n# CBU: {} ({})\n\nCREATE CBU {} '{}' ; 'CBU created via DSL IDE: {}' WITH\n    status = 'active'\n    business_model = 'Standard'\n;\n\n# Ready for additional entities via Entity Picker",
            js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default(),
            cbu_name,
            cbu_id,
            cbu_id,
            cbu_name,
            cbu_name
        );

        wasm_bindgen_futures::spawn_local(async move {
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

                                        // Store the completed CBU and DSL for UI to pick up
                                        let window = web_sys::window().unwrap();
                                        let storage = window.local_storage().unwrap().unwrap();
                                        let _ = storage.set_item("data_designer_new_cbu_created", &serde_json::to_string(&cbu).unwrap_or_default());
                                        let _ = storage.set_item("data_designer_new_cbu_dsl", &dsl_content);
                                        let _ = storage.set_item("data_designer_new_cbu_name", &cbu.cbu_name);
                                        let _ = storage.set_item("data_designer_cbu_creation_complete", "true");
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
    fn manage_dsl_state(&mut self, operation: DslOperation) {
        let new_dsl = match operation {
            DslOperation::LoadForCreateNew { cbu_key } => {
                self.selected_cbu_id = None;
                self.selected_entities.clear();

                // Generate LISP format for new CBU creation
                let dsl = if self.lisp_mode {
                    "; Create a new CBU - add entities below using Entity Picker\n(create-cbu \"New CBU Name\" \"CBU Purpose Description\")\n; Use Entity Picker to add entities".to_string()
                } else {
                    format!(
                        "# Create a new CBU - add entities below using Entity Picker\nCREATE CBU {} 'New CBU Name' ; 'CBU Purpose Description' WITH\n  # Use Entity Picker to add entities",
                        cbu_key
                    )
                };

                wasm_utils::console_log(&format!("📝 DSL initialized for CREATE NEW: {}", cbu_key));
                dsl
            },
            DslOperation::LoadForEdit { cbu_id, cbu_name, cbu_purpose } => {
                self.selected_cbu_id = Some(cbu_id.clone());
                self.selected_entities.clear();

                // Generate LISP format for existing CBU editing
                let dsl = if self.lisp_mode {
                    format!(
                        "; Editing CBU: {}\n; Original CBU Key: {}\n(update-cbu \"{}\" \"{}\" \"{}\")\n; Entity associations will be loaded - use Entity Picker to add entities",
                        cbu_name, cbu_id, cbu_id, cbu_name, cbu_purpose
                    )
                } else {
                    format!(
                        "# Editing CBU: {}\n# Original CBU Key: {}\nUPDATE CBU {} '{}' ; '{}' WITH\n  # Entity associations will be loaded - use Entity Picker to add entities",
                        cbu_name, cbu_id, cbu_id, cbu_name, cbu_purpose
                    )
                };

                wasm_utils::console_log(&format!("📝 DSL initialized for EDIT: {} ({})", cbu_name, cbu_id));
                dsl
            },
            DslOperation::UpdateWithEntities { preserve_header } => {
                if self.selected_entities.is_empty() {
                    wasm_utils::console_log("⚠️  No entities selected for DSL update");
                    return;
                }

                let dsl = if self.lisp_mode {
                    // Generate LISP-style DSL
                    wasm_utils::console_log("🔧 Generating LISP-style DSL");
                    self.build_lisp_dsl()
                } else if preserve_header {
                    // Preserve existing header and update entities (EBNF format)
                    self.build_dsl_preserving_header()
                } else {
                    // Generate completely new DSL (EBNF format)
                    self.build_dsl_from_scratch()
                };
                wasm_utils::console_log(&format!("✅ DSL updated with {} entities", self.selected_entities.len()));
                dsl
            },
            DslOperation::Clear => {
                self.selected_entities.clear();
                self.selected_cbu_id = None;
                wasm_utils::console_log("🧹 DSL state cleared");
                String::new()
            },
        };

        // **EBNF VALIDATION** - Validate DSL syntax before applying changes
        if !new_dsl.is_empty() {
            match self.validate_dsl_syntax(&new_dsl) {
                Ok(_) => {
                    self.dsl_script = new_dsl;
                    wasm_utils::console_log("✅ DSL syntax validation passed");
                },
                Err(validation_error) => {
                    wasm_utils::console_log(&format!("❌ DSL syntax validation failed: {}", validation_error));
                    // Still apply the DSL but log the validation error
                    // This allows the user to see and fix syntax issues
                    self.dsl_script = new_dsl;
                    // TODO: Show validation error in UI
                }
            }
        } else {
            self.dsl_script = new_dsl;
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

        // Check for basic CBU DSL structure
        let dsl_text = dsl.to_string();

        // Validate CREATE CBU syntax
        if dsl_text.contains("CREATE CBU")
            && !self.validate_create_cbu_syntax(&dsl_text) {
                return Err("CREATE CBU syntax error: Expected format 'CREATE CBU <id> '<name>' ; '<purpose>' WITH'".to_string());
            }

        // Validate UPDATE CBU syntax
        if dsl_text.contains("UPDATE CBU")
            && !self.validate_update_cbu_syntax(&dsl_text) {
                return Err("UPDATE CBU syntax error: Expected format 'UPDATE CBU <id> '<name>' ; '<purpose>' WITH'".to_string());
            }

        // Validate ENTITY syntax
        if dsl_text.contains("ENTITY")
            && !self.validate_entity_syntax(&dsl_text) {
                return Err("ENTITY syntax error: Expected format 'ENTITY <id> AS '<role>' # <name>'".to_string());
            }

        Ok(())
    }

    /// Validate CREATE CBU command syntax
    fn validate_create_cbu_syntax(&self, dsl: &str) -> bool {
        // Basic regex-like validation for CREATE CBU pattern
        // TODO: Replace with proper nom parser integration
        for line in dsl.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("CREATE CBU") {
                // Must have: CREATE CBU <id> '<name>' ; '<purpose>' WITH
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 3 {
                    return false;
                }
                if !trimmed.contains('\'') || !trimmed.contains(';') || !trimmed.contains("WITH") {
                    return false;
                }
                return true;
            }
        }
        true // No CREATE CBU found, that's fine
    }

    /// Validate UPDATE CBU command syntax
    fn validate_update_cbu_syntax(&self, dsl: &str) -> bool {
        // Basic validation for UPDATE CBU pattern
        for line in dsl.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("UPDATE CBU") {
                // Must have: UPDATE CBU <id> '<name>' ; '<purpose>' WITH
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() < 3 {
                    return false;
                }
                if !trimmed.contains('\'') || !trimmed.contains(';') || !trimmed.contains("WITH") {
                    return false;
                }
                return true;
            }
        }
        true // No UPDATE CBU found, that's fine
    }

    /// Validate ENTITY command syntax
    fn validate_entity_syntax(&self, dsl: &str) -> bool {
        // Basic validation for ENTITY pattern
        for line in dsl.lines() {
            let trimmed = line.trim();
            if trimmed.contains("ENTITY") && !trimmed.starts_with('#') {
                // Must have: ENTITY <id> AS '<role>' # <name>
                if !trimmed.contains(" AS ") || !trimmed.contains('\'') {
                    return false;
                }
                // Check for valid role
                let valid_roles = ["Asset Owner", "Investment Manager", "Managing Company"];
                let has_valid_role = valid_roles.iter().any(|role| trimmed.contains(role));
                if !has_valid_role {
                    return false;
                }
            }
        }
        true
    }

    /// Helper: Build DSL preserving existing header (used by single DSL manager)
    fn build_dsl_preserving_header(&self) -> String {
        // Parse existing DSL to preserve CBU-level information
        let existing_lines: Vec<&str> = self.dsl_script.lines().collect();
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

        // If no CBU header found, use fallback
        if cbu_header_lines.is_empty() || !found_with {
            if self.cbu_context == CbuContext::CreateNew {
                let new_cbu_key = format!("CBU_{:05}", (js_sys::Date::now() as u64) % 100000);
                let header = format!("CREATE CBU {} 'New CBU Name' ; 'CBU Description' WITH", new_cbu_key);
                cbu_header_lines = vec![header];
            } else {
                let default_id = "CBU_ID".to_string();
                let cbu_id = self.selected_cbu_id.as_ref().unwrap_or(&default_id);
                let header = format!("UPDATE CBU {} WITH", cbu_id);
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
        self.append_entities_to_dsl(&mut new_dsl);

        new_dsl
    }

    /// Helper: Build DSL from scratch (used by single DSL manager)
    fn build_dsl_from_scratch(&self) -> String {
        let mut new_dsl = String::new();

        // Create header based on context
        if self.cbu_context == CbuContext::CreateNew {
            let new_cbu_key = format!("CBU_{:05}", (js_sys::Date::now() as u64) % 100000);
            new_dsl.push_str(&format!("CREATE CBU {} 'New CBU Name' ; 'CBU Description' WITH\n", new_cbu_key));
        } else {
            let default_id = "CBU_ID".to_string();
            let cbu_id = self.selected_cbu_id.as_ref().unwrap_or(&default_id);
            new_dsl.push_str(&format!("UPDATE CBU {} WITH\n", cbu_id));
        }

        // Add entity definitions
        self.append_entities_to_dsl(&mut new_dsl);

        new_dsl
    }

    /// Helper: Append entities to DSL (used by both DSL builders)
    fn append_entities_to_dsl(&self, dsl: &mut String) {
        for (i, (entity_info, role)) in self.selected_entities.iter().enumerate() {
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
    fn build_lisp_dsl(&self) -> String {
        let mut lisp_dsl = String::new();

        // Start comment
        lisp_dsl.push_str("; LISP-style CBU DSL - list processing for financial entities\n");

        // Build S-expression based on context
        if self.cbu_context == CbuContext::CreateNew {
            // Extract CBU name and description from existing DSL if available
            let (cbu_name, cbu_description) = self.extract_cbu_info_from_dsl();

            lisp_dsl.push_str(&format!(
                "(create-cbu \"{}\" \"{}\"\n",
                cbu_name.unwrap_or_else(|| "New CBU Name".to_string()),
                cbu_description.unwrap_or_else(|| "CBU Description".to_string())
            ));
        } else {
            let cbu_id = self.selected_cbu_id.as_ref().unwrap_or(&"CBU_ID".to_string()).clone();
            lisp_dsl.push_str(&format!("(update-cbu \"{}\"\n", cbu_id));
        }

        // Add entities list if we have any
        if !self.selected_entities.is_empty() {
            lisp_dsl.push_str("  (entities\n");

            for (entity_info, role) in &self.selected_entities {
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
    fn extract_cbu_info_from_dsl(&self) -> (Option<String>, Option<String>) {
        if self.dsl_script.is_empty() {
            return (None, None);
        }

        // Parse existing DSL to extract CBU name and description
        for line in self.dsl_script.lines() {
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

    pub fn render(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        // Update entities from async state (60fps compatible)
        self.update_entities_from_async_state();
        self.update_cbus_from_async_state();
        // **60FPS THREAD-SAFE STATE READ** - Read execution state from Arc/Mutex cache

        // Store context for floating window (outside UI constraints)
        let ctx = ui.ctx().clone();
        ui.heading("🏢 CBU DSL Management");
        ui.separator();

        // CBU Context Selection - Prominent at the top
        self.render_cbu_context_selection(ui, grpc_client);

        // Only show the rest of the UI if context is selected
        if self.cbu_context == CbuContext::None {
            return;
        }

        // Auto-load entities if not already loaded and gRPC client is available
        if self.available_entities.is_empty() && !self.loading_entities && grpc_client.is_some() {
            wasm_utils::console_log("🔄 Auto-loading entities for CBU DSL IDE");
            self.load_available_entities(grpc_client, ui.ctx());
        }

        // Toolbar
        self.render_toolbar(ui, grpc_client);

        // Debug info
        ui.horizontal(|ui| {
            ui.label(format!("📊 Entities loaded: {}", self.available_entities.len()));
            if self.loading_entities {
                ui.spinner();
                ui.label("Loading...");
            }
        });

        ui.add_space(10.0);

        // Main content area
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 100.0)
            .show(ui, |ui| {
                self.render_main_content(ui, grpc_client);
            });

        // Render floating entity picker if open (pass ctx directly to avoid CentralPanel constraints)
        // Note: This is called OUTSIDE the ui context to avoid layout constraints from CentralPanel
        self.render_floating_entity_picker(&ctx);
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.horizontal(|ui| {
            // Execute button
            let execute_button = ui.add_enabled(
                !self.dsl_script.trim().is_empty() && !self.executing.load(Ordering::SeqCst) && grpc_client.is_some(),
                egui::Button::new("▶ Execute DSL")
            );

            if execute_button.clicked() {
                self.execute_dsl(grpc_client);
            }

            // Clear button - REMOVED DEFAULT ACTION
            // if ui.button("🗑 Clear").clicked() {
            //     self.dsl_script.clear(); // REMOVED: default action that bypassed gRPC state management
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
                !self.loading_entities && grpc_client.is_some(),
                egui::Button::new("🔄 Load Entities")
            );

            if load_entities_button.clicked() {
                self.load_available_entities(grpc_client, ui.ctx());
            }

            // Entity picker - compact display with expand button
            ui.horizontal(|ui| {
                // Show selected entities count and expand button
                let selected_count = self.selected_entities.len();
                let entities_count = self.available_entities.len();

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

            if self.loading_entities {
                ui.spinner();
                ui.label("Loading entities...");
            }
        });
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, _grpc_client: Option<&GrpcClient>) {
        // Two-column layout
        ui.columns(2, |columns| {
            // Left column: DSL Editor
            columns[0].group(|ui| {
                ui.heading("📝 DSL Script Editor");
                ui.separator();

                // DSL text editor with enhanced IDE features
                let hint_text = r#"Write CBU DSL commands here. Example:

CREATE CBU 'Growth Fund Alpha' ; 'Diversified growth fund' WITH
  ENTITY AC001 AS 'Asset Owner' # Alpha Capital
  ENTITY BM002 AS 'Investment Manager' # Beta Management
  ENTITY GS003 AS 'Managing Company' # Gamma Services"#;

                // Clipboard controls above editor
                ui.horizontal(|ui| {
                    ui.label("📝 DSL Editor:");
                    ui.separator();
                    if ui.button("📋 Copy").on_hover_text("Copy DSL to clipboard").clicked() {
                        self.copy_to_clipboard();
                    }
                    if ui.button("📄 Paste").on_hover_text("Paste from clipboard").clicked() {
                        self.paste_from_clipboard();
                    }
                    // Clear button - REMOVED DEFAULT ACTION
                    // if ui.button("🗑️ Clear").on_hover_text("Clear DSL editor").clicked() {
                    //     self.dsl_script.clear(); // REMOVED: default action that bypassed gRPC state management
                    // }
                });

                // Enhanced DSL editor with hover support
                let editor_response = self.render_enhanced_dsl_editor(ui, hint_text);

                // Auto-completion suggestions
                if editor_response.has_focus() && !self.available_entities.is_empty() {
                    self.show_auto_completion_popup(ui);
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

                if ui.button("📋 Use Example").clicked() {
                    self.dsl_script = selected_example_code.to_string();
                }
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
            ui.label("CBU DSL Syntax Reference:");

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("CREATE CBU");
                    ui.code("CREATE CBU 'name' ; 'description' WITH");
                    ui.code("  ENTITY ('name', 'id') AS 'Asset Owner' AND");
                    ui.code("  ENTITY ('name', 'id') AS 'Investment Manager' AND");
                    ui.code("  ENTITY ('name', 'id') AS 'Managing Company'");

                    ui.add_space(10.0);

                    ui.heading("UPDATE CBU");
                    ui.code("UPDATE CBU 'cbu_id' SET field = 'value'");

                    ui.add_space(10.0);

                    ui.heading("DELETE CBU");
                    ui.code("DELETE CBU 'cbu_id'");

                    ui.add_space(10.0);

                    ui.heading("QUERY CBU");
                    ui.code("QUERY CBU [WHERE condition]");

                    ui.add_space(10.0);

                    ui.heading("Required Roles:");
                    ui.label("• Asset Owner - The entity that owns the assets");
                    ui.label("• Investment Manager - The entity managing investments");
                    ui.label("• Managing Company - The entity providing management services");

                    ui.add_space(10.0);

                    ui.heading("Notes:");
                    ui.label("• All entities must exist in the client entities table");
                    ui.label("• Strings must be quoted with single quotes");
                    ui.label("• CBU IDs are auto-generated for CREATE operations");
                });
            });
        });
    }

    fn show_auto_completion_popup(&self, ui: &mut egui::Ui) {
        // Simple auto-completion based on available entities
        // Simplified implementation - just show entities in a collapsing section
        ui.collapsing("Available Entities", |ui| {
            for entity in &self.available_entities {
                if ui.button(format!("'{}' ({})", entity.entity_name, entity.entity_id)).clicked() {
                    // Insert entity into DSL script (simplified)
                    // In a real implementation, this would insert at cursor position
                }
            }
        });
    }

    fn render_enhanced_dsl_editor(&mut self, ui: &mut egui::Ui, hint_text: &str) -> egui::Response {
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
            if !self.dsl_script.is_empty() {
                self.syntax_errors = self.syntax_highlighter.validate_syntax(&self.dsl_script);
            }

            // Show syntax errors if any
            if !self.syntax_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, "⚠️ Syntax Errors:");
                ui.indent("syntax_errors", |ui| {
                    for error in &self.syntax_errors {
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                });
                ui.separator();
            }

            let editor_response = if self.show_syntax_highlighting && !self.dsl_script.is_empty() {
                // Show syntax-highlighted preview alongside editor
                ui.horizontal(|ui| {
                    // Editor on the left with completion
                    let text_response = ui.vertical(|ui| {
                        let text_response = ui.add_sized(
                            [ui.available_width() * 0.5 - 10.0, 400.0],
                            egui::TextEdit::multiline(&mut self.dsl_script)
                                .code_editor()
                                .hint_text(hint_text)
                        );

                        // Handle completion trigger
                        if text_response.has_focus() {
                            self.handle_completion_input(ui, &text_response);
                        }

                        // Show completion popup if active
                        if self.show_completion_popup {
                            self.render_completion_popup(ui);
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
                                if self.dsl_script.lines().count() > 20 {
                                    // For large files, show with line numbers
                                    self.syntax_highlighter.render_with_line_numbers(ui, &self.dsl_script);
                                } else {
                                    // For smaller files, show highlighted lines
                                    self.syntax_highlighter.render_highlighted_lines(ui, &self.dsl_script);
                                }
                            });
                    });

                    text_response
                }).inner
            } else {
                // Standard editor with completion
                ui.vertical(|ui| {
                    let text_response = ui.add(
                        egui::TextEdit::multiline(&mut self.dsl_script)
                            .desired_width(f32::INFINITY)
                            .desired_rows(15)
                            .code_editor()
                            .hint_text(hint_text)
                    );

                    // Handle completion trigger
                    if text_response.has_focus() {
                        self.handle_completion_input(ui, &text_response);
                    }

                    // Show completion popup if active
                    if self.show_completion_popup {
                        self.render_completion_popup(ui);
                    }

                    text_response
                }).inner
            };

            // Show tooltip on hover - let egui handle the hover detection
            editor_response.on_hover_ui(|ui| {
                self.show_dsl_content_tooltip(ui);
            })
        }).inner
    }

    /// Handle code completion input triggers
    fn handle_completion_input(&mut self, ui: &mut egui::Ui, text_response: &egui::Response) {
        // Check for completion triggers
        let ctx = ui.ctx();

        // Trigger completion on Ctrl+Space
        if ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) {
            self.trigger_completion();
        }

        // Handle completion navigation
        if self.show_completion_popup {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowUp) {
                    if self.selected_completion > 0 {
                        self.selected_completion -= 1;
                    }
                } else if i.key_pressed(egui::Key::ArrowDown) {
                    if self.selected_completion < self.completion_suggestions.len().saturating_sub(1) {
                        self.selected_completion += 1;
                    }
                } else if i.key_pressed(egui::Key::Enter) {
                    self.apply_completion();
                } else if i.key_pressed(egui::Key::Escape) {
                    self.show_completion_popup = false;
                }
            });
        }

        // Auto-trigger completion on typing
        if text_response.changed() {
            // Simple auto-trigger when typing certain characters
            if self.dsl_script.ends_with('(') || self.dsl_script.ends_with(' ') {
                self.trigger_completion();
            }
        }
    }

    /// Trigger code completion
    fn trigger_completion(&mut self) {
        let cursor_pos = self.dsl_script.len();
        self.completion_suggestions = self.syntax_highlighter.get_completions(&self.dsl_script, cursor_pos);

        if !self.completion_suggestions.is_empty() {
            self.show_completion_popup = true;
            self.selected_completion = 0;
            self.completion_trigger_pos = cursor_pos;
        }
    }

    /// Apply the selected completion
    fn apply_completion(&mut self) {
        if let Some(completion) = self.completion_suggestions.get(self.selected_completion) {
            // Find the word at cursor to replace
            let word_start = self.find_word_start(self.completion_trigger_pos);

            // Replace the partial word with the completion
            let before = &self.dsl_script[..word_start];
            let after = &self.dsl_script[self.completion_trigger_pos..];

            self.dsl_script = format!("{}{}{}", before, completion, after);
        }

        self.show_completion_popup = false;
    }

    /// Find the start of the current word
    fn find_word_start(&self, pos: usize) -> usize {
        let chars: Vec<char> = self.dsl_script.chars().collect();
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
    fn render_completion_popup(&mut self, ui: &mut egui::Ui) {
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
                                self.apply_completion();
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

    fn show_dsl_content_tooltip(&self, ui: &mut egui::Ui) {
        // Show contextual tooltips based on DSL content
        ui.label("📝 CBU DSL Editor");
        ui.separator();

        // Count and show CBU and entity references
        let cbu_count = self.dsl_script.lines().filter(|line|
            line.trim().starts_with("CREATE CBU ") || line.trim().starts_with("UPDATE CBU ")
        ).count();

        let entity_count = self.dsl_script.lines().filter(|line|
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
        for line in self.dsl_script.lines() {
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
                    if let Some(cbu) = self.available_cbus.iter().find(|c| c.cbu_id == cbu_key) {
                        ui.label(format!("  Current Name: {}", cbu.cbu_name));
                        ui.label(format!("  Status: {}", cbu.status));
                    }
                    break;
                }
            }
        }

        // Show entity summary
        let entities: Vec<_> = self.dsl_script.lines()
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

CREATE CBU 'Growth Fund Alpha' ; 'Diversified growth fund' WITH
  ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
  ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
  ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'

UPDATE CBU 'CBU001' SET description = 'Updated description'

DELETE CBU 'CBU001'

QUERY CBU WHERE status = 'active'"#
    }

    fn get_dsl_examples(&self) -> Vec<(&str, &str)> {
        vec![
            ("Create CBU", r#"CREATE CBU 'Growth Fund Alpha' ; 'A diversified growth-focused investment fund' WITH
  ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
  ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
  ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'"#),

            ("Update CBU Description", "UPDATE CBU 'CBU001' SET description = 'Updated fund description'"),

            ("Update Multiple Fields", "UPDATE CBU 'CBU001' SET description = 'New description' AND business_model = 'Hedge Fund'"),

            ("Delete CBU", "DELETE CBU 'CBU001'"),

            ("Query All CBUs", "QUERY CBU"),

            ("Query Active CBUs", "QUERY CBU WHERE status = 'active'"),

            ("Query by Name", "QUERY CBU WHERE cbu_name LIKE '%Growth%'"),
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

    fn execute_dsl(&mut self, grpc_client: Option<&GrpcClient>) {
        // **CENTRALIZED DSL EXECUTION** - All execution goes through validation first
        wasm_utils::console_log("🚀 EXECUTE DSL CALLED - Starting execution process");
        wasm_utils::console_log(&format!("📝 DSL Script: '{}'", self.dsl_script));
        wasm_utils::console_log(&format!("🔗 gRPC Client: {}", if grpc_client.is_some() { "Available" } else { "None" }));

        // Step 1: Validate DSL through central manager before execution
        match self.validate_dsl_syntax(&self.dsl_script) {
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

        if let Some(client) = grpc_client {
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
                dsl_script: self.dsl_script.clone(),
            };

            // Clone for async operation - includes thread-safe state
            let client_clone = client.clone();
            let executing_clone = self.executing.clone();
            let result_clone = self.execution_result.clone();

            // **PERFORMANT ASYNC WITH THREAD-SAFE STATE**
            // Async can now safely update UI state through Arc/Mutex
            wasm_bindgen_futures::spawn_local(async move {
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

                // **THREAD-SAFE UI STATE UPDATE** - Update result atomically
                if let Ok(mut result) = result_clone.lock() {
                    *result = Some(final_result);
                }

                // **ATOMIC EXECUTION FLAG** - Clear executing state
                executing_clone.store(false, Ordering::SeqCst);

                wasm_utils::console_log("💡 Execution complete - UI will refresh on next frame");
            });

            // **60FPS EGUI PATTERN** - Immediate UI feedback, async updates state cache
            // UI thread reads atomic flags and mutex state for smooth 60fps performance
        }
    }

    fn simulate_execution(&mut self) {
        // Simulate execution result
        let script = self.dsl_script.trim();

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

    fn render_cbu_context_selection(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.group(|ui| {
            ui.heading("📋 CBU Operation Mode");
            ui.add_space(5.0);

            match self.cbu_context {
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
                            self.start_create_new_cbu();
                        }

                        ui.add_space(20.0);

                        // Edit Existing CBU
                        let edit_button = ui.add_sized(
                            [180.0, 40.0],
                            egui::Button::new("✏️ Edit Existing CBU")
                        );

                        if edit_button.clicked() {
                            self.start_edit_existing_cbu(grpc_client);
                        }
                    });
                },
                CbuContext::CreateNew => {
                    ui.horizontal(|ui| {
                        ui.label("🆕 Mode: Creating New CBU");
                        ui.separator();
                        if ui.button("🔙 Back to Selection").clicked() {
                            self.reset_context();
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
                        let create_enabled = !self.new_cbu_name.trim().is_empty() && !self.creating_cbu;
                        let create_button = ui.add_enabled(
                            create_enabled,
                            egui::Button::new(if self.creating_cbu { "🔄 Creating..." } else { "🔨 Create CBU" })
                                .fill(if create_enabled { egui::Color32::from_rgb(34, 139, 34) } else { egui::Color32::GRAY })
                        );

                        if create_button.clicked() && create_enabled {
                            self.create_new_cbu(grpc_client);
                        }

                        // Auto-focus the text input
                        if name_input.gained_focus() || name_input.has_focus() {
                            // Keep focus active
                        }
                    });

                    // Show creation status
                    if self.creating_cbu {
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
                            self.refresh_cbu_list(grpc_client);
                        }

                        ui.separator();
                        if ui.button("🔙 Back to Selection").clicked() {
                            self.cbu_context = CbuContext::None;
                        }
                    });

                    ui.add_space(10.0);

                    // CBU Selection Section
                    if self.loading_cbus {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading CBUs...");
                        });
                    } else if self.available_cbus.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("⚠️ No CBUs found. Click 'Refresh List' to try again.");
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(format!("📋 Found {} CBUs. Select one:", self.available_cbus.len()));
                        });

                        ui.add_space(5.0);

                        let selected_name = self.selected_cbu_id.as_ref()
                            .and_then(|id| self.available_cbus.iter().find(|cbu| cbu.cbu_id == *id))
                            .map(|cbu| cbu.cbu_name.as_str())
                            .unwrap_or("Choose CBU...");

                        let mut selected_cbu_id_for_loading = None;
                        egui::ComboBox::from_id_salt("cbu_selector")
                            .selected_text(selected_name)
                            .width(400.0)
                            .show_ui(ui, |ui| {
                                for cbu in &self.available_cbus {
                                    let selected = ui.selectable_value(
                                        &mut self.selected_cbu_id,
                                        Some(cbu.cbu_id.clone()),
                                        format!("{} ({})", cbu.cbu_name, cbu.cbu_id)
                                    );

                                    if selected.clicked() {
                                        // Store the ID to load DSL after borrowing ends
                                        selected_cbu_id_for_loading = Some(cbu.cbu_id.clone());
                                        wasm_utils::console_log(&format!("🎯 CBU selected: {} ({})", cbu.cbu_name, cbu.cbu_id));

                                        // Immediately set active CBU context (don't wait for async DSL load)
                                        self.active_cbu_id = Some(cbu.cbu_id.clone());
                                        self.active_cbu_name = cbu.cbu_name.clone();
                                        wasm_utils::console_log(&format!("✅ Active CBU context set: {} ({})", cbu.cbu_name, cbu.cbu_id));
                                    }
                                }
                            });

                        // Load DSL if a CBU was selected
                        if let Some(cbu_id) = selected_cbu_id_for_loading {
                            self.load_cbu_dsl(&cbu_id, grpc_client);
                        }

                        // Show selected CBU info
                        if let Some(selected_id) = &self.selected_cbu_id {
                            if let Some(cbu) = self.available_cbus.iter().find(|c| &c.cbu_id == selected_id) {
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

    fn load_available_cbus(&mut self, grpc_client: Option<&GrpcClient>) {
        let frame_id = crate::trace_enter!("load_available_cbus", "cbu_dsl_ide.rs:1613", grpc_client.is_some());

        if let Some(client) = grpc_client {
            wasm_utils::console_log("🔍 Loading CBUs from gRPC database");
            crate::trace_state!("CbuDslIDE", "loading_cbus", self.loading_cbus, true);
            self.loading_cbus = true;

            let client_clone = client.clone();
            let cbus_state = Arc::new(Mutex::new(Vec::<CbuRecord>::new()));
            let cbus_clone = cbus_state.clone();

            // Store reference for UI updates
            self.cbus_loading_state = Some(cbus_state);

            wasm_bindgen_futures::spawn_local(async move {
                crate::trace_async!("list_cbus_grpc_call", "starting", Some("active filter, limit 100"));

                match client_clone.list_cbus(crate::grpc_client::ListCbusRequest {
                    status_filter: Some("active".to_string()),
                    limit: Some(100),
                    offset: Some(0),
                }).await {
                    Ok(response) => {
                        crate::trace_grpc!("list_cbus", response.cbus.len(), "success");
                        let mut cbus = cbus_clone.lock().unwrap();
                        let old_count = cbus.len();

                        // Add gRPC CBUs directly (they are already CbuRecord)
                        for cbu in response.cbus {
                            cbus.push(cbu);
                        }

                        let new_count = cbus.len();
                        crate::trace_state!("async_cbu_state", "count", old_count, new_count);
                        wasm_utils::console_log(&format!("✅ Successfully loaded {} CBUs from gRPC", new_count));
                        crate::trace_async!("list_cbus_grpc_call", "completed", Some(&format!("{} CBUs stored", new_count)));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load CBUs from gRPC: {} - UI will show empty state", e));
                        crate::trace_async!("list_cbus_grpc_call", "failed", Some(&format!("{:?}", e)));
                        // Don't provide fallback mock data - let UI handle empty state properly
                    }
                }
            });
            crate::trace_exit!(frame_id, "async_task_spawned");
        } else {
            wasm_utils::console_log("⚠️ No gRPC client available - CBU list will be empty");
            self.available_cbus.clear();
            self.loading_cbus = false;
            crate::trace_exit!(frame_id, "no_grpc_client");
        }
    }

    fn load_cbu_dsl(&mut self, cbu_id: &str, grpc_client: Option<&GrpcClient>) {
        let Some(client) = grpc_client else {
            wasm_utils::console_log("❌ No gRPC client available for CBU DSL loading");
            return;
        };

        // Load actual DSL from database via gRPC GetCbu call
        let client_clone = client.clone();
        let cbu_id_clone = cbu_id.to_string();

        wasm_utils::console_log(&format!("🔍 Loading DSL for CBU: {}", cbu_id));

        // Use async task to load CBU and DSL content from database
        wasm_bindgen_futures::spawn_local(async move {
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
                                // Store empty DSL to indicate we need to create one
                                let window = web_sys::window().unwrap();
                                let storage = window.local_storage().unwrap().unwrap();
                                let _ = storage.set_item("data_designer_cbu_dsl_loaded", "");
                                let _ = storage.set_item("data_designer_cbu_dsl_cbu_id", &cbu_id_clone);
                            } else {
                                wasm_utils::console_log(&format!("✅ Loaded DSL for CBU: {} ({} chars)", cbu.cbu_name, dsl_content.len()));
                                // Store actual DSL content for UI to pick up
                                let window = web_sys::window().unwrap();
                                let storage = window.local_storage().unwrap().unwrap();
                                let _ = storage.set_item("data_designer_cbu_dsl_loaded", &dsl_content);
                                let _ = storage.set_item("data_designer_cbu_dsl_cbu_id", &cbu_id_clone);
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

    fn load_available_entities(&mut self, grpc_client: Option<&GrpcClient>, _ctx: &egui::Context) {
        // Load entities from gRPC API - centralized through manage_dsl_state
        self.load_entities_from_grpc(grpc_client);

        // Don't set loading_entities = false here - let the async task complete first
        // The loading state will be updated in update_entities_from_async_state()
        wasm_utils::console_log("🔄 Started loading entities from gRPC...");
    }

    fn update_entities_from_async_state(&mut self) {
        // Check if entities have been loaded from async task
        let should_clear_state = if let Some(loading_state) = &self.entities_loading_state {
            if let Ok(entities) = loading_state.try_lock() {
                if !entities.is_empty() {
                    // Transfer entities from async state to UI state
                    self.available_entities = entities.clone();
                    self.loading_entities = false; // Stop loading animation
                    wasm_utils::console_log(&format!("✅ Updated UI with {} async-loaded entities", entities.len()));
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

        // Clear the async state if we consumed the entities
        if should_clear_state {
            self.entities_loading_state = None;
        }
    }

    fn update_cbus_from_async_state(&mut self) {
        let frame_id = crate::trace_enter!("update_cbus_from_async_state", "cbu_dsl_ide.rs:1728", self.cbus_loading_state.is_some());

        // Check if CBUs have been loaded from async task
        let should_clear_state = if let Some(loading_state) = &self.cbus_loading_state {
            crate::trace_state!("CbuDslIDE", "has_loading_state", false, true);

            if let Ok(cbus) = loading_state.try_lock() {
                crate::trace_state!("CbuDslIDE", "async_lock_acquired", false, true);
                let async_count = cbus.len();

                if !cbus.is_empty() {
                    crate::trace_state!("CbuDslIDE", "async_cbus_available", 0, async_count);

                    // Transfer CBUs from async state to UI state
                    let old_ui_count = self.available_cbus.len();
                    self.available_cbus = cbus.clone();

                    crate::trace_state!("CbuDslIDE", "loading_cbus", self.loading_cbus, false);
                    self.loading_cbus = false; // Stop loading animation

                    crate::trace_state!("CbuDslIDE", "ui_cbu_count", old_ui_count, self.available_cbus.len());
                    wasm_utils::console_log(&format!("✅ Updated UI with {} async-loaded CBUs", self.available_cbus.len()));
                    true
                } else {
                    crate::trace_state!("CbuDslIDE", "async_cbus_empty", 0, 0);
                    false
                }
            } else {
                crate::trace_state!("CbuDslIDE", "async_lock_failed", false, true);
                false
            }
        } else {
            crate::trace_state!("CbuDslIDE", "no_loading_state", false, false);
            false
        };

        // Clear the async state after successful transfer
        if should_clear_state {
            self.cbus_loading_state = None;
            crate::trace_exit!(frame_id, "state_cleared");
        } else {
            crate::trace_exit!(frame_id, "no_state_change");
        }

        // Check for new CBU creation completion from localStorage
        self.check_for_new_cbu_creation();

        // Check for CBU DSL loading completion from localStorage
        self.check_for_cbu_dsl_loaded();
    }

    /// Check for completed CBU creation from async task and switch to active CBU context
    fn check_for_new_cbu_creation(&mut self) {
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
                                    self.cbu_context = CbuContext::EditExisting;
                                    self.active_cbu_id = Some(cbu.cbu_id.clone());
                                    self.active_cbu_name = cbu.cbu_name.clone();
                                    self.dsl_script = dsl_content;
                                    self.creating_cbu = false;
                                    self.new_cbu_name.clear();

                                    // Refresh CBU list to include the new CBU
                                    if !self.available_cbus.iter().any(|c| c.cbu_id == cbu.cbu_id) {
                                        self.available_cbus.push(cbu);
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

    /// Check for completed CBU DSL loading from async task and update DSL content
    fn check_for_cbu_dsl_loaded(&mut self) {
        if let Ok(window) = web_sys::window().ok_or("no window") {
            if let Ok(storage) = window.local_storage().ok().flatten().ok_or("no storage") {
                // Check if DSL has been loaded from the database
                if let Ok(Some(dsl_content)) = storage.get_item("data_designer_cbu_dsl_loaded") {
                    if let Ok(Some(cbu_id)) = storage.get_item("data_designer_cbu_dsl_cbu_id") {
                        // Find the CBU name from available CBUs
                        let cbu_name = self.available_cbus.iter()
                            .find(|cbu| cbu.cbu_id == cbu_id)
                            .map(|cbu| cbu.cbu_name.clone())
                            .unwrap_or_else(|| format!("CBU {}", cbu_id));

                        if dsl_content.is_empty() {
                            wasm_utils::console_log(&format!("📭 No DSL found for CBU: {} - showing empty template", cbu_name));
                            // Show empty template for CBU without DSL
                            self.dsl_script = format!(
                                "# No DSL found for CBU: {}\n# Creating new DSL template\n\nUPDATE CBU {} '{}' ; 'Updated via DSL IDE' WITH\n    status = 'active'\n;\n\n# Add entities using Entity Picker",
                                cbu_name, cbu_id, cbu_name
                            );
                        } else {
                            wasm_utils::console_log(&format!("✅ Loaded DSL for CBU: {} ({} chars)", cbu_name, dsl_content.len()));
                            self.dsl_script = dsl_content;
                        }

                        // Set active CBU context
                        self.active_cbu_id = Some(cbu_id.clone());
                        self.active_cbu_name = cbu_name;
                        self.selected_cbu_id = Some(cbu_id);

                        // Clear localStorage flags
                        let _ = storage.remove_item("data_designer_cbu_dsl_loaded");
                        let _ = storage.remove_item("data_designer_cbu_dsl_cbu_id");

                        wasm_utils::console_log("✅ DSL loaded and UI updated - user can now edit the DSL");
                    }
                }
            }
        }
    }

    fn load_entities_from_grpc(&mut self, grpc_client: Option<&GrpcClient>) {
        if let Some(client) = grpc_client {
            // Use shared state pattern for async-to-UI communication
            let client_clone = client.clone();
            let entities_state = Arc::new(Mutex::new(Vec::<EntityInfo>::new()));
            let entities_clone = entities_state.clone();

            // Store reference for UI updates (using field)
            self.entities_loading_state = Some(entities_state);

            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log("🔄 Loading entities from gRPC API...");

                // Call gRPC GetEntities method
                let request = crate::grpc_client::GetEntitiesRequest {
                    jurisdiction: None,
                    entity_type: None,
                    status: None,
                };

                match client_clone.get_entities(request).await {
                    Ok(response) => {
                        let mut entities = entities_clone.lock().unwrap();

                        // Convert gRPC entities to EntityInfo
                        for entity in response.entities {
                            entities.push(EntityInfo {
                                entity_id: entity.entity_id,
                                entity_name: entity.entity_name,
                                jurisdiction: entity.jurisdiction,
                                entity_type: entity.entity_type,
                                country_code: entity.country_code,
                                lei_code: entity.lei_code, // Already Option<String>
                                status: "active".to_string(), // Default status since gRPC EntityInfo doesn't have status field
                            });
                        }

                        wasm_utils::console_log(&format!("✅ Loaded {} entities from gRPC", entities.len()));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load entities from gRPC: {}", e));

                        // Fallback: Add sample entities for development
                        let mut entities = entities_clone.lock().unwrap();
                        entities.push(EntityInfo {
                            entity_id: "DEV001".to_string(),
                            entity_name: "Development Entity 1".to_string(),
                            jurisdiction: "United States".to_string(),
                            entity_type: "Investment Manager".to_string(),
                            country_code: "US".to_string(),
                            lei_code: Some("DEV123456789012345".to_string()),
                            status: "Active".to_string(),
                        });
                        entities.push(EntityInfo {
                            entity_id: "DEV002".to_string(),
                            entity_name: "Development Entity 2".to_string(),
                            jurisdiction: "United Kingdom".to_string(),
                            entity_type: "Asset Owner".to_string(),
                            country_code: "GB".to_string(),
                            lei_code: Some("DEV987654321098765".to_string()),
                            status: "Active".to_string(),
                        });
                        wasm_utils::console_log("✅ Loaded fallback development entities");
                    }
                }
            });

            // Clear existing entities while loading
            self.available_entities.clear();
        } else {
            wasm_utils::console_log("⚠️ No gRPC client available for loading entities");

            // Provide development entities when no gRPC client
            self.available_entities = vec![
                EntityInfo {
                    entity_id: "LOCAL001".to_string(),
                    entity_name: "Local Development Entity".to_string(),
                    jurisdiction: "United States".to_string(),
                    entity_type: "Investment Manager".to_string(),
                    country_code: "US".to_string(),
                    lei_code: Some("LOCAL12345678901234".to_string()),
                    status: "Active".to_string(),
                },
            ];
        }
    }



    fn render_entity_picker_panel(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.group(|ui| {
            ui.heading("👥 Smart Entity Picker - Client Entity Table");
            wasm_utils::console_log(&format!("🎯 Rendering entity picker panel with {} entities available", self.available_entities.len()));

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
            let filtered_entities: Vec<&EntityInfo> = self.available_entities.iter()
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
            wasm_utils::console_log(&format!("🔍 Filtering {} entities -> {} results", self.available_entities.len(), filtered_entities.len()));
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

                    if filtered_entities.is_empty() && !self.available_entities.is_empty() {
                        ui.label("🔍 No entities match your search criteria. Try adjusting the filters.");
                    } else if self.available_entities.is_empty() && !self.loading_entities {
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

            if !self.selected_entities.is_empty() {
                ui.separator();
                ui.label("✅ Selected Entities for CBU:");
                for (i, (entity_id, role)) in self.selected_entities.iter().enumerate() {
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
                self.add_entity_to_dsl(&entity_id, &entity_name, &role);
            }

            // Remove entities (in reverse order to maintain indices)
            let mut _entities_removed = false;
            for &i in entities_to_remove.iter().rev() {
                if i < self.selected_entities.len() {
                    let removed_entity = self.selected_entities.remove(i);
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
                self.generate_cbu_dsl_from_selection();
            }
        });
    }

    fn render_floating_entity_picker(&mut self, ctx: &egui::Context) {
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
                        let filtered_entities: Vec<&EntityInfo> = self.available_entities.iter()
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
                        if filtered_entities.is_empty() && !self.available_entities.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label("🔍 No entities match your search criteria.");
                                ui.label("Try adjusting the filters above.");
                            });
                        } else if self.available_entities.is_empty() && !self.loading_entities {
                            ui.vertical_centered(|ui| {
                                ui.label("📭 No entities loaded");
                                ui.label("Click 'Load Entities' to fetch from the client entity table");
                            });
                        }

                        // Selected entities list (INSIDE ScrollArea to prevent springing)
                        if !self.selected_entities.is_empty() {
                            ui.separator();
                            ui.label("✅ Selected Entities for CBU:");

                            for (i, (entity_info, role)) in self.selected_entities.iter().enumerate() {
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
                        if !self.selected_entities.is_empty() {
                            self.manage_dsl_state(DslOperation::UpdateWithEntities { preserve_header: true });
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
                    self.add_entity_to_dsl(&entity_id, &entity_name, &role);
                }

                // Remove entities (in reverse order to maintain indices)
                for &i in entities_to_remove.iter().rev() {
                    if i < self.selected_entities.len() {
                        self.selected_entities.remove(i);
                    }
                }

                // Generate DSL if requested and auto-close panel
                if generate_dsl {
                    self.generate_cbu_dsl_from_selection();
                    self.show_floating_entity_picker = false; // Auto-close after generating DSL
                }
            });

        // Update state if window was closed via X button
        self.show_floating_entity_picker = open;
    }

    fn add_entity_to_dsl(&mut self, entity_id: &str, entity_name: &str, role: &str) {
        // Check if this entity+role combination already exists
        let entity_info = format!("{} ({})", entity_name, entity_id);
        if !self.selected_entities.iter().any(|(id, r)| id == &entity_info && r == role) {
            self.selected_entities.push((entity_info, role.to_string()));
            wasm_utils::console_log(&format!("➕ Added entity: {} as {}", entity_name, role));

            // REMOVED: Auto-update DSL in real-time - this was overriding user edits
            // if self.cbu_context == CbuContext::CreateNew {
            //     self.update_dsl_with_current_entities();
            // }
        } else {
            wasm_utils::console_log(&format!("⚠️  Entity {} with role {} already selected", entity_name, role));
        }
    }

    fn update_dsl_with_current_entities(&mut self) {
        if self.selected_entities.is_empty() {
            // Only reset to template if no entities AND DSL is empty or still the default template
            if self.cbu_context == CbuContext::CreateNew {
                let default_template = "# Create a new CBU - add entities below using Entity Picker\nCREATE CBU 'New CBU Name' ; 'CBU Purpose Description' WITH\n  ";
                // Only overwrite if the DSL is empty or still the default template - preserve user edits!
                if self.dsl_script.is_empty() || self.dsl_script.trim() == default_template.trim() {
                    self.dsl_script = default_template.to_string();
                }
                // If user has made edits, don't overwrite them!
            }
            return;
        }

        // Generate DSL with current entities
        self.generate_cbu_dsl_from_selection();
        wasm_utils::console_log("🔄 Auto-updated DSL with current entities");
    }

    fn generate_cbu_dsl_from_selection(&mut self) {
        // Use single DSL management function with header preservation
        self.manage_dsl_state(DslOperation::UpdateWithEntities { preserve_header: true });
    }

    /// Copy DSL script to clipboard (simplified for egui/gaming context)
    fn copy_to_clipboard(&self) {
        // egui games typically don't focus on text editing - simplified implementation
        if !self.dsl_script.is_empty() {
            wasm_utils::console_log(&format!("📋 DSL Content:\n{}", self.dsl_script));
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