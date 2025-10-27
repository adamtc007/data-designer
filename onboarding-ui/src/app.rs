use eframe::egui;
use crate::wasm_utils;
use crate::http_client::GrpcClient;
use crate::state_manager::OnboardingStateManager;

/// Main Onboarding Workflow Platform Application
#[allow(dead_code)] // Constructed in main.rs and lib.rs
pub struct OnboardingApp {
    state: OnboardingStateManager,
    show_yaml_editor: bool,
    show_create_request: bool,
    show_intent_editor: bool,
    show_output: bool,
    current_view: AppView,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppView {
    CreateRequest,
    EditRequest,
}

#[allow(dead_code)] // All methods used by eframe::App trait impl
impl OnboardingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting Onboarding Workflow Platform");

        let client = GrpcClient::new("http://localhost:8080");

        Self {
            state: OnboardingStateManager::new(Some(client)),
            show_yaml_editor: false,
            show_create_request: true,
            show_intent_editor: false,
            show_output: false,
            current_view: AppView::CreateRequest,
        }
    }
}

impl eframe::App for OnboardingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Update state from async operations
        self.state.update_from_async();

        // Load onboarding requests on first render
        if self.state.onboarding_requests.is_empty() && !self.state.metadata_loading {
            self.state.load_metadata();
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.selectable_label(self.current_view == AppView::CreateRequest, "Create Request").clicked() {
                        self.current_view = AppView::CreateRequest;
                    }
                    if ui.selectable_label(self.current_view == AppView::EditRequest, "Edit Request").clicked() {
                        self.current_view = AppView::EditRequest;
                    }
                    ui.separator();
                    ui.checkbox(&mut self.show_yaml_editor, "YAML Editor");
                    ui.checkbox(&mut self.show_intent_editor, "Intent Editor");
                    ui.checkbox(&mut self.show_output, "Output Viewer");
                });
            });
        });

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 Onboarding Workflow Platform");
                ui.separator();

                if ui.button("🔄 Reload Metadata").clicked() {
                    self.state.load_metadata();
                }

                if self.state.content_modified && ui.button("💾 Save").clicked() {
                    self.state.save_current_file();
                }

                ui.separator();

                if ui.button("⚙ Compile Workflow").clicked() {
                    self.state.compile_workflow();
                }

                if self.state.compile_result.as_ref().map(|r| r.success).unwrap_or(false)
                    && ui.button("▶ Execute Workflow").clicked() {
                    self.state.execute_workflow();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.show_yaml_editor, "YAML");
                    ui.checkbox(&mut self.show_intent_editor, "Intent");
                    ui.checkbox(&mut self.show_output, "Output");
                });
            });

            // Status bar
            ui.horizontal(|ui| {
                if self.state.metadata_loading {
                    ui.spinner();
                    ui.label("Loading metadata...");
                }
                if self.state.compiling {
                    ui.spinner();
                    ui.label("Compiling workflow...");
                }
                if self.state.executing {
                    ui.spinner();
                    ui.label("Executing workflow...");
                }
                if let Some(error) = &self.state.metadata_error {
                    ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
                }
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                AppView::CreateRequest => {
                    self.render_create_request_view(ui);
                }
                AppView::EditRequest => {
                    self.render_edit_request_view(ui);
                }
            }
        });
    }
}

impl OnboardingApp {
    fn render_create_request_view(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Create New Onboarding Request");
            ui.add_space(10.0);

            // Database Records List
            if !self.state.onboarding_requests.is_empty() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(format!("📊 Onboarding Requests ({} records)", self.state.onboarding_requests.len()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄 Refresh").clicked() {
                                self.state.load_metadata();
                            }
                            if self.state.metadata_loading {
                                ui.spinner();
                                ui.label("Loading...");
                            }
                        });
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            egui::Grid::new("onboarding_requests_grid")
                                .num_columns(6)
                                .spacing([8.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    // Header
                                    ui.strong("ID");
                                    ui.strong("Onboarding ID");
                                    ui.strong("Name");
                                    ui.strong("Description");
                                    ui.strong("Status");
                                    ui.strong("CBU ID");
                                    ui.end_row();

                                    // Data rows
                                    for request in &self.state.onboarding_requests {
                                        ui.label(request.id.to_string());
                                        ui.label(&request.onboarding_id);
                                        ui.label(request.name.as_deref().unwrap_or("N/A"));
                                        ui.label(request.description.as_deref().unwrap_or("N/A"));
                                        ui.label(&request.status);
                                        ui.label(request.cbu_id.as_deref().unwrap_or("N/A"));
                                        ui.end_row();
                                    }
                                });
                        });
                });
                ui.add_space(15.0);
            } else if self.state.metadata_loading {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading onboarding requests...");
                    });
                });
                ui.add_space(15.0);
            } else {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("📊 No onboarding requests loaded");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄 Load Requests").clicked() {
                                self.state.load_metadata();
                            }
                        });
                    });
                });
                ui.add_space(15.0);
            }

            ui.group(|ui| {
                ui.label("Request Details");
                ui.separator();

                egui::Grid::new("create_request_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.state.request_name);
                        ui.end_row();

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.state.request_description);
                        ui.end_row();

                        ui.label("CBU ID:");
                        egui::ComboBox::new("cbu_selector", "")
                            .selected_text(&self.state.request_cbu_id)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-001".to_string(), "CBU-001: US Growth Equity Fund Alpha");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-002".to_string(), "CBU-002: European Infrastructure Fund Beta");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-003".to_string(), "CBU-003: Asia-Pacific Trade Finance Consortium");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-004".to_string(), "CBU-004: Global Multi-Asset Pension Scheme");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-005".to_string(), "CBU-005: Cross-Border Digital Payments Network");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-006".to_string(), "CBU-006: Emerging Markets Debt Fund");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-007".to_string(), "CBU-007: Nordic Private Equity Fund");
                                ui.selectable_value(&mut self.state.request_cbu_id, "CBU-008".to_string(), "CBU-008: Global Commodity Opportunities Fund");
                            });
                        ui.end_row();
                    });
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let can_create = !self.state.request_name.is_empty() && !self.state.creating_request;

                if ui.add_enabled(can_create, egui::Button::new("Create Request")).clicked() {
                    self.state.create_onboarding_request();
                }

                if self.state.creating_request {
                    ui.spinner();
                    ui.label("Creating...");
                }
            });

            ui.add_space(10.0);

            // Show result
            if let Some(result) = &self.state.create_request_result {
                ui.group(|ui| {
                    if result.success {
                        ui.colored_label(egui::Color32::from_rgb(0, 255, 0), "✓ Request Created Successfully");
                        ui.label(&result.message);
                        if let Some(ref onboarding_id) = result.onboarding_id {
                            ui.label(format!("Onboarding ID: {}", onboarding_id));

                            ui.add_space(5.0);
                            if ui.button("Switch to Edit Request View").clicked() {
                                self.state.current_onboarding_id = Some(onboarding_id.clone());
                                self.current_view = AppView::EditRequest;
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Failed to Create Request");
                        ui.label(&result.message);
                    }
                });

                ui.add_space(10.0);

                // Database Records (Round-trip visibility) - Two Read-Only Windows
                if result.success {
                    ui.add_space(10.0);

                    // Main OB Entity Record Display Window
                    if let Some(ref request_record) = self.state.db_request_record {
                        ui.group(|ui| {
                            ui.heading("📊 Main OB Entity Record");
                            ui.separator();

                            // Format as readable text content
                            let entity_content = format!(
                                "Onboarding Request Entity\n\
                                ========================\n\n\
                                ID: {}\n\
                                Onboarding ID: {}\n\
                                Name: {}\n\
                                Description: {}\n\
                                Status: {}\n\
                                CBU ID: {}\n\
                                Created: {}\n\
                                Updated: {}\n\n\
                                Record successfully retrieved from database via gRPC → HTTP → WASM UI flow.",
                                request_record.id,
                                request_record.onboarding_id,
                                request_record.name.as_deref().unwrap_or("None"),
                                request_record.description.as_deref().unwrap_or("None"),
                                request_record.status,
                                request_record.cbu_id.as_deref().unwrap_or("None"),
                                request_record.created_at,
                                request_record.updated_at
                            );

                            ui.add(
                                egui::TextEdit::multiline(&mut entity_content.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_rows(12)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false) // Read-only
                            );
                        });
                    }

                    ui.add_space(10.0);

                    // OB DSL Content Display Window
                    if let Some(ref dsl_record) = self.state.db_dsl_record {
                        ui.group(|ui| {
                            ui.heading("📋 OB DSL Content");
                            ui.separator();

                            // Format DSL content as readable text
                            let mut dsl_content = format!(
                                "Onboarding DSL Metadata\n\
                                =======================\n\n\
                                DSL ID: {}\n\
                                Request ID: {}\n\
                                Instance ID: {}\n\
                                Products: {}\n\
                                Template Version: {}\n\
                                Created: {}\n\
                                Updated: {}\n\n",
                                dsl_record.id,
                                dsl_record.onboarding_request_id,
                                dsl_record.instance_id.as_deref().unwrap_or("None"),
                                dsl_record.products.as_ref().map(|p| p.join(", ")).unwrap_or("None".to_string()),
                                dsl_record.template_version.as_deref().unwrap_or("None"),
                                dsl_record.created_at,
                                dsl_record.updated_at
                            );

                            // Add JSON content sections
                            if let Some(ref team_users) = dsl_record.team_users {
                                dsl_content.push_str("Team Users JSON:\n");
                                dsl_content.push_str("----------------\n");
                                if let Ok(json_str) = serde_json::to_string_pretty(team_users) {
                                    dsl_content.push_str(&json_str);
                                    dsl_content.push_str("\n\n");
                                }
                            }

                            if let Some(ref cbu_profile) = dsl_record.cbu_profile {
                                dsl_content.push_str("CBU Profile JSON:\n");
                                dsl_content.push_str("-----------------\n");
                                if let Ok(json_str) = serde_json::to_string_pretty(cbu_profile) {
                                    dsl_content.push_str(&json_str);
                                    dsl_content.push_str("\n\n");
                                }
                            }

                            dsl_content.push_str("DSL metadata successfully retrieved from database via gRPC → HTTP → WASM UI flow.");

                            ui.add(
                                egui::TextEdit::multiline(&mut dsl_content.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_rows(20)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false) // Read-only
                            );
                        });
                    }

                    // Loading indicator for database records
                    if self.state.loading_db_records {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading database records...");
                        });
                    }
                }
            }

            if let Some(error) = &self.state.metadata_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }
        });
    }

    fn render_edit_request_view(&mut self, ui: &mut egui::Ui) {
        let panel_count = [self.show_yaml_editor, self.show_intent_editor, self.show_output]
            .iter()
            .filter(|&&x| x)
            .count();

        if panel_count == 0 {
            ui.centered_and_justified(|ui| {
                ui.label("Enable at least one panel using View menu");
            });
            return;
        }

        // Use equal-width columns
        ui.columns(panel_count, |columns| {
            let mut col_idx = 0;

            if self.show_yaml_editor {
                self.render_yaml_editor(&mut columns[col_idx]);
                col_idx += 1;
            }

            if self.show_intent_editor {
                self.render_intent_editor(&mut columns[col_idx]);
                col_idx += 1;
            }

            if self.show_output {
                self.render_output_viewer(&mut columns[col_idx]);
            }
        });
    }
}

impl OnboardingApp {
    fn render_yaml_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("📄 YAML Configuration");

        let dict_names: Vec<String> = self.state.metadata
            .as_ref()
            .map(|m| m.resource_dicts.keys().cloned().collect())
            .unwrap_or_default();

        let has_metadata = self.state.metadata.is_some();

        if has_metadata {
            ui.horizontal_wrapped(|ui| {
                ui.label("File:");

                if ui.selectable_label(self.state.selected_file == "product_catalog", "📦 Product Catalog").clicked() {
                    self.state.select_file("product_catalog");
                }

                if ui.selectable_label(self.state.selected_file == "cbu_templates", "📋 CBU Templates").clicked() {
                    self.state.select_file("cbu_templates");
                }

                for dict_name in &dict_names {
                    if ui.selectable_label(&self.state.selected_file == dict_name, format!("📚 {}", dict_name)).clicked() {
                        self.state.select_file(dict_name);
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.state.current_content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                    );

                    if response.changed() {
                        self.state.content_modified = true;
                    }
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.label("Loading metadata...");
                });
            });
        }
    }

    fn render_intent_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Onboard Intent");

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.group(|ui| {
                    ui.label("Instance Details");
                    ui.separator();

                    egui::Grid::new("intent_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Instance ID:");
                            ui.text_edit_singleline(&mut self.state.instance_id);
                            ui.end_row();

                            ui.label("CBU ID:");
                            ui.text_edit_singleline(&mut self.state.cbu_id);
                            ui.end_row();

                            ui.label("Products:");
                            ui.text_edit_singleline(&mut self.state.products_input);
                            ui.end_row();
                        });

                    ui.label("(comma-separated product IDs)");
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label("👥 Team Users (JSON Array)");
                    ui.separator();

                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.team_users_input)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                    );
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label("🏢 CBU Profile (JSON Object)");
                    ui.separator();

                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.cbu_profile_input)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(6)
                    );
                });
            });
    }

    fn render_output_viewer(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Compiled Output");

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                if let Some(result) = &self.state.compile_result {
                    if result.success {
                        ui.colored_label(egui::Color32::from_rgb(0, 255, 0), "✓ Compilation Successful");
                        ui.label(&result.message);
                        ui.separator();

                        // Execution Plan
                        if let Some(plan) = &result.plan {
                            ui.collapsing("📋 Execution Plan", |ui| {
                                let plan_json = serde_json::to_string_pretty(plan).unwrap_or_default();
                                ui.add(
                                    egui::TextEdit::multiline(&mut plan_json.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        }

                        ui.add_space(5.0);

                        // IDD (Information Dependency Diagram)
                        if let Some(idd) = &result.idd {
                            ui.collapsing("📊 IDD (Information Dependency Diagram)", |ui| {
                                let idd_json = serde_json::to_string_pretty(idd).unwrap_or_default();
                                ui.add(
                                    egui::TextEdit::multiline(&mut idd_json.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                );
                            });
                        }

                        ui.add_space(5.0);

                        // Bindings
                        if let Some(bindings) = &result.bindings {
                            ui.collapsing("🔗 Bindings", |ui| {
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

                // Execution Results
                if let Some(result) = &self.state.execute_result {
                    ui.heading("Execution Results");

                    if result.success {
                        ui.colored_label(egui::Color32::from_rgb(0, 255, 0), "✓ Execution Successful");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Execution Failed");
                    }

                    ui.label(&result.message);

                    if !result.execution_log.is_empty() {
                        ui.separator();
                        ui.label("📝 Execution Log:");

                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for log_entry in &result.execution_log {
                                    ui.label(format!("  • {}", log_entry));
                                }
                            });
                    }
                }

                if self.state.compile_result.is_none() && self.state.execute_result.is_none() {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Click '⚙ Compile Workflow' to generate execution plan");
                        });
                    });
                }
            });
    }
}
