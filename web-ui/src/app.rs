use eframe::egui;
use crate::{WebRouter, wasm_utils};
use crate::grpc_client::GrpcClient;
use crate::cbu_dsl_ide::CbuDslIDE;
use crate::cbu_state_manager::CbuStateManager;

/// CBU DSL Management Application - Simplified for CBU-only functionality
pub struct DataDesignerWebApp {
    router: WebRouter,

    // Central state manager - single source of truth
    state: CbuStateManager,

    // CBU DSL IDE - UI only, references state
    cbu_dsl_ide: CbuDslIDE,
}

impl DataDesignerWebApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting CBU DSL Management App with centralized state");

        let grpc_client = Some(GrpcClient::new("http://localhost:8080"));

        Self {
            router: WebRouter::new(),
            state: CbuStateManager::new(grpc_client),
            cbu_dsl_ide: CbuDslIDE::new(),
        }
    }
}

impl eframe::App for DataDesignerWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Force continuous repainting to ensure responsiveness
        ctx.request_repaint();

        // Update state from async operations (polling pattern - will be improved)
        self.state.update_from_async();

        // Top panel with simple title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🏢 CBU DSL Management System");
            ui.separator();
        });

        // Main content panel - only CBU DSL IDE
        egui::CentralPanel::default().show(ctx, |ui| {
            // Render UI with state - UI captures intent, state handles logic
            self.cbu_dsl_ide.render(ui, &mut self.state);
        });
    }
}