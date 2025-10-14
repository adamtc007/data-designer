use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    println!("🚀 Starting Data Designer - Pure Rust Edition!");
    println!("✅ Tauri completely removed!");
    println!("✅ All HTML/JS/TS garbage deleted!");
    println!("🦀 Pure Rust egui immediate mode GUI");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Data Designer - Pure Rust CRUD",
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
    status_message: String,
    counter: i32,
}

#[derive(PartialEq, Default)]
enum Tab {
    #[default]
    Dashboard,
    Database,
    Status,
}

impl DataDesignerApp {
    fn new() -> Self {
        Self {
            status_message: "🎉 Pure Rust application - Tauri completely removed!".to_string(),
            ..Default::default()
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
                    if ui.button("Connect").clicked() {
                        self.status_message = "Database connection - coming soon!".to_string();
                        ui.close();
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(egui::Color32::GREEN, &self.status_message);
            });
        });

        // Tab panel
        egui::TopBottomPanel::top("tab_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "🏠 Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::Database, "🗄️ Database");
                ui.selectable_value(&mut self.current_tab, Tab::Status, "📊 Status");
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Dashboard => self.show_dashboard(ui),
                Tab::Database => self.show_database(ui),
                Tab::Status => self.show_status(ui),
            }
        });
    }
}

impl DataDesignerApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎉 Welcome to Pure Rust Data Designer!");

        ui.separator();

        ui.label("This is a complete rewrite using egui immediate mode GUI.");
        ui.label("No more Tauri, no more HTML/JS/TS - just pure Rust!");

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ Increment").clicked() {
                self.counter += 1;
                self.status_message = format!("Counter: {}", self.counter);
            }

            if ui.button("➖ Decrement").clicked() {
                self.counter -= 1;
                self.status_message = format!("Counter: {}", self.counter);
            }

            ui.label(format!("Value: {}", self.counter));
        });

        ui.separator();

        ui.collapsing("✅ Completed Tasks", |ui| {
            ui.label("• Removed all Tauri dependencies");
            ui.label("• Deleted all HTML/JS/TS files");
            ui.label("• Created pure Rust egui frontend");
            ui.label("• Set up Cargo workspace structure");
            ui.label("• Moved database code to shared library");
        });

        ui.collapsing("🚧 Next Steps", |ui| {
            ui.label("• Connect egui app to database");
            ui.label("• Implement CRUD operations");
            ui.label("• Add rule editor interface");
            ui.label("• Integrate existing parser/engine");
        });
    }

    fn show_database(&mut self, ui: &mut egui::Ui) {
        ui.heading("🗄️ Database Management");
        ui.label("Database integration will be implemented here.");
        ui.label("The core database library is already available in data-designer-core.");

        ui.separator();

        if ui.button("🔌 Test Connection").clicked() {
            self.status_message = "Database connection test - not implemented yet".to_string();
        }
    }

    fn show_status(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 System Status");

        ui.separator();

        egui::Grid::new("status_grid").show(ui, |ui| {
            ui.label("Frontend:");
            ui.colored_label(egui::Color32::GREEN, "✅ Pure Rust egui");
            ui.end_row();

            ui.label("Backend:");
            ui.colored_label(egui::Color32::GREEN, "✅ Rust core library");
            ui.end_row();

            ui.label("Database:");
            ui.colored_label(egui::Color32::YELLOW, "🚧 Ready to connect");
            ui.end_row();

            ui.label("Tauri:");
            ui.colored_label(egui::Color32::RED, "❌ Completely removed");
            ui.end_row();

            ui.label("HTML/JS/TS:");
            ui.colored_label(egui::Color32::RED, "❌ All deleted");
            ui.end_row();
        });
    }
}