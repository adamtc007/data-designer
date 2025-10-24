use eframe::egui;
use crate::{WebRouter, wasm_utils};
use crate::grpc_client::GrpcClient;
use crate::cbu_dsl_ide::CbuDslIDE;

/// CBU DSL Management Application - Simplified for CBU-only functionality
pub struct DataDesignerWebApp {
    router: WebRouter,

    // gRPC client for CBU operations
    grpc_client: Option<GrpcClient>,

    // CBU DSL IDE - main functionality
    cbu_dsl_ide: CbuDslIDE,
}

impl DataDesignerWebApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting CBU DSL Management App");

        Self {
            router: WebRouter::new(),
            grpc_client: Some(GrpcClient::new("http://localhost:50051")), // Use gRPC port
            cbu_dsl_ide: CbuDslIDE::new(),
        }
    }
}

impl eframe::App for DataDesignerWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Force continuous repainting to ensure responsiveness
        ctx.request_repaint();

        // Top panel with simple title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🏢 CBU DSL Management System");
            ui.separator();
        });

        // Main content panel - only CBU DSL IDE
        egui::CentralPanel::default().show(ctx, |ui| {
            // Only render CBU DSL IDE
            self.cbu_dsl_ide.render(ui, self.grpc_client.as_ref());
        });
    }
}