/// Call Tree Builder and Execution Tracer
///
/// This module provides debugging utilities to trace function calls,
/// async operations, and state changes for debugging in ZED.
use std::sync::{Arc, Mutex};
use crate::wasm_utils;

// Platform-specific time functions
#[cfg(target_arch = "wasm32")]
fn get_timestamp_ms() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn get_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub file_location: String,
    pub args: Vec<String>,
    pub timestamp: u64,
    pub depth: usize,
    pub thread_id: String,
    pub result: Option<String>,
    pub duration_ms: Option<u64>,
    pub children: Vec<CallFrame>,
}

#[derive(Debug, Clone)]
pub struct StateChange {
    pub component: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: u64,
    pub stack_trace: Vec<String>,
}

#[derive(Debug)]
pub struct CallTracer {
    call_stack: Vec<CallFrame>,
    state_changes: Vec<StateChange>,
    current_depth: usize,
    start_time: u64,
    max_stack_size: usize,
    max_state_changes: usize,
}

impl Default for CallTracer {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_TRACER: std::sync::OnceLock<Arc<Mutex<CallTracer>>> = std::sync::OnceLock::new();

impl CallTracer {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            state_changes: Vec::new(),
            current_depth: 0,
            start_time: get_timestamp_ms(),
            max_stack_size: 1000,      // Limit to last 1000 calls
            max_state_changes: 1000,   // Limit to last 1000 state changes
        }
    }

    pub fn global() -> Arc<Mutex<CallTracer>> {
        GLOBAL_TRACER.get_or_init(|| Arc::new(Mutex::new(CallTracer::new()))).clone()
    }

    pub fn enter_function(&mut self, function_name: &str, file_location: &str, args: Vec<String>) -> usize {
        // Clear old entries if we hit the limit
        if self.call_stack.len() >= self.max_stack_size {
            // Keep only the last half to avoid constant resizing
            let keep_from = self.max_stack_size / 2;
            self.call_stack.drain(0..keep_from);
        }

        let frame = CallFrame {
            function_name: function_name.to_string(),
            file_location: file_location.to_string(),
            args,
            timestamp: get_timestamp_ms() - self.start_time,
            depth: self.current_depth,
            thread_id: "main".to_string(), // WASM is single-threaded
            result: None,
            duration_ms: None,
            children: Vec::new(),
        };

        self.call_stack.push(frame);
        self.current_depth += 1;

        let frame_id = self.call_stack.len() - 1;
        wasm_utils::console_log(&format!(
            "🔍 ENTER[{}] {}:{} ({})",
            self.current_depth, function_name, file_location,
            self.call_stack[frame_id].args.join(", ")
        ));

        frame_id
    }

    pub fn exit_function(&mut self, frame_id: usize, result: Option<String>) {
        if frame_id >= self.call_stack.len() {
            return;
        }

        let start_time = self.call_stack[frame_id].timestamp;
        let duration = get_timestamp_ms() - self.start_time - start_time;

        self.call_stack[frame_id].result = result.clone();
        self.call_stack[frame_id].duration_ms = Some(duration);

        self.current_depth = self.current_depth.saturating_sub(1);

        wasm_utils::console_log(&format!(
            "🔍 EXIT[{}] {} -> {} ({}ms)",
            self.current_depth + 1,
            self.call_stack[frame_id].function_name,
            result.unwrap_or_else(|| "()".to_string()),
            duration
        ));
    }

    pub fn log_state_change(&mut self, component: &str, field: &str, old_value: &str, new_value: &str) {
        // Clear old entries if we hit the limit
        if self.state_changes.len() >= self.max_state_changes {
            // Keep only the last half to avoid constant resizing
            let keep_from = self.max_state_changes / 2;
            self.state_changes.drain(0..keep_from);
        }

        let change = StateChange {
            component: component.to_string(),
            field: field.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
            timestamp: get_timestamp_ms() - self.start_time,
            stack_trace: self.get_current_stack_trace(),
        };

        wasm_utils::console_log(&format!(
            "📝 STATE[{}] {}.{}: '{}' -> '{}'",
            self.current_depth, component, field, old_value, new_value
        ));

        self.state_changes.push(change);
    }

    pub fn log_async_operation(&mut self, operation: &str, status: &str, data: Option<&str>) {
        wasm_utils::console_log(&format!(
            "⚡ ASYNC[{}] {} -> {} {}",
            self.current_depth,
            operation,
            status,
            data.map(|d| format!("({})", d)).unwrap_or_default()
        ));
    }

    pub fn log_grpc_call(&mut self, endpoint: &str, request_size: usize, response_status: &str) {
        wasm_utils::console_log(&format!(
            "🌐 GRPC[{}] {} ({} bytes) -> {}",
            self.current_depth, endpoint, request_size, response_status
        ));
    }

    pub fn log_ui_event(&mut self, event_type: &str, component: &str, details: &str) {
        wasm_utils::console_log(&format!(
            "🖱️ UI[{}] {} in {} ({})",
            self.current_depth, event_type, component, details
        ));
    }

    fn get_current_stack_trace(&self) -> Vec<String> {
        self.call_stack
            .iter()
            .map(|frame| format!("{}:{}", frame.function_name, frame.file_location))
            .collect()
    }

    pub fn dump_call_tree(&self) -> String {
        let mut output = String::new();
        output.push_str("📋 CALL TREE:\n");

        for (i, frame) in self.call_stack.iter().enumerate() {
            let indent = "  ".repeat(frame.depth);
            let duration = frame.duration_ms.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
            let result = frame.result.as_ref().map(|r| format!(" -> {}", r)).unwrap_or_default();

            output.push_str(&format!(
                "{}{}. {}:{}{}{}\n",
                indent, i, frame.function_name, frame.file_location, result, duration
            ));
        }

        output.push_str("\n📝 STATE CHANGES:\n");
        for change in &self.state_changes {
            output.push_str(&format!(
                "  {}.{}: '{}' -> '{}' ({}ms)\n",
                change.component, change.field, change.old_value, change.new_value, change.timestamp
            ));
        }

        output
    }

    pub fn export_for_zed(&self) -> String {
        // Export in a format that ZED can use for step-through debugging
        let mut output = String::new();

        output.push_str("// Call Tree Export for ZED\n");
        output.push_str("// Use this to trace through the execution flow\n\n");

        for frame in &self.call_stack {
            output.push_str(&format!(
                "// {}:{}:0 - {}({}) -> {}\n",
                frame.file_location,
                0, // Line number placeholder
                frame.function_name,
                frame.args.join(", "),
                frame.result.as_ref().unwrap_or(&"?".to_string())
            ));
        }

        output.push_str("\n// Critical State Changes:\n");
        for change in &self.state_changes {
            if change.component.contains("cbu") || change.component.contains("entity") {
                output.push_str(&format!(
                    "// {}: {}.{} = '{}'\n",
                    change.timestamp, change.component, change.field, change.new_value
                ));
            }
        }

        output
    }
}

// Convenience macros for easy tracing
// Tagged with #[cfg(debug_assertions)] so they compile out in release builds
#[macro_export]
macro_rules! trace_enter {
    ($func_name:expr, $file:expr, $($arg:expr),*) => {
        {
            #[cfg(debug_assertions)]
            {
                let args = vec![$(format!("{:?}", $arg)),*];
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.enter_function($func_name, $file, args)
            }
            #[cfg(not(debug_assertions))]
            {
                0 // Return dummy frame_id in release builds
            }
        }
    };
}

#[macro_export]
macro_rules! trace_exit {
    ($frame_id:expr, $result:expr) => {
        {
            #[cfg(debug_assertions)]
            {
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.exit_function($frame_id, Some(format!("{:?}", $result)));
            }
        }
    };
}

#[macro_export]
macro_rules! trace_state {
    ($component:expr, $field:expr, $old:expr, $new:expr) => {
        {
            #[cfg(debug_assertions)]
            {
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.log_state_change($component, $field, &format!("{:?}", $old), &format!("{:?}", $new));
            }
        }
    };
}

#[macro_export]
macro_rules! trace_async {
    ($operation:expr, $status:expr, $data:expr) => {
        {
            #[cfg(debug_assertions)]
            {
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.log_async_operation($operation, $status, $data);
            }
        }
    };
}

#[macro_export]
macro_rules! trace_grpc {
    ($endpoint:expr, $req_size:expr, $status:expr) => {
        {
            #[cfg(debug_assertions)]
            {
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.log_grpc_call($endpoint, $req_size, $status);
            }
        }
    };
}

#[macro_export]
macro_rules! trace_ui {
    ($event:expr, $component:expr, $details:expr) => {
        {
            #[cfg(debug_assertions)]
            {
                let tracer = $crate::call_tracer::CallTracer::global();
                let mut t = tracer.lock().unwrap();
                t.log_ui_event($event, $component, $details);
            }
        }
    };
}

// Function to dump trace to console for ZED
#[cfg(debug_assertions)]
pub fn dump_trace_for_zed() {
    let tracer = CallTracer::global();
    let t = tracer.lock().unwrap();
    let export = t.export_for_zed();
    crate::wasm_utils::console_log("=== ZED TRACE EXPORT ===");
    crate::wasm_utils::console_log(&export);
    crate::wasm_utils::console_log("=== END TRACE ===");
}

#[cfg(not(debug_assertions))]
pub fn dump_trace_for_zed() {
    // No-op in release builds
}