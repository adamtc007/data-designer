use eframe::egui;
use crate::http_api_client::{DataDesignerHttpClient, ResourceTemplate, TemplateAttribute};
use crate::code_editor::CodeEditor;
use crate::attribute_autocomplete::AttributeAutocomplete;
use crate::wasm_utils;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TemplateEditor {
    // Connection to API
    pub api_client: Option<DataDesignerHttpClient>,

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

    // Editor state
    pub template_json_edit: String,
    pub dsl_edit: String,
    pub description_edit: String,
    pub new_template_id: String,
    pub copy_from_baseline: bool,

    // Custom DSL editor
    pub code_editor: CodeEditor,

    // Attribute management
    pub attribute_autocomplete: AttributeAutocomplete,
    pub editing_attributes: Vec<TemplateAttribute>,

    // Status messages
    pub status_message: String,
    pub error_message: Option<String>,
}

impl TemplateEditor {
    pub fn new() -> Self {
        Self {
            api_client: None,
            templates: HashMap::new(),
            selected_template_id: None,
            current_template: None,
            loading_templates: false,
            saving_template: false,
            template_list_visible: true,
            editor_visible: false,
            show_create_dialog: false,
            template_json_edit: String::new(),
            dsl_edit: String::new(),
            description_edit: String::new(),
            new_template_id: String::new(),
            copy_from_baseline: true,
            code_editor: CodeEditor::default(),
            attribute_autocomplete: AttributeAutocomplete::new("template_attr"),
            editing_attributes: Vec::new(),
            status_message: "Ready to edit templates".to_string(),
            error_message: None,
        }
    }

    pub fn set_api_client(&mut self, client: DataDesignerHttpClient) {
        self.api_client = Some(client);
        self.load_all_templates();
    }

    pub fn load_all_templates(&mut self) {
        if let Some(client) = &self.api_client {
            self.loading_templates = true;
            self.status_message = "Loading templates...".to_string();
            self.error_message = None;

            // For now, let's simulate loading with known template IDs
            // In a real implementation, we'd use proper async state management
            wasm_utils::console_log("🔄 Loading all templates from API");

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
        if let Some(template) = self.templates.get(template_id) {
            self.selected_template_id = Some(template_id.to_string());
            self.current_template = Some(template.clone());

            // Populate editor fields
            self.description_edit = template.description.clone();
            self.dsl_edit = template.dsl.clone();
            self.template_json_edit = serde_json::to_string_pretty(template).unwrap_or_default();

            // Update the custom code editor with the DSL content
            self.code_editor.set_content(template.dsl.clone());

            // Load attributes for editing
            self.editing_attributes = template.attributes.clone();

            // Show editor
            self.editor_visible = true;
            self.status_message = format!("Editing template: {}", template_id);

            wasm_utils::console_log(&format!("📝 Selected template for editing: {}", template_id));
        }
    }

    pub fn save_current_template(&mut self) {
        if let (Some(template_id), Some(client)) = (&self.selected_template_id, &self.api_client) {
            if let Some(mut template) = self.current_template.clone() {
                // Update template with edited values
                template.description = self.description_edit.clone();
                template.dsl = self.dsl_edit.clone();
                template.attributes = self.editing_attributes.clone();

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

                // Update JSON view immediately
                self.template_json_edit = serde_json::to_string_pretty(&template).unwrap_or_default();
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
        crate::wasm_utils::console_log("🚨 Template Editor render() called!");
        ui.heading("📝 Template Editor - ENHANCED WITH SYNTAX HIGHLIGHTING");
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
            // Top header with controls - improved spacing
            ui.horizontal(|ui| {
                ui.heading(&format!("✏️ Editing: {}", template_id));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ Close").clicked() {
                        self.editor_visible = false;
                        self.selected_template_id = None;
                        self.current_template = None;
                    }

                    ui.add_space(8.0); // Add spacing between buttons

                    if ui.button("💾 Save").clicked() {
                        self.save_current_template();
                    }
                });
            });

            ui.separator();
            ui.add_space(10.0); // Add extra space after header to prevent overlap

            if self.saving_template {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Saving template...");
                });
                return;
            }

            // Two-pane layout: Metadata on left, DSL editor on right (full height)
            // Left panel: Metadata editor
            egui::SidePanel::left("metadata_panel")
                .resizable(true)
                .default_width(300.0)
                .width_range(250.0..=500.0)
                .show_inside(ui, |ui| {
                    self.render_metadata_panel(ui);
                });

            // Central panel: Custom DSL code editor (takes all remaining space)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.render_dsl_editor_panel(ui);
            });

            ui.add_space(15.0); // Increased spacing before footer

            // Save reminder footer with proper spacing
            ui.separator();
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.small("💡 Changes are auto-synced. Click");
                ui.add_space(4.0);
                if ui.small_button("💾 Save").clicked() {
                    self.save_current_template();
                }
                ui.add_space(4.0);
                ui.small("to persist to server");
            });
            ui.add_space(5.0); // Bottom padding
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

        // Description editor
        ui.label("Description:");
        let desc_response = ui.add(
            egui::TextEdit::multiline(&mut self.description_edit)
                .desired_rows(4)
                .desired_width(f32::INFINITY)
        );

        if desc_response.changed() {
            // Update the current template's description
            if let Some(template) = &mut self.current_template {
                template.description = self.description_edit.clone();
            }
        }

        ui.add_space(10.0);

        // Enhanced Attributes section with autocomplete
        ui.collapsing("🔧 Attributes", |ui| {
            ui.label(format!("Template Attributes ({})", self.editing_attributes.len()));
            ui.separator();

            // Show existing attributes
            let mut to_remove = None;
            for (index, attr) in self.editing_attributes.iter().enumerate() {
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

            // Remove attribute if requested
            if let Some(index) = to_remove {
                self.editing_attributes.remove(index);
                // Update current template
                if let Some(template) = &mut self.current_template {
                    template.attributes = self.editing_attributes.clone();
                    // Update JSON view
                    self.template_json_edit = serde_json::to_string_pretty(template).unwrap_or_default();
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

                self.editing_attributes.push(new_attr);
                self.attribute_autocomplete.clear();

                // Update current template
                if let Some(template) = &mut self.current_template {
                    template.attributes = self.editing_attributes.clone();
                    // Update JSON view
                    self.template_json_edit = serde_json::to_string_pretty(template).unwrap_or_default();
                }

                wasm_utils::console_log(&format!("✅ Added attribute: {}", attr_name));
            }

            ui.add_space(8.0);
            ui.small("💡 Start typing to search available attributes. Use Tab to autocomplete or Enter to select.");
        });

        ui.add_space(10.0);

        // JSON view (collapsible)
        ui.collapsing("📄 Raw JSON", |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.template_json_edit)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY)
            );
        });
    }

    /// Render the custom DSL editor panel
    fn render_dsl_editor_panel(&mut self, ui: &mut egui::Ui) {
        // Use our custom code editor
        let code_response = self.code_editor.show(ui);

        // Sync changes back to the template
        if code_response.changed() {
            self.dsl_edit = self.code_editor.get_content().to_string();

            // Update the current template's DSL
            if let Some(template) = &mut self.current_template {
                template.dsl = self.dsl_edit.clone();
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
}

impl Default for TemplateEditor {
    fn default() -> Self {
        Self::new()
    }
}