use eframe::egui;
use data_designer_core::db::{
    init_db, DbPool,
    ClientBusinessUnit, CreateCbuRequest,
    DbOperations
};
use tokio::runtime::Runtime;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    println!("🚀 Starting Data Designer - Pure Rust Edition!");
    println!("🔌 Connecting to PostgreSQL database...");

    // Initialize database connection
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let pool = rt.block_on(async {
        match init_db().await {
            Ok(pool) => {
                println!("✅ Database connected successfully");
                Some(pool)
            }
            Err(e) => {
                eprintln!("❌ Database connection failed: {}", e);
                eprintln!("   Continuing with offline mode...");
                None
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Data Designer - Pure Rust + Database",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DataDesignerApp::new(pool, rt)))
        }),
    )
}

struct DataDesignerApp {
    current_tab: Tab,
    db_pool: Option<DbPool>,
    runtime: Arc<Runtime>,

    // Data
    cbus: Vec<ClientBusinessUnit>,
    selected_cbu: Option<usize>,

    // UI State
    show_cbu_form: bool,
    cbu_form: CbuForm,
    status_message: String,
    loading: bool,
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    Dashboard,
    CBUs,
    Database,
}

#[derive(Default)]
struct CbuForm {
    cbu_name: String,
    description: String,
    primary_entity_id: String,
    primary_lei: String,
    domicile_country: String,
    business_type: String,
}

impl DataDesignerApp {
    fn new(db_pool: Option<DbPool>, runtime: Arc<Runtime>) -> Self {
        let mut app = Self {
            current_tab: Tab::default(),
            db_pool,
            runtime,
            cbus: Vec::new(),
            selected_cbu: None,
            show_cbu_form: false,
            cbu_form: CbuForm::default(),
            status_message: "Initializing...".to_string(),
            loading: false,
        };

        // Load initial data
        app.load_cbus();
        app
    }

    fn load_cbus(&mut self) {
        if let Some(ref pool) = self.db_pool {
            self.loading = true;
            self.status_message = "Loading CBUs from database...".to_string();

            let _pool = pool.clone();
            let rt = self.runtime.clone();

            match rt.block_on(async {
                DbOperations::list_cbus().await
            }) {
                Ok(cbu_summaries) => {
                    // Convert summaries to full CBUs - for now just create basic ones
                    self.cbus = cbu_summaries.into_iter().map(|summary| ClientBusinessUnit {
                        id: summary.id,
                        cbu_id: summary.cbu_id,
                        cbu_name: summary.cbu_name,
                        description: summary.description,
                        primary_entity_id: None,
                        primary_lei: None,
                        domicile_country: None,
                        regulatory_jurisdiction: None,
                        business_type: None,
                        status: summary.status,
                        created_date: None,
                        last_review_date: None,
                        next_review_date: None,
                        created_by: None,
                        created_at: summary.created_at,
                        updated_by: None,
                        updated_at: summary.updated_at,
                        metadata: None,
                    }).collect();

                    self.status_message = format!("✅ Loaded {} CBUs from database", self.cbus.len());
                }
                Err(e) => {
                    eprintln!("Failed to load CBUs: {}", e);
                    self.status_message = format!("❌ Failed to load CBUs: {}", e);
                    self.load_sample_data();
                }
            }
            self.loading = false;
        } else {
            self.load_sample_data();
        }
    }

    fn load_sample_data(&mut self) {
        use chrono::Utc;

        self.cbus = vec![
            ClientBusinessUnit {
                id: 0,
                cbu_id: "OFFLINE001".to_string(),
                cbu_name: "Sample CBU (Offline Mode)".to_string(),
                description: Some("No database connection - sample data".to_string()),
                primary_entity_id: None,
                primary_lei: None,
                domicile_country: None,
                regulatory_jurisdiction: None,
                business_type: None,
                status: "Sample".to_string(),
                created_date: None,
                last_review_date: None,
                next_review_date: None,
                created_by: None,
                created_at: Utc::now(),
                updated_by: None,
                updated_at: Utc::now(),
                metadata: None,
            }
        ];
        self.status_message = "⚠️ Offline mode - using sample data".to_string();
    }

    fn create_cbu(&mut self) {
        if let Some(ref _pool) = self.db_pool {
            let request = CreateCbuRequest {
                cbu_name: self.cbu_form.cbu_name.clone(),
                description: if self.cbu_form.description.is_empty() { None } else { Some(self.cbu_form.description.clone()) },
                primary_entity_id: if self.cbu_form.primary_entity_id.is_empty() { None } else { Some(self.cbu_form.primary_entity_id.clone()) },
                primary_lei: if self.cbu_form.primary_lei.is_empty() { None } else { Some(self.cbu_form.primary_lei.clone()) },
                domicile_country: if self.cbu_form.domicile_country.is_empty() { None } else { Some(self.cbu_form.domicile_country.clone()) },
                regulatory_jurisdiction: None,
                business_type: if self.cbu_form.business_type.is_empty() { None } else { Some(self.cbu_form.business_type.clone()) },
                created_by: Some("egui-app".to_string()),
            };

            let rt = self.runtime.clone();
            match rt.block_on(async {
                DbOperations::create_cbu(request).await
            }) {
                Ok(cbu) => {
                    self.cbus.push(cbu);
                    self.status_message = "✅ CBU created successfully".to_string();
                    self.show_cbu_form = false;
                    self.cbu_form = CbuForm::default();
                }
                Err(e) => {
                    self.status_message = format!("❌ Failed to create CBU: {}", e);
                }
            }
        } else {
            self.status_message = "❌ No database connection".to_string();
        }
    }
}

impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ui.close();
                    }
                });
                ui.menu_button("Database", |ui| {
                    if ui.button("Refresh").clicked() {
                        self.load_cbus();
                        ui.close();
                    }
                    if ui.button("Test Connection").clicked() {
                        if self.db_pool.is_some() {
                            self.status_message = "✅ Database connected".to_string();
                        } else {
                            self.status_message = "❌ No database connection".to_string();
                        }
                        ui.close();
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                let color = if self.db_pool.is_some() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::YELLOW
                };
                ui.colored_label(color, &self.status_message);

                if self.loading {
                    ui.spinner();
                }
            });
        });

        // Tab panel
        egui::TopBottomPanel::top("tab_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "🏠 Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::CBUs, "🏢 CBUs");
                ui.selectable_value(&mut self.current_tab, Tab::Database, "🗄️ Database");
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Dashboard => self.show_dashboard(ui),
                Tab::CBUs => self.show_cbu_tab(ui),
                Tab::Database => self.show_database_tab(ui),
            }
        });

        // CBU form modal
        if self.show_cbu_form {
            egui::Window::new("Create CBU")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    self.show_cbu_form_ui(ui);
                });
        }
    }
}

impl DataDesignerApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🦀 Pure Rust Data Designer");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Database Status:");
            if self.db_pool.is_some() {
                ui.colored_label(egui::Color32::GREEN, "✅ Connected to PostgreSQL");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Offline mode");
            }
        });

        ui.horizontal(|ui| {
            ui.label("CBUs Loaded:");
            ui.label(format!("{}", self.cbus.len()));
        });

        ui.separator();

        if ui.button("🔄 Refresh Data").clicked() {
            self.load_cbus();
        }
    }

    fn show_cbu_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Client Business Units");

        ui.horizontal(|ui| {
            if ui.button("➕ Create CBU").clicked() {
                self.show_cbu_form = true;
                self.cbu_form = CbuForm::default();
            }

            if ui.button("🔄 Refresh").clicked() {
                self.load_cbus();
            }
        });

        ui.separator();

        // CBU table
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("cbu_grid").striped(true).show(ui, |ui| {
                ui.label("ID");
                ui.label("CBU ID");
                ui.label("Name");
                ui.label("Status");
                ui.label("Description");
                ui.label("Created");
                ui.end_row();

                for (_index, cbu) in self.cbus.iter().enumerate() {
                    ui.label(cbu.id.to_string());
                    ui.label(&cbu.cbu_id);
                    ui.label(&cbu.cbu_name);
                    ui.label(&cbu.status);
                    ui.label(cbu.description.as_ref().unwrap_or(&"N/A".to_string()));
                    ui.label(cbu.created_at.format("%Y-%m-%d").to_string());
                    ui.end_row();
                }
            });
        });
    }

    fn show_database_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🗄️ Database Management");

        ui.separator();

        if let Some(ref _pool) = self.db_pool {
            ui.colored_label(egui::Color32::GREEN, "✅ PostgreSQL Connected");
            ui.label("Connection pool active and ready");

            ui.separator();

            if ui.button("🧪 Test Query").clicked() {
                self.load_cbus();
            }
        } else {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ No Database Connection");
            ui.label("The application is running in offline mode");
            ui.label("Check config.toml or DATABASE_URL environment variable");
        }
    }

    fn show_cbu_form_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("cbu_form_grid").show(ui, |ui| {
            ui.label("CBU Name:");
            ui.text_edit_singleline(&mut self.cbu_form.cbu_name);
            ui.end_row();

            ui.label("Description:");
            ui.text_edit_multiline(&mut self.cbu_form.description);
            ui.end_row();

            ui.label("Primary Entity ID:");
            ui.text_edit_singleline(&mut self.cbu_form.primary_entity_id);
            ui.end_row();

            ui.label("Primary LEI:");
            ui.text_edit_singleline(&mut self.cbu_form.primary_lei);
            ui.end_row();

            ui.label("Domicile Country:");
            ui.text_edit_singleline(&mut self.cbu_form.domicile_country);
            ui.end_row();

            ui.label("Business Type:");
            ui.text_edit_singleline(&mut self.cbu_form.business_type);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("💾 Create").clicked() {
                self.create_cbu();
            }

            if ui.button("❌ Cancel").clicked() {
                self.show_cbu_form = false;
                self.cbu_form = CbuForm::default();
            }
        });
    }
}