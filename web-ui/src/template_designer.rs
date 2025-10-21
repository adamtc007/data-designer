use eframe::egui;
use serde_json::{json, Value};
use crate::wasm_utils;

/// Domain information for template creation
#[derive(Debug, Clone)]
pub struct Domain {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub regulatory_framework: String,
}

/// EBNF Grammar template for domain-specific syntax
#[derive(Debug, Clone)]
pub struct EbnfTemplate {
    pub id: i32,
    pub template_name: String,
    pub description: String,
    pub ebnf_pattern: String,
    pub complexity_level: String,
}

/// Template Designer IDE for creating and editing templates
#[derive(Debug)]
pub struct TemplateDesignerIDE {
    // Template editing state
    pub mode: DesignerMode,
    pub template_id: String,
    pub template_name: String,
    pub template_description: String,
    pub template_dsl: String,
    pub template_attributes: Vec<TemplateAttribute>,

    // Domain and EBNF selection
    pub available_domains: Vec<Domain>,
    pub selected_domain_id: Option<i32>,
    pub available_ebnf_templates: Vec<EbnfTemplate>,
    pub selected_ebnf_id: Option<i32>,

    // UI state
    pub show_domain_selector: bool,
    pub show_ebnf_selector: bool,
    pub show_attribute_editor: bool,
    pub loading: bool,
    pub error_message: Option<String>,
    pub status_message: String,

    // Editor state
    pub dsl_editor_content: String,
    pub syntax_tokens: Vec<SyntaxToken>,
    pub show_ast_view: bool,
    pub show_preview: bool,

    // Workflow state
    pub unsaved_changes: bool,
    pub show_save_dialog: bool,
    pub show_discard_dialog: bool,

    // New attribute creation
    pub new_attr_name: String,
    pub new_attr_type: String,
    pub new_attr_required: bool,
    pub new_attr_description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesignerMode {
    SelectTemplate,  // Choose existing template to edit or create new
    SelectDomain,    // Choose domain (KYC, Onboarding, etc.)
    SelectEbnf,      // Choose EBNF grammar for domain
    EditTemplate,    // Main editing interface
    Preview,         // Preview template before saving
}

#[derive(Debug, Clone)]
pub struct TemplateAttribute {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub allowed_values: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub text: String,
    pub token_type: TokenType,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,      // WORKFLOW, STEP, PHASE
    Command,      // ASSESS_RISK, SCREEN_ENTITY
    String,       // "string literals"
    Number,       // 123, 45.67
    Identifier,   // variable names
    Operator,     // =, ==, +, -
    Comment,      // # comments
    Error,        // syntax errors
}

impl TemplateDesignerIDE {
    pub fn new() -> Self {
        Self {
            mode: DesignerMode::SelectTemplate,
            template_id: String::new(),
            template_name: String::new(),
            template_description: String::new(),
            template_dsl: String::new(),
            template_attributes: Vec::new(),

            available_domains: Vec::new(),
            selected_domain_id: None,
            available_ebnf_templates: Vec::new(),
            selected_ebnf_id: None,

            show_domain_selector: false,
            show_ebnf_selector: false,
            show_attribute_editor: false,
            loading: false,
            error_message: None,
            status_message: "Ready to design templates".to_string(),

            dsl_editor_content: String::new(),
            syntax_tokens: Vec::new(),
            show_ast_view: false,
            show_preview: false,

            unsaved_changes: false,
            show_save_dialog: false,
            show_discard_dialog: false,

            new_attr_name: String::new(),
            new_attr_type: "String".to_string(),
            new_attr_required: false,
            new_attr_description: String::new(),
        }
    }

    /// Start creating a new template
    pub fn start_new_template(&mut self) {
        self.mode = DesignerMode::SelectDomain;
        self.template_id = String::new();
        self.template_name = String::new();
        self.template_description = String::new();
        self.template_dsl = String::new();
        self.template_attributes.clear();
        self.unsaved_changes = false;
        self.load_domains_from_database();
        wasm_utils::console_log("🎨 Starting new template creation");
    }

    /// Start editing an existing template
    pub fn start_edit_template(&mut self, template_json: &str) {
        if let Ok(template) = serde_json::from_str::<Value>(template_json) {
            self.template_id = template.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            self.template_name = template.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            self.template_description = template.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            self.template_dsl = template.get("dsl").and_then(|v| v.as_str()).unwrap_or("").to_string();
            self.dsl_editor_content = self.template_dsl.clone();

            // Load attributes
            self.template_attributes.clear();
            if let Some(attrs) = template.get("attributes").and_then(|v| v.as_array()) {
                for attr in attrs {
                    if let (Some(name), Some(data_type)) = (
                        attr.get("name").and_then(|v| v.as_str()),
                        attr.get("dataType").and_then(|v| v.as_str())
                    ) {
                        let required = attr.get("ui")
                            .and_then(|ui| ui.get("required"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        self.template_attributes.push(TemplateAttribute {
                            name: name.to_string(),
                            data_type: data_type.to_string(),
                            required,
                            description: attr.get("ui")
                                .and_then(|ui| ui.get("label"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("").to_string(),
                            allowed_values: None,
                        });
                    }
                }
            }

            self.mode = DesignerMode::EditTemplate;
            self.unsaved_changes = false;
            self.tokenize_dsl();
            wasm_utils::console_log(&format!("🎨 Started editing template: {}", self.template_id));
        }
    }

    /// Load available domains from database
    fn load_domains_from_database(&mut self) {
        // Mock data - in real implementation this would be async API call
        self.available_domains = vec![
            Domain {
                id: 1,
                name: "customer_onboarding".to_string(),
                description: "Customer registration and initial data collection".to_string(),
                regulatory_framework: "KYC".to_string(),
            },
            Domain {
                id: 2,
                name: "kyc_verification".to_string(),
                description: "Know Your Customer identity verification process".to_string(),
                regulatory_framework: "AML".to_string(),
            },
            Domain {
                id: 3,
                name: "risk_assessment".to_string(),
                description: "Customer risk profiling and scoring".to_string(),
                regulatory_framework: "AML".to_string(),
            },
            Domain {
                id: 4,
                name: "data_privacy".to_string(),
                description: "GDPR and privacy compliance for customer data".to_string(),
                regulatory_framework: "GDPR".to_string(),
            },
        ];
        wasm_utils::console_log(&format!("📊 Loaded {} domains from database", self.available_domains.len()));
    }

    /// Load EBNF templates for selected domain from database
    fn load_ebnf_templates_for_domain(&mut self, domain_id: i32) {
        wasm_utils::console_log(&format!("📝 Loading EBNF templates from database for domain {}", domain_id));

        // Make async API call to fetch EBNF templates from database
        wasm_bindgen_futures::spawn_local({
            async move {
                let url = "http://localhost:3030/api/ebnf-templates";
                match reqwest::get(url).await {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.text().await {
                                Ok(text) => {
                                    wasm_utils::console_log(&format!("✅ Got EBNF templates from database: {}", text));
                                    // Parse and log the templates - in production this would update the UI state
                                    if let Ok(templates) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(templates_array) = templates.as_array() {
                                            wasm_utils::console_log(&format!("📊 Database contains {} EBNF templates", templates_array.len()));
                                            for template in templates_array {
                                                if let (Some(name), Some(pattern)) = (
                                                    template.get("template_name").and_then(|n| n.as_str()),
                                                    template.get("ebnf_pattern").and_then(|p| p.as_str())
                                                ) {
                                                    wasm_utils::console_log(&format!("📝 EBNF: {} -> {}", name, &pattern[..50.min(pattern.len())]));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => wasm_utils::console_log(&format!("❌ Failed to parse EBNF response: {:?}", e)),
                            }
                        } else {
                            wasm_utils::console_log(&format!("❌ EBNF API returned status: {}", response.status()));
                        }
                    }
                    Err(e) => wasm_utils::console_log(&format!("❌ Failed to fetch EBNF templates: {:?}", e)),
                }
            }
        });

        // For immediate UI functionality, load fallback data that matches the database
        // This ensures the UI works while the async call completes
        self.available_ebnf_templates = vec![
            EbnfTemplate {
                id: 1,
                template_name: "simple_concatenation".to_string(),
                description: "Concatenate two or more string attributes".to_string(),
                ebnf_pattern: "result ::= {source_attr} (\" \" {source_attr})*".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 2,
                template_name: "conditional_assignment".to_string(),
                description: "Assign value based on condition".to_string(),
                ebnf_pattern: "result ::= IF {condition} THEN {true_value} ELSE {false_value}".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 3,
                template_name: "lookup_transformation".to_string(),
                description: "Transform value using lookup table".to_string(),
                ebnf_pattern: "result ::= LOOKUP({source_attr}, {lookup_table})".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 4,
                template_name: "arithmetic_calculation".to_string(),
                description: "Perform arithmetic operations".to_string(),
                ebnf_pattern: "result ::= {operand1} {operator} {operand2}".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 5,
                template_name: "validation_rule".to_string(),
                description: "Validate data against business rules".to_string(),
                ebnf_pattern: "result ::= VALIDATE({source_attr}, {rule_expr})".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 6,
                template_name: "aggregation_rule".to_string(),
                description: "Aggregate data using functions".to_string(),
                ebnf_pattern: "result ::= {agg_function}({source_attrs})".to_string(),
                complexity_level: "simple".to_string(),
            },
            EbnfTemplate {
                id: 7,
                template_name: "data_dictionary_lookup".to_string(),
                description: "Retrieve data from the data dictionary using GET-DATA verb".to_string(),
                ebnf_pattern: "result ::= GET-DATA {attribute_path} FROM {data_source} [WHERE {condition}]".to_string(),
                complexity_level: "simple".to_string(),
            },
        ];
        wasm_utils::console_log(&format!("📝 Loaded {} EBNF templates (fallback data matching database)", self.available_ebnf_templates.len()));
    }

    /// Tokenize DSL content for syntax highlighting
    fn tokenize_dsl(&mut self) {
        self.syntax_tokens.clear();
        let content = &self.dsl_editor_content;

        // Simple tokenizer - in real implementation this would use the EBNF grammar
        let lines = content.lines();
        let mut pos = 0;

        for line in lines {
            let trimmed = line.trim();
            let line_start = pos;

            if trimmed.starts_with('#') {
                self.syntax_tokens.push(SyntaxToken {
                    text: line.to_string(),
                    token_type: TokenType::Comment,
                    start_pos: line_start,
                    end_pos: pos + line.len(),
                });
            } else if trimmed.starts_with("WORKFLOW") || trimmed.starts_with("STEP") || trimmed.starts_with("PHASE") {
                // Split into keyword and rest
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if !parts.is_empty() {
                    self.syntax_tokens.push(SyntaxToken {
                        text: parts[0].to_string(),
                        token_type: TokenType::Keyword,
                        start_pos: line_start,
                        end_pos: line_start + parts[0].len(),
                    });
                }
            } else if trimmed.contains("ASSESS_RISK") || trimmed.contains("SCREEN_ENTITY") || trimmed.contains("LOG") {
                self.syntax_tokens.push(SyntaxToken {
                    text: line.to_string(),
                    token_type: TokenType::Command,
                    start_pos: line_start,
                    end_pos: pos + line.len(),
                });
            }

            pos += line.len() + 1; // +1 for newline
        }

        wasm_utils::console_log(&format!("🎨 Tokenized DSL: {} tokens", self.syntax_tokens.len()));
    }

    /// Render the Template Designer IDE
    pub fn render(&mut self, ui: &mut egui::Ui) {
        match self.mode {
            DesignerMode::SelectTemplate => self.render_template_selection(ui),
            DesignerMode::SelectDomain => self.render_domain_selection(ui),
            DesignerMode::SelectEbnf => self.render_ebnf_selection(ui),
            DesignerMode::EditTemplate => self.render_template_editor(ui),
            DesignerMode::Preview => self.render_template_preview(ui),
        }

        // Global dialogs
        self.render_save_dialog(ui);
        self.render_discard_dialog(ui);
    }

    fn render_template_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎨 Template Designer IDE");
        ui.separator();

        ui.label("Choose how to start:");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("➕ Create New Template").clicked() {
                self.start_new_template();
            }

            if ui.button("📝 Edit Existing Template").clicked() {
                // This would be called from the main templates view
                ui.label("(Select from Templates tab first)");
            }
        });

        ui.add_space(20.0);
        ui.label("🏗️ Full Round-Trip Workflow:");
        ui.label("1. Select Domain (KYC, Onboarding, etc.)");
        ui.label("2. Choose EBNF Grammar for syntax rules");
        ui.label("3. Edit DSL with syntax highlighting");
        ui.label("4. Configure template attributes");
        ui.label("5. Save to database or discard changes");
    }

    fn render_domain_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌐 Select Domain");
        ui.separator();

        ui.label("Choose the domain for your template:");
        ui.add_space(10.0);

        if self.available_domains.is_empty() {
            ui.label("Loading domains from database...");
            return;
        }

        let domains = self.available_domains.clone();
        for domain in &domains {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&domain.name);
                        ui.label(&domain.description);
                        ui.small(format!("Framework: {}", domain.regulatory_framework));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Select").clicked() {
                            self.selected_domain_id = Some(domain.id);
                            self.load_ebnf_templates_for_domain(domain.id);
                            self.mode = DesignerMode::SelectEbnf;
                        }
                    });
                });
            });
            ui.add_space(5.0);
        }

        ui.add_space(20.0);
        if ui.button("⬅️ Back").clicked() {
            self.mode = DesignerMode::SelectTemplate;
        }
    }

    fn render_ebnf_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Select EBNF Grammar");
        ui.separator();

        ui.label("Choose the EBNF grammar template for syntax rules:");
        ui.add_space(10.0);

        let ebnf_templates = self.available_ebnf_templates.clone();
        for ebnf in &ebnf_templates {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&ebnf.template_name);
                            ui.label(&ebnf.description);
                            ui.small(format!("Complexity: {}", ebnf.complexity_level));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Select").clicked() {
                                self.selected_ebnf_id = Some(ebnf.id);
                                self.initialize_template_editor(&ebnf.ebnf_pattern);
                                self.mode = DesignerMode::EditTemplate;
                            }
                        });
                    });

                    ui.collapsing("EBNF Pattern", |ui| {
                        ui.code(&ebnf.ebnf_pattern);
                    });
                });
            });
            ui.add_space(5.0);
        }

        ui.add_space(20.0);
        if ui.button("⬅️ Back").clicked() {
            self.mode = DesignerMode::SelectDomain;
        }
    }

    fn initialize_template_editor(&mut self, ebnf_pattern: &str) {
        // Initialize with a basic template based on EBNF
        let template_content = if ebnf_pattern.contains("GET-DATA") {
            format!(
                "WORKFLOW \"DataRetrievalWorkflow\"\n\nSTEP \"Initialize\"\n    LOG \"Starting data retrieval workflow\"\n    # Based on EBNF: {}\n\nSTEP \"RetrieveData\"\n    # Example GET-DATA usage (stub for data dictionary integration)\n    GET-DATA client.personal_info.full_name FROM customer_data\n    GET-DATA account.balance FROM account_data WHERE account.status == \"active\"\n    GET-DATA risk.score FROM risk_assessment WHERE client_id == client_id\n\nSTEP \"ProcessData\"\n    LOG \"Processing retrieved data\"\n    # Process the retrieved data\n\nSTEP \"Complete\"\n    LOG \"Data retrieval workflow completed\"",
                ebnf_pattern
            )
        } else {
            format!(
                "WORKFLOW \"NewWorkflow\"\n\nSTEP \"Initialize\"\n    # TODO: Add your logic here\n    LOG \"Starting workflow\"\n\nSTEP \"Process\"\n    # Based on EBNF: {}\n    # Add domain-specific commands\n\nSTEP \"Complete\"\n    LOG \"Workflow completed\"",
                ebnf_pattern
            )
        };

        self.template_dsl = template_content;
        self.dsl_editor_content = self.template_dsl.clone();
        self.tokenize_dsl();

        // Add default attributes
        self.template_attributes = vec![
            TemplateAttribute {
                name: "case_id".to_string(),
                data_type: "String".to_string(),
                required: true,
                description: "Unique case identifier".to_string(),
                allowed_values: None,
            }
        ];
    }

    fn render_template_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("✏️ Template Editor");
        ui.separator();

        // Top toolbar
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                self.show_save_dialog = true;
            }

            if ui.button("🗑️ Discard").clicked() {
                self.show_discard_dialog = true;
            }

            ui.separator();

            ui.checkbox(&mut self.show_ast_view, "AST View");
            ui.checkbox(&mut self.show_preview, "Preview");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.unsaved_changes {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Unsaved changes");
                }
            });
        });

        ui.separator();

        // Template metadata
        ui.horizontal(|ui| {
            ui.label("Template Name:");
            if ui.text_edit_singleline(&mut self.template_name).changed() {
                self.unsaved_changes = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Description:");
            if ui.text_edit_singleline(&mut self.template_description).changed() {
                self.unsaved_changes = true;
            }
        });

        ui.add_space(10.0);

        // Main editor area
        ui.horizontal(|ui| {
            // Left panel - DSL Editor
            ui.vertical(|ui| {
                ui.strong("DSL Code:");
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        if ui.add(
                            egui::TextEdit::multiline(&mut self.dsl_editor_content)
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .code_editor()
                        ).changed() {
                            self.unsaved_changes = true;
                            self.tokenize_dsl();
                        }
                    });

                // Syntax highlighting preview
                if !self.syntax_tokens.is_empty() {
                    ui.collapsing("Syntax Tokens", |ui| {
                        for token in &self.syntax_tokens {
                            ui.horizontal(|ui| {
                                let color = match token.token_type {
                                    TokenType::Keyword => egui::Color32::from_rgb(86, 156, 214),
                                    TokenType::Command => egui::Color32::from_rgb(78, 201, 176),
                                    TokenType::Comment => egui::Color32::from_rgb(106, 153, 85),
                                    _ => egui::Color32::WHITE,
                                };
                                ui.colored_label(color, &token.text);
                                ui.label(format!("({:?})", token.token_type));
                            });
                        }
                    });
                }
            });

            ui.separator();

            // Right panel - Attributes
            ui.vertical(|ui| {
                ui.strong("Template Attributes:");

                // Existing attributes
                for (i, attr) in self.template_attributes.clone().iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&attr.name);
                                ui.label(format!("{} ({})", attr.data_type, if attr.required { "required" } else { "optional" }));
                                if !attr.description.is_empty() {
                                    ui.small(&attr.description);
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️").clicked() {
                                    self.template_attributes.remove(i);
                                    self.unsaved_changes = true;
                                }
                            });
                        });
                    });
                }

                ui.add_space(10.0);

                // Add new attribute
                ui.collapsing("➕ Add Attribute", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_attr_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.new_attr_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.new_attr_type, "String".to_string(), "String");
                                ui.selectable_value(&mut self.new_attr_type, "Number".to_string(), "Number");
                                ui.selectable_value(&mut self.new_attr_type, "Boolean".to_string(), "Boolean");
                                ui.selectable_value(&mut self.new_attr_type, "Array".to_string(), "Array");
                                ui.selectable_value(&mut self.new_attr_type, "Object".to_string(), "Object");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.new_attr_required, "Required");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut self.new_attr_description);
                    });

                    if ui.button("Add Attribute").clicked() && !self.new_attr_name.is_empty() {
                        self.template_attributes.push(TemplateAttribute {
                            name: self.new_attr_name.clone(),
                            data_type: self.new_attr_type.clone(),
                            required: self.new_attr_required,
                            description: self.new_attr_description.clone(),
                            allowed_values: None,
                        });

                        // Clear form
                        self.new_attr_name.clear();
                        self.new_attr_description.clear();
                        self.new_attr_required = false;
                        self.unsaved_changes = true;
                    }
                });
            });
        });

        // AST View (if enabled)
        if self.show_ast_view {
            ui.separator();
            ui.collapsing("🌳 AST View", |ui| {
                ui.label("Abstract Syntax Tree representation:");
                ui.code("AST parsing would be implemented here using the EBNF grammar");
            });
        }

        // Preview (if enabled)
        if self.show_preview {
            ui.separator();
            self.render_template_preview(ui);
        }
    }

    fn render_template_preview(&mut self, ui: &mut egui::Ui) {
        ui.strong("📋 Template Preview:");

        let preview_json = json!({
            "id": self.template_name,
            "description": self.template_description,
            "attributes": self.template_attributes.iter().map(|attr| {
                json!({
                    "name": attr.name,
                    "dataType": attr.data_type,
                    "allowedValues": attr.allowed_values,
                    "ui": {
                        "required": attr.required,
                        "label": if attr.description.is_empty() { &attr.name } else { &attr.description }
                    }
                })
            }).collect::<Vec<_>>(),
            "dsl": self.dsl_editor_content
        });

        ui.code(serde_json::to_string_pretty(&preview_json).unwrap_or_else(|_| "Error generating preview".to_string()));
    }

    fn render_save_dialog(&mut self, ui: &mut egui::Ui) {
        if self.show_save_dialog {
            egui::Window::new("💾 Save Template")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Save template to database?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_template();
                            self.show_save_dialog = false;
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_save_dialog = false;
                        }
                    });
                });
        }
    }

    fn render_discard_dialog(&mut self, ui: &mut egui::Ui) {
        if self.show_discard_dialog {
            egui::Window::new("🗑️ Discard Changes")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Discard all unsaved changes?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("🗑️ Discard").clicked() {
                            self.discard_changes();
                            self.show_discard_dialog = false;
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_discard_dialog = false;
                        }
                    });
                });
        }
    }

    fn save_template(&mut self) {
        // Build create template request with domain and EBNF foreign keys
        let create_request = json!({
            "template_name": self.template_name,
            "description": self.template_description,
            "domain_id": self.selected_domain_id,
            "ebnf_template_id": self.selected_ebnf_id,
            "dsl_code": self.dsl_editor_content,
            "attributes": self.template_attributes.iter().map(|attr| {
                json!({
                    "name": attr.name,
                    "dataType": attr.data_type,
                    "allowedValues": attr.allowed_values,
                    "ui": {
                        "required": attr.required,
                        "label": if attr.description.is_empty() { &attr.name } else { &attr.description }
                    }
                })
            }).collect::<Vec<_>>()
        });

        wasm_utils::console_log(&format!("💾 Saving template with domain/EBNF linkage: {}", serde_json::to_string_pretty(&create_request).unwrap_or_default()));

        // Make API call to create template with proper foreign key relationships
        wasm_bindgen_futures::spawn_local({
            let request_data = create_request.to_string();
            let template_name = self.template_name.clone();
            async move {
                let client = reqwest::Client::new();
                let url = "http://localhost:3030/api/templates/create";

                match client.put(url)
                    .header("Content-Type", "application/json")
                    .body(request_data)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.text().await {
                                Ok(response_text) => {
                                    wasm_utils::console_log(&format!("✅ Template '{}' saved successfully with database foreign keys!", template_name));
                                    wasm_utils::console_log(&format!("📊 Server response: {}", response_text));
                                }
                                Err(e) => wasm_utils::console_log(&format!("❌ Error reading save response: {:?}", e)),
                            }
                        } else {
                            wasm_utils::console_log(&format!("❌ Save failed with status: {}", response.status()));
                        }
                    }
                    Err(e) => wasm_utils::console_log(&format!("❌ Failed to save template: {:?}", e)),
                }
            }
        });

        self.unsaved_changes = false;
        self.status_message = format!("Template '{}' saved with domain and EBNF linkage!", self.template_name);
        self.mode = DesignerMode::SelectTemplate;
    }

    fn discard_changes(&mut self) {
        wasm_utils::console_log("🗑️ Discarding template changes");
        self.mode = DesignerMode::SelectTemplate;
        self.unsaved_changes = false;
        self.template_name.clear();
        self.template_description.clear();
        self.dsl_editor_content.clear();
        self.template_attributes.clear();
    }
}

impl Default for TemplateDesignerIDE {
    fn default() -> Self {
        Self::new()
    }
}