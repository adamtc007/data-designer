// CBU DSL IDE - Interactive panel for writing and executing CBU CRUD operations
use eframe::egui;
use crate::grpc_client::{GrpcClient, GetEntitiesRequest};
use crate::wasm_utils;
use serde::{Deserialize, Serialize};

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

    // Execution state
    executing: bool,
    last_result: Option<CbuDslResponse>,

    // UI state
    show_examples: bool,
    show_help: bool,
    selected_example: usize,

    // Entity lookup for auto-completion
    available_entities: Vec<EntityInfo>,
    loading_entities: bool,

    // Entity picker state
    show_entity_picker: bool,
    show_floating_entity_picker: bool, // New floating panel state
    entity_picker_window_size: egui::Vec2, // Track window size
    entity_picker_first_open: bool, // Track first open to apply default size only once
    entity_filter_jurisdiction: String,
    entity_filter_type: String,
    entity_search_name: String,
    selected_entities: Vec<(String, String)>, // (entity_id, role)
}

#[derive(Debug, Clone)]
struct EntityInfo {
    entity_id: String,
    entity_name: String,
    entity_type: String,
    jurisdiction: String,
    country_code: String,
    lei_code: Option<String>,
}

impl Default for CbuDslIDE {
    fn default() -> Self {
        Self::new()
    }
}

impl CbuDslIDE {
    pub fn new() -> Self {
        Self {
            dsl_script: String::new(),
            executing: false,
            last_result: None,
            show_examples: false,
            show_help: false,
            selected_example: 0,
            available_entities: Vec::new(),
            loading_entities: false,
            show_entity_picker: false,
            show_floating_entity_picker: false,
            entity_picker_window_size: egui::Vec2::new(720.0, 420.0),
            entity_picker_first_open: true,
            entity_filter_jurisdiction: "All".to_string(),
            entity_filter_type: "All".to_string(),
            entity_search_name: String::new(),
            selected_entities: Vec::new(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.heading("🏢 CBU DSL Management");
        ui.separator();

        // Auto-load entities if not already loaded and gRPC client is available
        if self.available_entities.is_empty() && !self.loading_entities && grpc_client.is_some() {
            wasm_utils::console_log("🔄 Auto-loading entities for CBU DSL IDE");
            self.load_available_entities(grpc_client);
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

        // Render floating entity picker if open
        self.render_floating_entity_picker(ui.ctx());
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.horizontal(|ui| {
            // Execute button
            let execute_button = ui.add_enabled(
                !self.dsl_script.trim().is_empty() && !self.executing && grpc_client.is_some(),
                egui::Button::new("▶ Execute DSL")
            );

            if execute_button.clicked() {
                self.execute_dsl(grpc_client);
            }

            // Clear button
            if ui.button("🗑 Clear").clicked() {
                self.dsl_script.clear();
                self.last_result = None;
            }

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
                self.load_available_entities(grpc_client);
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
                    wasm_utils::console_log("🔍 Opening entity picker window");
                    self.show_floating_entity_picker = true;
                    self.entity_picker_first_open = true; // Reset for default size only

                    // CRITICAL: Reset egui's stored window size/position memory
                    ui.ctx().memory_mut(|mem| {
                        mem.reset_areas();
                    });
                    wasm_utils::console_log("🧹 Reset egui areas memory to clear stored window sizes");
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

                // DSL text editor with syntax highlighting
                let hint_text = r#"Write CBU DSL commands here. Example:

CREATE CBU 'Growth Fund Alpha' ; 'Diversified growth fund' WITH
  ENTITY ('Alpha Capital', 'AC001') AS 'Asset Owner' AND
  ENTITY ('Beta Management', 'BM002') AS 'Investment Manager' AND
  ENTITY ('Gamma Services', 'GS003') AS 'Managing Company'"#;

                let editor_response = ui.add(
                    egui::TextEdit::multiline(&mut self.dsl_script)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15)
                        .code_editor()
                        .hint_text(hint_text)
                );

                // Auto-completion suggestions
                if editor_response.has_focus() && !self.available_entities.is_empty() {
                    self.show_auto_completion_popup(ui);
                }

                ui.add_space(10.0);

                // Execution status
                if self.executing {
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

        if let Some(result) = &self.last_result {
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

    fn execute_dsl(&mut self, grpc_client: Option<&GrpcClient>) {
        if let Some(_client) = grpc_client {
            self.executing = true;

            // TODO: Implement actual gRPC call to execute CBU DSL
            // For now, simulate execution
            self.simulate_execution();
        }
    }

    fn simulate_execution(&mut self) {
        // Simulate execution result
        let script = self.dsl_script.trim();

        if script.to_uppercase().starts_with("CREATE CBU") {
            self.last_result = Some(CbuDslResponse {
                success: true,
                message: "CBU created successfully".to_string(),
                cbu_id: Some(format!("CBU{:06}", 123456)), // Simplified for demo
                validation_errors: Vec::new(),
                data: None,
            });
        } else if script.to_uppercase().starts_with("UPDATE CBU") {
            self.last_result = Some(CbuDslResponse {
                success: true,
                message: "CBU updated successfully".to_string(),
                cbu_id: None,
                validation_errors: Vec::new(),
                data: None,
            });
        } else if script.to_uppercase().starts_with("DELETE CBU") {
            self.last_result = Some(CbuDslResponse {
                success: true,
                message: "CBU deleted successfully".to_string(),
                cbu_id: None,
                validation_errors: Vec::new(),
                data: None,
            });
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

            self.last_result = Some(CbuDslResponse {
                success: true,
                message: "Query executed successfully".to_string(),
                cbu_id: None,
                validation_errors: Vec::new(),
                data: Some(sample_data),
            });
        } else {
            self.last_result = Some(CbuDslResponse {
                success: false,
                message: "Invalid DSL command".to_string(),
                cbu_id: None,
                validation_errors: vec!["Command must start with CREATE CBU, UPDATE CBU, DELETE CBU, or QUERY CBU".to_string()],
                data: None,
            });
        }

        self.executing = false;
    }

    fn load_available_entities(&mut self, grpc_client: Option<&GrpcClient>) {
        if let Some(client) = grpc_client {
            wasm_utils::console_log("🔄 Starting entity loading process");
            self.loading_entities = true;

            // Create gRPC request
            let request = GetEntitiesRequest {
                jurisdiction: None, // Load all jurisdictions
                entity_type: None,  // Load all types
                status: Some("active".to_string()), // Only active entities
            };

            wasm_utils::console_log("📡 Making gRPC request for entities");

            // Clone client for async operation
            let client_clone = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match client_clone.get_entities(request).await {
                    Ok(response) => {
                        // TODO: Update UI with entities
                        // Since this is async, we need a callback mechanism
                        wasm_utils::console_log(&format!("✅ Loaded {} entities from gRPC", response.entities.len()));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Failed to load entities: {}", e));
                    }
                }
            });

            // For now, also call simulate to have immediate data
            wasm_utils::console_log("📊 Loading simulated entity data for immediate display");
            self.simulate_entity_loading();
        } else {
            wasm_utils::console_log("⚠️ No gRPC client available for entity loading");
        }
    }

    fn simulate_entity_loading(&mut self) {
        // For now, provide immediate data while gRPC loads in background
        wasm_utils::console_log("🏗️ Creating simulated entity data");
        self.available_entities = vec![
            // US Entities
            EntityInfo {
                entity_id: "US001".to_string(),
                entity_name: "Manhattan Asset Management LLC".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "Delaware".to_string(),
                country_code: "US".to_string(),
                lei_code: Some("549300VPLTI2JI1A8N82".to_string()),
            },
            EntityInfo {
                entity_id: "US002".to_string(),
                entity_name: "Goldman Sachs Asset Management".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "New York".to_string(),
                country_code: "US".to_string(),
                lei_code: Some("784F5XWPLTWKTBV3E584".to_string()),
            },
            EntityInfo {
                entity_id: "US003".to_string(),
                entity_name: "BlackRock Institutional Trust".to_string(),
                entity_type: "Asset Owner".to_string(),
                jurisdiction: "Delaware".to_string(),
                country_code: "US".to_string(),
                lei_code: Some("549300WOTC9L6FP6DY29".to_string()),
            },
            EntityInfo {
                entity_id: "US004".to_string(),
                entity_name: "State Street Global Services".to_string(),
                entity_type: "Service Provider".to_string(),
                jurisdiction: "Massachusetts".to_string(),
                country_code: "US".to_string(),
                lei_code: Some("571474TGEMMWANRLN572".to_string()),
            },
            // EU Entities
            EntityInfo {
                entity_id: "EU001".to_string(),
                entity_name: "Deutsche Asset Management".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "Germany".to_string(),
                country_code: "DE".to_string(),
                lei_code: Some("529900T8BM49AURSDO55".to_string()),
            },
            EntityInfo {
                entity_id: "EU002".to_string(),
                entity_name: "BNP Paribas Asset Management".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "France".to_string(),
                country_code: "FR".to_string(),
                lei_code: Some("969500UP76J52A9OXU27".to_string()),
            },
            EntityInfo {
                entity_id: "EU003".to_string(),
                entity_name: "UBS Asset Management AG".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "Switzerland".to_string(),
                country_code: "CH".to_string(),
                lei_code: Some("549300ZZK73H1MR76N74".to_string()),
            },
            // APAC Entities
            EntityInfo {
                entity_id: "AP001".to_string(),
                entity_name: "Nomura Asset Management".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "Japan".to_string(),
                country_code: "JP".to_string(),
                lei_code: Some("353800MLJIGSLQ3JGP81".to_string()),
            },
            EntityInfo {
                entity_id: "AP002".to_string(),
                entity_name: "China Asset Management Co".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "China".to_string(),
                country_code: "CN".to_string(),
                lei_code: Some("300300S39XTBSNH66F17".to_string()),
            },
            EntityInfo {
                entity_id: "AP003".to_string(),
                entity_name: "DBS Asset Management".to_string(),
                entity_type: "Investment Manager".to_string(),
                jurisdiction: "Singapore".to_string(),
                country_code: "SG".to_string(),
                lei_code: Some("549300F4WH7V9NCKXX55".to_string()),
            },
        ];

        wasm_utils::console_log(&format!("✅ Simulated entity loading complete: {} entities loaded", self.available_entities.len()));
        self.loading_entities = false;
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
            if filtered_entities.len() > 0 {
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
            for &i in entities_to_remove.iter().rev() {
                if i < self.selected_entities.len() {
                    self.selected_entities.remove(i);
                }
            }

            // Generate DSL if requested
            if generate_dsl {
                self.generate_cbu_dsl_from_selection();
            }
        });
    }

    fn render_floating_entity_picker(&mut self, ctx: &egui::Context) {
        if !self.show_floating_entity_picker {
            return;
        }

        wasm_utils::console_log(&format!("🎯 Rendering floating entity picker, current stored size: {:?}", self.entity_picker_window_size));

        let mut open = self.show_floating_entity_picker;

        // Create window with stable ID
        let mut window = egui::Window::new("👥 Smart Entity Picker - Client Entity Table")
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .id(egui::Id::new("entity_picker_window")); // Stable ID for size persistence

        // Only apply default size on first open, then let egui handle persistence
        if std::mem::take(&mut self.entity_picker_first_open) {
            wasm_utils::console_log("🎯 First open: applying default size 720x420");
            window = window.default_size(egui::Vec2::new(720.0, 420.0));
        } else {
            wasm_utils::console_log(&format!("🔄 Subsequent open: using stored size {:?}", self.entity_picker_window_size));
        }

        if let Some(window_response) = window.show(ctx, |ui| {
            // Track entity selections to avoid borrowing issues
            let mut entity_selections: Vec<(String, String, String)> = Vec::new(); // (entity_id, entity_name, role)

            // Filter controls in a horizontal layout
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

            // Scrollable list of filtered entities - avoid layout clamps that fight vertical resizing
            egui::ScrollArea::vertical()
                .auto_shrink([false, false]) // Don't auto-shrink in either direction
                .show(ui, |ui| {
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
                });

            ui.add_space(10.0);

            // Selected entities preview
            let mut entities_to_remove = Vec::new();
            let mut generate_dsl = false;

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
                    if ui.button("🗑 Clear All").clicked() {
                        self.selected_entities.clear();
                    }
                });
            }

            // Close button at bottom
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("✅ Done").clicked() {
                    self.show_floating_entity_picker = false;
                }
                ui.label("Select entities and roles, then click 'Generate CBU DSL' or 'Done'");
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
        }) {
            // Persist window size after layout completes
            let new_size = window_response.response.rect.size();
            wasm_utils::console_log(&format!("💾 Persisting window size: {:?}", new_size));
            self.entity_picker_window_size = new_size;
        }

        // Update state if window was closed via X button
        self.show_floating_entity_picker = open;
    }

    fn add_entity_to_dsl(&mut self, entity_id: &str, entity_name: &str, role: &str) {
        // Check if this entity+role combination already exists
        if !self.selected_entities.iter().any(|(id, r)| id == entity_id && r == role) {
            self.selected_entities.push((format!("{} ({})", entity_name, entity_id), role.to_string()));
        }
    }

    fn generate_cbu_dsl_from_selection(&mut self) {
        if self.selected_entities.len() < 3 {
            return; // Need at least Asset Owner, Investment Manager, Managing Company
        }

        let mut dsl = String::from("CREATE CBU 'New CBU Name' ; 'CBU Description' WITH\n");

        for (i, (entity_info, role)) in self.selected_entities.iter().enumerate() {
            let parts: Vec<&str> = entity_info.split(" (").collect();
            if parts.len() == 2 {
                let name = parts[0];
                let id = parts[1].trim_end_matches(')');

                if i > 0 {
                    dsl.push_str(" AND\n");
                }
                dsl.push_str(&format!("  ENTITY ('{}', '{}') AS '{}'", name, id, role));
            }
        }

        self.dsl_script = dsl;
        self.selected_entities.clear(); // Clear selection
    }
}

// Syntax highlighting for CBU DSL (simplified)
pub fn highlight_cbu_dsl(ui: &mut egui::Ui, text: &str) {
    let keywords = ["CREATE", "UPDATE", "DELETE", "QUERY", "CBU", "WITH", "ENTITY", "AS", "AND", "SET", "WHERE"];
    let roles = ["Asset Owner", "Investment Manager", "Managing Company"];

    // Simple syntax highlighting implementation
    // In a real implementation, this would use proper tokenization
    for line in text.lines() {
        ui.horizontal(|ui| {
            for word in line.split_whitespace() {
                if keywords.contains(&word.to_uppercase().as_str()) {
                    ui.colored_label(egui::Color32::BLUE, word);
                } else if roles.iter().any(|role| word.contains(role)) {
                    ui.colored_label(egui::Color32::GREEN, word);
                } else if word.starts_with('\'') && word.ends_with('\'') {
                    ui.colored_label(egui::Color32::YELLOW, word);
                } else {
                    ui.label(word);
                }
                ui.label(" ");
            }
        });
    }
}