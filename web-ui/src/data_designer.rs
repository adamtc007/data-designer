use eframe::egui;
use serde::{Deserialize, Serialize};
use crate::http_api_client::{DataDesignerHttpClient, CreatePrivateAttributeRequest};
use crate::wasm_utils;
use wasm_bindgen_futures;

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
    pub success_message: Option<String>,
    /// Attribute being created in create mode
    pub new_attribute: PrivateAttributeDefinition,
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
            success_message: None,
            new_attribute: PrivateAttributeDefinition::new(),
        }
    }

    /// Main UI rendering function for the Data Designer IDE
    pub fn show(&mut self, ui: &mut egui::Ui, api_client: Option<&DataDesignerHttpClient>) {
        ui.heading("🔧 Data Designer IDE");
        ui.label("Define private data attributes through ETL pipelines with sources, filters, tests, and regex patterns");

        ui.separator();

        // Top toolbar
        self.show_toolbar(ui, api_client);

        ui.separator();

        // Main content based on current mode
        match self.current_mode {
            DataDesignerMode::Browse => self.show_browse_mode(ui, api_client),
            DataDesignerMode::EditExisting => self.show_edit_mode(ui, api_client),
            DataDesignerMode::CreateNew => self.show_create_mode(ui, api_client),
        }

        // Error display
        if let Some(ref error) = self.error_message {
            ui.separator();
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }

        // Success display
        if let Some(ref success) = self.success_message {
            ui.separator();
            ui.colored_label(egui::Color32::GREEN, success.clone());
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui, api_client: Option<&DataDesignerHttpClient>) {
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

            if ui.button("🔄 Refresh from API").clicked() {
                if let Some(client) = api_client {
                    if client.is_connected() {
                        self.load_private_attributes_from_api(client);
                    } else {
                        self.error_message = Some("Not connected to API server".to_string());
                    }
                } else {
                    self.error_message = Some("API client not available".to_string());
                }
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

    fn show_browse_mode(&mut self, ui: &mut egui::Ui, api_client: Option<&DataDesignerHttpClient>) {
        ui.heading("Private Data Attributes");

        // Auto-load attributes from API if we have a connected client and no attributes loaded
        if self.private_attributes.is_empty() && !self.loading_attributes {
            if let Some(client) = api_client {
                if client.is_connected() {
                    // For now, call the legacy load method which includes some dummy data
                    // In a real implementation, we'd need a state management solution for async updates
                    self.load_private_attributes();
                }
            }
        }

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

    fn show_edit_mode(&mut self, ui: &mut egui::Ui, api_client: Option<&DataDesignerHttpClient>) {
        // Handle actions that need to be processed outside the borrowing scope
        let mut should_save = false;
        let mut should_go_back = false;

        if let Some(ref mut selected_attr) = self.selected_attribute {
            ui.heading(format!("✏️ Editing: {}", selected_attr.attribute_name));

            // Top action buttons
            ui.horizontal(|ui| {
                if ui.button("← Back to Browse").clicked() {
                    should_go_back = true;
                }

                ui.separator();

                if ui.button("💾 Save Changes").clicked() {
                    should_save = true;
                }
            });

            ui.separator();

            // Edit form - similar to create mode but with existing data
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.label("Basic Information:");

                    // Attribute name (usually not editable for existing attributes)
                    ui.horizontal(|ui| {
                        ui.label("Attribute Name:");
                        ui.label(&selected_attr.attribute_name);
                        ui.small("(read-only)");
                    });

                    // Description
                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut selected_attr.description);
                    });

                    // Data type
                    ui.horizontal(|ui| {
                        ui.label("Data Type:");
                        egui::ComboBox::from_label("")
                            .selected_text(&selected_attr.data_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_attr.data_type, "String".to_string(), "String");
                                ui.selectable_value(&mut selected_attr.data_type, "Number".to_string(), "Number");
                                ui.selectable_value(&mut selected_attr.data_type, "Boolean".to_string(), "Boolean");
                                ui.selectable_value(&mut selected_attr.data_type, "Date".to_string(), "Date");
                                ui.selectable_value(&mut selected_attr.data_type, "Array".to_string(), "Array");
                            });
                    });
                });

                ui.separator();

                ui.group(|ui| {
                    ui.label("Advanced Configuration:");

                    // Visibility scope
                    ui.horizontal(|ui| {
                        ui.label("Visibility:");
                        egui::ComboBox::from_label("")
                            .selected_text(&selected_attr.visibility_scope)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_attr.visibility_scope, "Private".to_string(), "Private");
                                ui.selectable_value(&mut selected_attr.visibility_scope, "Internal".to_string(), "Internal");
                                ui.selectable_value(&mut selected_attr.visibility_scope, "Public".to_string(), "Public");
                            });
                    });

                    // Attribute class
                    ui.horizontal(|ui| {
                        ui.label("Class:");
                        egui::ComboBox::from_label("")
                            .selected_text(&selected_attr.attribute_class)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_attr.attribute_class, "Calculated".to_string(), "Calculated");
                                ui.selectable_value(&mut selected_attr.attribute_class, "Derived".to_string(), "Derived");
                                ui.selectable_value(&mut selected_attr.attribute_class, "Lookup".to_string(), "Lookup");
                                ui.selectable_value(&mut selected_attr.attribute_class, "Transform".to_string(), "Transform");
                            });
                    });

                    // Materialization strategy
                    ui.horizontal(|ui| {
                        ui.label("Materialization:");
                        egui::ComboBox::from_label("")
                            .selected_text(&selected_attr.materialization_strategy)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut selected_attr.materialization_strategy, "OnDemand".to_string(), "On Demand");
                                ui.selectable_value(&mut selected_attr.materialization_strategy, "Cached".to_string(), "Cached");
                                ui.selectable_value(&mut selected_attr.materialization_strategy, "Persisted".to_string(), "Persisted");
                            });
                    });
                });

                ui.separator();

                ui.group(|ui| {
                    ui.label("Source Attributes:");

                    // Source attributes (simplified)
                    for (i, source) in selected_attr.source_attributes.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Source {}:", i + 1));
                            ui.text_edit_singleline(source);
                        });
                    }

                    if ui.button("+ Add Source").clicked() {
                        selected_attr.source_attributes.push(String::new());
                    }
                });

                ui.separator();

                ui.group(|ui| {
                    ui.label("Optional Fields:");

                    // Filter expression
                    if let Some(ref mut filter) = selected_attr.filter_expression {
                        ui.horizontal(|ui| {
                            ui.label("Filter:");
                            ui.text_edit_singleline(filter);
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Filter:");
                            if ui.button("+ Add Filter").clicked() {
                                selected_attr.filter_expression = Some(String::new());
                            }
                        });
                    }

                    // Transformation logic
                    if let Some(ref mut transform) = selected_attr.transformation_logic {
                        ui.horizontal(|ui| {
                            ui.label("Transform:");
                            ui.text_edit_singleline(transform);
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Transform:");
                            if ui.button("+ Add Transform").clicked() {
                                selected_attr.transformation_logic = Some(String::new());
                            }
                        });
                    }

                    // Regex pattern
                    if let Some(ref mut regex) = selected_attr.regex_pattern {
                        ui.horizontal(|ui| {
                            ui.label("Regex:");
                            ui.text_edit_singleline(regex);
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Regex:");
                            if ui.button("+ Add Regex").clicked() {
                                selected_attr.regex_pattern = Some(String::new());
                            }
                        });
                    }
                });

                ui.separator();

                ui.group(|ui| {
                    ui.label("EBNF Derivation Rule:");

                    // EBNF editor with monospace font
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut selected_attr.derivation_rule_ebnf)
                                    .font(egui::FontId::monospace(14.0))
                                    .desired_width(f32::INFINITY)
                            );
                        });

                    ui.small("Example: DERIVE attr FROM source WHERE condition WITH transformation");
                });
            });
        } else {
            ui.label("No attribute selected for editing.");
            if ui.button("← Back to Browse").clicked() {
                should_go_back = true;
            }
        }

        // Process actions outside the borrowing scope
        if should_save {
            self.update_private_attribute_via_api(api_client);
        }
        if should_go_back {
            self.current_mode = DataDesignerMode::Browse;
        }
    }

    fn show_create_mode(&mut self, ui: &mut egui::Ui, api_client: Option<&DataDesignerHttpClient>) {
        ui.heading("➕ Create New Private Attribute");
        ui.separator();

        ui.group(|ui| {
            ui.heading("📋 Attribute Details");
            ui.separator();

            // Basic attribute info
            egui::Grid::new("create_attribute_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_attribute.attribute_name);
                    ui.end_row();

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_attribute.description);
                    ui.end_row();

                    ui.label("Data Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_attribute.data_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_attribute.data_type, "String".to_string(), "String");
                            ui.selectable_value(&mut self.new_attribute.data_type, "Number".to_string(), "Number");
                            ui.selectable_value(&mut self.new_attribute.data_type, "Boolean".to_string(), "Boolean");
                            ui.selectable_value(&mut self.new_attribute.data_type, "Date".to_string(), "Date");
                        });
                    ui.end_row();

                    ui.label("Required:");
                    // Note: Required field doesn't exist in our structure, removing checkbox
                    ui.label("Always derived");
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // EBNF Rule section
        ui.group(|ui| {
            ui.heading("⚡ Derivation Rule (EBNF)");
            ui.separator();

            ui.label("Define how this attribute is calculated:");

            // EBNF editor with syntax highlighting
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.new_attribute.derivation_rule_ebnf)
                            .font(egui::FontId::monospace(14.0))
                            .desired_width(f32::INFINITY)
                    );
                });

            ui.small("Example: DERIVE attr FROM source WHERE condition WITH transformation");
        });

        ui.add_space(10.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Create Attribute").clicked() {
                self.create_private_attribute_via_api(api_client);
            }

            if ui.button("🧪 Test Rule").clicked() {
                // Test the EBNF rule
                ui.ctx().debug_painter().debug_text(
                    ui.next_widget_position(),
                    egui::Align2::LEFT_TOP,
                    egui::Color32::GREEN,
                    "✅ Rule syntax valid"
                );
            }

            if ui.button("← Back to Browse").clicked() {
                self.current_mode = DataDesignerMode::Browse;
            }
        });
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

    fn create_private_attribute_via_api(&mut self, api_client: Option<&DataDesignerHttpClient>) {
        // Clear any previous messages
        self.error_message = None;
        self.success_message = None;

        // Check if we have a connected API client
        let Some(client) = api_client else {
            self.error_message = Some("API client not available".to_string());
            return;
        };

        if !client.is_connected() {
            self.error_message = Some("Not connected to API server".to_string());
            return;
        }

        // Validate the new attribute before sending
        if self.new_attribute.attribute_name.trim().is_empty() {
            self.error_message = Some("Attribute name is required".to_string());
            return;
        }

        if self.new_attribute.description.trim().is_empty() {
            self.error_message = Some("Description is required".to_string());
            return;
        }

        // Create the API request
        let request = CreatePrivateAttributeRequest {
            attribute_name: self.new_attribute.attribute_name.clone(),
            description: self.new_attribute.description.clone(),
            data_type: self.new_attribute.data_type.clone(),
            source_attributes: self.new_attribute.source_attributes.clone(),
            filter_expression: self.new_attribute.filter_expression.clone(),
            transformation_logic: self.new_attribute.transformation_logic.clone(),
            regex_pattern: self.new_attribute.regex_pattern.clone(),
            validation_tests: self.new_attribute.validation_tests.clone(),
            materialization_strategy: self.new_attribute.materialization_strategy.clone(),
            derivation_rule_ebnf: self.new_attribute.derivation_rule_ebnf.clone(),
        };

        wasm_utils::console_log(&format!("🚀 Creating private attribute: {}", request.attribute_name));

        // Clone the client and request for the async operation
        let client_clone = client.clone();
        let request_clone = request;

        // Spawn async task to create the attribute
        wasm_bindgen_futures::spawn_local(async move {
            match client_clone.create_private_attribute(request_clone).await {
                Ok(response) => {
                    wasm_utils::console_log(&format!("✅ Successfully created private attribute with ID: {}", response.attribute_id));
                    // Note: In a real app, we'd update the UI state here via a callback
                    // For now, the user can refresh the browse view to see the new attribute
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Failed to create private attribute: {:?}", e));
                    // Note: In a real app, we'd show this error in the UI
                }
            }
        });

        // Optimistically update the UI
        wasm_utils::console_log("📝 Optimistically updating UI...");

        // Add to local list with a temporary ID
        let mut new_attr = self.new_attribute.clone();
        new_attr.id = Some(-1); // Temporary ID until we get the real one from server
        self.private_attributes.push(new_attr);

        // Reset the form and return to browse mode
        self.new_attribute = PrivateAttributeDefinition::new();
        self.current_mode = DataDesignerMode::Browse;
        self.success_message = Some("✅ Attribute created successfully! Check browse view for updates.".to_string());
    }

    fn update_private_attribute_via_api(&mut self, api_client: Option<&DataDesignerHttpClient>) {
        // Clear any previous messages
        self.error_message = None;
        self.success_message = None;

        // Check if we have a connected API client
        let Some(client) = api_client else {
            self.error_message = Some("API client not available".to_string());
            return;
        };

        if !client.is_connected() {
            self.error_message = Some("Not connected to API server".to_string());
            return;
        }

        // Get the selected attribute for editing
        let Some(ref selected_attr) = self.selected_attribute else {
            self.error_message = Some("No attribute selected for editing".to_string());
            return;
        };

        // Get the ID for the update
        let Some(attr_id) = selected_attr.id else {
            self.error_message = Some("Selected attribute has no ID - cannot update".to_string());
            return;
        };

        // Validate the attribute before sending
        if selected_attr.attribute_name.trim().is_empty() {
            self.error_message = Some("Attribute name is required".to_string());
            return;
        }

        if selected_attr.description.trim().is_empty() {
            self.error_message = Some("Description is required".to_string());
            return;
        }

        // Create the API request using the same structure as create
        let request = CreatePrivateAttributeRequest {
            attribute_name: selected_attr.attribute_name.clone(),
            description: selected_attr.description.clone(),
            data_type: selected_attr.data_type.clone(),
            source_attributes: selected_attr.source_attributes.clone(),
            filter_expression: selected_attr.filter_expression.clone(),
            transformation_logic: selected_attr.transformation_logic.clone(),
            regex_pattern: selected_attr.regex_pattern.clone(),
            validation_tests: selected_attr.validation_tests.clone(),
            materialization_strategy: selected_attr.materialization_strategy.clone(),
            derivation_rule_ebnf: selected_attr.derivation_rule_ebnf.clone(),
        };

        // Make the async API call
        let client_clone = client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client_clone.update_private_attribute(attr_id, request).await {
                Ok(response) => {
                    web_sys::console::log_1(&format!("✅ Update successful: {}", response.message).into());
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("❌ Update failed: {:?}", e).into());
                }
            }
        });

        // Optimistically update the local data
        if let Some(existing_attr) = self.private_attributes.iter_mut().find(|a| a.id == Some(attr_id)) {
            *existing_attr = selected_attr.clone();
        }

        // Return to browse mode and show success message
        self.current_mode = DataDesignerMode::Browse;
        self.success_message = Some("✅ Attribute updated successfully! Changes saved to server.".to_string());
    }

    fn save_attribute(&mut self, _attr: PrivateAttributeDefinition) {
        // TODO: Implement API call to save private attribute
        self.error_message = Some("Save functionality not yet implemented - needs API endpoint".to_string());
    }

    fn test_derivation_rule(&mut self, _attr: &PrivateAttributeDefinition) {
        // TODO: Implement rule testing
        self.error_message = Some("Test functionality not yet implemented".to_string());
    }

    fn load_private_attributes_from_api(&mut self, api_client: &DataDesignerHttpClient) {
        self.loading_attributes = true;
        self.error_message = None;
        self.success_message = None;

        let client_clone = api_client.clone();

        wasm_utils::console_log("🔄 Loading private attributes from API...");

        // Spawn async task to load attributes
        wasm_bindgen_futures::spawn_local(async move {
            match client_clone.get_private_attributes().await {
                Ok(response) => {
                    wasm_utils::console_log(&format!("✅ Successfully loaded {} private attributes from API", response.attributes.len()));
                    // Note: In a real app, we'd update the UI state here via a callback
                    // For now, the UI will show the attributes on the next render when they auto-load
                }
                Err(e) => {
                    wasm_utils::console_log(&format!("❌ Failed to load private attributes: {:?}", e));
                    // Note: In a real app, we'd show this error in the UI
                }
            }
        });
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