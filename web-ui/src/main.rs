//! Native Desktop Version of Data Designer
//!
//! This provides the same egui application as the WASM version but running natively
//! for better debugging capabilities with full IDE integration, breakpoints, and profiling.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod grpc_client;
mod cbu_state_manager;
mod resource_state_manager;
mod cbu_dsl_ide;
mod resource_dsl_ide;
mod dsl_syntax_highlighter;
mod dsl_state_manager;
mod call_tracer;
mod wasm_utils;

use cbu_dsl_ide::CbuDslIDE;
use cbu_state_manager::CbuStateManager;
use resource_dsl_ide::ResourceDslIDE;
use resource_state_manager::ResourceStateManager;
use grpc_client::GrpcClient;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveView {
    Cbu,
    Resource,
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
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
    active_view: ActiveView,

    // Dual state managers - single source of truth
    cbu_state: CbuStateManager,
    resource_state: ResourceStateManager,

    // IDE components - UI only
    cbu_dsl_ide: CbuDslIDE,
    resource_dsl_ide: ResourceDslIDE,

    grpc_endpoint: String,
    connection_status: String,
}

impl DataDesignerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Default HTTP endpoint for local development (gRPC server exposes HTTP on 8080)
        let grpc_endpoint = "http://localhost:8080".to_string();
        let grpc_client = GrpcClient::new(&grpc_endpoint);

        Self {
            active_view: ActiveView::Cbu,
            cbu_state: CbuStateManager::new(Some(grpc_client.clone())),
            resource_state: ResourceStateManager::new(Some(grpc_client)),
            cbu_dsl_ide: CbuDslIDE::new(),
            resource_dsl_ide: ResourceDslIDE::new(),
            grpc_endpoint,
            connection_status: "Connected to localhost:8080 (HTTP/gRPC bridge)".to_string(),
        }
    }
}

impl eframe::App for DataDesignerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update state from async operations for both managers
        self.cbu_state.update_from_async();
        self.resource_state.update_from_async();

        // Top panel with connection info and view tabs
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
                        let grpc_client = GrpcClient::new(&self.grpc_endpoint);
                        self.cbu_state = CbuStateManager::new(Some(grpc_client.clone()));
                        self.resource_state = ResourceStateManager::new(Some(grpc_client));
                        self.connection_status = format!("Connected to {}", self.grpc_endpoint);
                    }

                    if ui.button("Disconnect").clicked() {
                        self.cbu_state = CbuStateManager::new(None);
                        self.resource_state = ResourceStateManager::new(None);
                        self.connection_status = "Disconnected".to_string();
                    }
                });

                ui.separator();
                ui.label(&self.connection_status);

                ui.separator();
                ui.label("🦀 Desktop Edition - wgpu Renderer");
            });

            ui.separator();

            // View tabs - CBU vs Resource
            ui.horizontal(|ui| {
                ui.heading("🏢 Data Designer");
                ui.separator();

                if ui.selectable_label(
                    self.active_view == ActiveView::Cbu,
                    "📋 CBU DSL"
                ).clicked() {
                    self.active_view = ActiveView::Cbu;
                }

                if ui.selectable_label(
                    self.active_view == ActiveView::Resource,
                    "🔧 Resource DSL"
                ).clicked() {
                    self.active_view = ActiveView::Resource;
                }
            });
        });

        // Main content area - render active view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_view {
                ActiveView::Cbu => {
                    self.cbu_dsl_ide.render(ui, &mut self.cbu_state);
                }
                ActiveView::Resource => {
                    self.resource_dsl_ide.render(ui, &mut self.resource_state);
                }
            }
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.cbu_state.get_grpc_client().is_some() {
                    ui.colored_label(egui::Color32::GREEN, "✅ gRPC Connected");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ gRPC Disconnected");
                }

                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| i.stable_dt).recip()));

                ui.separator();
                ui.label("Press F11 for fullscreen");

                ui.separator();
                match self.active_view {
                    ActiveView::Cbu => ui.label("Active: CBU DSL"),
                    ActiveView::Resource => ui.label("Active: Resource DSL"),
                };
            });
        });
    }
}