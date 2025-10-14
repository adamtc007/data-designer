use eframe::egui;
use serde::{Deserialize, Serialize};

mod modals;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Data Designer - CRUD Management",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DataDesignerApp::new()))
        }),
    )
}

#[derive(Default)]
struct DataDesignerApp {
    current_tab: Tab,
    db_connected: bool,

    // CBU Management
    cbus: Vec<CBU>,
    selected_cbu: Option<usize>,
    cbu_form: CBUForm,
    show_cbu_form: bool,

    // Product Management
    products: Vec<Product>,
    selected_product: Option<usize>,
    product_form: ProductForm,
    show_product_form: bool,

    // Resource Management
    resources: Vec<Resource>,
    selected_resource: Option<usize>,
    resource_form: ResourceForm,
    show_resource_form: bool,
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    CBUs,
    Products,
    Resources,
    Database,
}

#[derive(Clone, Serialize, Deserialize)]
struct CBU {
    id: Option<i32>,
    cbu_id: String,
    cbu_name: String,
    status: String,
    description: Option<String>,
    primary_lei: Option<String>,
    domicile_country: Option<String>,
    business_type: Option<String>,
    member_count: Option<i32>,
    role_count: Option<i32>,
    roles: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Product {
    id: Option<i32>,
    product_name: String,
    product_type: String,
    description: Option<String>,
    status: String,
    created_date: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Resource {
    id: Option<i32>,
    resource_name: String,
    resource_type: String,
    description: Option<String>,
    status: String,
    location: Option<String>,
}

#[derive(Default)]
struct CBUForm {
    cbu_id: String,
    cbu_name: String,
    status: String,
    description: String,
    primary_lei: String,
    domicile_country: String,
    business_type: String,
}

#[derive(Default)]
struct ProductForm {
    product_name: String,
    product_type: String,
    description: String,
    status: String,
}

#[derive(Default)]
struct ResourceForm {
    resource_name: String,
    resource_type: String,
    description: String,
    status: String,
    location: String,
}

impl DataDesignerApp {
    fn new() -> Self {
        let mut app = Self::default();

        // Initialize with sample data
        app.cbus = vec![
            CBU {
                id: Some(1),
                cbu_id: "CBU001".to_string(),
                cbu_name: "Financial Services CBU".to_string(),
                status: "Active".to_string(),
                description: Some("Primary financial services business unit".to_string()),
                primary_lei: Some("LEI123456789".to_string()),
                domicile_country: Some("USA".to_string()),
                business_type: Some("Financial".to_string()),
                member_count: Some(150),
                role_count: Some(25),
                roles: Some("Trading, Risk Management".to_string()),
            },
        ];

        app.products = vec![
            Product {
                id: Some(1),
                product_name: "Risk Management System".to_string(),
                product_type: "Software".to_string(),
                description: Some("Enterprise risk management platform".to_string()),
                status: "Active".to_string(),
                created_date: Some("2024-01-15".to_string()),
            },
        ];

        app.resources = vec![
            Resource {
                id: Some(1),
                resource_name: "PostgreSQL Database".to_string(),
                resource_type: "Database".to_string(),
                description: Some("Primary data storage".to_string()),
                status: "Active".to_string(),
                location: Some("us-east-1".to_string()),
            },
        ];

        app
    }
}

impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Export Data").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Import Data").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Database", |ui| {
                    if ui.button("Connect").clicked() {
                        self.db_connected = !self.db_connected;
                        ui.close_menu();
                    }
                    if ui.button("Sync Data").clicked() {
                        ui.close_menu();
                    }
                });

                ui.separator();

                // Connection status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let color = if self.db_connected { egui::Color32::GREEN } else { egui::Color32::RED };
                    ui.colored_label(color, if self.db_connected { "🟢 Connected" } else { "🔴 Disconnected" });
                });
            });
        });

        // Left panel for navigation
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Data Designer");
                ui.separator();

                // Tab selection
                ui.selectable_value(&mut self.current_tab, Tab::CBUs, "🏢 CBUs");
                ui.selectable_value(&mut self.current_tab, Tab::Products, "📦 Products");
                ui.selectable_value(&mut self.current_tab, Tab::Resources, "🔧 Resources");
                ui.selectable_value(&mut self.current_tab, Tab::Database, "🗄️ Database");
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::CBUs => self.show_cbu_management(ui),
                Tab::Products => self.show_product_management(ui),
                Tab::Resources => self.show_resource_management(ui),
                Tab::Database => self.show_database_info(ui),
            }
        });

        // Modal windows
        self.show_cbu_form_modal(ctx);
        self.show_product_form_modal(ctx);
        self.show_resource_form_modal(ctx);
    }
}

impl DataDesignerApp {
    fn show_cbu_management(&mut self, ui: &mut egui::Ui) {
        ui.heading("CBU Management");

        ui.horizontal(|ui| {
            if ui.button("➕ Add CBU").clicked() {
                self.cbu_form = CBUForm::default();
                self.show_cbu_form = true;
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Reload from database
            }
        });

        ui.separator();

        let mut edit_index = None;
        let mut delete_index = None;

        // CBU table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("cbu_grid")
                .num_columns(8)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label("ID");
                    ui.label("CBU ID");
                    ui.label("Name");
                    ui.label("Status");
                    ui.label("Type");
                    ui.label("Members");
                    ui.label("Roles");
                    ui.label("Actions");
                    ui.end_row();

                    // Data rows
                    for (index, cbu) in self.cbus.iter().enumerate() {
                        ui.label(format!("{}", cbu.id.unwrap_or(0)));
                        ui.label(&cbu.cbu_id);
                        ui.label(&cbu.cbu_name);
                        ui.label(&cbu.status);
                        ui.label(cbu.business_type.as_ref().unwrap_or(&"N/A".to_string()));
                        ui.label(format!("{}", cbu.member_count.unwrap_or(0)));
                        ui.label(format!("{}", cbu.role_count.unwrap_or(0)));

                        ui.horizontal(|ui| {
                            if ui.small_button("✏️").clicked() {
                                edit_index = Some(index);
                            }
                            if ui.small_button("🗑️").clicked() {
                                delete_index = Some(index);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        // Handle actions after borrowing is complete
        if let Some(index) = edit_index {
            self.load_cbu_for_edit(index);
            self.show_cbu_form = true;
        }
        if let Some(index) = delete_index {
            self.delete_cbu(index);
        }
    }

    fn show_product_management(&mut self, ui: &mut egui::Ui) {
        ui.heading("Product Management");

        ui.horizontal(|ui| {
            if ui.button("➕ Add Product").clicked() {
                self.product_form = ProductForm::default();
                self.show_product_form = true;
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Reload from database
            }
        });

        ui.separator();

        let mut edit_index = None;
        let mut delete_index = None;

        // Product table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("product_grid")
                .num_columns(6)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label("ID");
                    ui.label("Name");
                    ui.label("Type");
                    ui.label("Status");
                    ui.label("Created");
                    ui.label("Actions");
                    ui.end_row();

                    // Data rows
                    for (index, product) in self.products.iter().enumerate() {
                        ui.label(format!("{}", product.id.unwrap_or(0)));
                        ui.label(&product.product_name);
                        ui.label(&product.product_type);
                        ui.label(&product.status);
                        ui.label(product.created_date.as_ref().unwrap_or(&"N/A".to_string()));

                        ui.horizontal(|ui| {
                            if ui.small_button("✏️").clicked() {
                                edit_index = Some(index);
                            }
                            if ui.small_button("🗑️").clicked() {
                                delete_index = Some(index);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        // Handle actions after borrowing is complete
        if let Some(index) = edit_index {
            self.load_product_for_edit(index);
            self.show_product_form = true;
        }
        if let Some(index) = delete_index {
            self.delete_product(index);
        }
    }

    fn show_resource_management(&mut self, ui: &mut egui::Ui) {
        ui.heading("Resource Management");

        ui.horizontal(|ui| {
            if ui.button("➕ Add Resource").clicked() {
                self.resource_form = ResourceForm::default();
                self.show_resource_form = true;
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Reload from database
            }
        });

        ui.separator();

        let mut edit_index = None;
        let mut delete_index = None;

        // Resource table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("resource_grid")
                .num_columns(6)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label("ID");
                    ui.label("Name");
                    ui.label("Type");
                    ui.label("Status");
                    ui.label("Location");
                    ui.label("Actions");
                    ui.end_row();

                    // Data rows
                    for (index, resource) in self.resources.iter().enumerate() {
                        ui.label(format!("{}", resource.id.unwrap_or(0)));
                        ui.label(&resource.resource_name);
                        ui.label(&resource.resource_type);
                        ui.label(&resource.status);
                        ui.label(resource.location.as_ref().unwrap_or(&"N/A".to_string()));

                        ui.horizontal(|ui| {
                            if ui.small_button("✏️").clicked() {
                                edit_index = Some(index);
                            }
                            if ui.small_button("🗑️").clicked() {
                                delete_index = Some(index);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        // Handle actions after borrowing is complete
        if let Some(index) = edit_index {
            self.load_resource_for_edit(index);
            self.show_resource_form = true;
        }
        if let Some(index) = delete_index {
            self.delete_resource(index);
        }
    }

    fn show_database_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Database Information");

        ui.group(|ui| {
            ui.label("Connection Status:");
            let color = if self.db_connected { egui::Color32::GREEN } else { egui::Color32::RED };
            ui.colored_label(color, if self.db_connected { "Connected" } else { "Disconnected" });
        });

        ui.separator();

        ui.label("Statistics:");
        ui.label(format!("CBUs: {}", self.cbus.len()));
        ui.label(format!("Products: {}", self.products.len()));
        ui.label(format!("Resources: {}", self.resources.len()));
    }

    // CRUD Operations
    fn load_cbu_for_edit(&mut self, index: usize) {
        if let Some(cbu) = self.cbus.get(index) {
            self.cbu_form.cbu_id = cbu.cbu_id.clone();
            self.cbu_form.cbu_name = cbu.cbu_name.clone();
            self.cbu_form.status = cbu.status.clone();
            self.cbu_form.description = cbu.description.clone().unwrap_or_default();
            self.cbu_form.primary_lei = cbu.primary_lei.clone().unwrap_or_default();
            self.cbu_form.domicile_country = cbu.domicile_country.clone().unwrap_or_default();
            self.cbu_form.business_type = cbu.business_type.clone().unwrap_or_default();
            self.selected_cbu = Some(index);
        }
    }

    fn delete_cbu(&mut self, index: usize) {
        self.cbus.remove(index);
    }

    fn load_product_for_edit(&mut self, index: usize) {
        if let Some(product) = self.products.get(index) {
            self.product_form.product_name = product.product_name.clone();
            self.product_form.product_type = product.product_type.clone();
            self.product_form.status = product.status.clone();
            self.product_form.description = product.description.clone().unwrap_or_default();
            self.selected_product = Some(index);
        }
    }

    fn delete_product(&mut self, index: usize) {
        self.products.remove(index);
    }

    fn load_resource_for_edit(&mut self, index: usize) {
        if let Some(resource) = self.resources.get(index) {
            self.resource_form.resource_name = resource.resource_name.clone();
            self.resource_form.resource_type = resource.resource_type.clone();
            self.resource_form.status = resource.status.clone();
            self.resource_form.description = resource.description.clone().unwrap_or_default();
            self.resource_form.location = resource.location.clone().unwrap_or_default();
            self.selected_resource = Some(index);
        }
    }

    fn delete_resource(&mut self, index: usize) {
        self.resources.remove(index);
    }
}
