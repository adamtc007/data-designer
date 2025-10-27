// Desktop entry point for onboarding UI
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod http_client;
pub mod onboarding;
mod state_manager;
pub mod wasm_utils;

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Onboarding Workflow Platform"),
        ..Default::default()
    };

    eframe::run_native(
        "Onboarding Platform",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::OnboardingApp::new(cc)))
        }),
    )
}

#[cfg(not(feature = "tokio"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Onboarding Workflow Platform"),
        ..Default::default()
    };

    eframe::run_native(
        "Onboarding Platform",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(app::OnboardingApp::new(cc)))
        }),
    )
}
