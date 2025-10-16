use eframe::egui;
use crate::http_api_client::{DataDesignerHttpClient, ResourceTemplate};
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

            let client = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log("🔄 Loading all templates from API");

                match client.get_all_templates().await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ Loaded {} templates", response.templates.len()));
                        // Note: In a real implementation, we'd need to update the UI state here
                        // For now, this is the async pattern
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load templates: {:?}", e));
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

                self.saving_template = true;
                self.status_message = format!("Saving template: {}", template_id);

                let client = client.clone();
                let id = template_id.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    wasm_utils::console_log(&format!("💾 Saving template: {}", id));

                    match client.upsert_template(&id, template).await {
                        Ok(response) => {
                            wasm_utils::console_log(&format!("✅ {}", response.message));
                        }
                        Err(e) => {
                            wasm_utils::console_log(&format!("❌ Failed to save template: {:?}", e));
                        }
                    }
                });
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
        ui.heading("📝 Template Editor");
        ui.separator();

        // Connection status and controls
        self.render_connection_status(ui);
        ui.separator();

        // Main layout: Template list on left, Editor on right
        ui.horizontal(|ui| {
            // Left panel: Template list
            ui.vertical(|ui| {
                ui.set_min_width(250.0);
                self.render_template_list(ui);
            });

            ui.separator();

            // Right panel: Template editor
            ui.vertical(|ui| {
                if self.editor_visible {
                    self.render_template_editor(ui);
                } else {
                    self.render_welcome_message(ui);
                }
            });
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
            let template_ids: Vec<String> = self.templates.keys().cloned().collect();
            for template_id in template_ids {
                let is_selected = self.selected_template_id.as_ref() == Some(&template_id);

                // Clone template data for display
                if let Some(template) = self.templates.get(&template_id) {
                    let template_description = template.description.clone();
                    let template_attr_count = template.attributes.len();
                    let template_dsl_lines = template.dsl.lines().count();

                    if ui.selectable_label(is_selected, &template_id).clicked() {
                        self.select_template(&template_id);
                    }

                    // Show template description
                    if is_selected {
                        ui.indent("template_desc", |ui| {
                            ui.small(&template_description);
                            ui.small(format!("{} attributes, {} lines of DSL",
                                template_attr_count,
                                template_dsl_lines));
                        });
                    }
                }
            }
        });
    }

    fn render_template_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(template_id) = self.selected_template_id.clone() {
            ui.horizontal(|ui| {
                ui.heading(&format!("✏️ Editing: {}", template_id));

                if ui.button("💾 Save").clicked() {
                    self.save_current_template();
                }

                if ui.button("❌ Close").clicked() {
                    self.editor_visible = false;
                    self.selected_template_id = None;
                    self.current_template = None;
                }
            });

            ui.separator();

            if self.saving_template {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Saving template...");
                });
                return;
            }

            // Tabbed editor interface
            ui.horizontal(|ui| {
                let _ = ui.selectable_label(true, "📝 Description & DSL");
                let _ = ui.selectable_label(false, "🔧 Attributes");
                let _ = ui.selectable_label(false, "📄 JSON View");
            });

            ui.separator();

            // Description editor
            ui.horizontal(|ui| {
                ui.label("Description:");
            });
            ui.text_edit_multiline(&mut self.description_edit);

            ui.add_space(10.0);

            // DSL editor with larger area
            ui.horizontal(|ui| {
                ui.label("DSL Code:");
            });

            let dsl_response = ui.add(
                egui::TextEdit::multiline(&mut self.dsl_edit)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(20)
                    .desired_width(f32::INFINITY)
            );

            if dsl_response.changed() {
                // Update the current template's DSL
                if let Some(template) = &mut self.current_template {
                    template.dsl = self.dsl_edit.clone();
                }
            }

            ui.add_space(10.0);

            // Quick save reminder
            ui.horizontal(|ui| {
                ui.small("💡 Remember to click");
                if ui.small_button("💾 Save").clicked() {
                    self.save_current_template();
                }
                ui.small("to persist your changes");
            });
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