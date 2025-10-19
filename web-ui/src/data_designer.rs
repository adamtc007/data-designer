use eframe::egui;
use serde::{Deserialize, Serialize};

/// Data Designer IDE for defining private data attributes via ETL pipelines
#[derive(Debug, Clone)]
pub struct DataDesignerIDE {
    /// Current mode of the Data Designer
    pub current_mode: DataDesignerMode,
    /// List of existing private attributes
    pub private_attributes: Vec<PrivateAttributeDefinition>,
    /// Currently selected/editing attribute
    pub selected_attribute: Option<PrivateAttributeDefinition>,
    /// Available public attributes for source selection
    pub public_attributes: Vec<PublicAttributeInfo>,
    /// Loading states
    pub loading_attributes: bool,
    pub loading_public_data: bool,
    /// UI state
    pub show_create_dialog: bool,
    pub filter_text: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataDesignerMode {
    Browse,         // Browse existing private attributes
    EditExisting,   // Edit an existing private attribute
    CreateNew,      // Create a new private attribute
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateAttributeDefinition {
    pub id: Option<i32>,
    pub attribute_name: String,
    pub description: String,
    pub data_type: String,
    pub visibility_scope: String, // Always "private" for this IDE
    pub attribute_class: String,  // Always "derived" for this IDE

    // ETL Pipeline Definition
    pub source_attributes: Vec<String>,
    pub filter_expression: Option<String>,
    pub transformation_logic: Option<String>,
    pub regex_pattern: Option<String>,
    pub validation_tests: Option<String>,
    pub materialization_strategy: String,

    // EBNF Derivation Rule (auto-generated from above)
    pub derivation_rule_ebnf: String,

    // Metadata
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAttributeInfo {
    pub full_path: String,
    pub attribute_name: String,
    pub entity_name: String,
    pub data_type: String,
    pub description: Option<String>,
    pub attribute_type: String, // "business" or "system"
}

impl Default for DataDesignerIDE {
    fn default() -> Self {
        Self::new()
    }
}

impl DataDesignerIDE {
    pub fn new() -> Self {
        Self {
            current_mode: DataDesignerMode::Browse,
            private_attributes: Vec::new(),
            selected_attribute: None,
            public_attributes: Vec::new(),
            loading_attributes: false,
            loading_public_data: false,
            show_create_dialog: false,
            filter_text: String::new(),
            error_message: None,
        }
    }

    /// Main UI rendering function for the Data Designer IDE
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔧 Data Designer IDE");
        ui.label("Define private data attributes through ETL pipelines with sources, filters, tests, and regex patterns");

        ui.separator();

        // Top toolbar
        self.show_toolbar(ui);

        ui.separator();

        // Main content based on current mode
        match self.current_mode {
            DataDesignerMode::Browse => self.show_browse_mode(ui),
            DataDesignerMode::EditExisting => self.show_edit_mode(ui),
            DataDesignerMode::CreateNew => self.show_create_mode(ui),
        }

        // Error display
        if let Some(ref error) = self.error_message {
            ui.separator();
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Mode buttons
            if ui.selectable_label(self.current_mode == DataDesignerMode::Browse, "📋 Browse").clicked() {
                self.current_mode = DataDesignerMode::Browse;
                self.load_private_attributes();
            }

            if ui.button("➕ Create New Private Attribute").clicked() {
                self.current_mode = DataDesignerMode::CreateNew;
                self.selected_attribute = Some(PrivateAttributeDefinition::new());
                self.load_public_attributes();
            }

            ui.separator();

            // Refresh button
            if ui.button("🔄 Refresh").clicked() {
                self.load_private_attributes();
                self.load_public_attributes();
            }

            // Search
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.filter_text);
        });
    }

    fn show_browse_mode(&mut self, ui: &mut egui::Ui) {
        ui.heading("Private Data Attributes");

        if self.loading_attributes {
            ui.spinner();
            ui.label("Loading private attributes...");
            return;
        }

        // Filter private attributes
        let filtered_attributes: Vec<_> = self.private_attributes
            .iter()
            .filter(|attr| {
                if self.filter_text.is_empty() {
                    true
                } else {
                    attr.attribute_name.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                    attr.description.to_lowercase().contains(&self.filter_text.to_lowercase())
                }
            })
            .cloned()
            .collect();

        if filtered_attributes.is_empty() {
            ui.label("No private attributes found. Create your first private attribute using the 'Create New' button.");
            return;
        }

        let mut edit_attribute = None;
        let mut delete_attribute = None;

        // Table of private attributes
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("private_attributes_grid")
                .num_columns(6)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.strong("Attribute Name");
                    ui.strong("Data Type");
                    ui.strong("Sources");
                    ui.strong("Strategy");
                    ui.strong("Description");
                    ui.strong("Actions");
                    ui.end_row();

                    // Rows
                    for attr in &filtered_attributes {
                        ui.label(&attr.attribute_name);
                        ui.label(&attr.data_type);
                        ui.label(attr.source_attributes.join(", "));
                        ui.label(&attr.materialization_strategy);
                        ui.label(&attr.description);

                        ui.horizontal(|ui| {
                            if ui.small_button("✏️ Edit").clicked() {
                                edit_attribute = Some(attr.clone());
                            }
                            if ui.small_button("🗑️ Delete").clicked() {
                                delete_attribute = Some(attr.id);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        // Handle actions outside the closure
        if let Some(attr) = edit_attribute {
            self.current_mode = DataDesignerMode::EditExisting;
            self.selected_attribute = Some(attr);
            self.load_public_attributes();
        }
        if let Some(_id) = delete_attribute {
            self.error_message = Some("Delete functionality not yet implemented".to_string());
        }
    }

    fn show_edit_mode(&mut self, ui: &mut egui::Ui) {
        if self.selected_attribute.is_some() {
            let attr_name = self.selected_attribute.as_ref().unwrap().attribute_name.clone();
            ui.heading(format!("✏️ Editing: {}", attr_name));

            // Simple editor placeholder to fix borrowing issues
            ui.label("Data Designer IDE - Edit mode placeholder");
            ui.label("(Borrowing conflicts need architectural fix)");

            if ui.button("← Back to Browse").clicked() {
                self.current_mode = DataDesignerMode::Browse;
            }
        } else {
            ui.label("No attribute selected for editing.");
            if ui.button("← Back to Browse").clicked() {
                self.current_mode = DataDesignerMode::Browse;
            }
        }
    }

    fn show_create_mode(&mut self, ui: &mut egui::Ui) {
        ui.heading("➕ Create New Private Attribute");

        // Simple creator placeholder to fix borrowing issues
        ui.label("Data Designer IDE - Create mode placeholder");
        ui.label("(Borrowing conflicts need architectural fix)");

        if ui.button("← Back to Browse").clicked() {
            self.current_mode = DataDesignerMode::Browse;
        }
    }

    fn show_attribute_editor(&mut self, ui: &mut egui::Ui, attr: &mut PrivateAttributeDefinition) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("attribute_editor")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    // Basic Information
                    ui.strong("Attribute Name:");
                    ui.text_edit_singleline(&mut attr.attribute_name);
                    ui.end_row();

                    ui.strong("Description:");
                    ui.text_edit_multiline(&mut attr.description);
                    ui.end_row();

                    ui.strong("Data Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(&attr.data_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut attr.data_type, "String".to_string(), "String");
                            ui.selectable_value(&mut attr.data_type, "Number".to_string(), "Number");
                            ui.selectable_value(&mut attr.data_type, "Boolean".to_string(), "Boolean");
                            ui.selectable_value(&mut attr.data_type, "Date".to_string(), "Date");
                            ui.selectable_value(&mut attr.data_type, "Decimal".to_string(), "Decimal");
                            ui.selectable_value(&mut attr.data_type, "Enum".to_string(), "Enum");
                        });
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    // ETL Pipeline Definition
                    ui.strong("Source Attributes:");
                    ui.vertical(|ui| {
                        self.show_source_attribute_selector(ui, attr);
                    });
                    ui.end_row();

                    ui.strong("Filter Expression:");
                    ui.text_edit_multiline(attr.filter_expression.get_or_insert_with(String::new));
                    ui.end_row();

                    ui.strong("Transformation Logic:");
                    ui.text_edit_multiline(attr.transformation_logic.get_or_insert_with(String::new));
                    ui.end_row();

                    ui.strong("Regex Pattern:");
                    ui.text_edit_singleline(attr.regex_pattern.get_or_insert_with(String::new));
                    ui.end_row();

                    ui.strong("Validation Tests:");
                    ui.text_edit_multiline(attr.validation_tests.get_or_insert_with(String::new));
                    ui.end_row();

                    ui.strong("Materialization:");
                    egui::ComboBox::from_label("")
                        .selected_text(&attr.materialization_strategy)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut attr.materialization_strategy, "computed".to_string(), "computed");
                            ui.selectable_value(&mut attr.materialization_strategy, "cached".to_string(), "cached");
                            ui.selectable_value(&mut attr.materialization_strategy, "persisted".to_string(), "persisted");
                            ui.selectable_value(&mut attr.materialization_strategy, "hybrid".to_string(), "hybrid");
                        });
                    ui.end_row();
                });

            ui.separator();

            // Generated EBNF Preview
            ui.heading("📝 Generated EBNF Rule");
            self.update_ebnf_rule(attr);
            ui.text_edit_multiline(&mut attr.derivation_rule_ebnf);

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("💾 Save Attribute").clicked() {
                    self.save_attribute(attr.clone());
                }

                if ui.button("🧪 Test Rule").clicked() {
                    self.test_derivation_rule(attr);
                }

                if ui.button("← Cancel").clicked() {
                    self.current_mode = DataDesignerMode::Browse;
                    self.selected_attribute = None;
                }
            });
        });
    }

    fn show_source_attribute_selector(&mut self, ui: &mut egui::Ui, attr: &mut PrivateAttributeDefinition) {
        if self.loading_public_data {
            ui.spinner();
            ui.label("Loading public attributes...");
            return;
        }

        // Show selected sources
        ui.label("Selected Sources:");
        let mut to_remove = Vec::new();
        for (i, source) in attr.source_attributes.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(source);
                if ui.small_button("❌").clicked() {
                    to_remove.push(i);
                }
            });
        }

        // Remove selected items (in reverse order to maintain indices)
        for &i in to_remove.iter().rev() {
            attr.source_attributes.remove(i);
        }

        ui.separator();

        // Add new sources
        ui.label("Available Public Attributes:");
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for pub_attr in &self.public_attributes {
                    if !attr.source_attributes.contains(&pub_attr.full_path) {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} ({})", pub_attr.full_path, pub_attr.data_type));
                            if ui.small_button("➕ Add").clicked() {
                                attr.source_attributes.push(pub_attr.full_path.clone());
                            }
                        });
                    }
                }
            });
    }

    fn update_ebnf_rule(&self, attr: &mut PrivateAttributeDefinition) {
        let mut rule = format!("DERIVE {}", attr.attribute_name);

        if !attr.source_attributes.is_empty() {
            rule.push_str(&format!(" FROM {}", attr.source_attributes.join(", ")));
        }

        if let Some(ref filter) = attr.filter_expression {
            if !filter.is_empty() {
                rule.push_str(&format!(" WHERE {}", filter));
            }
        }

        if let Some(ref transform) = attr.transformation_logic {
            if !transform.is_empty() {
                rule.push_str(&format!(" WITH {}", transform));
            }
        }

        if let Some(ref regex) = attr.regex_pattern {
            if !regex.is_empty() {
                rule.push_str(&format!(" EXTRACT REGEX \"{}\"", regex));
            }
        }

        if let Some(ref test) = attr.validation_tests {
            if !test.is_empty() {
                rule.push_str(&format!(" TEST {}", test));
            }
        }

        rule.push_str(&format!(" MATERIALIZE {}", attr.materialization_strategy));

        attr.derivation_rule_ebnf = rule;
    }

    fn save_attribute(&mut self, _attr: PrivateAttributeDefinition) {
        // TODO: Implement API call to save private attribute
        self.error_message = Some("Save functionality not yet implemented - needs API endpoint".to_string());
    }

    fn test_derivation_rule(&mut self, _attr: &PrivateAttributeDefinition) {
        // TODO: Implement rule testing
        self.error_message = Some("Test functionality not yet implemented".to_string());
    }

    fn load_private_attributes(&mut self) {
        self.loading_attributes = true;
        // TODO: Load from API - for now use dummy data
        self.private_attributes = vec![
            PrivateAttributeDefinition {
                id: Some(1),
                attribute_name: "internal_risk_tier".to_string(),
                description: "Internal risk classification tier".to_string(),
                data_type: "Enum".to_string(),
                visibility_scope: "private".to_string(),
                attribute_class: "derived".to_string(),
                source_attributes: vec!["Client.risk_score".to_string(), "Client.aum_usd".to_string()],
                filter_expression: Some("Client.kyc_status = \"approved\"".to_string()),
                transformation_logic: Some("COMPUTE_RISK_TIER(risk_score, aum_usd)".to_string()),
                regex_pattern: Some("^[A-Z]+$".to_string()),
                validation_tests: Some("result IN [\"LOW\", \"MEDIUM\", \"HIGH\"]".to_string()),
                materialization_strategy: "cached".to_string(),
                derivation_rule_ebnf: "DERIVE internal_risk_tier FROM Client.risk_score, Client.aum_usd WHERE Client.kyc_status = \"approved\" WITH COMPUTE_RISK_TIER(risk_score, aum_usd) EXTRACT REGEX \"^[A-Z]+$\" TEST result IN [\"LOW\", \"MEDIUM\", \"HIGH\"] MATERIALIZE cached".to_string(),
                created_at: Some("2025-01-19T10:00:00Z".to_string()),
                updated_at: Some("2025-01-19T10:00:00Z".to_string()),
            }
        ];
        self.loading_attributes = false;
    }

    fn load_public_attributes(&mut self) {
        self.loading_public_data = true;
        // TODO: Load from data dictionary API - for now use dummy data
        self.public_attributes = vec![
            PublicAttributeInfo {
                full_path: "Client.legal_entity_name".to_string(),
                attribute_name: "legal_entity_name".to_string(),
                entity_name: "Client".to_string(),
                data_type: "String".to_string(),
                description: Some("Legal name of the entity".to_string()),
                attribute_type: "business".to_string(),
            },
            PublicAttributeInfo {
                full_path: "Client.risk_score".to_string(),
                attribute_name: "risk_score".to_string(),
                entity_name: "Client".to_string(),
                data_type: "Number".to_string(),
                description: Some("Risk score".to_string()),
                attribute_type: "derived".to_string(),
            },
            PublicAttributeInfo {
                full_path: "Client.aum_usd".to_string(),
                attribute_name: "aum_usd".to_string(),
                entity_name: "Client".to_string(),
                data_type: "Decimal".to_string(),
                description: Some("Assets under management".to_string()),
                attribute_type: "business".to_string(),
            },
        ];
        self.loading_public_data = false;
    }
}

impl PrivateAttributeDefinition {
    fn new() -> Self {
        Self {
            id: None,
            attribute_name: String::new(),
            description: String::new(),
            data_type: "String".to_string(),
            visibility_scope: "private".to_string(),
            attribute_class: "derived".to_string(),
            source_attributes: Vec::new(),
            filter_expression: None,
            transformation_logic: None,
            regex_pattern: None,
            validation_tests: None,
            materialization_strategy: "computed".to_string(),
            derivation_rule_ebnf: String::new(),
            created_at: None,
            updated_at: None,
        }
    }
}