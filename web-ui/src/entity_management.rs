use eframe::egui;
use crate::grpc_client::GrpcClient;

/// Entity Management UI Components for CRUD operations
pub struct EntityManagementUI {
    // CBU Management
    cbu_list: Vec<CbuRecord>,
    selected_cbu: Option<CbuRecord>,
    new_cbu_form: CbuForm,
    show_cbu_form: bool,

    // Product Management
    product_list: Vec<ProductRecord>,
    selected_product: Option<ProductRecord>,
    new_product_form: ProductForm,
    show_product_form: bool,

    // Service Management
    service_list: Vec<ServiceRecord>,
    selected_service: Option<ServiceRecord>,
    new_service_form: ServiceForm,
    show_service_form: bool,

    // Resource Management
    resource_list: Vec<ResourceRecord>,
    selected_resource: Option<ResourceRecord>,
    new_resource_form: ResourceForm,
    show_resource_form: bool,

    // UI State
    loading: bool,
    error_message: Option<String>,
    success_message: Option<String>,

    // Picker Windows State
    show_product_picker: bool,
    show_service_picker: bool,
    product_picker_filter: String,
    service_picker_filter: String,
}

// Data structures for entity records
#[derive(Debug, Clone)]
pub struct CbuRecord {
    pub id: i32,
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: Option<String>,
    pub legal_entity_name: Option<String>,
    pub jurisdiction: Option<String>,
    pub business_model: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductRecord {
    pub id: i32,
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: Option<String>,
    pub contract_type: Option<String>,
    pub commercial_status: Option<String>,
    pub pricing_model: Option<String>,
    pub target_market: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ServiceRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub service_type: String,
    pub service_category: String,
    pub delivery_model: String,
    pub billable: bool,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ResourceRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource_type: String,
    pub location: Option<String>,
    pub capabilities: String,
    pub status: String,
    pub visibility: String,
}

// Form structures for creating/editing entities
#[derive(Debug, Clone, Default)]
pub struct CbuForm {
    pub cbu_id: String,
    pub cbu_name: String,
    pub description: String,
    pub legal_entity_name: String,
    pub jurisdiction: String,
    pub business_model: String,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProductForm {
    pub product_id: String,
    pub product_name: String,
    pub line_of_business: String,
    pub description: String,
    pub contract_type: String,
    pub commercial_status: String,
    pub pricing_model: String,
    pub target_market: String,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceForm {
    pub service_id: String,
    pub service_name: String,
    pub service_category: String,
    pub description: String,
    pub service_type: String,
    pub delivery_model: String,
    pub billable: bool,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceForm {
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
    pub description: String,
    pub location: String,
    pub capabilities: String,
    pub status: String,
    pub visibility: String,
}

impl Default for EntityManagementUI {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityManagementUI {
    pub fn new() -> Self {
        Self {
            cbu_list: Vec::new(),
            selected_cbu: None,
            new_cbu_form: CbuForm::default(),
            show_cbu_form: false,

            product_list: Vec::new(),
            selected_product: None,
            new_product_form: ProductForm::default(),
            show_product_form: false,

            service_list: Vec::new(),
            selected_service: None,
            new_service_form: ServiceForm::default(),
            show_service_form: false,

            resource_list: Vec::new(),
            selected_resource: None,
            new_resource_form: ResourceForm::default(),
            show_resource_form: false,

            loading: false,
            error_message: None,
            success_message: None,

            // Picker Windows State
            show_product_picker: false,
            show_service_picker: false,
            product_picker_filter: String::new(),
            service_picker_filter: String::new(),
        }
    }

    /// Render CBU Management UI
    pub fn show_cbu_management(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.heading("🏢 CBU Management");
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ New CBU").clicked() {
                self.show_cbu_form = true;
                // self.new_cbu_form = CbuForm::default(); // REMOVED: default action - form should retain state
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_cbu_list(grpc_client);
            }
        });

        ui.separator();

        // Status messages
        self.show_status_messages(ui);

        // CBU List
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.cbu_list.is_empty() && !self.loading {
                ui.label("No CBUs found. Click 'New CBU' to create one.");
            } else {
                for cbu in &self.cbu_list.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&cbu.cbu_name);
                                ui.label(format!("ID: {}", cbu.cbu_id));
                                if let Some(desc) = &cbu.description {
                                    ui.label(format!("Description: {}", desc));
                                }
                                ui.label(format!("Status: {}", cbu.status));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️ Delete").clicked() {
                                    self.delete_cbu(&cbu.cbu_id, grpc_client);
                                }
                                if ui.button("✏️ Edit").clicked() {
                                    self.edit_cbu(cbu);
                                }
                                if ui.button("👁️ View").clicked() {
                                    self.selected_cbu = Some(cbu.clone());
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }
        });

        // CBU Form Modal
        if self.show_cbu_form {
            self.show_cbu_form_modal(ui, grpc_client);
        }

        // CBU Details Panel
        if let Some(cbu) = self.selected_cbu.clone() {
            self.show_cbu_details(ui, &cbu);
        }
    }

    /// Render Product Management UI
    pub fn show_product_management(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        // Check for async product loading results
        self.check_async_product_results();

        ui.heading("📦 Product Management");
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ New Product").clicked() {
                self.show_product_form = true;
                // self.new_product_form = ProductForm::default(); // REMOVED: default action - form should retain state
            }

            if ui.button("🔍 Pick Product").clicked() {
                self.show_product_picker = true;
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_product_list(grpc_client);
            }
        });

        ui.separator();

        // Status messages
        self.show_status_messages(ui);

        // Product List
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.product_list.is_empty() && !self.loading {
                ui.label("No products found. Click 'New Product' to create one.");
            } else {
                for product in &self.product_list.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&product.product_name);
                                ui.label(format!("ID: {}", product.product_id));
                                ui.label(format!("Line of Business: {}", product.line_of_business));
                                if let Some(desc) = &product.description {
                                    ui.label(format!("Description: {}", desc));
                                }
                                ui.label(format!("Status: {}", product.status));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️ Delete").clicked() {
                                    self.delete_product(&product.product_id, grpc_client);
                                }
                                if ui.button("✏️ Edit").clicked() {
                                    self.edit_product(product);
                                }
                                if ui.button("👁️ View").clicked() {
                                    self.selected_product = Some(product.clone());
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }
        });

        // Product Form Modal
        if self.show_product_form {
            self.show_product_form_modal(ui, grpc_client);
        }

        // Product Details Panel
        if let Some(product) = self.selected_product.clone() {
            self.show_product_details(ui, &product);
        }

        // Product Picker Window (called outside UI constraints for proper resizing)
        if self.show_product_picker {
            let ctx = ui.ctx().clone();
            self.render_product_picker(&ctx, grpc_client);
        }
    }

    /// Render Service Management UI
    pub fn show_service_management(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.heading("⚙️ Service Management");
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ New Service").clicked() {
                self.show_service_form = true;
                // self.new_service_form = ServiceForm::default(); // REMOVED: default action - form should retain state
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_service_list(grpc_client);
            }
        });

        ui.separator();

        // Status messages
        self.show_status_messages(ui);

        // Service List
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.service_list.is_empty() && !self.loading {
                ui.label("No services found. Click 'New Service' to create one.");
            } else {
                for service in &self.service_list.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&service.name);
                                ui.label(format!("ID: {}", service.id));
                                ui.label(format!("Type: {}", service.service_type));
                                ui.label(format!("Category: {}", service.service_category));
                                ui.label(format!("Billable: {}", service.billable));
                                ui.label(format!("Status: {}", service.status));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️ Delete").clicked() {
                                    self.delete_service(&service.id, grpc_client);
                                }
                                if ui.button("✏️ Edit").clicked() {
                                    self.edit_service(service);
                                }
                                if ui.button("👁️ View").clicked() {
                                    self.selected_service = Some(service.clone());
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }
        });

        // Service Form Modal
        if self.show_service_form {
            self.show_service_form_modal(ui, grpc_client);
        }

        // Service Details Panel
        if let Some(service) = self.selected_service.clone() {
            self.show_service_details(ui, &service);
        }
    }

    /// Render Resource Management UI
    pub fn show_resource_management(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        ui.heading("🔧 Resource Management");
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ New Resource").clicked() {
                self.show_resource_form = true;
                // self.new_resource_form = ResourceForm::default(); // REMOVED: default action - form should retain state
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_resource_list(grpc_client);
            }
        });

        ui.separator();

        // Status messages
        self.show_status_messages(ui);

        // Resource List
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.resource_list.is_empty() && !self.loading {
                ui.label("No resources found. Click 'New Resource' to create one.");
            } else {
                for resource in &self.resource_list.clone() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&resource.name);
                                ui.label(format!("ID: {}", resource.id));
                                ui.label(format!("Type: {}", resource.resource_type));
                                ui.label(format!("Visibility: {}", resource.visibility));
                                ui.label(format!("Status: {}", resource.status));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️ Delete").clicked() {
                                    self.delete_resource(&resource.id, grpc_client);
                                }
                                if ui.button("✏️ Edit").clicked() {
                                    self.edit_resource(resource);
                                }
                                if ui.button("👁️ View").clicked() {
                                    self.selected_resource = Some(resource.clone());
                                }
                            });
                        });
                    });
                    ui.separator();
                }
            }
        });

        // Resource Form Modal
        if self.show_resource_form {
            self.show_resource_form_modal(ui, grpc_client);
        }

        // Resource Details Panel
        if let Some(resource) = self.selected_resource.clone() {
            self.show_resource_details(ui, &resource);
        }
    }

    // Helper methods for UI components
    fn show_status_messages(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
        }

        if let Some(success) = &self.success_message {
            ui.colored_label(egui::Color32::GREEN, format!("✅ {}", success));
        }

        if self.loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading...");
            });
        }
    }

    // CBU-specific methods
    fn show_cbu_form_modal(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        egui::Window::new("CBU Form")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("CBU ID:");
                    ui.text_edit_singleline(&mut self.new_cbu_form.cbu_id);

                    ui.label("CBU Name:");
                    ui.text_edit_singleline(&mut self.new_cbu_form.cbu_name);

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_cbu_form.description);

                    ui.label("Legal Entity Name:");
                    ui.text_edit_singleline(&mut self.new_cbu_form.legal_entity_name);

                    ui.label("Jurisdiction:");
                    ui.text_edit_singleline(&mut self.new_cbu_form.jurisdiction);

                    ui.label("Business Model:");
                    ui.text_edit_singleline(&mut self.new_cbu_form.business_model);

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_cbu_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_cbu_form.status, "active".to_string(), "Active");
                            ui.selectable_value(&mut self.new_cbu_form.status, "inactive".to_string(), "Inactive");
                            ui.selectable_value(&mut self.new_cbu_form.status, "pending".to_string(), "Pending");
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_cbu(grpc_client);
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_cbu_form = false;
                        }
                    });
                });
            });
    }

    fn show_cbu_details(&mut self, ui: &mut egui::Ui, cbu: &CbuRecord) {
        egui::Window::new("CBU Details")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.heading(&cbu.cbu_name);
                    ui.separator();

                    ui.label(format!("ID: {}", cbu.cbu_id));
                    if let Some(desc) = &cbu.description {
                        ui.label(format!("Description: {}", desc));
                    }
                    if let Some(entity) = &cbu.legal_entity_name {
                        ui.label(format!("Legal Entity: {}", entity));
                    }
                    if let Some(jurisdiction) = &cbu.jurisdiction {
                        ui.label(format!("Jurisdiction: {}", jurisdiction));
                    }
                    if let Some(model) = &cbu.business_model {
                        ui.label(format!("Business Model: {}", model));
                    }
                    ui.label(format!("Status: {}", cbu.status));

                    ui.separator();

                    if ui.button("✏️ Edit").clicked() {
                        self.edit_cbu(cbu);
                    }

                    if ui.button("❌ Close").clicked() {
                        self.selected_cbu = None;
                    }
                });
            });
    }

    // Product-specific methods
    fn show_product_form_modal(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        egui::Window::new("Product Form")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Product ID:");
                    ui.text_edit_singleline(&mut self.new_product_form.product_id);

                    ui.label("Product Name:");
                    ui.text_edit_singleline(&mut self.new_product_form.product_name);

                    ui.label("Line of Business:");
                    ui.text_edit_singleline(&mut self.new_product_form.line_of_business);

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_product_form.description);

                    ui.label("Contract Type:");
                    ui.text_edit_singleline(&mut self.new_product_form.contract_type);

                    ui.label("Commercial Status:");
                    ui.text_edit_singleline(&mut self.new_product_form.commercial_status);

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_product_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_product_form.status, "active".to_string(), "Active");
                            ui.selectable_value(&mut self.new_product_form.status, "inactive".to_string(), "Inactive");
                            ui.selectable_value(&mut self.new_product_form.status, "pending".to_string(), "Pending");
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_product(grpc_client);
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_product_form = false;
                        }
                    });
                });
            });
    }

    fn show_product_details(&mut self, ui: &mut egui::Ui, product: &ProductRecord) {
        egui::Window::new("Product Details")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.heading(&product.product_name);
                    ui.separator();

                    ui.label(format!("ID: {}", product.product_id));
                    ui.label(format!("Line of Business: {}", product.line_of_business));
                    if let Some(desc) = &product.description {
                        ui.label(format!("Description: {}", desc));
                    }
                    if let Some(contract) = &product.contract_type {
                        ui.label(format!("Contract Type: {}", contract));
                    }
                    ui.label(format!("Status: {}", product.status));

                    ui.separator();

                    if ui.button("✏️ Edit").clicked() {
                        self.edit_product(product);
                    }

                    if ui.button("❌ Close").clicked() {
                        self.selected_product = None;
                    }
                });
            });
    }

    // Service-specific methods
    fn show_service_form_modal(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        egui::Window::new("Service Form")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Service ID:");
                    ui.text_edit_singleline(&mut self.new_service_form.service_id);

                    ui.label("Service Name:");
                    ui.text_edit_singleline(&mut self.new_service_form.service_name);

                    ui.label("Service Category:");
                    ui.text_edit_singleline(&mut self.new_service_form.service_category);

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_service_form.description);

                    ui.label("Service Type:");
                    ui.text_edit_singleline(&mut self.new_service_form.service_type);

                    ui.label("Delivery Model:");
                    ui.text_edit_singleline(&mut self.new_service_form.delivery_model);

                    ui.checkbox(&mut self.new_service_form.billable, "Billable");

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_service_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_service_form.status, "active".to_string(), "Active");
                            ui.selectable_value(&mut self.new_service_form.status, "inactive".to_string(), "Inactive");
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_service(grpc_client);
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_service_form = false;
                        }
                    });
                });
            });
    }

    fn show_service_details(&mut self, ui: &mut egui::Ui, service: &ServiceRecord) {
        egui::Window::new("Service Details")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.heading(&service.name);
                    ui.separator();

                    ui.label(format!("ID: {}", service.id));
                    ui.label(format!("Type: {}", service.service_type));
                    ui.label(format!("Category: {}", service.service_category));
                    ui.label(format!("Description: {}", service.description));
                    ui.label(format!("Delivery Model: {}", service.delivery_model));
                    ui.label(format!("Billable: {}", service.billable));
                    ui.label(format!("Status: {}", service.status));

                    ui.separator();

                    if ui.button("✏️ Edit").clicked() {
                        self.edit_service(service);
                    }

                    if ui.button("❌ Close").clicked() {
                        self.selected_service = None;
                    }
                });
            });
    }

    // Resource-specific methods
    fn show_resource_form_modal(&mut self, ui: &mut egui::Ui, grpc_client: Option<&GrpcClient>) {
        egui::Window::new("Resource Form")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Resource ID:");
                    ui.text_edit_singleline(&mut self.new_resource_form.resource_id);

                    ui.label("Resource Name:");
                    ui.text_edit_singleline(&mut self.new_resource_form.resource_name);

                    ui.label("Resource Type:");
                    ui.text_edit_singleline(&mut self.new_resource_form.resource_type);

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_resource_form.description);

                    ui.label("Location:");
                    ui.text_edit_singleline(&mut self.new_resource_form.location);

                    ui.label("Capabilities (JSON):");
                    ui.text_edit_multiline(&mut self.new_resource_form.capabilities);

                    ui.label("Visibility:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_resource_form.visibility)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_resource_form.visibility, "private".to_string(), "Private");
                            ui.selectable_value(&mut self.new_resource_form.visibility, "public".to_string(), "Public");
                        });

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.new_resource_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_resource_form.status, "active".to_string(), "Active");
                            ui.selectable_value(&mut self.new_resource_form.status, "inactive".to_string(), "Inactive");
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            self.save_resource(grpc_client);
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.show_resource_form = false;
                        }
                    });
                });
            });
    }

    fn show_resource_details(&mut self, ui: &mut egui::Ui, resource: &ResourceRecord) {
        egui::Window::new("Resource Details")
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.heading(&resource.name);
                    ui.separator();

                    ui.label(format!("ID: {}", resource.id));
                    ui.label(format!("Type: {}", resource.resource_type));
                    ui.label(format!("Description: {}", resource.description));
                    if let Some(location) = &resource.location {
                        ui.label(format!("Location: {}", location));
                    }
                    ui.label(format!("Capabilities: {}", resource.capabilities));
                    ui.label(format!("Visibility: {}", resource.visibility));
                    ui.label(format!("Status: {}", resource.status));

                    ui.separator();

                    if ui.button("✏️ Edit").clicked() {
                        self.edit_resource(resource);
                    }

                    if ui.button("❌ Close").clicked() {
                        self.selected_resource = None;
                    }
                });
            });
    }

    // Data loading methods (placeholder implementations)
    fn load_cbu_list(&mut self, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to load CBU list
        self.loading = true;
        // For now, load sample data
        self.cbu_list = vec![
            CbuRecord {
                id: 1,
                cbu_id: "CBU001".to_string(),
                cbu_name: "Sample CBU".to_string(),
                description: Some("Sample CBU for testing".to_string()),
                legal_entity_name: Some("Sample Entity LLC".to_string()),
                jurisdiction: Some("US".to_string()),
                business_model: Some("Investment Management".to_string()),
                status: "active".to_string(),
            }
        ];
        self.loading = false;
    }

    fn load_product_list(&mut self, grpc_client: Option<&GrpcClient>) {
        self.loading = true;
        self.error_message = None;

        if let Some(client) = grpc_client {
            let request = crate::grpc_client::ListProductsRequest {
                status_filter: Some("active".to_string()),
                limit: Some(100),
                offset: None,
            };

            // Store the client for async operation
            let client_clone = client.clone();

            // Use spawn_local for WASM async operation
            wasm_bindgen_futures::spawn_local(async move {
                match client_clone.list_products(request).await {
                    Ok(response) => {
                        crate::wasm_utils::console_log(&format!("✅ Retrieved {} products from gRPC", response.products.len()));

                        // Convert ProductDetails to ProductRecord for UI
                        let ui_products: Vec<ProductRecord> = response.products.into_iter().enumerate()
                            .map(|(index, product)| ProductRecord {
                                id: index as i32 + 1, // Generate sequential ID for UI
                                product_id: product.product_id,
                                product_name: product.product_name,
                                line_of_business: product.line_of_business,
                                description: if product.description.is_empty() { None } else { Some(product.description) },
                                contract_type: if product.contract_type.is_empty() { None } else { Some(product.contract_type) },
                                commercial_status: if product.commercial_status.is_empty() { None } else { Some(product.commercial_status) },
                                pricing_model: if product.pricing_model.is_empty() { None } else { Some(product.pricing_model) },
                                target_market: if product.target_market.is_empty() { None } else { Some(product.target_market) },
                                status: product.status,
                            })
                            .collect();

                        crate::wasm_utils::console_log(&format!("📦 Converted {} products for UI display", ui_products.len()));

                        // Store in localStorage as JSON to communicate with UI
                        if let Ok(json) = serde_json::to_string(&ui_products) {
                            let window = web_sys::window().unwrap();
                            let storage = window.local_storage().unwrap().unwrap();
                            if let Err(e) = storage.set_item("data_designer_products", &json) {
                                crate::wasm_utils::console_log(&format!("❌ Failed to store products in localStorage: {:?}", e));
                            } else {
                                crate::wasm_utils::console_log("💾 Products stored in localStorage for UI update");
                            }
                        }
                    }
                    Err(e) => {
                        crate::wasm_utils::console_log(&format!("❌ Error loading products: {:?}", e));

                        // Store error in localStorage
                        let window = web_sys::window().unwrap();
                        let storage = window.local_storage().unwrap().unwrap();
                        let _ = storage.set_item("data_designer_products_error", &format!("{:?}", e));
                    }
                }
            });

            // Show a loading message
            crate::wasm_utils::console_log("📡 Loading products from gRPC server...");
        } else {
            crate::wasm_utils::console_log("⚠️ No gRPC client available");
            self.error_message = Some("gRPC client not available. Products must be loaded from database via gRPC server.".to_string());
            self.loading = false;
            return;
        }

        // Products will be loaded from database via gRPC - no hardcoded data
        // Note: The gRPC call above will populate the products when async call completes
        // DON'T clear existing data - only clear when gRPC call successfully returns new data
        // self.product_list.clear(); // REMOVED: this was a default action that bypassed gRPC authority
        self.loading = true; // Set to true while waiting for gRPC response
    }

    fn check_async_product_results(&mut self) {
        // Check if async product loading completed by polling localStorage
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                // Check for successful product data
                if let Ok(Some(json)) = storage.get_item("data_designer_products") {
                    if let Ok(products) = serde_json::from_str::<Vec<ProductRecord>>(&json) {
                        crate::wasm_utils::console_log(&format!("🔄 Updating UI with {} products from async result", products.len()));
                        self.product_list = products;
                        self.loading = false;

                        // Clear the storage item to avoid re-processing
                        let _ = storage.remove_item("data_designer_products");
                    }
                }

                // Check for error results
                if let Ok(Some(error)) = storage.get_item("data_designer_products_error") {
                    crate::wasm_utils::console_log(&format!("🔄 Setting error message from async result: {}", error));
                    self.error_message = Some(format!("Error loading products: {}", error));
                    self.loading = false;

                    // Clear the storage item to avoid re-processing
                    let _ = storage.remove_item("data_designer_products_error");
                }
            }
        }
    }

    fn load_service_list(&mut self, grpc_client: Option<&GrpcClient>) {
        self.loading = true;
        self.error_message = None;

        if let Some(client) = grpc_client {
            // Use gRPC GetServices endpoint to load from database
            let client_clone = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Note: Need to implement GetServicesRequest in grpc_client.rs
                crate::wasm_utils::console_log("📡 Loading services from database via gRPC server...");
                // TODO: Implement GetServicesRequest and call get_services() method
                crate::wasm_utils::console_log("⚠️ GetServices gRPC call not yet implemented in client");
            });

            crate::wasm_utils::console_log("📡 Services will be loaded from database via gRPC server...");
        } else {
            crate::wasm_utils::console_log("⚠️ No gRPC client available");
            self.error_message = Some("gRPC client not available. Services must be loaded from database via gRPC server.".to_string());
        }

        // Services will be loaded from database via gRPC - no hardcoded data
        // self.service_list.clear(); // REMOVED: default action that bypassed gRPC authority
        self.loading = true; // Set to true while waiting for gRPC response
    }

    fn load_resource_list(&mut self, grpc_client: Option<&GrpcClient>) {
        self.loading = true;
        self.error_message = None;

        if let Some(client) = grpc_client {
            // Use gRPC GetResources endpoint to load from database
            let client_clone = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Note: Need to implement GetResourcesRequest in grpc_client.rs
                crate::wasm_utils::console_log("📡 Loading resources from database via gRPC server...");
                // TODO: Implement GetResourcesRequest and call get_resources() method
                crate::wasm_utils::console_log("⚠️ GetResources gRPC call not yet implemented in client");
            });

            crate::wasm_utils::console_log("📡 Resources will be loaded from database via gRPC server...");
        } else {
            crate::wasm_utils::console_log("⚠️ No gRPC client available");
            self.error_message = Some("gRPC client not available. Resources must be loaded from database via gRPC server.".to_string());
        }

        // Resources will be loaded from database via gRPC - no hardcoded data
        // self.resource_list.clear(); // REMOVED: default action that bypassed gRPC authority
        self.loading = true; // Set to true while waiting for gRPC response
    }

    // CRUD operation methods (placeholder implementations)
    fn save_cbu(&mut self, grpc_client: Option<&GrpcClient>) {
        if let Some(client) = grpc_client {
            // Create some test entity associations to demonstrate DSL round-trip functionality
            let test_entities = vec![
                crate::grpc_client::CbuEntityAssociation {
                    entity_id: "TEST_ENT_001".to_string(),
                    entity_name: "Test Primary Entity".to_string(),
                    role_in_cbu: "Manager".to_string(),
                    entity_type: Some("Corporation".to_string()),
                    active_in_cbu: true,
                },
                crate::grpc_client::CbuEntityAssociation {
                    entity_id: "TEST_ENT_002".to_string(),
                    entity_name: "Test Secondary Entity".to_string(),
                    role_in_cbu: "Custodian".to_string(),
                    entity_type: Some("Trust".to_string()),
                    active_in_cbu: true,
                },
            ];

            // Create UpdateCbuRequest from form data
            let request = crate::grpc_client::UpdateCbuRequest {
                cbu_id: self.new_cbu_form.cbu_id.clone(),
                cbu_name: if self.new_cbu_form.cbu_name.is_empty() { None } else { Some(self.new_cbu_form.cbu_name.clone()) },
                description: if self.new_cbu_form.description.is_empty() { None } else { Some(self.new_cbu_form.description.clone()) },
                legal_entity_name: if self.new_cbu_form.legal_entity_name.is_empty() { None } else { Some(self.new_cbu_form.legal_entity_name.clone()) },
                jurisdiction: if self.new_cbu_form.jurisdiction.is_empty() { None } else { Some(self.new_cbu_form.jurisdiction.clone()) },
                business_model: if self.new_cbu_form.business_model.is_empty() { None } else { Some(self.new_cbu_form.business_model.clone()) },
                status: if self.new_cbu_form.status.is_empty() { None } else { Some(self.new_cbu_form.status.clone()) },
                entities: test_entities, // Include test entities to demonstrate DSL round-trip
            };

            // Store client and request for async call
            let client_clone = client.clone();
            let request_clone = request;

            // Use wasm_bindgen_futures to spawn async task
            wasm_bindgen_futures::spawn_local(async move {
                match client_clone.update_cbu(request_clone).await {
                    Ok(response) => {
                        if response.success {
                            crate::wasm_utils::console_log(&format!("✅ CBU updated successfully: {}", response.message));
                        } else {
                            crate::wasm_utils::console_log(&format!("❌ CBU update failed: {}", response.message));
                        }
                    },
                    Err(e) => {
                        crate::wasm_utils::console_log(&format!("❌ CBU update error: {}", e));
                    }
                }
            });

            self.success_message = Some("CBU save request sent - check console for status".to_string());
            self.show_cbu_form = false;
            self.load_cbu_list(Some(client));
        } else {
            self.error_message = Some("gRPC client not available".to_string());
        }
    }

    fn edit_cbu(&mut self, cbu: &CbuRecord) {
        self.new_cbu_form = CbuForm {
            cbu_id: cbu.cbu_id.clone(),
            cbu_name: cbu.cbu_name.clone(),
            description: cbu.description.clone().unwrap_or_default(),
            legal_entity_name: cbu.legal_entity_name.clone().unwrap_or_default(),
            jurisdiction: cbu.jurisdiction.clone().unwrap_or_default(),
            business_model: cbu.business_model.clone().unwrap_or_default(),
            status: cbu.status.clone(),
        };
        self.show_cbu_form = true;
    }

    fn delete_cbu(&mut self, _cbu_id: &str, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to delete CBU
        self.success_message = Some("CBU deleted successfully".to_string());
        self.load_cbu_list(_grpc_client);
    }

    fn save_product(&mut self, grpc_client: Option<&GrpcClient>) {
        if let Some(client) = grpc_client {
            // Check if this is a new product (empty ID) or an update
            let is_new_product = self.new_product_form.product_id.is_empty();

            if is_new_product {
                // Generate a new product ID for new products
                let product_id = format!("PROD{:03}", self.product_list.len() + 1);

                let request = crate::grpc_client::CreateProductRequest {
                    product_id: product_id.clone(),
                    product_name: self.new_product_form.product_name.clone(),
                    line_of_business: self.new_product_form.line_of_business.clone(),
                    description: if self.new_product_form.description.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.description.clone())
                    },
                    contract_type: if self.new_product_form.contract_type.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.contract_type.clone())
                    },
                    commercial_status: if self.new_product_form.commercial_status.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.commercial_status.clone())
                    },
                    pricing_model: if self.new_product_form.pricing_model.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.pricing_model.clone())
                    },
                    target_market: if self.new_product_form.target_market.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.target_market.clone())
                    },
                    status: self.new_product_form.status.clone(),
                };

                // Store the client for async operation
                let client_clone = client.clone();

                // Use spawn_local for WASM async operation
                wasm_bindgen_futures::spawn_local(async move {
                    match client_clone.create_product(request).await {
                        Ok(response) => {
                            if response.success {
                                crate::wasm_utils::console_log("✅ Product created successfully");
                            } else {
                                crate::wasm_utils::console_log(&format!("❌ Failed to create product: {}", response.message));
                            }
                        }
                        Err(e) => {
                            crate::wasm_utils::console_log(&format!("❌ Error creating product: {:?}", e));
                        }
                    }
                });

                self.success_message = Some("Product creation request sent".to_string());
            } else {
                // Update existing product
                let request = crate::grpc_client::UpdateProductRequest {
                    product_id: self.new_product_form.product_id.clone(),
                    product_name: self.new_product_form.product_name.clone(),
                    line_of_business: self.new_product_form.line_of_business.clone(),
                    description: if self.new_product_form.description.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.description.clone())
                    },
                    contract_type: if self.new_product_form.contract_type.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.contract_type.clone())
                    },
                    commercial_status: if self.new_product_form.commercial_status.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.commercial_status.clone())
                    },
                    pricing_model: if self.new_product_form.pricing_model.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.pricing_model.clone())
                    },
                    target_market: if self.new_product_form.target_market.is_empty() {
                        None
                    } else {
                        Some(self.new_product_form.target_market.clone())
                    },
                    status: self.new_product_form.status.clone(),
                };

                // Store the client for async operation
                let client_clone = client.clone();

                // Use spawn_local for WASM async operation
                wasm_bindgen_futures::spawn_local(async move {
                    match client_clone.update_product(request).await {
                        Ok(response) => {
                            if response.success {
                                crate::wasm_utils::console_log("✅ Product updated successfully");
                            } else {
                                crate::wasm_utils::console_log(&format!("❌ Failed to update product: {}", response.message));
                            }
                        }
                        Err(e) => {
                            crate::wasm_utils::console_log(&format!("❌ Error updating product: {:?}", e));
                        }
                    }
                });

                self.success_message = Some("Product update request sent".to_string());
            }
        } else {
            self.success_message = Some("No gRPC client available - using fallback".to_string());
        }

        self.show_product_form = false;
        self.load_product_list(grpc_client);
    }

    fn edit_product(&mut self, product: &ProductRecord) {
        self.new_product_form = ProductForm {
            product_id: product.product_id.clone(),
            product_name: product.product_name.clone(),
            line_of_business: product.line_of_business.clone(),
            description: product.description.clone().unwrap_or_default(),
            contract_type: product.contract_type.clone().unwrap_or_default(),
            commercial_status: product.commercial_status.clone().unwrap_or_default(),
            pricing_model: product.pricing_model.clone().unwrap_or_default(),
            target_market: product.target_market.clone().unwrap_or_default(),
            status: product.status.clone(),
        };
        self.show_product_form = true;
    }

    fn delete_product(&mut self, _product_id: &str, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to delete Product
        self.success_message = Some("Product deleted successfully".to_string());
        self.load_product_list(_grpc_client);
    }

    fn save_service(&mut self, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to create/update Service
        self.success_message = Some("Service saved successfully".to_string());
        self.show_service_form = false;
        self.load_service_list(_grpc_client);
    }

    fn edit_service(&mut self, service: &ServiceRecord) {
        self.new_service_form = ServiceForm {
            service_id: service.id.clone(),
            service_name: service.name.clone(),
            service_category: service.service_category.clone(),
            description: service.description.clone(),
            service_type: service.service_type.clone(),
            delivery_model: service.delivery_model.clone(),
            billable: service.billable,
            status: service.status.clone(),
        };
        self.show_service_form = true;
    }

    fn delete_service(&mut self, _service_id: &str, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to delete Service
        self.success_message = Some("Service deleted successfully".to_string());
        self.load_service_list(_grpc_client);
    }

    fn save_resource(&mut self, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to create/update Resource
        self.success_message = Some("Resource saved successfully".to_string());
        self.show_resource_form = false;
        self.load_resource_list(_grpc_client);
    }

    fn edit_resource(&mut self, resource: &ResourceRecord) {
        self.new_resource_form = ResourceForm {
            resource_id: resource.id.clone(),
            resource_name: resource.name.clone(),
            resource_type: resource.resource_type.clone(),
            description: resource.description.clone(),
            location: resource.location.clone().unwrap_or_default(),
            capabilities: resource.capabilities.clone(),
            status: resource.status.clone(),
            visibility: resource.visibility.clone(),
        };
        self.show_resource_form = true;
    }

    fn delete_resource(&mut self, _resource_id: &str, _grpc_client: Option<&GrpcClient>) {
        // TODO: Implement actual gRPC call to delete Resource
        self.success_message = Some("Resource deleted successfully".to_string());
        self.load_resource_list(_grpc_client);
    }

    // Picker Windows Implementation using the successful resizing pattern
    fn render_product_picker(&mut self, ctx: &egui::Context, grpc_client: Option<&GrpcClient>) {
        // Check for async product loading results
        self.check_async_product_results();

        let mut open = self.show_product_picker;

        // Product Picker with proper resizing using the successful pattern
        egui::Window::new("📦 Product Picker - Select Product")
            .open(&mut open)
            .resizable(true)
            .default_size([680.0, 500.0])
            .show(ctx, |ui| {
                // Filter controls
                ui.horizontal(|ui| {
                    ui.label("🔍 Search:");
                    ui.text_edit_singleline(&mut self.product_picker_filter);

                    if ui.button("🔄 Refresh Products").clicked() {
                        self.load_product_list(grpc_client);
                    }
                });

                ui.separator();

                // Filter products based on search
                let filtered_products: Vec<&ProductRecord> = self.product_list.iter()
                    .filter(|product| {
                        self.product_picker_filter.is_empty() ||
                        product.product_name.to_lowercase().contains(&self.product_picker_filter.to_lowercase()) ||
                        product.product_id.to_lowercase().contains(&self.product_picker_filter.to_lowercase()) ||
                        product.line_of_business.to_lowercase().contains(&self.product_picker_filter.to_lowercase())
                    })
                    .collect();

                ui.label(format!("📋 Found {} products:", filtered_products.len()));
                ui.separator();

                // CRITICAL: Dynamic height allocation for proper resizing
                let available_height = ui.available_height();
                let content_height = (available_height - 150.0).max(200.0); // Reserve space for controls

                // Scrollable list with proper space allocation
                egui::ScrollArea::vertical()
                    .max_height(content_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if filtered_products.is_empty() && self.product_list.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label("📭 No products available");
                                ui.label("Click 'Refresh Products' to load from gRPC server");
                            });
                        } else if filtered_products.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label("🔍 No products match your search");
                                ui.label("Try adjusting the search filter");
                            });
                        } else {
                            for product in &filtered_products {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.heading(format!("📦 {}", product.product_name));
                                            ui.horizontal(|ui| {
                                                ui.label(format!("🆔 {}", product.product_id));
                                                ui.label("•");
                                                ui.label(format!("🏢 {}", product.line_of_business));
                                            });
                                            if let Some(desc) = &product.description {
                                                ui.label(format!("📄 {}", desc));
                                            }
                                            ui.label(format!("📊 Status: {}", product.status));
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.add_sized([100.0, 30.0], egui::Button::new("✅ Select")).clicked() {
                                                // TODO: Handle product selection - could set selected_product or trigger callback
                                                self.selected_product = Some((*product).clone());
                                                self.success_message = Some(format!("Selected product: {}", product.product_name));
                                                self.show_product_picker = false;
                                            }
                                            if ui.add_sized([80.0, 30.0], egui::Button::new("👁️ View")).clicked() {
                                                self.selected_product = Some((*product).clone());
                                            }
                                        });
                                    });
                                });
                                ui.add_space(5.0);
                            }
                        }

                        // CRITICAL: Allocate remaining space for proper window resizing
                        ui.allocate_space(ui.available_size());
                    });

                // Close button
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("❌ Cancel").clicked() {
                        self.show_product_picker = false;
                    }
                    ui.label("Search and select a product from the list above");
                });
            });

        // Update state if window was closed via X button
        self.show_product_picker = open;
    }
}