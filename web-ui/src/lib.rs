use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod app;
mod resource_sheet_ui;
mod minimal_types;
mod http_api_client;
mod dsl_syntax_test;
mod grpc_client;
mod debug_ui;
mod template_designer;
mod data_designer;
mod entity_management;
mod cbu_dsl_ide;
mod dsl_syntax_highlighter;
mod dsl_state_manager;
mod call_tracer;


/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    // Redirect `log` message to `console.log` and friends:
    #[cfg(target_arch = "wasm32")]
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    #[cfg(target_arch = "wasm32")]
    let web_options = eframe::WebOptions::default();

    let canvas_id = canvas_id.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&canvas_id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Failed to find canvas element");

        #[cfg(target_arch = "wasm32")]
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
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
    Dashboard,
    // Main functional areas
    ResourceTemplates,     // Manage Resource Templates
    PrivateData,          // Manage Private Data
    OnboardingRequests,   // Create Onboarding Request
    // Design tools (accessed from main areas)
    TemplateDesigner,     // Template creation tool
    DataDesigner,         // Data design tool
    // Capability Management
    // Entity Management (CRUD)
    CbuDslIde,           // CBU DSL management IDE
    ProductManagement,    // Product entity CRUD
    ServiceManagement,    // Service entity CRUD
    ResourceManagement,   // Resource entity CRUD
    WorkflowManagement,   // Onboarding workflow CRUD
    // Supporting areas
    Transpiler,
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
            current_route: AppRoute::Dashboard,
        }
    }

    pub fn navigate_to(&mut self, route: AppRoute) {
        self.current_route = route;
    }

    pub fn current_route(&self) -> &AppRoute {
        &self.current_route
    }
}

// WASM-specific utilities
pub mod wasm_utils {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }

    pub fn console_log(s: &str) {
        log(s);
    }

    pub fn set_panic_hook() {
        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    pub fn get_browser_storage() -> Option<web_sys::Storage> {
        web_sys::window()?
            .local_storage()
            .ok()?
    }

    pub fn save_to_storage(key: &str, value: &str) -> Result<(), JsValue> {
        if let Some(storage) = get_browser_storage() {
            storage.set_item(key, value)?;
        }
        Ok(())
    }

    pub fn load_from_storage(key: &str) -> Option<String> {
        get_browser_storage()?
            .get_item(key)
            .ok()?
    }

    // Convenience aliases for the async pattern used in entity_management.rs
    pub fn set_local_storage(key: &str, value: &str) {
        let _ = save_to_storage(key, value);
    }

    pub fn get_local_storage(key: &str) -> Option<String> {
        load_from_storage(key)
    }

    pub fn remove_local_storage(key: &str) {
        if let Some(storage) = get_browser_storage() {
            let _ = storage.remove_item(key);
        }
    }
}