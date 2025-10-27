use eframe::egui;
use crate::onboarding_state_manager::OnboardingStateManager;

pub struct OnboardingIDE {
    // UI state
    show_yaml_editor: bool,
    show_intent_editor: bool,
    show_output: bool,
}

impl OnboardingIDE {
    pub fn new() -> Self {
        Self {
            show_yaml_editor: true,
            show_intent_editor: true,
            show_output: true,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut OnboardingStateManager) {
        // Load metadata on first render
        if state.metadata.is_none() && !state.metadata_loading {
            state.load_metadata();
        }

        // Top toolbar
        ui.horizontal(|ui| {
            ui.heading("Onboarding Workflow Designer");
            ui.separator();

            if ui.button("Reload Metadata").clicked() {
                state.load_metadata();
            }

            if state.content_modified {
                if ui.button("💾 Save").clicked() {
                    state.save_current_file();
                }
            }

            ui.separator();

            if ui.button("⚙ Compile").clicked() {
                state.compile_workflow();
            }

            if state.compile_result.as_ref().map(|r| r.success).unwrap_or(false) {
                if ui.button("▶ Execute").clicked() {
                    state.execute_workflow();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.show_yaml_editor, "YAML");
                ui.checkbox(&mut self.show_intent_editor, "Intent");
                ui.checkbox(&mut self.show_output, "Output");
            });
        });

        ui.separator();

        // Status bar
        if state.metadata_loading {
            ui.label("Loading metadata...");
        }
        if state.compiling {
            ui.label("Compiling workflow...");
        }
        if state.executing {
            ui.label("Executing workflow...");
        }
        if let Some(error) = &state.metadata_error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
        }

        ui.separator();

        // Main content - three panels
        let panel_count = [self.show_yaml_editor, self.show_intent_editor, self.show_output]
            .iter()
            .filter(|&&x| x)
            .count();

        if panel_count == 0 {
            ui.label("Enable at least one panel to view content");
            return;
        }

        let panel_width = ui.available_width() / panel_count as f32;

        ui.horizontal(|ui| {
            // Panel 1: YAML Config Editor
            if self.show_yaml_editor {
                ui.vertical(|ui| {
                    ui.set_min_width(panel_width - 10.0);
                    ui.set_max_width(panel_width);
                    self.render_yaml_editor(ui, state);
                });
                ui.separator();
            }

            // Panel 2: Intent Editor
            if self.show_intent_editor {
                ui.vertical(|ui| {
                    ui.set_min_width(panel_width - 10.0);
                    ui.set_max_width(panel_width);
                    self.render_intent_editor(ui, state);
                });
                ui.separator();
            }

            // Panel 3: Output Viewer
            if self.show_output {
                ui.vertical(|ui| {
                    ui.set_min_width(panel_width - 10.0);
                    ui.set_max_width(panel_width);
                    self.render_output_viewer(ui, state);
                });
            }
        });
    }

    fn render_yaml_editor(&mut self, ui: &mut egui::Ui, state: &mut OnboardingStateManager) {
        ui.heading("YAML Configuration");

        // Clone dict names to avoid borrowing issues
        let dict_names: Vec<String> = state.metadata
            .as_ref()
            .map(|m| m.resource_dicts.keys().cloned().collect())
            .unwrap_or_default();

        let has_metadata = state.metadata.is_some();

        if has_metadata {
            // File selector
            ui.horizontal(|ui| {
                ui.label("File:");

                if ui.selectable_label(state.selected_file == "product_catalog", "Product Catalog").clicked() {
                    state.select_file("product_catalog");
                }

                if ui.selectable_label(state.selected_file == "cbu_templates", "CBU Templates").clicked() {
                    state.select_file("cbu_templates");
                }

                // Resource dictionaries
                for dict_name in &dict_names {
                    if ui.selectable_label(&state.selected_file == dict_name, dict_name).clicked() {
                        state.select_file(dict_name);
                    }
                }
            });

            ui.separator();

            // Editor
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut state.current_content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                    );

                    if response.changed() {
                        state.content_modified = true;
                    }
                });
        } else {
            ui.label("Loading metadata...");
        }
    }

    fn render_intent_editor(&mut self, ui: &mut egui::Ui, state: &mut OnboardingStateManager) {
        ui.heading("Simple Onboarding");

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.group(|ui| {
                    ui.label("Onboarding Details");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("CBU Name:");
                        ui.text_edit_singleline(&mut state.cbu_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("CBU Description:");
                        ui.text_edit_singleline(&mut state.cbu_description);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Product Name:");
                        ui.text_edit_singleline(&mut state.product_name);
                    });
                });
            });
    }

    fn render_output_viewer(&mut self, ui: &mut egui::Ui, state: &mut OnboardingStateManager) {
        ui.heading("Compiled Output");

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                if let Some(result) = &state.compile_result {
                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, "✓ Compilation Successful");
                        ui.label(&result.message);
                        ui.separator();

                        // Show Plan
                        if let Some(plan) = &result.plan {
                            ui.group(|ui| {
                                ui.label("📋 Execution Plan");
                                ui.separator();

                                let plan_json = serde_json::to_string_pretty(plan).unwrap_or_default();
                                ui.add(
                                    egui::TextEdit::multiline(&mut plan_json.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        }

                        ui.add_space(10.0);

                        // Show IDD (Data Gaps)
                        if let Some(idd) = &result.idd {
                            ui.group(|ui| {
                                ui.label("📊 IDD (Information Dependency Diagram)");
                                ui.separator();

                                let idd_json = serde_json::to_string_pretty(idd).unwrap_or_default();
                                ui.add(
                                    egui::TextEdit::multiline(&mut idd_json.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        }

                        ui.add_space(10.0);

                        // Show Bindings
                        if let Some(bindings) = &result.bindings {
                            ui.group(|ui| {
                                ui.label("🔗 Bindings");
                                ui.separator();

                                let bindings_json = serde_json::to_string_pretty(bindings).unwrap_or_default();
                                ui.add(
                                    egui::TextEdit::multiline(&mut bindings_json.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        }
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Compilation Failed");
                        ui.label(&result.message);
                    }

                    ui.add_space(10.0);
                    ui.separator();
                }

                // Show execution results
                if let Some(result) = &state.execute_result {
                    ui.heading("Execution Results");

                    if result.success {
                        ui.colored_label(egui::Color32::GREEN, "✓ Execution Successful");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Execution Failed");
                    }

                    ui.label(&result.message);

                    if !result.execution_log.is_empty() {
                        ui.separator();
                        ui.label("Execution Log:");

                        for log_entry in &result.execution_log {
                            ui.label(format!("  • {}", log_entry));
                        }
                    }
                }

                if state.compile_result.is_none() && state.execute_result.is_none() {
                    ui.label("Click 'Compile' to generate execution plan");
                }
            });
    }
}
