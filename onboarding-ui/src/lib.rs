#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod app;
mod http_client;
pub mod onboarding;
mod state_manager;
pub mod wasm_utils;

/// WASM entry point for onboarding UI
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    #[cfg(feature = "log")]
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    let canvas_id = canvas_id.to_string();

    wasm_utils::spawn_async(async move {
        let _canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&canvas_id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Failed to find canvas element");

        let start_result = eframe::WebRunner::new()
            .start(
                _canvas,
                web_options,
                Box::new(|cc| {
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());
                    Ok(Box::new(app::OnboardingApp::new(cc)))
                }),
            )
            .await;

        if let Some(loading_text) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"))
        {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p>The app has crashed. See the developer console for details.</p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });

    Ok(())
}
