//! Cross-platform utilities that work in both WASM and native contexts

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
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
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    pub fn now_timestamp() -> u64 {
        js_sys::Date::now() as u64
    }

    pub fn now_iso_string() -> String {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default()
    }

    pub fn spawn_async<F>(future: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native_impl {
    pub fn console_log(s: &str) {
        // Use standard Rust logging for native
        log::info!("{}", s);
        // Also print to stdout for immediate visibility during debugging
        println!("[DEBUG] {}", s);
    }

    pub fn set_panic_hook() {
        // Native panic hook is already set up by env_logger
    }

    pub fn now_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn now_iso_string() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    pub fn spawn_async<F>(future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(future);
    }
}

// Re-export the appropriate implementation
#[cfg(target_arch = "wasm32")]
pub use wasm_impl::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native_impl::*;