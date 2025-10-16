use eframe::egui;
use crate::minimal_types::ResourceSheetRecord;
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use crate::wasm_utils;

/// Web-compatible resource sheet management UI component
#[derive(Debug, Clone)]
pub struct ResourceSheetManager {
    // List view state
    pub resource_sheets: Vec<ResourceSheetRecord>,
    pub selected_sheet_index: Option<usize>,
    pub filter_type: String,
    pub filter_status: String,
    pub search_query: String,
    pub loading: bool,
    pub error_message: Option<String>,

    // Details view state
    pub showing_details: bool,
    pub selected_sheet: Option<ResourceSheetRecord>,

    // JSON editor state
    pub editing_json: bool,
    pub json_editor_content: String,
    pub json_parse_error: Option<String>,

    // Creation state (simplified for web demo)
    pub showing_create_dialog: bool,

    // Web-specific state
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    List,
    Details,
    JsonEditor,
}

impl Default for ResourceSheetManager {
    fn default() -> Self {
        Self {
            resource_sheets: Vec::new(),
            selected_sheet_index: None,
            filter_type: "All".to_string(),
            filter_status: "All".to_string(),
            search_query: String::new(),
            loading: false,
            error_message: None,
            showing_details: false,
            selected_sheet: None,
            editing_json: false,
            json_editor_content: String::new(),
            json_parse_error: None,
            showing_create_dialog: false,
            view_mode: ViewMode::List,
        }
    }
}

impl ResourceSheetManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the web version of the resource sheet manager
    pub fn render_web_version(&mut self, ui: &mut egui::Ui) {
        ui.heading("🗂️ Resource Sheets - Web Demo");
        ui.separator();

        // Web demo notice
        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 100),
            "📱 Web Demo Mode - Sample data loaded for demonstration"
        );
        ui.add_space(10.0);

        // Quick stats
        ui.horizontal(|ui| {
            ui.label(format!("📊 Total Sheets: {}", self.resource_sheets.len()));
            ui.separator();

            let kyc_count = self.resource_sheets.iter()
                .filter(|sheet| sheet.resource_type.contains("KYC"))
                .count();
            ui.label(format!("🔍 KYC: {}", kyc_count));

            let orchestrator_count = self.resource_sheets.iter()
                .filter(|sheet| sheet.resource_type == "Orchestrator")
                .count();
            ui.label(format!("🎭 Orchestrators: {}", orchestrator_count));
        });

        ui.add_space(10.0);

        // View mode selector
        ui.horizontal(|ui| {
            ui.label("View:");
            if ui.selectable_label(self.view_mode == ViewMode::List, "📋 List").clicked() {
                self.view_mode = ViewMode::List;
            }
            if ui.selectable_label(self.view_mode == ViewMode::Details, "🔍 Details").clicked() {
                self.view_mode = ViewMode::Details;
            }
            if ui.selectable_label(self.view_mode == ViewMode::JsonEditor, "📝 JSON").clicked() {
                self.view_mode = ViewMode::JsonEditor;
            }
        });

        ui.separator();

        // Render based on view mode
        match self.view_mode {
            ViewMode::List => self.render_list_view(ui),
            ViewMode::Details => self.render_details_view(ui),
            ViewMode::JsonEditor => self.render_json_editor(ui),
        }
    }

    fn render_list_view(&mut self, ui: &mut egui::Ui) {
        // Search and filters
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.search_query);

            ui.separator();
            ui.label("Type:");
            egui::ComboBox::from_id_salt("filter_type")
                .selected_text(&self.filter_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_type, "All".to_string(), "All");
                    ui.selectable_value(&mut self.filter_type, "Domain_KYC".to_string(), "Domain KYC");
                    ui.selectable_value(&mut self.filter_type, "Orchestrator".to_string(), "Orchestrator");
                });

            ui.separator();
            ui.label("Status:");
            egui::ComboBox::from_id_salt("filter_status")
                .selected_text(&self.filter_status)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_status, "All".to_string(), "All");
                    ui.selectable_value(&mut self.filter_status, "Pending".to_string(), "Pending");
                    ui.selectable_value(&mut self.filter_status, "Executing".to_string(), "Executing");
                    ui.selectable_value(&mut self.filter_status, "Complete".to_string(), "Complete");
                });
        });

        ui.add_space(10.0);

        // Filter resource sheets and clone data to avoid borrowing issues
        let filtered_sheets: Vec<_> = self.resource_sheets
            .iter()
            .enumerate()
            .filter(|(_, sheet)| {
                let matches_search = self.search_query.is_empty() ||
                    sheet.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                    sheet.resource_id.to_lowercase().contains(&self.search_query.to_lowercase());

                let matches_type = self.filter_type == "All" ||
                    sheet.resource_type == self.filter_type;

                let matches_status = self.filter_status == "All" ||
                    sheet.status == self.filter_status;

                matches_search && matches_type && matches_status
            })
            .map(|(index, sheet)| (index, sheet.clone()))
            .collect();

        // Check if empty before iteration
        let is_empty = filtered_sheets.is_empty();

        // Display resource sheets
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (original_index, sheet) in &filtered_sheets {
                self.render_resource_sheet_card_cloned(ui, sheet, *original_index);
            }

            if is_empty {
                ui.centered_and_justified(|ui| {
                    ui.label("📭 No resource sheets match the current filters");
                });
            }
        });
    }

    fn render_resource_sheet_card_cloned(&mut self, ui: &mut egui::Ui, sheet: &ResourceSheetRecord, index: usize) {
        let _selected = self.selected_sheet_index == Some(index);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Status indicator
                let status_color = match sheet.status.as_str() {
                    "Pending" => egui::Color32::YELLOW,
                    "Executing" => egui::Color32::BLUE,
                    "Complete" => egui::Color32::GREEN,
                    "Failed" => egui::Color32::RED,
                    _ => egui::Color32::GRAY,
                };

                ui.colored_label(status_color, "●");

                // Main content
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&sheet.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(&sheet.status);
                        });
                    });

                    ui.label(format!("🆔 {}", sheet.resource_id));
                    ui.label(format!("🏷️ {}", sheet.resource_type));

                    if let Some(description) = &sheet.description {
                        ui.label(format!("📄 {}", description));
                    }

                    if let Some(client_id) = &sheet.client_id {
                        ui.label(format!("👤 Client: {}", client_id));
                    }

                    if let Some(product_id) = &sheet.product_id {
                        ui.label(format!("📦 Product: {}", product_id));
                    }

                    ui.add_space(5.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("🔍 View Details").clicked() {
                            self.selected_sheet_index = Some(index);
                            self.selected_sheet = Some(sheet.clone());
                            self.view_mode = ViewMode::Details;
                            wasm_utils::console_log(&format!("Viewing details for: {}", sheet.resource_id));
                        }

                        if ui.button("📝 Edit JSON").clicked() {
                            self.selected_sheet_index = Some(index);
                            self.selected_sheet = Some(sheet.clone());
                            self.json_editor_content = serde_json::to_string_pretty(&sheet.json_data).unwrap_or_default();
                            self.view_mode = ViewMode::JsonEditor;
                            wasm_utils::console_log(&format!("Editing JSON for: {}", sheet.resource_id));
                        }

                        if ui.button("🎯 Execute").clicked() {
                            wasm_utils::console_log(&format!("Execute clicked for: {}", sheet.resource_id));
                            // In a real implementation, this would trigger DSL execution
                        }
                    });
                });
            });
        });

        ui.add_space(5.0);
    }

    fn render_details_view(&mut self, ui: &mut egui::Ui) {
        if let Some(sheet) = &self.selected_sheet {
            ui.horizontal(|ui| {
                if ui.button("← Back to List").clicked() {
                    self.view_mode = ViewMode::List;
                }
                ui.separator();
                ui.heading("🔍 Resource Sheet Details");
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.heading(&sheet.name);

                    ui.add_space(10.0);

                    // Basic information
                    ui.strong("Basic Information");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("🆔 Resource ID:");
                        ui.label(&sheet.resource_id);
                    });

                    ui.horizontal(|ui| {
                        ui.label("🏷️ Type:");
                        ui.label(&sheet.resource_type);
                    });

                    ui.horizontal(|ui| {
                        ui.label("📊 Status:");
                        let status_color = match sheet.status.as_str() {
                            "Pending" => egui::Color32::YELLOW,
                            "Executing" => egui::Color32::BLUE,
                            "Complete" => egui::Color32::GREEN,
                            "Failed" => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        };
                        ui.colored_label(status_color, &sheet.status);
                    });

                    ui.horizontal(|ui| {
                        ui.label("📅 Version:");
                        ui.label(&sheet.version);
                    });

                    if let Some(description) = &sheet.description {
                        ui.horizontal(|ui| {
                            ui.label("📄 Description:");
                            ui.label(description);
                        });
                    }

                    if let Some(client_id) = &sheet.client_id {
                        ui.horizontal(|ui| {
                            ui.label("👤 Client ID:");
                            ui.label(client_id);
                        });
                    }

                    if let Some(product_id) = &sheet.product_id {
                        ui.horizontal(|ui| {
                            ui.label("📦 Product ID:");
                            ui.label(product_id);
                        });
                    }
                });

                ui.add_space(10.0);

                // JSON data preview
                ui.group(|ui| {
                    ui.strong("📋 Data Preview");
                    ui.separator();

                    if let Ok(formatted_json) = serde_json::to_string_pretty(&sheet.json_data) {
                        let lines: Vec<&str> = formatted_json.lines().take(20).collect();
                        let preview = lines.join("\n");
                        let truncated = lines.len() >= 20;

                        ui.add(
                            egui::TextEdit::multiline(&mut preview.as_str())
                                .code_editor()
                                .desired_rows(15)
                                .desired_width(f32::INFINITY)
                        );

                        if truncated {
                            ui.label("... (truncated, use JSON editor for full view)");
                        }
                    }
                });

                ui.add_space(10.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("📝 Edit JSON").clicked() {
                        self.json_editor_content = serde_json::to_string_pretty(&sheet.json_data).unwrap_or_default();
                        self.view_mode = ViewMode::JsonEditor;
                    }

                    if ui.button("🎯 Execute DSL").clicked() {
                        wasm_utils::console_log(&format!("Execute DSL for: {}", sheet.resource_id));
                        // In a real implementation, this would execute the DSL
                    }
                });
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("📭 No resource sheet selected");
                if ui.button("Back to List").clicked() {
                    self.view_mode = ViewMode::List;
                }
            });
        }
    }

    fn render_json_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← Back to List").clicked() {
                self.view_mode = ViewMode::List;
            }
            ui.separator();
            ui.heading("📝 JSON Editor");
        });

        ui.separator();

        if let Some(sheet) = &self.selected_sheet {
            ui.horizontal(|ui| {
                ui.label(format!("Editing: {}", sheet.name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💾 Save (Demo)").clicked() {
                        // In web demo mode, just validate JSON
                        match serde_json::from_str::<JsonValue>(&self.json_editor_content) {
                            Ok(_) => {
                                self.json_parse_error = None;
                                wasm_utils::console_log("JSON is valid! (Demo mode - not actually saved)");
                            },
                            Err(e) => {
                                self.json_parse_error = Some(format!("JSON Parse Error: {}", e));
                            }
                        }
                    }
                });
            });

            if let Some(error) = &self.json_parse_error {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
            }

            ui.add_space(10.0);

            // JSON editor
            ui.add_sized(
                [ui.available_width(), ui.available_height() - 40.0],
                egui::TextEdit::multiline(&mut self.json_editor_content)
                    .code_editor()
                    .desired_width(f32::INFINITY)
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("📭 No resource sheet selected for editing");
                if ui.button("Back to List").clicked() {
                    self.view_mode = ViewMode::List;
                }
            });
        }
    }
}