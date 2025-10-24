// Resource DSL IDE - Pure UI Component for Resource DSL Management
// Mirrors CbuDslIDE structure for consistency

use eframe::egui;
use crate::resource_state_manager::{ResourceStateManager, ResourceContext};

pub struct ResourceDslIDE {
    // UI state only - no business logic
    search_filter: String,
    show_resource_picker: bool,
}

impl ResourceDslIDE {
    pub fn new() -> Self {
        Self {
            search_filter: String::new(),
            show_resource_picker: false,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut ResourceStateManager) {
        // Load resources on first render
        if state.available_resources.is_empty() && !state.loading_resources {
            state.load_resources();
        }

        // Poll async updates
        state.update_from_async();

        // Header section
        ui.horizontal(|ui| {
            ui.heading("🔧 Resource DSL IDE");
            ui.separator();

            if state.loading_resources {
                ui.spinner();
                ui.label("Loading resources...");
            } else {
                ui.label(format!("{} resources available", state.available_resources.len()));
            }
        });

        ui.separator();

        // Context selection area
        self.render_context_selection(ui, state);

        ui.separator();

        // Main content area - DSL editor or resource picker
        match state.resource_context {
            ResourceContext::None => {
                self.render_welcome_screen(ui, state);
            }
            ResourceContext::CreateNew | ResourceContext::EditExisting => {
                self.render_dsl_editor(ui, state);
            }
        }
    }

    fn render_context_selection(&mut self, ui: &mut egui::Ui, state: &mut ResourceStateManager) {
        ui.horizontal(|ui| {
            ui.label("Mode:");

            if ui.selectable_label(
                state.resource_context == ResourceContext::CreateNew,
                "➕ Create New"
            ).clicked() {
                state.set_resource_context(ResourceContext::CreateNew);
                state.clear();
            }

            if ui.selectable_label(
                state.resource_context == ResourceContext::EditExisting,
                "✏️ Edit Existing"
            ).clicked() {
                state.set_resource_context(ResourceContext::EditExisting);
                self.show_resource_picker = true;
            }

            if state.resource_context != ResourceContext::None {
                ui.separator();
                if ui.button("🔄 Clear").clicked() {
                    state.clear();
                    state.set_resource_context(ResourceContext::None);
                }
            }
        });

        // Show active resource info
        if let Some(resource_id) = &state.active_resource_id {
            ui.horizontal(|ui| {
                ui.label("Active Resource:");
                ui.strong(&state.active_resource_name);
                ui.label(format!("({})", resource_id));
            });
        }
    }

    fn render_welcome_screen(&mut self, ui: &mut egui::Ui, _state: &mut ResourceStateManager) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Welcome to Resource DSL IDE");
            ui.add_space(20.0);
            ui.label("Select a mode above to get started:");
            ui.add_space(10.0);
            ui.label("• Create New - Define a new Resource DSL from scratch");
            ui.label("• Edit Existing - Load and modify an existing Resource DSL");
            ui.add_space(50.0);
        });
    }

    fn render_dsl_editor(&mut self, ui: &mut egui::Ui, state: &mut ResourceStateManager) {
        // Show resource picker window if needed
        if self.show_resource_picker && state.resource_context == ResourceContext::EditExisting {
            self.render_resource_picker_window(ui, state);
        }

        // DSL Editor area
        ui.horizontal(|ui| {
            ui.heading("S-Expression DSL Editor");
            ui.separator();

            if !state.active_resource_id.is_some() && state.resource_context == ResourceContext::EditExisting {
                if ui.button("📂 Pick Resource").clicked() {
                    self.show_resource_picker = true;
                }
            }
        });

        ui.add_space(10.0);

        // Main text editor
        let available_height = ui.available_height() - 100.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(available_height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.dsl_script)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .hint_text(
                            "(resource\n  (kind YourResourceType)\n  (version 1)\n  (attributes\n    (attr name (type string) (visibility public) (required true))\n  )\n  (provisioning\n    (endpoint \"grpc://service/Method\")\n  )\n)"
                        )
                );
            });

        ui.add_space(10.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                state.save_resource_dsl();
            }

            if ui.button("⚙️ Execute").clicked() {
                state.execute_dsl();
            }

            if ui.button("✓ Validate").clicked() {
                // TODO: Add validation
            }

            ui.separator();

            if state.updating_dsl {
                ui.spinner();
                ui.label("Saving...");
            }

            if state.executing_dsl {
                ui.spinner();
                ui.label("Executing...");
            }
        });

        // Show errors if any
        if let Some(error) = &state.last_error {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }

        // Show syntax errors
        if !state.syntax_errors.is_empty() {
            ui.add_space(10.0);
            ui.label("⚠️ Syntax Errors:");
            for error in &state.syntax_errors {
                ui.colored_label(egui::Color32::YELLOW, format!("  • {}", error));
            }
        }
    }

    fn render_resource_picker_window(&mut self, ui: &mut egui::Ui, state: &mut ResourceStateManager) {
        let mut close_picker = false;
        let mut selected_resource_id: Option<String> = None;

        egui::Window::new("📂 Select Resource")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .show(ui.ctx(), |ui| {
                // Search filter
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    ui.text_edit_singleline(&mut self.search_filter);
                    if ui.button("Clear").clicked() {
                        self.search_filter.clear();
                    }
                });

                ui.separator();

                // Resource list
                let available_height = 400.0;
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(available_height)
                    .show(ui, |ui| {
                        let filter = self.search_filter.to_lowercase();

                        for resource in &state.available_resources {
                            if !filter.is_empty() &&
                               !resource.resource_name.to_lowercase().contains(&filter) &&
                               !resource.resource_id.to_lowercase().contains(&filter) {
                                continue;
                            }

                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.strong(&resource.resource_name);
                                        ui.label(format!("ID: {} | Type: {}", resource.resource_id, resource.resource_type));
                                        if let Some(desc) = &resource.description {
                                            ui.label(desc);
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Load DSL").clicked() {
                                            selected_resource_id = Some(resource.resource_id.clone());
                                            close_picker = true;
                                        }
                                    });
                                });
                            });
                        }

                        if state.available_resources.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("No resources found");
                            });
                        }
                    });

                ui.separator();

                // Footer buttons
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close_picker = true;
                    }
                });
            });

        // Load selected resource DSL after window is closed (avoids borrow conflicts)
        if let Some(resource_id) = selected_resource_id {
            state.load_resource_dsl(resource_id);
        }

        if close_picker {
            self.show_resource_picker = false;
            self.search_filter.clear();
        }
    }
}

impl Default for ResourceDslIDE {
    fn default() -> Self {
        Self::new()
    }
}
