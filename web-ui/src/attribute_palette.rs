use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::http_api_client::TemplateAttribute;

/// Master attribute definition from the palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterAttribute {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub category: String,
    pub required: bool,
    pub allowed_values: Option<Vec<String>>,
    pub validation_rules: HashMap<String, serde_json::Value>,
    pub ui: AttributeUIMetadata,
}

/// UI metadata for attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeUIMetadata {
    pub label: String,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub input_type: Option<String>,
    #[serde(default)]
    pub help_text: Option<String>,
}

/// Category definition for organizing attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeCategory {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
}

/// Master attributes data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterAttributesData {
    pub attributes: Vec<MasterAttribute>,
    pub categories: Vec<AttributeCategory>,
}

/// Attribute Palette UI component for compositional template editing
#[derive(Debug, Clone)]
pub struct AttributePalette {
    /// Master attributes loaded from the dictionary
    pub master_attributes: Vec<MasterAttribute>,
    /// Categories for organizing attributes
    pub categories: Vec<AttributeCategory>,
    /// Current search filter
    pub search_filter: String,
    /// Selected category filter (None = all categories)
    pub selected_category: Option<String>,
    /// Filtered attributes based on search and category
    pub filtered_attributes: Vec<MasterAttribute>,
    /// Whether the palette is visible
    pub visible: bool,
    /// Loading state
    pub loading: bool,
    /// Error message if loading fails
    pub error_message: Option<String>,
    /// Recently added attributes (for visual feedback)
    pub recently_added: Vec<String>,
    /// Compact view mode
    pub compact_mode: bool,
}

impl AttributePalette {
    pub fn new() -> Self {
        let mut palette = Self {
            master_attributes: Vec::new(),
            categories: Vec::new(),
            search_filter: String::new(),
            selected_category: None,
            filtered_attributes: Vec::new(),
            visible: true,
            loading: true,
            error_message: None,
            recently_added: Vec::new(),
            compact_mode: false,
        };

        palette.load_master_attributes();
        palette
    }

    /// Load master attributes from the JSON file
    fn load_master_attributes(&mut self) {
        // In a real implementation, this would load from the JSON file
        // For now, we'll include the data directly
        match self.load_from_embedded_data() {
            Ok((attributes, categories)) => {
                self.master_attributes = attributes;
                self.categories = categories;
                self.update_filtered_attributes();
                self.loading = false;
                self.error_message = None;
            }
            Err(e) => {
                self.loading = false;
                self.error_message = Some(format!("Failed to load master attributes: {}", e));
            }
        }
    }

    /// Load from embedded data (in a real implementation, this would read the JSON file)
    fn load_from_embedded_data(&self) -> Result<(Vec<MasterAttribute>, Vec<AttributeCategory>), String> {
        // This would normally read from the master_attributes.json file
        // For now, we'll include a subset of the data directly
        let categories = vec![
            AttributeCategory {
                name: "Identity".to_string(),
                description: "Identity and identification attributes".to_string(),
                icon: "👤".to_string(),
                color: "#3B82F6".to_string(),
            },
            AttributeCategory {
                name: "Contact".to_string(),
                description: "Contact information and communication details".to_string(),
                icon: "📞".to_string(),
                color: "#10B981".to_string(),
            },
            AttributeCategory {
                name: "Financial".to_string(),
                description: "Financial products, accounts, and monetary information".to_string(),
                icon: "💰".to_string(),
                color: "#F59E0B".to_string(),
            },
            AttributeCategory {
                name: "Regulatory".to_string(),
                description: "Regulatory compliance and jurisdiction information".to_string(),
                icon: "⚖️".to_string(),
                color: "#EF4444".to_string(),
            },
            AttributeCategory {
                name: "Risk".to_string(),
                description: "Risk assessment and scoring attributes".to_string(),
                icon: "⚠️".to_string(),
                color: "#F97316".to_string(),
            },
            AttributeCategory {
                name: "Workflow".to_string(),
                description: "Process flow and orchestration attributes".to_string(),
                icon: "🔄".to_string(),
                color: "#8B5CF6".to_string(),
            },
            AttributeCategory {
                name: "Documents".to_string(),
                description: "Document types and verification information".to_string(),
                icon: "📄".to_string(),
                color: "#06B6D4".to_string(),
            },
            AttributeCategory {
                name: "Audit".to_string(),
                description: "Audit trail and tracking information".to_string(),
                icon: "📊".to_string(),
                color: "#84CC16".to_string(),
            },
        ];

        let attributes = vec![
            MasterAttribute {
                name: "case_id".to_string(),
                data_type: "String".to_string(),
                description: "Unique identifier for a case or process instance".to_string(),
                category: "Identity".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Case ID".to_string(),
                    placeholder: Some("Enter unique case identifier".to_string()),
                    input_type: Some("text".to_string()),
                    help_text: Some("Must be unique across all templates".to_string()),
                },
            },
            MasterAttribute {
                name: "client_id".to_string(),
                data_type: "String".to_string(),
                description: "Unique identifier for a client entity".to_string(),
                category: "Identity".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Client ID".to_string(),
                    placeholder: Some("Enter client identifier".to_string()),
                    input_type: Some("text".to_string()),
                    help_text: Some("Primary client reference".to_string()),
                },
            },
            MasterAttribute {
                name: "full_name".to_string(),
                data_type: "String".to_string(),
                description: "Complete legal name as it appears on official documents".to_string(),
                category: "Identity".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Full Legal Name".to_string(),
                    placeholder: Some("First Middle Last".to_string()),
                    input_type: Some("text".to_string()),
                    help_text: Some("Must match official documentation".to_string()),
                },
            },
            MasterAttribute {
                name: "email".to_string(),
                data_type: "Email".to_string(),
                description: "Primary email address for customer communication".to_string(),
                category: "Contact".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Email Address".to_string(),
                    placeholder: Some("user@example.com".to_string()),
                    input_type: Some("email".to_string()),
                    help_text: Some("Primary contact email".to_string()),
                },
            },
            MasterAttribute {
                name: "phone_number".to_string(),
                data_type: "PhoneNumber".to_string(),
                description: "Primary contact phone number with country code".to_string(),
                category: "Contact".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Phone Number".to_string(),
                    placeholder: Some("+1234567890".to_string()),
                    input_type: Some("tel".to_string()),
                    help_text: Some("Include country code".to_string()),
                },
            },
            MasterAttribute {
                name: "account_number".to_string(),
                data_type: "String".to_string(),
                description: "Account number for financial services".to_string(),
                category: "Financial".to_string(),
                required: false,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Account Number".to_string(),
                    placeholder: Some("Generated automatically".to_string()),
                    input_type: Some("text".to_string()),
                    help_text: Some("Usually auto-generated during account setup".to_string()),
                },
            },
            MasterAttribute {
                name: "jurisdiction".to_string(),
                data_type: "String".to_string(),
                description: "Legal jurisdiction or regulatory domain".to_string(),
                category: "Regulatory".to_string(),
                required: true,
                allowed_values: Some(vec![
                    "US".to_string(),
                    "UK".to_string(),
                    "EU".to_string(),
                    "APAC".to_string(),
                    "Canada".to_string(),
                ]),
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Jurisdiction".to_string(),
                    placeholder: None,
                    input_type: Some("select".to_string()),
                    help_text: Some("Primary regulatory jurisdiction".to_string()),
                },
            },
            MasterAttribute {
                name: "risk_rating".to_string(),
                data_type: "String".to_string(),
                description: "Risk assessment rating for the client".to_string(),
                category: "Risk".to_string(),
                required: false,
                allowed_values: Some(vec![
                    "Low".to_string(),
                    "Medium".to_string(),
                    "High".to_string(),
                    "Critical".to_string(),
                ]),
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Risk Rating".to_string(),
                    placeholder: None,
                    input_type: Some("select".to_string()),
                    help_text: Some("Client risk assessment".to_string()),
                },
            },
            MasterAttribute {
                name: "status".to_string(),
                data_type: "String".to_string(),
                description: "Current status of the process or case".to_string(),
                category: "Workflow".to_string(),
                required: true,
                allowed_values: Some(vec![
                    "Pending".to_string(),
                    "InProgress".to_string(),
                    "Complete".to_string(),
                    "Approved".to_string(),
                    "Rejected".to_string(),
                ]),
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Status".to_string(),
                    placeholder: None,
                    input_type: Some("select".to_string()),
                    help_text: Some("Current process status".to_string()),
                },
            },
            MasterAttribute {
                name: "document_type".to_string(),
                data_type: "String".to_string(),
                description: "Type of identification document provided".to_string(),
                category: "Documents".to_string(),
                required: false,
                allowed_values: Some(vec![
                    "passport".to_string(),
                    "driver_license".to_string(),
                    "national_id".to_string(),
                    "birth_certificate".to_string(),
                ]),
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Document Type".to_string(),
                    placeholder: None,
                    input_type: Some("select".to_string()),
                    help_text: Some("Type of supporting document".to_string()),
                },
            },
            MasterAttribute {
                name: "created_by".to_string(),
                data_type: "String".to_string(),
                description: "User who created this case or process".to_string(),
                category: "Audit".to_string(),
                required: true,
                allowed_values: None,
                validation_rules: HashMap::new(),
                ui: AttributeUIMetadata {
                    label: "Created By".to_string(),
                    placeholder: Some("Username or ID".to_string()),
                    input_type: Some("text".to_string()),
                    help_text: Some("User identifier".to_string()),
                },
            },
        ];

        Ok((attributes, categories))
    }

    /// Update filtered attributes based on search and category filters
    fn update_filtered_attributes(&mut self) {
        let search_lower = self.search_filter.to_lowercase();

        self.filtered_attributes = self.master_attributes
            .iter()
            .filter(|attr| {
                // Category filter
                if let Some(ref category) = self.selected_category {
                    if &attr.category != category {
                        return false;
                    }
                }

                // Search filter
                if !search_lower.is_empty() {
                    return attr.name.to_lowercase().contains(&search_lower)
                        || attr.description.to_lowercase().contains(&search_lower)
                        || attr.category.to_lowercase().contains(&search_lower)
                        || attr.ui.label.to_lowercase().contains(&search_lower);
                }

                true
            })
            .cloned()
            .collect();
    }

    /// Check if an attribute is already in the template
    fn is_attribute_in_template(&self, attr_name: &str, template_attributes: &[TemplateAttribute]) -> bool {
        template_attributes.iter().any(|attr| attr.name == attr_name)
    }

    /// Add an attribute to the recently added list for visual feedback
    fn mark_recently_added(&mut self, attr_name: String) {
        self.recently_added.push(attr_name);

        // Keep only the last 5 recently added items
        if self.recently_added.len() > 5 {
            self.recently_added.remove(0);
        }
    }

    /// Clear recently added list
    pub fn clear_recently_added(&mut self) {
        self.recently_added.clear();
    }

    /// Convert a MasterAttribute to a TemplateAttribute
    fn to_template_attribute(&self, master_attr: &MasterAttribute) -> TemplateAttribute {
        let mut ui_map = std::collections::HashMap::new();
        ui_map.insert("label".to_string(), serde_json::Value::String(master_attr.ui.label.clone()));
        ui_map.insert("required".to_string(), serde_json::Value::Bool(master_attr.required));

        if let Some(ref placeholder) = master_attr.ui.placeholder {
            ui_map.insert("placeholder".to_string(), serde_json::Value::String(placeholder.clone()));
        }
        if let Some(ref help_text) = master_attr.ui.help_text {
            ui_map.insert("help_text".to_string(), serde_json::Value::String(help_text.clone()));
        }

        TemplateAttribute {
            name: master_attr.name.clone(),
            data_type: master_attr.data_type.clone(),
            allowed_values: master_attr.allowed_values.clone(),
            ui: ui_map,
        }
    }

    /// Main rendering method for the attribute palette
    pub fn show(&mut self, ui: &mut egui::Ui, template_attributes: &[TemplateAttribute]) -> Option<TemplateAttribute> {
        let mut added_attribute = None;

        ui.vertical(|ui| {
            // Header with controls
            ui.horizontal(|ui| {
                ui.heading("🎨 Attribute Palette");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Toggle compact mode
                    if ui.toggle_value(&mut self.compact_mode, "📋").on_hover_text("Toggle compact view").clicked() {
                        // Visual feedback handled by the toggle itself
                    }

                    ui.add_space(8.0);

                    // Visibility toggle
                    if ui.button(if self.visible { "▼ Hide" } else { "▶ Show" }).clicked() {
                        self.visible = !self.visible;
                    }
                });
            });

            if !self.visible {
                return;
            }

            ui.separator();

            // Show loading or error state
            if self.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading attribute dictionary...");
                });
                return;
            }

            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
                if ui.button("🔄 Retry").clicked() {
                    self.loading = true;
                    self.load_master_attributes();
                }
                return;
            }

            // Search box
            ui.horizontal(|ui| {
                ui.label("🔍");
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_filter)
                        .hint_text("Search attributes...")
                        .desired_width(f32::INFINITY)
                );

                if search_response.changed() {
                    self.update_filtered_attributes();
                }

                if ui.button("✕").on_hover_text("Clear search").clicked() {
                    self.search_filter.clear();
                    self.update_filtered_attributes();
                }
            });

            ui.add_space(5.0);

            // Category filters
            ui.horizontal_wrapped(|ui| {
                // All categories button
                let all_selected = self.selected_category.is_none();
                if ui.selectable_label(all_selected, "📂 All").clicked() {
                    self.selected_category = None;
                    self.update_filtered_attributes();
                }

                // Individual category buttons - clone to avoid borrow issues
                let categories = self.categories.clone();
                for category in &categories {
                    let is_selected = self.selected_category.as_ref() == Some(&category.name);
                    let label = format!("{} {}", category.icon, category.name);

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_category = if is_selected {
                            None
                        } else {
                            Some(category.name.clone())
                        };
                        self.update_filtered_attributes();
                    }
                }
            });

            ui.add_space(5.0);

            // Statistics
            ui.horizontal(|ui| {
                ui.small(format!("📊 Showing {} of {} attributes",
                    self.filtered_attributes.len(),
                    self.master_attributes.len()
                ));

                if !self.recently_added.is_empty() {
                    ui.separator();
                    ui.small(format!("✅ Recently added: {}", self.recently_added.len()));
                }
            });

            ui.separator();

            // Attributes list
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    if self.filtered_attributes.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            if self.search_filter.is_empty() && self.selected_category.is_none() {
                                ui.label("No attributes available");
                            } else {
                                ui.label("No matching attributes found");
                                ui.small("Try adjusting your search or category filter");
                            }
                            ui.add_space(20.0);
                        });
                        return;
                    }

                    // Clone filtered attributes to avoid borrow checker issues
                    let filtered_attributes = self.filtered_attributes.clone();
                    for attr in &filtered_attributes {
                        let already_in_template = self.is_attribute_in_template(&attr.name, template_attributes);
                        let recently_added = self.recently_added.contains(&attr.name);

                        ui.group(|ui| {
                            if self.compact_mode {
                                // Compact view
                                ui.horizontal(|ui| {
                                    ui.label(&attr.name);
                                    ui.small(format!("({})", attr.data_type));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if already_in_template {
                                            ui.small("✅ Added");
                                        } else if recently_added {
                                            ui.colored_label(egui::Color32::GREEN, "✅ Just added");
                                        } else {
                                            if ui.button("➕ Add").clicked() {
                                                added_attribute = Some(self.to_template_attribute(attr));
                                                self.mark_recently_added(attr.name.clone());
                                            }
                                        }
                                    });
                                });
                            } else {
                                // Full view
                                ui.vertical(|ui| {
                                    // Header with name and type
                                    ui.horizontal(|ui| {
                                        ui.strong(&attr.name);
                                        ui.small(format!("({})", attr.data_type));

                                        // Category badge
                                        let category = self.categories.iter()
                                            .find(|c| c.name == attr.category);
                                        if let Some(cat) = category {
                                            ui.small(format!("{} {}", cat.icon, cat.name));
                                        }

                                        // Required indicator
                                        if attr.required {
                                            ui.small("*");
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if already_in_template {
                                                ui.small("✅ Already in template");
                                            } else if recently_added {
                                                ui.colored_label(egui::Color32::GREEN, "✅ Just added");
                                            } else {
                                                if ui.button("➕ Add to Template").clicked() {
                                                    added_attribute = Some(self.to_template_attribute(attr));
                                                    self.mark_recently_added(attr.name.clone());
                                                }
                                            }
                                        });
                                    });

                                    // Description
                                    ui.small(&attr.description);

                                    // Allowed values if present
                                    if let Some(ref values) = attr.allowed_values {
                                        ui.horizontal(|ui| {
                                            ui.small("Values:");
                                            ui.small(values.join(", "));
                                        });
                                    }

                                    // UI help text if present
                                    if let Some(ref help_text) = attr.ui.help_text {
                                        ui.small(format!("💡 {}", help_text));
                                    }
                                });
                            }
                        });

                        ui.add_space(3.0);
                    }
                });
        });

        added_attribute
    }

    /// Set visibility of the palette
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the number of filtered attributes
    pub fn filtered_count(&self) -> usize {
        self.filtered_attributes.len()
    }

    /// Get the total number of master attributes
    pub fn total_count(&self) -> usize {
        self.master_attributes.len()
    }
}

impl Default for AttributePalette {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_api_client::TemplateAttribute;
    use std::collections::HashMap;

    #[test]
    fn test_attribute_palette_creation() {
        let palette = AttributePalette::new();

        assert!(!palette.master_attributes.is_empty(), "Should have master attributes loaded");
        assert!(!palette.categories.is_empty(), "Should have categories loaded");
        assert!(palette.visible, "Should be visible by default");
        assert!(!palette.loading, "Should not be loading after creation");
        assert!(palette.error_message.is_none(), "Should not have error messages");
    }

    #[test]
    fn test_attribute_palette_filtering() {
        let mut palette = AttributePalette::new();

        // Test initial state - should have all attributes
        let initial_count = palette.filtered_attributes.len();
        assert!(initial_count > 0, "Should have filtered attributes initially");
        assert_eq!(initial_count, palette.master_attributes.len(), "Initially should show all attributes");

        // Test category filtering
        palette.selected_category = Some("Identity".to_string());
        palette.update_filtered_attributes();

        let identity_count = palette.filtered_attributes.len();
        assert!(identity_count > 0, "Should have identity attributes");
        assert!(identity_count < initial_count, "Should filter down to identity only");

        // Verify all filtered attributes are from Identity category
        for attr in &palette.filtered_attributes {
            assert_eq!(attr.category, "Identity", "All filtered attributes should be from Identity category");
        }

        // Test search filtering
        palette.selected_category = None; // Clear category filter
        palette.search_filter = "case".to_string();
        palette.update_filtered_attributes();

        let search_count = palette.filtered_attributes.len();
        assert!(search_count > 0, "Should find attributes containing 'case'");
        assert!(search_count < initial_count, "Should filter down for search");

        // Verify all filtered attributes contain the search term
        for attr in &palette.filtered_attributes {
            let contains_search = attr.name.to_lowercase().contains("case") ||
                                attr.description.to_lowercase().contains("case") ||
                                attr.category.to_lowercase().contains("case") ||
                                attr.ui.label.to_lowercase().contains("case");
            assert!(contains_search, "Attribute {} should contain search term 'case'", attr.name);
        }
    }

    #[test]
    fn test_attribute_in_template_detection() {
        let palette = AttributePalette::new();

        // Create a template with some attributes
        let template_attributes = vec![
            TemplateAttribute {
                name: "case_id".to_string(),
                data_type: "String".to_string(),
                allowed_values: None,
                ui: HashMap::new(),
            },
            TemplateAttribute {
                name: "status".to_string(),
                data_type: "String".to_string(),
                allowed_values: Some(vec!["Pending".to_string(), "Complete".to_string()]),
                ui: HashMap::new(),
            },
        ];

        // Test detection
        assert!(palette.is_attribute_in_template("case_id", &template_attributes), "Should detect case_id in template");
        assert!(palette.is_attribute_in_template("status", &template_attributes), "Should detect status in template");
        assert!(!palette.is_attribute_in_template("email", &template_attributes), "Should not detect email in template");
        assert!(!palette.is_attribute_in_template("nonexistent", &template_attributes), "Should not detect nonexistent attribute");
    }

    #[test]
    fn test_master_to_template_attribute_conversion() {
        let palette = AttributePalette::new();

        // Find a known master attribute
        let case_id_attr = palette.master_attributes.iter()
            .find(|attr| attr.name == "case_id")
            .expect("Should have case_id master attribute");

        // Convert to template attribute
        let template_attr = palette.to_template_attribute(case_id_attr);

        // Verify conversion
        assert_eq!(template_attr.name, case_id_attr.name);
        assert_eq!(template_attr.data_type, case_id_attr.data_type);
        assert_eq!(template_attr.allowed_values, case_id_attr.allowed_values);

        // Check UI metadata conversion
        assert!(template_attr.ui.contains_key("label"));
        assert!(template_attr.ui.contains_key("required"));

        if let Some(serde_json::Value::String(label)) = template_attr.ui.get("label") {
            assert_eq!(label, &case_id_attr.ui.label);
        } else {
            panic!("Label should be a string");
        }

        if let Some(serde_json::Value::Bool(required)) = template_attr.ui.get("required") {
            assert_eq!(*required, case_id_attr.required);
        } else {
            panic!("Required should be a boolean");
        }
    }

    #[test]
    fn test_recently_added_tracking() {
        let mut palette = AttributePalette::new();

        assert!(palette.recently_added.is_empty(), "Should start with empty recently added list");

        // Add some attributes
        palette.mark_recently_added("case_id".to_string());
        palette.mark_recently_added("email".to_string());

        assert_eq!(palette.recently_added.len(), 2, "Should have 2 recently added attributes");
        assert!(palette.recently_added.contains(&"case_id".to_string()), "Should contain case_id");
        assert!(palette.recently_added.contains(&"email".to_string()), "Should contain email");

        // Test overflow (should keep only last 5)
        for i in 1..=10 {
            palette.mark_recently_added(format!("attr_{}", i));
        }

        assert_eq!(palette.recently_added.len(), 5, "Should keep only last 5 recently added");
        assert!(palette.recently_added.contains(&"attr_10".to_string()), "Should contain the last added");
        assert!(palette.recently_added.contains(&"attr_6".to_string()), "Should contain attr_6 (5th from end)");
        assert!(!palette.recently_added.contains(&"case_id".to_string()), "Should not contain early additions");

        // Test clearing
        palette.clear_recently_added();
        assert!(palette.recently_added.is_empty(), "Should be empty after clearing");
    }

    #[test]
    fn test_visibility_controls() {
        let mut palette = AttributePalette::new();

        assert!(palette.is_visible(), "Should be visible by default");

        palette.set_visible(false);
        assert!(!palette.is_visible(), "Should be hidden after setting to false");

        palette.set_visible(true);
        assert!(palette.is_visible(), "Should be visible after setting to true");
    }

    #[test]
    fn test_count_methods() {
        let mut palette = AttributePalette::new();

        let total = palette.total_count();
        assert!(total > 0, "Should have total attributes");

        let filtered = palette.filtered_count();
        assert_eq!(filtered, total, "Initially filtered should equal total");

        // Apply a filter
        palette.search_filter = "xyz_nonexistent".to_string();
        palette.update_filtered_attributes();

        assert_eq!(palette.filtered_count(), 0, "Should have no matches for nonexistent search");
        assert_eq!(palette.total_count(), total, "Total should remain unchanged");
    }

    #[test]
    fn test_category_structure() {
        let palette = AttributePalette::new();

        // Verify we have the expected categories
        let expected_categories = vec![
            "Identity", "Contact", "Financial", "Regulatory",
            "Risk", "Workflow", "Documents", "Audit"
        ];

        for expected in &expected_categories {
            let found = palette.categories.iter().any(|cat| &cat.name == expected);
            assert!(found, "Should have category: {}", expected);
        }

        // Verify each category has proper structure
        for category in &palette.categories {
            assert!(!category.name.is_empty(), "Category name should not be empty");
            assert!(!category.description.is_empty(), "Category description should not be empty");
            assert!(!category.icon.is_empty(), "Category icon should not be empty");
            assert!(!category.color.is_empty(), "Category color should not be empty");
            assert!(category.color.starts_with('#'), "Category color should be a hex color");
        }
    }
}