//! Native Desktop Version of Data Designer
//!
//! This provides the same egui application as the WASM version but running natively
//! for better debugging capabilities with full IDE integration, breakpoints, and profiling.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod grpc_client;
mod cbu_dsl_ide;
mod dsl_syntax_highlighter;
mod dsl_state_manager;
mod call_tracer;
mod wasm_utils;

use cbu_dsl_ide::CbuDslIDE;
use grpc_client::GrpcClient;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Data Designer - Desktop Edition",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(DataDesignerApp::new(cc)))
        }),
    )
}

struct DataDesignerApp {
    cbu_dsl_ide: CbuDslIDE,
    grpc_client: Option<GrpcClient>,
    grpc_endpoint: String,
    connection_status: String,
}

impl DataDesignerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Default HTTP endpoint for local development (gRPC server exposes HTTP on 8080)
        let grpc_endpoint = "http://localhost:8080".to_string();
        let grpc_client = Some(GrpcClient::new(&grpc_endpoint));

        Self {
            cbu_dsl_ide: CbuDslIDE::new(),
            grpc_client,
            grpc_endpoint,
            connection_status: "Connected to localhost:8080 (HTTP/gRPC bridge)".to_string(),
        }
    }
}

impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with connection info
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Connection", |ui| {
                    ui.label("gRPC Endpoint:");
                    ui.text_edit_singleline(&mut self.grpc_endpoint);

                    if ui.button("Reconnect").clicked() {
                        self.grpc_client = Some(GrpcClient::new(&self.grpc_endpoint));
                        self.connection_status = format!("Connected to {}", self.grpc_endpoint);
                    }

                    if ui.button("Disconnect").clicked() {
                        self.grpc_client = None;
                        self.connection_status = "Disconnected".to_string();
                    }
                });

                ui.separator();
                ui.label(&self.connection_status);

                ui.separator();
                ui.label("🦀 Desktop Edition - Full Debugging");
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CBU DSL IDE - Desktop Edition");
            ui.separator();

            // Render the same CBU DSL IDE as the WASM version
            self.cbu_dsl_ide.render(ui, self.grpc_client.as_ref());
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.grpc_client.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ gRPC Connected");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ gRPC Disconnected");
                }

                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| i.stable_dt).recip()));

                ui.separator();
                ui.label("Press F11 for fullscreen");
            });
        });
    }
}