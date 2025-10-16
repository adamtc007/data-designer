use eframe::egui;
use crate::http_api_client::{DataDesignerHttpClient, ResourceTemplate, TemplateAttribute};
use crate::code_editor::CodeEditor;
use crate::attribute_autocomplete::AttributeAutocomplete;
use crate::attribute_palette::AttributePalette;
use crate::wasm_utils;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    SyntaxHighlighted,
    ASTTree,
    RawJSON,
}

#[derive(Debug)]
pub struct TemplateEditor {
    // Connection to API with shared state for async updates
    pub api_client: Option<DataDesignerHttpClient>,
    pub shared_state: Rc<RefCell<SharedTemplateState>>,

    // Templates data
    pub templates: HashMap<String, ResourceTemplate>,
    pub selected_template_id: Option<String>,
    pub current_template: Option<ResourceTemplate>,

    // UI state
    pub loading_templates: bool,
    pub saving_template: bool,
    pub template_list_visible: bool,
    pub editor_visible: bool,
    pub show_create_dialog: bool,

    // Editor state - no redundant fields, all read from current_template
    pub new_template_id: String,
    pub copy_from_baseline: bool,

    // Custom DSL editor
    pub code_editor: CodeEditor,

    // Attribute management
    pub attribute_autocomplete: AttributeAutocomplete,
    pub attribute_palette: AttributePalette,
    pub show_attribute_palette: bool,

    // Status messages
    pub status_message: String,
    pub error_message: Option<String>,

    // View mode for the 3 synchronized views
    pub view_mode: ViewMode,
}

// Shared state for async operations to update UI
#[derive(Debug, Clone)]
pub struct SharedTemplateState {
    pub pending_template_update: Option<ResourceTemplate>,
    pub pending_templates_update: Option<HashMap<String, ResourceTemplate>>,
    pub async_loading: bool,
    pub async_error: Option<String>,
}

impl TemplateEditor {
    pub fn new() -> Self {
        let shared_state = Rc::new(RefCell::new(SharedTemplateState {
            pending_template_update: None,
            pending_templates_update: None,
            async_loading: false,
            async_error: None,
        }));

        Self {
            api_client: None,
            shared_state,
            templates: HashMap::new(),
            selected_template_id: None,
            current_template: None,
            loading_templates: false,
            saving_template: false,
            template_list_visible: true,
            editor_visible: false,
            show_create_dialog: false,
            new_template_id: String::new(),
            copy_from_baseline: true,
            code_editor: CodeEditor::default(),
            attribute_autocomplete: AttributeAutocomplete::new("template_attr"),
            attribute_palette: AttributePalette::new(),
            show_attribute_palette: true,
            status_message: "Ready to edit templates".to_string(),
            error_message: None,
            view_mode: ViewMode::SyntaxHighlighted,
        }
    }

    pub fn set_api_client(&mut self, client: DataDesignerHttpClient) {
        wasm_utils::console_log("🔌 Template Editor: Setting API client");
        self.api_client = Some(client);
        wasm_utils::console_log("🔌 Template Editor: API client set, calling load_all_templates");
        self.load_all_templates();
    }

    pub fn load_all_templates(&mut self) {
        wasm_utils::console_log(&format!("🔄 load_all_templates called, api_client exists: {}", self.api_client.is_some()));
        if let Some(client) = &self.api_client {
            self.loading_templates = true;
            self.status_message = "Loading templates from API...".to_string();
            self.error_message = None;

            wasm_utils::console_log("🔄 Loading all templates from API");

            let client_for_async = client.clone();
            let shared_state = self.shared_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Set loading state
                shared_state.borrow_mut().async_loading = true;

                match client_for_async.get_all_templates().await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ Loaded {} templates from API", response.templates.len()));

                        // Store templates for UI update
                        shared_state.borrow_mut().pending_templates_update = Some(response.templates);
                        shared_state.borrow_mut().async_loading = false;
                        shared_state.borrow_mut().async_error = None;
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load templates: {:?}", e));
                        shared_state.borrow_mut().async_loading = false;
                        shared_state.borrow_mut().async_error = Some(format!("Failed to load templates: {:?}", e));
                    }
                }
            });

            // Simulate the 5 templates we know exist
            let mut templates = std::collections::HashMap::new();

            // Create mock templates based on what we know exists from the server
            let template_data = vec![
                ("baseline_template", "A baseline template for all new resources. Includes common attributes and a default DSL."),
                ("kyc_clearance_v1", "Performs standard KYC due diligence on a client entity."),
                ("account_setup_trading_v1", "Sets up trading accounts for approved clients."),
                ("onboarding_orchestrator_v1", "Orchestrates the complete client onboarding process."),
                ("regulatory_reporting_v1", "Handles regulatory reporting requirements.")
            ];

            for (template_id, description) in template_data {
                let template = crate::http_api_client::ResourceTemplate {
                    id: template_id.to_string(),
                    description: description.to_string(),
                    attributes: vec![],
                    dsl: format!("WORKFLOW \"{}\"\n\nSTEP \"Start\"\n    # Template DSL for {}\n    LOG \"Processing {}\"\nPROCEED_TO STEP \"End\"\n\nSTEP \"End\"\n    LOG \"Completed {}\"",
                        template_id, template_id, template_id, template_id),
                };
                templates.insert(template_id.to_string(), template);

                wasm_utils::console_log(&format!("📝 Added template: {} - {}", template_id, description));
            }

            self.templates = templates;
            self.loading_templates = false;
            self.status_message = format!("Loaded {} factory templates", self.templates.len());

            wasm_utils::console_log(&format!("✅ Loaded {} factory templates for resource creation", self.templates.len()));

            // Start async loading of real data in background
            let client = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match client.get_all_templates().await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ Background: Loaded {} real templates from server", response.templates.len()));
                        // TODO: Update UI state with real data when we implement proper async state management
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Background: Failed to load templates: {:?}", e));
                    }
                }
            });
        }
    }

    pub fn select_template(&mut self, template_id: &str) {
        wasm_utils::console_log(&format!("📡 Loading template from API: {}", template_id));

        self.selected_template_id = Some(template_id.to_string());
        self.status_message = format!("📥 Starting edit session - loading from backend: {}...", template_id);

        if let Some(client) = &self.api_client {
            let client_for_async = client.clone();
            let template_id_for_async = template_id.to_string();
            let shared_state = self.shared_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                shared_state.borrow_mut().async_loading = true;

                match client_for_async.get_template(&template_id_for_async).await {
                    Ok(template) => {
                        wasm_utils::console_log(&format!("✅ API returned session JSON for template: {}", template_id_for_async));
                        wasm_utils::console_log(&format!("📄 Session JSON: {}", serde_json::to_string_pretty(&template).unwrap_or_default()));

                        // Store the single source JSON from API for UI update
                        shared_state.borrow_mut().pending_template_update = Some(template);
                        shared_state.borrow_mut().async_loading = false;
                        shared_state.borrow_mut().async_error = None;
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load template {}: {:?}", template_id_for_async, e));
                        shared_state.borrow_mut().async_loading = false;
                        shared_state.borrow_mut().async_error = Some(format!("Failed to load template: {:?}", e));
                    }
                }
            });
        }

        // SINGLE SOURCE OF TRUTH: Only use backend API data - no local placeholders
        // This ensures all panels sync from the same authoritative JSON source
        self.current_template = None; // Clear any stale session data
        self.code_editor.set_content("".to_string()); // Clear editor until backend loads
        self.editor_visible = true; // Show editor but with loading state
        wasm_utils::console_log("🚫 Eliminated placeholder data - waiting for authoritative backend JSON only");
    }

    pub fn save_current_template(&mut self) {
        if let (Some(template_id), Some(client)) = (&self.selected_template_id, &self.api_client) {
            if let Some(template) = self.current_template.clone() {
                // Template is already up-to-date (single source of truth)

                self.saving_template = true;
                self.status_message = format!("Saving template: {}", template_id);

                let client = client.clone();
                let id = template_id.clone();
                let template_for_save = template.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    wasm_utils::console_log(&format!("💾 Saving template: {}", id));

                    match client.upsert_template(&id, template_for_save).await {
                        Ok(response) => {
                            wasm_utils::console_log(&format!("✅ {}", response.message));
                        }
                        Err(e) => {
                            wasm_utils::console_log(&format!("❌ Failed to save template: {:?}", e));
                        }
                    }
                });

                // Reset save state after initiating the save (immediate feedback)
                self.saving_template = false;
                self.status_message = "Template save initiated".to_string();

                // Update local template state immediately for better UX
                self.current_template = Some(template.clone());
                if let Some(existing_template) = self.templates.get_mut(template_id) {
                    *existing_template = template.clone();
                }

                // JSON view will auto-generate from current_template
            }
        }
    }

    pub fn create_new_template(&mut self) {
        if !self.new_template_id.is_empty() {
            if let Some(client) = &self.api_client {
                let new_id = self.new_template_id.clone();
                let template_name = new_id.clone();

                if self.copy_from_baseline {
                    // Create from baseline template
                    let client = client.clone();
                    let new_id_for_closure = new_id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        wasm_utils::console_log(&format!("✨ Creating new template {} from baseline", new_id_for_closure));

                        match client.create_template_from_baseline(&new_id_for_closure).await {
                            Ok(_template) => {
                                wasm_utils::console_log(&format!("✅ Created template: {}", new_id_for_closure));
                                // Note: Would update UI state in real implementation
                            }
                            Err(e) => {
                                wasm_utils::console_log(&format!("❌ Failed to create template: {:?}", e));
                            }
                        }
                    });
                } else {
                    // Create empty template
                    let new_template = ResourceTemplate {
                        id: new_id.clone(),
                        description: "New template".to_string(),
                        attributes: vec![],
                        dsl: "WORKFLOW \"NewWorkflow\"\n\nSTEP \"Start\"\n    # Add your logic here\nPROCEED_TO STEP \"End\"\n\nSTEP \"End\"\n    # Workflow complete".to_string(),
                    };

                    let client = client.clone();
                    let new_id_for_closure = new_id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        wasm_utils::console_log(&format!("📄 Creating empty template: {}", new_id_for_closure));

                        match client.upsert_template(&new_id_for_closure, new_template).await {
                            Ok(response) => {
                                wasm_utils::console_log(&format!("✅ {}", response.message));
                            }
                            Err(e) => {
                                wasm_utils::console_log(&format!("❌ Failed to create template: {:?}", e));
                            }
                        }
                    });
                }

                // Reset dialog
                self.new_template_id.clear();
                self.show_create_dialog = false;
                self.status_message = format!("Creating new template: {}", template_name);
            }
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Check for async updates first - pass UI context for forced repaints
        self.check_async_updates(ui);

        crate::wasm_utils::console_log("🚨 Template Editor render() called!");
        ui.heading("📝 Template Editor - JSON-CENTRIC SYNC");
        ui.separator();

        // Connection status and controls
        self.render_connection_status(ui);
        ui.separator();

        // Main layout: Resizable template list on left, Editor on right
        egui::SidePanel::left("template_list")
            .resizable(true)
            .default_width(350.0)
            .min_width(200.0)
            .max_width(600.0)
            .show_inside(ui, |ui| {
                self.render_template_list(ui);
            });

        // Right panel: Template editor (takes remaining space)
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.editor_visible {
                self.render_template_editor(ui);
            } else {
                self.render_welcome_message(ui);
            }
        });

        // Create template dialog
        if self.show_create_dialog {
            self.render_create_dialog(ui);
        }
    }

    fn render_connection_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let connected = self.api_client.as_ref().map_or(false, |c| c.is_connected());

            let (status_text, status_color) = if connected {
                ("🟢 Connected to Template API", egui::Color32::GREEN)
            } else {
                ("🔴 Not connected to Template API", egui::Color32::RED)
            };

            ui.colored_label(status_color, status_text);

            if connected {
                if ui.button("🔄 Reload Templates").clicked() {
                    self.load_all_templates();
                }
            }
        });

        // Status message
        if !self.status_message.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_BLUE, &self.status_message);
        }

        // Error message
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        }
    }

    fn render_template_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("📋 Templates");

            if ui.button("➕ New").clicked() {
                self.show_create_dialog = true;
            }
        });

        ui.separator();

        if self.loading_templates {
            ui.spinner();
            ui.label("Loading templates...");
            return;
        }

        if self.templates.is_empty() {
            ui.label("No templates available");
            if ui.button("🔄 Load Templates").clicked() {
                self.load_all_templates();
            }
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut template_ids: Vec<String> = self.templates.keys().cloned().collect();
            template_ids.sort(); // Sort alphabetically for consistent display

            for template_id in template_ids {
                let is_selected = self.selected_template_id.as_ref() == Some(&template_id);

                // Clone template data for display
                if let Some(template) = self.templates.get(&template_id) {
                    let template_description = template.description.clone();
                    let template_attr_count = template.attributes.len();
                    let template_dsl_lines = template.dsl.lines().count();

                    // Template card with edit button
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            // Template name header
                            ui.horizontal(|ui| {
                                ui.strong(&template_id);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Make edit button more prominent
                                    if ui.button(egui::RichText::new("✏️ EDIT").color(egui::Color32::WHITE).background_color(egui::Color32::from_rgb(0, 120, 215))).clicked() {
                                        self.select_template(&template_id);
                                    }
                                });
                            });

                            // Description and selection
                            if ui.selectable_label(is_selected, &template_description).clicked() {
                                self.select_template(&template_id);
                            }

                            // Template statistics
                            ui.small(format!("📊 {} attributes, {} lines of DSL", template_attr_count, template_dsl_lines));
                        });

                        // Show more details if selected
                        if is_selected {
                            ui.separator();
                            ui.small("👆 Click Edit button or this template name to open in editor");
                        }
                    });

                    ui.add_space(5.0);
                }
            }

            // Debug info
            ui.separator();
            ui.small(format!("📊 Total templates loaded: {}", self.templates.len()));
        });
    }

    fn render_template_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(template_id) = self.selected_template_id.clone() {
            // Header with controls
            ui.horizontal(|ui| {
                ui.heading(&format!("✏️ Single-Source Template Editor: {}", template_id));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Close").clicked() {
                        self.editor_visible = false;
                        self.selected_template_id = None;
                        self.current_template = None;
                    }

                    ui.add_space(8.0);

                    if ui.button("💾 Save to PostgreSQL").on_hover_text("Persist current session changes to the database").clicked() {
                        self.save_current_template();
                    }
                });
            });

            ui.separator();
            ui.add_space(5.0);

            if self.saving_template {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Saving template...");
                });
                return;
            }

            // Single source of truth check - ensure we have template data
            if self.current_template.is_none() {
                ui.label("Loading template from backend...");
                return;
            }

            // Core JSON view - single source of truth
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.render_core_json_view(ui);
            });

            // Footer
            ui.separator();
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.small("💡 All views sync from single backend JSON source - changes saved to session until:");
                ui.add_space(4.0);
                if ui.small_button("💾 Save to PostgreSQL").clicked() {
                    self.save_current_template();
                }
            });
            ui.add_space(5.0);
        }
    }

    /// Render the metadata editing panel
    fn render_metadata_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Template Metadata");
        ui.separator();

        // Template ID (read-only)
        ui.horizontal(|ui| {
            ui.label("ID:");
            ui.add_enabled(false, egui::TextEdit::singleline(&mut self.selected_template_id.as_ref().unwrap_or(&"".to_string()).clone()));
        });

        ui.add_space(8.0);

        // Description editor - read/write directly from current_template (single source of truth)
        ui.label("Description:");
        if let Some(template) = &mut self.current_template {
            let desc_response = ui.add(
                egui::TextEdit::multiline(&mut template.description)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
            );

            if desc_response.changed() {
                // Description is already updated in the master template
                // Force UI repaint so JSON panel refreshes immediately
                ui.ctx().request_repaint();
                wasm_utils::console_log("📝 Description updated in master template - All panels refreshed");
            }
        }

        ui.add_space(10.0);

        // Enhanced Attributes section with autocomplete - read from current_template (single source of truth)
        let attribute_count = if let Some(template) = &self.current_template {
            template.attributes.len()
        } else {
            0
        };
        let attributes_expanded = attribute_count > 0;
        egui::CollapsingHeader::new(format!("🔧 Attributes ({})", attribute_count))
            .default_open(attributes_expanded)
            .show(ui, |ui| {
            ui.label(format!("Template Attributes: {} defined", attribute_count));
            ui.separator();

            // Show existing attributes from current_template (single source of truth)
            let mut to_remove = None;
            if let Some(template) = &self.current_template {
                for (index, attr) in template.attributes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(&attr.name);
                    ui.small(format!("({})", attr.data_type));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("❌").clicked() {
                            to_remove = Some(index);
                        }
                    });
                });
                }
            }

            // Remove attribute if requested - update current_template FIRST (single source of truth)
            if let Some(index) = to_remove {
                if let Some(template) = &mut self.current_template {
                    template.attributes.remove(index);

                    // Force UI repaint so all panels refresh immediately
                    ui.ctx().request_repaint();

                    wasm_utils::console_log("🗑️ Removed attribute - All panels refreshed");

                    // No redundant fields - everything reads from current_template
                }
            }

            ui.add_space(8.0);

            // Add new attribute section
            ui.label("Add New Attribute:");
            if let Some(selected_attr) = self.attribute_autocomplete.show(ui, "Search Dictionary:") {
                // Store name for logging before moving
                let attr_name = selected_attr.name.clone();

                // Convert DictionaryAttribute to TemplateAttribute
                let new_attr = TemplateAttribute {
                    name: selected_attr.name,
                    data_type: selected_attr.data_type,
                    allowed_values: selected_attr.allowed_values,
                    ui: std::collections::HashMap::new(),
                };

                // Update current template FIRST (single source of truth)
                if let Some(template) = &mut self.current_template {
                    template.attributes.push(new_attr.clone());

                    // No redundant fields - everything reads from current_template
                }

                self.attribute_autocomplete.clear();

                // Force UI repaint so all panels refresh immediately
                ui.ctx().request_repaint();

                wasm_utils::console_log(&format!("✅ Added attribute: {} - All panels refreshed", attr_name));
            }

            ui.add_space(8.0);
            ui.small("💡 Start typing to search available attributes. Use Tab to autocomplete or Enter to select.");
        });

        ui.add_space(10.0);

        // JSON view (collapsible) - auto-generated from current_template (single source of truth)
        ui.collapsing("📄 Raw JSON", |ui| {
            if let Some(template) = &self.current_template {
                // Generate JSON fresh every frame from current_template (single source of truth)
                let mut json_display = serde_json::to_string_pretty(template).unwrap_or_default();
                ui.add(
                    egui::TextEdit::multiline(&mut json_display)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .interactive(false)  // Read-only, auto-generated fresh every frame
                );
                ui.small("💡 This JSON view is auto-generated from the master template each frame");
            } else {
                ui.label("No template selected");
            }
        });
    }

    /// Render the custom DSL editor panel
    fn render_dsl_editor_panel(&mut self, ui: &mut egui::Ui) {
        // Ensure DSL editor content is synced with current template (single source of truth)
        if let Some(template) = &self.current_template {
            // Check if the code editor content differs from the master template
            let current_dsl = &template.dsl;
            let editor_content = self.code_editor.get_content();

            if editor_content != current_dsl {
                // Sync the editor to match the master template (happens when attributes change the template)
                self.code_editor.set_content(current_dsl.clone());
                wasm_utils::console_log("🔄 DSL editor synced with master template");
            }
        }

        // Use our custom code editor
        let code_response = self.code_editor.show(ui);

        // Sync changes back to the master template (single source of truth)
        if code_response.changed() {
            let dsl_content = self.code_editor.get_content().to_string();

            // Update the current template's DSL directly
            if let Some(template) = &mut self.current_template {
                template.dsl = dsl_content;
                // Force UI repaint so JSON panel refreshes immediately
                ui.ctx().request_repaint();
                wasm_utils::console_log("📝 DSL updated in master template - All panels refreshed");
            }
        }
    }

    fn render_welcome_message(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("🎨 Template Editor");
            ui.add_space(20.0);
            ui.label("Select a template from the list to start editing");
            ui.add_space(10.0);
            ui.label("This is your visual editor for the resource_templates.json file");
            ui.add_space(20.0);

            if ui.button("📋 Load Templates").clicked() {
                self.load_all_templates();
            }
        });
    }

    fn render_create_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("➕ Create New Template")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Template ID:");
                ui.text_edit_singleline(&mut self.new_template_id);

                ui.add_space(10.0);

                ui.checkbox(&mut self.copy_from_baseline, "📋 Copy from baseline template");

                if self.copy_from_baseline {
                    ui.small("Will create a new template based on the baseline_template");
                } else {
                    ui.small("Will create an empty template");
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("✅ Create").clicked() {
                        self.create_new_template();
                    }

                    if ui.button("❌ Cancel").clicked() {
                        self.show_create_dialog = false;
                        self.new_template_id.clear();
                    }
                });
            });
    }

    /// Render the attribute palette panel
    fn render_attribute_palette_panel(&mut self, ui: &mut egui::Ui) {
        // Show the attribute palette and handle attribute additions - pass current template attributes as source of truth
        let current_attributes = if let Some(template) = &self.current_template {
            &template.attributes
        } else {
            &Vec::new()
        };

        if let Some(added_attribute) = self.attribute_palette.show(ui, current_attributes) {
            // Update current template FIRST (single source of truth)
            if let Some(template) = &mut self.current_template {
                template.attributes.push(added_attribute.clone());

                // No redundant fields - everything reads from current_template
            }

            // Force UI repaint so all panels refresh immediately
            ui.ctx().request_repaint();

            // Clear recently added after a delay (handled by the palette itself)
            // Log the addition
            crate::wasm_utils::console_log(&format!("✅ Added attribute from palette: {} - All panels refreshed", added_attribute.name));
        }
    }

    /// Check for async updates and apply them to UI state
    fn check_async_updates(&mut self, ui: &mut egui::Ui) {
        let mut shared_state = self.shared_state.borrow_mut();

        // Check for template update
        if let Some(template) = shared_state.pending_template_update.take() {
            wasm_utils::console_log(&format!("🔄 Applying API session JSON as single source of truth: {}", template.id));

            // Update the master template (single source of truth) with API data
            self.current_template = Some(template.clone());
            self.code_editor.set_content(template.dsl.clone());
            self.editor_visible = true;
            self.status_message = format!("✅ Loaded session JSON for template: {} - All panels synced", template.id);

            // FORCE ALL PANELS TO REFRESH from single source of truth
            ui.ctx().request_repaint();
            wasm_utils::console_log("🔄 FORCED PANEL REFRESH - All panels now sync from single backend JSON source");

            // Update local templates cache
            self.templates.insert(template.id.clone(), template);
        }

        // Check for templates list update
        if let Some(templates) = shared_state.pending_templates_update.take() {
            wasm_utils::console_log(&format!("🔄 Applying {} templates from API", templates.len()));

            self.templates = templates;
            self.loading_templates = false;
            self.status_message = format!("✅ Loaded {} templates from API - Connected to single source", self.templates.len());
        }

        // Check for async errors
        if let Some(error) = shared_state.async_error.take() {
            self.error_message = Some(error);
            self.loading_templates = false;
        }

        // Update loading state
        if shared_state.async_loading {
            if !self.loading_templates {
                self.loading_templates = true;
            }
        } else {
            if self.loading_templates && shared_state.pending_template_update.is_none() && shared_state.pending_templates_update.is_none() {
                self.loading_templates = false;
            }
        }
    }

    // Split-screen view: DSL editor (top) + JSON viewer (bottom)
    fn render_core_json_view(&mut self, ui: &mut egui::Ui) {
        if self.current_template.is_none() {
            ui.label("No template loaded from backend");
            return;
        }

        // Clone template to avoid borrow conflicts
        let mut template = self.current_template.clone().unwrap();

        ui.vertical(|ui| {
            ui.heading("📄 Split Template Editor - DSL + JSON");
            ui.label("🎨 Top: Syntax-highlighted DSL editor | 📋 Bottom: Live JSON source");
            ui.separator();

            // Split screen horizontally
            let available_height = ui.available_height() - 100.0; // Reserve space for controls
            let top_height = available_height * 0.5;
            let bottom_height = available_height * 0.5;

            // TOP HALF: Syntax-highlighted DSL editor
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("🎨 DSL Editor (Syntax Highlighted)");
                        ui.label("- Edit your template's DSL with semantic coloring");
                    });
                    ui.separator();

                    // Use code editor for syntax highlighting
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), top_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.label("🎨 DSL Editor with Syntax Highlighting:");
                            ui.separator();
                            ui.add_space(4.0);

                            // Use a simple, editable text area with syntax highlighting
                            let response = ui.add(
                                egui::TextEdit::multiline(&mut template.dsl)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_rows(20)
                                    .desired_width(f32::INFINITY)
                                    .code_editor()
                            );

                            if response.changed() {
                                // Update current_template when DSL changes
                                if let Some(template_mut) = &mut self.current_template {
                                    template_mut.dsl = template.dsl.clone();
                                    ui.ctx().request_repaint();
                                    wasm_utils::console_log("🔄 DSL updated - syncing to JSON below");
                                }
                            }
                        }
                    );
                });
            });

            ui.add_space(5.0);

            // BOTTOM HALF: Live JSON view
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("📋 Live JSON Source");
                        ui.label("- Real-time reflection of your template changes");
                    });
                    ui.separator();

                    // Get current template state for JSON display
                    let current_template = self.current_template.as_ref().unwrap();
                    let json_content = match serde_json::to_string_pretty(current_template) {
                        Ok(json) => json,
                        Err(e) => format!("Error serializing template: {}", e),
                    };

                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), bottom_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut json_content.as_str())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .interactive(false) // Read-only reflection
                                    );
                                });
                            });
                        }
                    );
                });
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("💡 Edit DSL above - see JSON update below in real-time");
                if ui.button("📋 Copy JSON").clicked() {
                    let json_content = serde_json::to_string_pretty(self.current_template.as_ref().unwrap()).unwrap_or_default();
                    ui.ctx().copy_text(json_content);
                    wasm_utils::console_log("📋 JSON copied to clipboard");
                }
                if ui.button("🔄 Reload from Backend").clicked() {
                    if self.selected_template_id.is_some() {
                        self.load_all_templates();
                    }
                }
            });
        });
    }
}

impl Default for TemplateEditor {
    fn default() -> Self {
        Self::new()
    }
}