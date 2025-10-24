#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

mod app;
mod grpc_client;
mod cbu_state_manager;
mod resource_state_manager;
mod cbu_dsl_ide;
mod resource_dsl_ide;
mod dsl_syntax_highlighter;
mod dsl_state_manager;
mod call_tracer;
pub mod wasm_utils;


/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    // Redirect `log` message to `console.log` and friends:
    #[cfg(target_arch = "wasm32")]
    // Initialize logging for WASM
    #[cfg(feature = "log")]
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    #[cfg(target_arch = "wasm32")]
    let web_options = eframe::WebOptions::default();

    let canvas_id = canvas_id.to_string();
    wasm_utils::spawn_async(async move {
        let _canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&canvas_id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Failed to find canvas element");

        #[cfg(target_arch = "wasm32")]
        let start_result = eframe::WebRunner::new()
            .start(
                _canvas,
                web_options,
                Box::new(|cc| {
                    // Set up dark theme
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());

                    Ok(Box::new(app::DataDesignerWebApp::new(cc)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        #[cfg(target_arch = "wasm32")]
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
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });

    Ok(())
}

/// Simple web routing without external dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum AppRoute {
    CbuDslIde,           // CBU DSL management IDE - main and only route
}

pub struct WebRouter {
    current_route: AppRoute,
}

impl Default for WebRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl WebRouter {
    pub fn new() -> Self {
        Self {
            current_route: AppRoute::CbuDslIde,
        }
    }

    pub fn navigate_to(&mut self, route: AppRoute) {
        self.current_route = route;
    }

    pub fn current_route(&self) -> &AppRoute {
        &self.current_route
    }
}

