use egui::{Color32, RichText, Stroke, Vec2};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CapabilityDefinition {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: CapabilityCategory,
    pub required_attributes: Vec<AttributeDefinition>,
    pub optional_attributes: Vec<AttributeDefinition>,
    pub execution_mode: ExecutionMode,
    pub status: CapabilityStatus,
    pub dependencies: Vec<String>,
    pub outputs: Vec<OutputDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityCategory {
    Setup,
    Configuration,
    Validation,
    Execution,
    Monitoring,
    Compliance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Synchronous,
    Asynchronous,
    Streaming,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityStatus {
    Available,
    Configured,
    Running,
    Completed,
    Failed,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct AttributeDefinition {
    pub name: String,
    pub display_name: String,
    pub attribute_type: AttributeType,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
    pub default_value: Option<Value>,
    pub current_value: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum AttributeType {
    String,
    Number,
    Boolean,
    Array(Box<AttributeType>),
    Object(HashMap<String, AttributeType>),
    Select(Vec<String>),
    FileUpload,
    Connection,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub message: String,
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum ValidationType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Range(f64, f64),
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct OutputDefinition {
    pub name: String,
    pub output_type: AttributeType,
    pub description: String,
}

pub struct CapabilityManagerUI {
    capabilities: Vec<CapabilityDefinition>,
    selected_capability: Option<String>,
    show_configuration_panel: bool,
    filter_category: Option<CapabilityCategory>,
    filter_status: Option<CapabilityStatus>,
    search_query: String,
    execution_history: Vec<ExecutionRecord>,
    attribute_values: HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub capability_name: String,
    pub execution_id: String,
    pub started_at: u64,  // Unix timestamp
    pub completed_at: Option<u64>,
    pub status: CapabilityStatus,
    pub inputs: HashMap<String, Value>,
    pub outputs: HashMap<String, Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<f64>,
}

impl Default for CapabilityManagerUI {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityManagerUI {
    pub fn new() -> Self {
        let mut ui = Self {
            capabilities: Vec::new(),
            selected_capability: None,
            show_configuration_panel: false,
            filter_category: None,
            filter_status: None,
            search_query: String::new(),
            execution_history: Vec::new(),
            attribute_values: HashMap::new(),
        };

        ui.load_built_in_capabilities();
        ui
    }

    fn load_built_in_capabilities(&mut self) {
        self.capabilities = vec![
            CapabilityDefinition {
                name: "AccountSetup".to_string(),
                display_name: "Account Setup".to_string(),
                description: "Configure client account structure and basic settings".to_string(),
                category: CapabilityCategory::Setup,
                required_attributes: vec![
                    AttributeDefinition {
                        name: "account_name".to_string(),
                        display_name: "Account Name".to_string(),
                        attribute_type: AttributeType::String,
                        description: "Primary account identifier".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Account name is required".to_string(),
                                parameters: HashMap::new(),
                            },
                            ValidationRule {
                                rule_type: ValidationType::MinLength(3),
                                message: "Account name must be at least 3 characters".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: None,
                        current_value: None,
                    },
                    AttributeDefinition {
                        name: "entity_lei".to_string(),
                        display_name: "Legal Entity Identifier".to_string(),
                        attribute_type: AttributeType::String,
                        description: "20-character LEI code".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Pattern("^[A-Z0-9]{20}$".to_string()),
                                message: "LEI must be 20 alphanumeric characters".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: None,
                        current_value: None,
                    },
                ],
                optional_attributes: vec![
                    AttributeDefinition {
                        name: "account_currency".to_string(),
                        display_name: "Base Currency".to_string(),
                        attribute_type: AttributeType::Select(vec![
                            "USD".to_string(), "EUR".to_string(), "GBP".to_string(),
                            "JPY".to_string(), "CHF".to_string()
                        ]),
                        description: "Primary account currency".to_string(),
                        validation_rules: vec![],
                        default_value: Some(Value::String("USD".to_string())),
                        current_value: None,
                    },
                ],
                execution_mode: ExecutionMode::Synchronous,
                status: CapabilityStatus::Available,
                dependencies: vec![],
                outputs: vec![
                    OutputDefinition {
                        name: "account_id".to_string(),
                        output_type: AttributeType::String,
                        description: "Generated unique account identifier".to_string(),
                    },
                ],
            },
            CapabilityDefinition {
                name: "TradeFeedSetup".to_string(),
                display_name: "Trade Feed Setup".to_string(),
                description: "Configure trade data ingestion and processing".to_string(),
                category: CapabilityCategory::Configuration,
                required_attributes: vec![
                    AttributeDefinition {
                        name: "feed_protocol".to_string(),
                        display_name: "Feed Protocol".to_string(),
                        attribute_type: AttributeType::Select(vec![
                            "FIX".to_string(), "REST".to_string(), "FTP".to_string(), "SFTP".to_string()
                        ]),
                        description: "Communication protocol for trade data".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Feed protocol is required".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: Some(Value::String("FIX".to_string())),
                        current_value: None,
                    },
                    AttributeDefinition {
                        name: "connection_details".to_string(),
                        display_name: "Connection Details".to_string(),
                        attribute_type: AttributeType::Connection,
                        description: "Endpoint configuration for trade feed".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Connection details are required".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: None,
                        current_value: None,
                    },
                ],
                optional_attributes: vec![
                    AttributeDefinition {
                        name: "batch_size".to_string(),
                        display_name: "Batch Size".to_string(),
                        attribute_type: AttributeType::Number,
                        description: "Number of trades to process in each batch".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Range(1.0, 10000.0),
                                message: "Batch size must be between 1 and 10,000".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: Some(Value::Number(serde_json::Number::from(1000))),
                        current_value: None,
                    },
                ],
                execution_mode: ExecutionMode::Asynchronous,
                status: CapabilityStatus::Available,
                dependencies: vec!["AccountSetup".to_string()],
                outputs: vec![
                    OutputDefinition {
                        name: "feed_id".to_string(),
                        output_type: AttributeType::String,
                        description: "Unique identifier for the configured feed".to_string(),
                    },
                    OutputDefinition {
                        name: "connection_status".to_string(),
                        output_type: AttributeType::Boolean,
                        description: "Current connection status".to_string(),
                    },
                ],
            },
            CapabilityDefinition {
                name: "HealthCheck".to_string(),
                display_name: "Health Check".to_string(),
                description: "Monitor system health and connectivity".to_string(),
                category: CapabilityCategory::Monitoring,
                required_attributes: vec![
                    AttributeDefinition {
                        name: "check_type".to_string(),
                        display_name: "Check Type".to_string(),
                        attribute_type: AttributeType::Select(vec![
                            "connectivity".to_string(), "performance".to_string(),
                            "data_quality".to_string(), "compliance".to_string()
                        ]),
                        description: "Type of health check to perform".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Required,
                                message: "Check type is required".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: Some(Value::String("connectivity".to_string())),
                        current_value: None,
                    },
                ],
                optional_attributes: vec![
                    AttributeDefinition {
                        name: "timeout_seconds".to_string(),
                        display_name: "Timeout (seconds)".to_string(),
                        attribute_type: AttributeType::Number,
                        description: "Maximum time to wait for health check completion".to_string(),
                        validation_rules: vec![
                            ValidationRule {
                                rule_type: ValidationType::Range(1.0, 300.0),
                                message: "Timeout must be between 1 and 300 seconds".to_string(),
                                parameters: HashMap::new(),
                            },
                        ],
                        default_value: Some(Value::Number(serde_json::Number::from(30))),
                        current_value: None,
                    },
                ],
                execution_mode: ExecutionMode::Synchronous,
                status: CapabilityStatus::Available,
                dependencies: vec![],
                outputs: vec![
                    OutputDefinition {
                        name: "check_result".to_string(),
                        output_type: AttributeType::Boolean,
                        description: "Overall health check result".to_string(),
                    },
                    OutputDefinition {
                        name: "details".to_string(),
                        output_type: AttributeType::Object(HashMap::new()),
                        description: "Detailed health check information".to_string(),
                    },
                ],
            },
        ];
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎛️ Capability Management");

        // Render filters and search
        self.render_filters(ui);
        ui.separator();

        // Main content area with split layout
        ui.horizontal(|ui| {
            // Left panel: Capability list
            ui.vertical(|ui| {
                ui.set_width(400.0);
                ui.heading("Available Capabilities");
                self.render_capability_list(ui);
            });

            ui.separator();

            // Right panel: Selected capability details
            ui.vertical(|ui| {
                if let Some(selected) = self.selected_capability.clone() {
                    self.render_capability_details(ui, &selected);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a capability to view details and configuration options");
                    });
                }
            });
        });

        ui.separator();

        // Bottom panel: Execution history
        if !self.execution_history.is_empty() {
            ui.collapsing("📊 Execution History", |ui| {
                self.render_execution_history(ui);
            });
        }
    }

    fn render_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.search_query);

            ui.separator();

            ui.label("Category:");
            egui::ComboBox::from_id_salt("category_filter")
                .selected_text(match &self.filter_category {
                    Some(cat) => format!("{:?}", cat),
                    None => "All".to_string(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_category, None, "All");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Setup), "Setup");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Configuration), "Configuration");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Validation), "Validation");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Execution), "Execution");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Monitoring), "Monitoring");
                    ui.selectable_value(&mut self.filter_category, Some(CapabilityCategory::Compliance), "Compliance");
                });

            ui.separator();

            ui.label("Status:");
            egui::ComboBox::from_id_salt("status_filter")
                .selected_text(match &self.filter_status {
                    Some(status) => format!("{:?}", status),
                    None => "All".to_string(),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_status, None, "All");
                    ui.selectable_value(&mut self.filter_status, Some(CapabilityStatus::Available), "Available");
                    ui.selectable_value(&mut self.filter_status, Some(CapabilityStatus::Configured), "Configured");
                    ui.selectable_value(&mut self.filter_status, Some(CapabilityStatus::Running), "Running");
                    ui.selectable_value(&mut self.filter_status, Some(CapabilityStatus::Completed), "Completed");
                    ui.selectable_value(&mut self.filter_status, Some(CapabilityStatus::Failed), "Failed");
                });
        });
    }

    fn render_capability_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let capabilities = self.capabilities.clone();
            for capability in capabilities {
                if self.matches_filters(&capability) {
                    self.render_capability_card(ui, &capability);
                    ui.add_space(8.0);
                }
            }
        });
    }

    fn matches_filters(&self, capability: &CapabilityDefinition) -> bool {
        // Search query filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            if !capability.name.to_lowercase().contains(&query)
                && !capability.display_name.to_lowercase().contains(&query)
                && !capability.description.to_lowercase().contains(&query) {
                return false;
            }
        }

        // Category filter
        if let Some(filter_cat) = &self.filter_category {
            if &capability.category != filter_cat {
                return false;
            }
        }

        // Status filter
        if let Some(filter_status) = &self.filter_status {
            if &capability.status != filter_status {
                return false;
            }
        }

        true
    }

    fn render_capability_card(&mut self, ui: &mut egui::Ui, capability: &CapabilityDefinition) {
        let is_selected = self.selected_capability.as_ref() == Some(&capability.name);

        let frame = egui::Frame::new()
            .fill(if is_selected { Color32::from_rgb(40, 60, 80) } else { Color32::from_rgb(30, 30, 30) })
            .stroke(if is_selected {
                Stroke::new(2.0, Color32::from_rgb(100, 150, 200))
            } else {
                Stroke::new(1.0, Color32::GRAY)
            })
            .corner_radius(6.0)
            .inner_margin(egui::Margin::same(12));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            let response = ui.allocate_response(Vec2::new(ui.available_width(), 80.0), egui::Sense::click());

            if response.clicked() {
                self.selected_capability = Some(capability.name.clone());
            }

            ui.horizontal(|ui| {
                // Status indicator
                let status_color = match capability.status {
                    CapabilityStatus::Available => Color32::from_rgb(100, 200, 100),
                    CapabilityStatus::Configured => Color32::from_rgb(100, 150, 200),
                    CapabilityStatus::Running => Color32::from_rgb(255, 200, 100),
                    CapabilityStatus::Completed => Color32::from_rgb(150, 255, 150),
                    CapabilityStatus::Failed => Color32::from_rgb(255, 100, 100),
                    CapabilityStatus::Disabled => Color32::GRAY,
                };

                ui.colored_label(status_color, "●");

                ui.vertical(|ui| {
                    ui.label(RichText::new(&capability.display_name).heading().strong());
                    ui.label(RichText::new(&capability.description).color(Color32::LIGHT_GRAY));

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{:?}", capability.category)).small().color(Color32::LIGHT_BLUE));
                        if !capability.dependencies.is_empty() {
                            ui.label(RichText::new(format!("Dependencies: {}", capability.dependencies.len())).small().color(Color32::YELLOW));
                        }
                    });
                });
            });
        });
    }

    fn render_capability_details(&mut self, ui: &mut egui::Ui, capability_name: &str) {
        if let Some(capability) = self.capabilities.iter().find(|c| c.name == capability_name).cloned() {
            ui.heading(&capability.display_name);
            ui.label(&capability.description);
            ui.add_space(10.0);

            // Status and basic info
            ui.horizontal(|ui| {
                ui.label("Status:");
                let status_color = match capability.status {
                    CapabilityStatus::Available => Color32::from_rgb(100, 200, 100),
                    CapabilityStatus::Configured => Color32::from_rgb(100, 150, 200),
                    CapabilityStatus::Running => Color32::from_rgb(255, 200, 100),
                    CapabilityStatus::Completed => Color32::from_rgb(150, 255, 150),
                    CapabilityStatus::Failed => Color32::from_rgb(255, 100, 100),
                    CapabilityStatus::Disabled => Color32::GRAY,
                };
                ui.colored_label(status_color, format!("{:?}", capability.status));

                ui.separator();
                ui.label(format!("Mode: {:?}", capability.execution_mode));
            });

            ui.separator();

            // Configuration section
            ui.collapsing("⚙️ Configuration", |ui| {
                self.render_capability_configuration(ui, &capability);
            });

            // Dependencies section
            if !capability.dependencies.is_empty() {
                ui.collapsing("🔗 Dependencies", |ui| {
                    for dep in &capability.dependencies {
                        ui.label(format!("• {}", dep));
                    }
                });
            }

            // Outputs section
            if !capability.outputs.is_empty() {
                ui.collapsing("📤 Outputs", |ui| {
                    for output in &capability.outputs {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&output.name).strong());
                            ui.label(format!("({:?})", output.output_type));
                        });
                        ui.label(RichText::new(&output.description).color(Color32::LIGHT_GRAY));
                        ui.add_space(5.0);
                    }
                });
            }

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("🔧 Configure").clicked() {
                    self.show_configuration_panel = true;
                }

                if ui.button("▶️ Execute").clicked() {
                    self.execute_capability(capability_name);
                }

                if ui.button("📋 Test").clicked() {
                    self.test_capability(capability_name);
                }
            });
        }
    }

    fn render_capability_configuration(&mut self, ui: &mut egui::Ui, capability: &CapabilityDefinition) {
        // Required attributes
        if !capability.required_attributes.is_empty() {
            ui.label(RichText::new("Required Attributes").strong().color(Color32::YELLOW));
            for attr in &capability.required_attributes {
                self.render_attribute_input(ui, &capability.name, attr, true);
            }
            ui.add_space(10.0);
        }

        // Optional attributes
        if !capability.optional_attributes.is_empty() {
            ui.label(RichText::new("Optional Attributes").strong().color(Color32::LIGHT_BLUE));
            for attr in &capability.optional_attributes {
                self.render_attribute_input(ui, &capability.name, attr, false);
            }
        }
    }

    fn render_attribute_input(&mut self, ui: &mut egui::Ui, capability_name: &str, attr: &AttributeDefinition, is_required: bool) {
        ui.horizontal(|ui| {
            let label_text = if is_required {
                format!("{}*", attr.display_name)
            } else {
                attr.display_name.clone()
            };

            ui.label(RichText::new(label_text).color(if is_required { Color32::YELLOW } else { Color32::WHITE }));
            ui.label(RichText::new(&attr.description).small().color(Color32::LIGHT_GRAY));
        });

        // Get or initialize the current value for this attribute
        let current_value = self.attribute_values
            .entry(capability_name.to_string())
            .or_insert_with(HashMap::new)
            .entry(attr.name.clone())
            .or_insert_with(|| attr.default_value.clone().unwrap_or(Value::Null));

        match &attr.attribute_type {
            AttributeType::String => {
                let mut text = match current_value {
                    Value::String(s) => s.clone(),
                    _ => String::new(),
                };
                if ui.text_edit_singleline(&mut text).changed() {
                    *current_value = Value::String(text);
                }
            }
            AttributeType::Number => {
                let mut number: f64 = match current_value {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => 0.0,
                };
                if ui.add(egui::DragValue::new(&mut number).speed(0.1)).changed() {
                    *current_value = Value::Number(serde_json::Number::from_f64(number).unwrap_or_else(|| serde_json::Number::from(0)));
                }
            }
            AttributeType::Boolean => {
                let mut bool_val = match current_value {
                    Value::Bool(b) => *b,
                    _ => false,
                };
                if ui.checkbox(&mut bool_val, "").changed() {
                    *current_value = Value::Bool(bool_val);
                }
            }
            AttributeType::Select(options) => {
                let current_text = match current_value {
                    Value::String(s) => s.clone(),
                    _ => options.first().cloned().unwrap_or_default(),
                };

                egui::ComboBox::from_id_salt(format!("{}_{}", capability_name, attr.name))
                    .selected_text(&current_text)
                    .show_ui(ui, |ui| {
                        for option in options {
                            if ui.selectable_label(&current_text == option, option).clicked() {
                                *current_value = Value::String(option.clone());
                            }
                        }
                    });
            }
            _ => {
                ui.label(format!("TODO: Input for {:?}", attr.attribute_type));
            }
        }

        ui.add_space(5.0);
    }

    fn render_execution_history(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for record in &self.execution_history {
                ui.horizontal(|ui| {
                    let status_color = match record.status {
                        CapabilityStatus::Running => Color32::from_rgb(255, 200, 100),
                        CapabilityStatus::Completed => Color32::from_rgb(150, 255, 150),
                        CapabilityStatus::Failed => Color32::from_rgb(255, 100, 100),
                        _ => Color32::GRAY,
                    };

                    ui.colored_label(status_color, "●");
                    ui.label(&record.capability_name);
                    ui.label(format!("{}", record.started_at));

                    if let Some(duration) = record.execution_time_ms {
                        ui.label(format!("{}ms", duration as u64));
                    }

                    if let Some(error) = &record.error_message {
                        ui.label(RichText::new("❌").color(Color32::RED)).on_hover_text(error);
                    }
                });
                ui.separator();
            }
        });
    }

    fn execute_capability(&mut self, capability_name: &str) {
        // Create execution record
        let record = ExecutionRecord {
            capability_name: capability_name.to_string(),
            execution_id: format!("exec-{}", js_sys::Date::now() as u64),
            started_at: js_sys::Date::now() as u64,
            completed_at: None,
            status: CapabilityStatus::Running,
            inputs: self.attribute_values.get(capability_name).cloned().unwrap_or_default(),
            outputs: HashMap::new(),
            error_message: None,
            execution_time_ms: None,
        };

        self.execution_history.insert(0, record);

        // TODO: Integrate with actual capability execution engine
        // For now, simulate execution completion after a short delay
    }

    fn test_capability(&mut self, capability_name: &str) {
        // Similar to execute but with test data
        let record = ExecutionRecord {
            capability_name: format!("TEST: {}", capability_name),
            execution_id: format!("test-{}", js_sys::Date::now() as u64),
            started_at: js_sys::Date::now() as u64,
            completed_at: Some(js_sys::Date::now() as u64),
            status: CapabilityStatus::Completed,
            inputs: HashMap::new(),
            outputs: serde_json::from_str(r#"{"test_result": true, "validation": "passed"}"#).unwrap(),
            error_message: None,
            execution_time_ms: Some(125.0),
        };

        self.execution_history.insert(0, record);
    }
}

impl CapabilityCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            CapabilityCategory::Setup => "🔧",
            CapabilityCategory::Configuration => "⚙️",
            CapabilityCategory::Validation => "✅",
            CapabilityCategory::Execution => "▶️",
            CapabilityCategory::Monitoring => "📊",
            CapabilityCategory::Compliance => "🛡️",
        }
    }
}

impl CapabilityStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            CapabilityStatus::Available => "⚪",
            CapabilityStatus::Configured => "🔵",
            CapabilityStatus::Running => "🟡",
            CapabilityStatus::Completed => "🟢",
            CapabilityStatus::Failed => "🔴",
            CapabilityStatus::Disabled => "⚫",
        }
    }
}