use crate::{DataDesignerApp, CBU, Product, Resource};
use eframe::egui;

impl DataDesignerApp {
    pub fn show_cbu_form_modal(&mut self, ctx: &egui::Context) {
        if !self.show_cbu_form {
            return;
        }

        egui::Window::new("CBU Form")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::Grid::new("cbu_form_grid").show(ui, |ui| {
                    ui.label("CBU ID:");
                    ui.text_edit_singleline(&mut self.cbu_form.cbu_id);
                    ui.end_row();

                    ui.label("CBU Name:");
                    ui.text_edit_singleline(&mut self.cbu_form.cbu_name);
                    ui.end_row();

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.cbu_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.cbu_form.status, "Active".to_string(), "Active");
                            ui.selectable_value(&mut self.cbu_form.status, "Inactive".to_string(), "Inactive");
                            ui.selectable_value(&mut self.cbu_form.status, "Pending".to_string(), "Pending");
                        });
                    ui.end_row();

                    ui.label("Business Type:");
                    ui.text_edit_singleline(&mut self.cbu_form.business_type);
                    ui.end_row();

                    ui.label("Primary LEI:");
                    ui.text_edit_singleline(&mut self.cbu_form.primary_lei);
                    ui.end_row();

                    ui.label("Domicile Country:");
                    ui.text_edit_singleline(&mut self.cbu_form.domicile_country);
                    ui.end_row();

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.cbu_form.description);
                    ui.end_row();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_cbu();
                        self.show_cbu_form = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_cbu_form = false;
                        self.selected_cbu = None;
                    }
                });
            });
    }

    pub fn show_product_form_modal(&mut self, ctx: &egui::Context) {
        if !self.show_product_form {
            return;
        }

        egui::Window::new("Product Form")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::Grid::new("product_form_grid").show(ui, |ui| {
                    ui.label("Product Name:");
                    ui.text_edit_singleline(&mut self.product_form.product_name);
                    ui.end_row();

                    ui.label("Product Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.product_form.product_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.product_form.product_type, "Software".to_string(), "Software");
                            ui.selectable_value(&mut self.product_form.product_type, "Hardware".to_string(), "Hardware");
                            ui.selectable_value(&mut self.product_form.product_type, "Service".to_string(), "Service");
                        });
                    ui.end_row();

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.product_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.product_form.status, "Active".to_string(), "Active");
                            ui.selectable_value(&mut self.product_form.status, "Inactive".to_string(), "Inactive");
                            ui.selectable_value(&mut self.product_form.status, "Development".to_string(), "Development");
                        });
                    ui.end_row();

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.product_form.description);
                    ui.end_row();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_product();
                        self.show_product_form = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_product_form = false;
                        self.selected_product = None;
                    }
                });
            });
    }

    pub fn show_resource_form_modal(&mut self, ctx: &egui::Context) {
        if !self.show_resource_form {
            return;
        }

        egui::Window::new("Resource Form")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::Grid::new("resource_form_grid").show(ui, |ui| {
                    ui.label("Resource Name:");
                    ui.text_edit_singleline(&mut self.resource_form.resource_name);
                    ui.end_row();

                    ui.label("Resource Type:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.resource_form.resource_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.resource_form.resource_type, "Database".to_string(), "Database");
                            ui.selectable_value(&mut self.resource_form.resource_type, "Server".to_string(), "Server");
                            ui.selectable_value(&mut self.resource_form.resource_type, "Storage".to_string(), "Storage");
                            ui.selectable_value(&mut self.resource_form.resource_type, "Network".to_string(), "Network");
                        });
                    ui.end_row();

                    ui.label("Status:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.resource_form.status)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.resource_form.status, "Active".to_string(), "Active");
                            ui.selectable_value(&mut self.resource_form.status, "Inactive".to_string(), "Inactive");
                            ui.selectable_value(&mut self.resource_form.status, "Maintenance".to_string(), "Maintenance");
                        });
                    ui.end_row();

                    ui.label("Location:");
                    ui.text_edit_singleline(&mut self.resource_form.location);
                    ui.end_row();

                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.resource_form.description);
                    ui.end_row();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_resource();
                        self.show_resource_form = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_resource_form = false;
                        self.selected_resource = None;
                    }
                });
            });
    }

    // Save operations
    fn save_cbu(&mut self) {
        let new_cbu = CBU {
            id: if let Some(index) = self.selected_cbu {
                self.cbus[index].id
            } else {
                Some(self.cbus.len() as i32 + 1)
            },
            cbu_id: self.cbu_form.cbu_id.clone(),
            cbu_name: self.cbu_form.cbu_name.clone(),
            status: self.cbu_form.status.clone(),
            description: if self.cbu_form.description.is_empty() { None } else { Some(self.cbu_form.description.clone()) },
            primary_lei: if self.cbu_form.primary_lei.is_empty() { None } else { Some(self.cbu_form.primary_lei.clone()) },
            domicile_country: if self.cbu_form.domicile_country.is_empty() { None } else { Some(self.cbu_form.domicile_country.clone()) },
            business_type: if self.cbu_form.business_type.is_empty() { None } else { Some(self.cbu_form.business_type.clone()) },
            member_count: Some(0),
            role_count: Some(0),
            roles: None,
        };

        if let Some(index) = self.selected_cbu {
            self.cbus[index] = new_cbu;
        } else {
            self.cbus.push(new_cbu);
        }

        self.selected_cbu = None;
        self.cbu_form = Default::default();
    }

    fn save_product(&mut self) {
        let new_product = Product {
            id: if let Some(index) = self.selected_product {
                self.products[index].id
            } else {
                Some(self.products.len() as i32 + 1)
            },
            product_name: self.product_form.product_name.clone(),
            product_type: self.product_form.product_type.clone(),
            status: self.product_form.status.clone(),
            description: if self.product_form.description.is_empty() { None } else { Some(self.product_form.description.clone()) },
            created_date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        };

        if let Some(index) = self.selected_product {
            self.products[index] = new_product;
        } else {
            self.products.push(new_product);
        }

        self.selected_product = None;
        self.product_form = Default::default();
    }

    fn save_resource(&mut self) {
        let new_resource = Resource {
            id: if let Some(index) = self.selected_resource {
                self.resources[index].id
            } else {
                Some(self.resources.len() as i32 + 1)
            },
            resource_name: self.resource_form.resource_name.clone(),
            resource_type: self.resource_form.resource_type.clone(),
            status: self.resource_form.status.clone(),
            description: if self.resource_form.description.is_empty() { None } else { Some(self.resource_form.description.clone()) },
            location: if self.resource_form.location.is_empty() { None } else { Some(self.resource_form.location.clone()) },
        };

        if let Some(index) = self.selected_resource {
            self.resources[index] = new_resource;
        } else {
            self.resources.push(new_resource);
        }

        self.selected_resource = None;
        self.resource_form = Default::default();
    }
}