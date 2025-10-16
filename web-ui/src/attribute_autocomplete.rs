use eframe::egui;

/// Represents an attribute from the data dictionary
#[derive(Debug, Clone)]
pub struct DictionaryAttribute {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub allowed_values: Option<Vec<String>>,
    pub category: String,
}

/// Autocomplete widget for attribute selection
#[derive(Debug, Clone)]
pub struct AttributeAutocomplete {
    /// Input text for filtering
    pub input_text: String,
    /// Available attributes from dictionary
    pub available_attributes: Vec<DictionaryAttribute>,
    /// Filtered attributes based on input
    pub filtered_attributes: Vec<DictionaryAttribute>,
    /// Whether the dropdown is currently shown
    pub show_dropdown: bool,
    /// Currently selected index in the dropdown
    pub selected_index: Option<usize>,
    /// ID for this autocomplete widget (for unique identification)
    pub widget_id: String,
}

impl AttributeAutocomplete {
    pub fn new(widget_id: &str) -> Self {
        let available_attributes = Self::load_dictionary_attributes();
        let filtered_attributes = available_attributes.clone();

        Self {
            input_text: String::new(),
            available_attributes,
            filtered_attributes,
            show_dropdown: false,
            selected_index: None,
            widget_id: widget_id.to_string(),
        }
    }

    /// Load predefined attributes from the data dictionary
    fn load_dictionary_attributes() -> Vec<DictionaryAttribute> {
        vec![
            DictionaryAttribute {
                name: "full_name".to_string(),
                data_type: "String".to_string(),
                description: "Complete legal name as it appears on official documents".to_string(),
                allowed_values: None,
                category: "Identity".to_string(),
            },
            DictionaryAttribute {
                name: "date_of_birth".to_string(),
                data_type: "Date".to_string(),
                description: "Customer date of birth for age verification and KYC".to_string(),
                allowed_values: None,
                category: "Identity".to_string(),
            },
            DictionaryAttribute {
                name: "email".to_string(),
                data_type: "Email".to_string(),
                description: "Primary email address for customer communication".to_string(),
                allowed_values: None,
                category: "Contact".to_string(),
            },
            DictionaryAttribute {
                name: "phone_number".to_string(),
                data_type: "PhoneNumber".to_string(),
                description: "Primary contact phone number with country code".to_string(),
                allowed_values: None,
                category: "Contact".to_string(),
            },
            DictionaryAttribute {
                name: "address".to_string(),
                data_type: "Address".to_string(),
                description: "Primary residential or business address".to_string(),
                allowed_values: None,
                category: "Contact".to_string(),
            },
            DictionaryAttribute {
                name: "document_type".to_string(),
                data_type: "Enum".to_string(),
                description: "Type of identification document provided".to_string(),
                allowed_values: Some(vec![
                    "passport".to_string(),
                    "driver_license".to_string(),
                    "national_id".to_string(),
                    "birth_certificate".to_string(),
                ]),
                category: "Documents".to_string(),
            },
            DictionaryAttribute {
                name: "risk_score".to_string(),
                data_type: "Integer".to_string(),
                description: "Calculated risk score from 0-100 based on KYC assessment".to_string(),
                allowed_values: None,
                category: "Risk".to_string(),
            },
            DictionaryAttribute {
                name: "customer_email".to_string(),
                data_type: "Email".to_string(),
                description: "Alternative email address for customer notifications".to_string(),
                allowed_values: None,
                category: "Contact".to_string(),
            },
        ]
    }

    /// Filter attributes based on current input
    fn update_filtered_attributes(&mut self) {
        if self.input_text.is_empty() {
            self.filtered_attributes = self.available_attributes.clone();
        } else {
            let input_lower = self.input_text.to_lowercase();
            self.filtered_attributes = self.available_attributes
                .iter()
                .filter(|attr| {
                    attr.name.to_lowercase().contains(&input_lower) ||
                    attr.description.to_lowercase().contains(&input_lower) ||
                    attr.category.to_lowercase().contains(&input_lower)
                })
                .cloned()
                .collect();
        }

        // Reset selection when filters change
        self.selected_index = if self.filtered_attributes.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Handle keyboard navigation
    fn handle_key_input(&mut self, ui: &egui::Ui) -> Option<DictionaryAttribute> {
        let mut selected_attribute = None;

        ui.input(|i| {
            // Handle Enter key to select current item
            if i.key_pressed(egui::Key::Enter) {
                if let Some(index) = self.selected_index {
                    if index < self.filtered_attributes.len() {
                        selected_attribute = Some(self.filtered_attributes[index].clone());
                        self.input_text = self.filtered_attributes[index].name.clone();
                        self.show_dropdown = false;
                    }
                }
            }

            // Handle Tab key to autocomplete
            if i.key_pressed(egui::Key::Tab) {
                if let Some(index) = self.selected_index {
                    if index < self.filtered_attributes.len() {
                        self.input_text = self.filtered_attributes[index].name.clone();
                        self.show_dropdown = false;
                    }
                }
            }

            // Handle Arrow Up
            if i.key_pressed(egui::Key::ArrowUp) && self.show_dropdown {
                if let Some(current) = self.selected_index {
                    if current > 0 {
                        self.selected_index = Some(current - 1);
                    } else {
                        self.selected_index = Some(self.filtered_attributes.len().saturating_sub(1));
                    }
                }
            }

            // Handle Arrow Down
            if i.key_pressed(egui::Key::ArrowDown) && self.show_dropdown {
                if let Some(current) = self.selected_index {
                    if current < self.filtered_attributes.len() - 1 {
                        self.selected_index = Some(current + 1);
                    } else {
                        self.selected_index = Some(0);
                    }
                } else if !self.filtered_attributes.is_empty() {
                    self.selected_index = Some(0);
                }
            }

            // Handle Escape to close dropdown
            if i.key_pressed(egui::Key::Escape) {
                self.show_dropdown = false;
            }
        });

        selected_attribute
    }

    /// Main rendering method for the autocomplete widget
    pub fn show(&mut self, ui: &mut egui::Ui, label: &str) -> Option<DictionaryAttribute> {
        let mut selected_attribute = None;

        ui.vertical(|ui| {
            ui.label(label);

            // Text input with autocomplete
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.input_text)
                    .hint_text("Start typing attribute name...")
                    .desired_width(f32::INFINITY)
            );

            // Handle input changes
            if response.changed() {
                self.update_filtered_attributes();
                self.show_dropdown = !self.input_text.is_empty() && !self.filtered_attributes.is_empty();
            }

            // Show dropdown when input is focused
            if response.gained_focus() {
                self.show_dropdown = !self.filtered_attributes.is_empty();
            }

            // Handle keyboard input
            if let Some(attr) = self.handle_key_input(ui) {
                selected_attribute = Some(attr);
            }

            // Show dropdown with filtered results
            if self.show_dropdown && !self.filtered_attributes.is_empty() {
                ui.add_space(2.0);

                egui::Frame::popup(ui.style())
                    .shadow(egui::epaint::Shadow::default())
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (index, attr) in self.filtered_attributes.iter().enumerate() {
                                    let is_selected = Some(index) == self.selected_index;

                                    // Style the selected item differently
                                    let (bg_color, text_color) = if is_selected {
                                        (ui.visuals().selection.bg_fill, ui.visuals().selection.stroke.color)
                                    } else {
                                        (egui::Color32::TRANSPARENT, ui.visuals().text_color())
                                    };

                                    let item_response = ui.allocate_response(
                                        egui::vec2(ui.available_width(), 40.0),
                                        egui::Sense::click()
                                    );

                                    // Draw background for selected item
                                    if is_selected {
                                        ui.painter().rect_filled(
                                            item_response.rect,
                                            2.0,
                                            bg_color
                                        );
                                    }

                                    // Draw the attribute item
                                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(item_response.rect), |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add_space(8.0);
                                            ui.vertical(|ui| {
                                                ui.add_space(4.0);

                                                // Attribute name
                                                ui.horizontal(|ui| {
                                                    ui.colored_label(text_color, &attr.name);
                                                    ui.small(format!("({})", attr.data_type));

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.small(&attr.category);
                                                    });
                                                });

                                                // Description
                                                ui.small(&attr.description);

                                                ui.add_space(4.0);
                                            });
                                        });
                                    });

                                    // Handle clicks
                                    if item_response.clicked() {
                                        selected_attribute = Some(attr.clone());
                                        self.input_text = attr.name.clone();
                                        self.show_dropdown = false;
                                    }

                                    // Handle hover to update selection
                                    if item_response.hovered() {
                                        self.selected_index = Some(index);
                                    }
                                }
                            });
                    });
            }

            // Hint text
            if !self.input_text.is_empty() && self.filtered_attributes.is_empty() {
                ui.small("No matching attributes found");
            } else if !self.show_dropdown && !self.input_text.is_empty() {
                ui.small("Press Enter to confirm, Tab to autocomplete, or continue typing");
            }
        });

        selected_attribute
    }

    /// Get the currently selected attribute name
    pub fn get_value(&self) -> &str {
        &self.input_text
    }

    /// Set the input value
    pub fn set_value(&mut self, value: &str) {
        self.input_text = value.to_string();
        self.update_filtered_attributes();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.input_text.clear();
        self.show_dropdown = false;
        self.update_filtered_attributes();
    }
}

impl Default for AttributeAutocomplete {
    fn default() -> Self {
        Self::new("default")
    }
}