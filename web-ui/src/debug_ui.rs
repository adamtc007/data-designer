use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::wasm_utils;

/// Comprehensive debugging and testing UI components
#[derive(Default)]
pub struct DebugTestInterface {
    // Test execution state
    pub active_tests: HashMap<String, TestExecution>,
    pub test_history: Vec<TestResult>,
    pub show_test_runner: bool,

    // Elasticsearch log analysis
    pub elasticsearch_logs: Vec<LogEntry>,
    pub log_filter: String,
    pub show_log_viewer: bool,

    // gRPC monitoring
    pub grpc_requests: Vec<GrpcRequest>,
    pub grpc_responses: Vec<GrpcResponse>,
    pub show_grpc_monitor: bool,

    // Performance metrics
    pub performance_metrics: PerformanceMetrics,
    pub show_performance_panel: bool,

    // End-to-end tracing
    pub active_traces: HashMap<String, TraceFlow>,
    pub show_trace_viewer: bool,

    // Error tracking
    pub error_log: Vec<ErrorEntry>,
    pub show_error_panel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecution {
    pub test_id: String,
    pub test_name: String,
    pub status: TestStatus,
    pub start_time: String,
    pub duration_ms: Option<u64>,
    pub trace_id: Option<String>,
    pub steps: Vec<TestStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Running,
    Passed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStep {
    pub step_name: String,
    pub status: TestStatus,
    pub duration_ms: Option<u64>,
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub timestamp: String,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub component: String,
    pub message: String,
    pub trace_id: Option<String>,
    pub test_run_id: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcRequest {
    pub request_id: String,
    pub method: String,
    pub timestamp: String,
    pub payload: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcResponse {
    pub request_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub payload: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_request_time: f64,
    pub max_request_time: u64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFlow {
    pub trace_id: String,
    pub test_name: String,
    pub status: String,
    pub start_time: String,
    pub components: Vec<TraceComponent>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceComponent {
    pub component: String,
    pub event_type: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    pub timestamp: String,
    pub component: String,
    pub error_type: String,
    pub message: String,
    pub trace_id: Option<String>,
    pub stack_trace: Option<String>,
}

impl DebugTestInterface {
    pub fn new() -> Self {
        Self::default()
    }

    /// Main debugging interface - renders all debug panels
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔍 Debug & Testing Interface");
        ui.separator();

        // Control panel for different views
        ui.horizontal(|ui| {
            ui.toggle_value(&mut self.show_test_runner, "🧪 Test Runner");
            ui.toggle_value(&mut self.show_log_viewer, "📋 Logs");
            ui.toggle_value(&mut self.show_grpc_monitor, "🌐 gRPC Monitor");
            ui.toggle_value(&mut self.show_performance_panel, "📊 Performance");
            ui.toggle_value(&mut self.show_trace_viewer, "🔍 Traces");
            ui.toggle_value(&mut self.show_error_panel, "❌ Errors");
        });

        ui.separator();

        // Render active panels
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.show_test_runner {
                self.render_test_runner(ui);
                ui.separator();
            }

            if self.show_log_viewer {
                self.render_log_viewer(ui);
                ui.separator();
            }

            if self.show_grpc_monitor {
                self.render_grpc_monitor(ui);
                ui.separator();
            }

            if self.show_performance_panel {
                self.render_performance_panel(ui);
                ui.separator();
            }

            if self.show_trace_viewer {
                self.render_trace_viewer(ui);
                ui.separator();
            }

            if self.show_error_panel {
                self.render_error_panel(ui);
            }
        });
    }

    /// Test Runner Panel - Execute and monitor tests
    fn render_test_runner(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🧪 Test Runner", |ui| {
            ui.horizontal(|ui| {
                if ui.button("▶️ Run Complete Pipeline").clicked() {
                    self.start_complete_pipeline_test();
                }
                if ui.button("🔄 Run Performance Test").clicked() {
                    self.start_performance_test();
                }
                if ui.button("🧠 Run AI Integration Test").clicked() {
                    self.start_ai_integration_test();
                }
                if ui.button("🗑️ Clear Results").clicked() {
                    self.test_history.clear();
                    self.active_tests.clear();
                }
            });

            ui.add_space(10.0);

            // Active tests
            if !self.active_tests.is_empty() {
                ui.heading("🔄 Running Tests");
                for (_test_id, test) in &self.active_tests {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&test.test_name);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                match test.status {
                                    TestStatus::Running => {
                                        ui.spinner();
                                        ui.colored_label(egui::Color32::YELLOW, "Running");
                                    }
                                    TestStatus::Passed => {
                                        ui.colored_label(egui::Color32::GREEN, "✅ Passed");
                                    }
                                    TestStatus::Failed => {
                                        ui.colored_label(egui::Color32::RED, "❌ Failed");
                                    }
                                    TestStatus::Timeout => {
                                        ui.colored_label(egui::Color32::ORANGE, "⏰ Timeout");
                                    }
                                }
                                if let Some(trace_id) = &test.trace_id {
                                    ui.small(format!("Trace: {}", &trace_id[..8]));
                                }
                            });
                        });

                        // Show test steps
                        if !test.steps.is_empty() {
                            ui.indent("test_steps", |ui| {
                                for step in &test.steps {
                                    ui.horizontal(|ui| {
                                        match step.status {
                                            TestStatus::Running => ui.label("🔄"),
                                            TestStatus::Passed => ui.label("✅"),
                                            TestStatus::Failed => ui.label("❌"),
                                            TestStatus::Timeout => ui.label("⏰"),
                                        };
                                        ui.label(&step.step_name);
                                        if let Some(duration) = step.duration_ms {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.small(format!("{}ms", duration));
                                            });
                                        }
                                    });
                                    if let Some(error) = &step.error {
                                        ui.small(egui::RichText::new(error).color(egui::Color32::RED));
                                    }
                                }
                            });
                        }
                    });
                }
            }

            // Test history
            if !self.test_history.is_empty() {
                ui.add_space(10.0);
                ui.heading("📊 Test History");
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for result in &self.test_history {
                        ui.horizontal(|ui| {
                            match result.status {
                                TestStatus::Passed => ui.label("✅"),
                                TestStatus::Failed => ui.label("❌"),
                                TestStatus::Timeout => ui.label("⏰"),
                                TestStatus::Running => ui.label("🔄"),
                            };
                            ui.label(&result.test_name);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.small(format!("{}ms", result.duration_ms));
                                ui.small(&result.timestamp);
                            });
                        });
                        if let Some(error) = &result.error_message {
                            ui.small(egui::RichText::new(error).color(egui::Color32::RED));
                        }
                    }
                });
            }
        });
    }

    /// Log Viewer Panel - View and filter Elasticsearch logs
    fn render_log_viewer(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📋 Elasticsearch Log Viewer", |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.log_filter);
                if ui.button("🔄 Refresh").clicked() {
                    self.fetch_elasticsearch_logs();
                }
                if ui.button("🗑️ Clear").clicked() {
                    self.elasticsearch_logs.clear();
                }
            });

            ui.add_space(10.0);

            if self.elasticsearch_logs.is_empty() {
                ui.label("No logs available. Click 'Refresh' to fetch from Elasticsearch.");
            } else {
                ui.label(format!("Showing {} log entries", self.elasticsearch_logs.len()));
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    egui::Grid::new("log_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            // Header
                            ui.strong("Time");
                            ui.strong("Level");
                            ui.strong("Component");
                            ui.strong("Message");
                            ui.strong("Trace");
                            ui.end_row();

                            // Log entries
                            for log in &self.elasticsearch_logs {
                                if self.log_filter.is_empty() ||
                                   log.message.contains(&self.log_filter) ||
                                   log.component.contains(&self.log_filter) {

                                    ui.small(&log.timestamp[11..19]); // Show time only

                                    let level_color = match log.level.as_str() {
                                        "Error" => egui::Color32::RED,
                                        "Warn" => egui::Color32::YELLOW,
                                        "Info" => egui::Color32::GREEN,
                                        _ => egui::Color32::GRAY,
                                    };
                                    ui.colored_label(level_color, &log.level);

                                    ui.small(&log.component);
                                    ui.small(&log.message);

                                    if let Some(trace_id) = &log.trace_id {
                                        ui.small(&trace_id[..8]);
                                    } else {
                                        ui.small("-");
                                    }
                                    ui.end_row();
                                }
                            }
                        });
                });
            }
        });
    }

    /// gRPC Monitor Panel - Monitor gRPC requests and responses
    fn render_grpc_monitor(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🌐 gRPC Request Monitor", |ui| {
            ui.horizontal(|ui| {
                if ui.button("🗑️ Clear Requests").clicked() {
                    self.grpc_requests.clear();
                    self.grpc_responses.clear();
                }
                ui.label(format!("Requests: {}", self.grpc_requests.len()));
            });

            ui.add_space(10.0);

            if !self.grpc_requests.is_empty() {
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for request in &self.grpc_requests {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(&request.method);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.small(&request.timestamp);
                                    if let Some(trace_id) = &request.trace_id {
                                        ui.small(format!("Trace: {}", &trace_id[..8]));
                                    }
                                });
                            });

                            // Find corresponding response
                            if let Some(response) = self.grpc_responses.iter()
                                .find(|r| r.request_id == request.request_id) {
                                ui.horizontal(|ui| {
                                    let status_color = if response.status == "OK" {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::RED
                                    };
                                    ui.colored_label(status_color, &response.status);
                                    ui.label(format!("{}ms", response.duration_ms));
                                    if let Some(error) = &response.error {
                                        ui.colored_label(egui::Color32::RED, error);
                                    }
                                });
                            } else {
                                ui.colored_label(egui::Color32::YELLOW, "Pending...");
                            }

                            ui.collapsing("Request Details", |ui| {
                                ui.small(&request.payload);
                            });
                        });
                    }
                });
            }
        });
    }

    /// Performance Panel - Show performance metrics and benchmarks
    fn render_performance_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📊 Performance Metrics", |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔄 Update Metrics").clicked() {
                    self.update_performance_metrics();
                }
                if ui.button("📈 Run Benchmark").clicked() {
                    self.run_performance_benchmark();
                }
            });

            ui.add_space(10.0);

            egui::Grid::new("perf_grid").show(ui, |ui| {
                ui.label("Average Request Time:");
                ui.label(format!("{:.2}ms", self.performance_metrics.avg_request_time));
                ui.end_row();

                ui.label("Max Request Time:");
                ui.label(format!("{}ms", self.performance_metrics.max_request_time));
                ui.end_row();

                ui.label("Total Requests:");
                ui.label(format!("{}", self.performance_metrics.total_requests));
                ui.end_row();

                ui.label("Failed Requests:");
                ui.label(format!("{}", self.performance_metrics.failed_requests));
                ui.end_row();

                ui.label("Success Rate:");
                let success_color = if self.performance_metrics.success_rate > 0.95 {
                    egui::Color32::GREEN
                } else if self.performance_metrics.success_rate > 0.8 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(success_color, format!("{:.1}%", self.performance_metrics.success_rate * 100.0));
                ui.end_row();

                ui.label("Memory Usage:");
                ui.label(format!("{:.1} MB", self.performance_metrics.memory_usage_mb));
                ui.end_row();
            });
        });
    }

    /// Trace Viewer Panel - Visualize end-to-end traces
    fn render_trace_viewer(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🔍 End-to-End Trace Viewer", |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh Traces").clicked() {
                    self.fetch_active_traces();
                }
                if ui.button("🗑️ Clear Traces").clicked() {
                    self.active_traces.clear();
                }
            });

            ui.add_space(10.0);

            if self.active_traces.is_empty() {
                ui.label("No active traces. Run tests to generate trace data.");
            } else {
                for (trace_id, trace) in &self.active_traces {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&trace.test_name);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if let Some(duration) = trace.total_duration_ms {
                                    ui.label(format!("{}ms", duration));
                                }
                                ui.small(format!("Trace: {}", &trace_id[..8]));
                            });
                        });

                        // Trace timeline
                        ui.collapsing("Component Timeline", |ui| {
                            for component in &trace.components {
                                ui.horizontal(|ui| {
                                    ui.label(&component.component);
                                    ui.label(&component.event_type);
                                    if let Some(duration) = component.duration_ms {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(format!("{}ms", duration));
                                        });
                                    }
                                    if let Some(_error) = &component.error {
                                        ui.colored_label(egui::Color32::RED, "❌");
                                    }
                                });
                                if let Some(data) = &component.data {
                                    ui.small(data);
                                }
                            }
                        });
                    });
                }
            }
        });
    }

    /// Error Panel - Track and analyze errors
    fn render_error_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("❌ Error Tracking", |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh Errors").clicked() {
                    self.fetch_error_logs();
                }
                if ui.button("🗑️ Clear Errors").clicked() {
                    self.error_log.clear();
                }
            });

            ui.add_space(10.0);

            if self.error_log.is_empty() {
                ui.colored_label(egui::Color32::GREEN, "✅ No errors detected");
            } else {
                ui.colored_label(egui::Color32::RED, format!("❌ {} errors found", self.error_log.len()));
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for error in &self.error_log {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::RED, &error.error_type);
                                ui.label(&error.component);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.small(&error.timestamp);
                                });
                            });
                            ui.label(&error.message);
                            if let Some(trace_id) = &error.trace_id {
                                ui.small(format!("Trace: {}", trace_id));
                            }
                            if let Some(stack) = &error.stack_trace {
                                ui.collapsing("Stack Trace", |ui| {
                                    ui.small(stack);
                                });
                            }
                        });
                    }
                });
            }
        });
    }

    // Test execution methods
    fn start_complete_pipeline_test(&mut self) {
        let test_id = format!("pipeline_test_{}", js_sys::Date::now());
        let trace_id = format!("trace_{}", uuid::Uuid::new_v4());

        let test = TestExecution {
            test_id: test_id.clone(),
            test_name: "Complete Data Pipeline Test".to_string(),
            status: TestStatus::Running,
            start_time: self.current_timestamp(),
            duration_ms: None,
            trace_id: Some(trace_id),
            steps: vec![
                TestStep {
                    step_name: "AI Suggestions".to_string(),
                    status: TestStatus::Running,
                    duration_ms: None,
                    data: None,
                    error: None,
                },
                TestStep {
                    step_name: "Template Instantiation".to_string(),
                    status: TestStatus::Running,
                    duration_ms: None,
                    data: None,
                    error: None,
                },
                TestStep {
                    step_name: "DSL Execution".to_string(),
                    status: TestStatus::Running,
                    duration_ms: None,
                    data: None,
                    error: None,
                },
            ],
        };

        self.active_tests.insert(test_id, test);
        wasm_utils::console_log("🧪 Started complete pipeline test");
    }

    fn start_performance_test(&mut self) {
        let test_id = format!("perf_test_{}", js_sys::Date::now());

        let test = TestExecution {
            test_id: test_id.clone(),
            test_name: "Performance Benchmark Test".to_string(),
            status: TestStatus::Running,
            start_time: self.current_timestamp(),
            duration_ms: None,
            trace_id: None,
            steps: vec![
                TestStep {
                    step_name: "Load Testing".to_string(),
                    status: TestStatus::Running,
                    duration_ms: None,
                    data: None,
                    error: None,
                },
            ],
        };

        self.active_tests.insert(test_id, test);
        wasm_utils::console_log("🏁 Started performance test");
    }

    fn start_ai_integration_test(&mut self) {
        let test_id = format!("ai_test_{}", js_sys::Date::now());

        let test = TestExecution {
            test_id: test_id.clone(),
            test_name: "AI Integration Test".to_string(),
            status: TestStatus::Running,
            start_time: self.current_timestamp(),
            duration_ms: None,
            trace_id: None,
            steps: vec![
                TestStep {
                    step_name: "AI Provider Connection".to_string(),
                    status: TestStatus::Running,
                    duration_ms: None,
                    data: None,
                    error: None,
                },
            ],
        };

        self.active_tests.insert(test_id, test);
        wasm_utils::console_log("🧠 Started AI integration test");
    }

    // Data fetching methods (simulated - would connect to real Elasticsearch in production)
    fn fetch_elasticsearch_logs(&mut self) {
        // Simulate fetching logs from Elasticsearch
        self.elasticsearch_logs = vec![
            LogEntry {
                timestamp: self.current_timestamp(),
                level: "Info".to_string(),
                component: "grpc_server".to_string(),
                message: "Rule executed successfully".to_string(),
                trace_id: Some("trace_12345".to_string()),
                test_run_id: Some("test_67890".to_string()),
                data: Some(r#"{"execution_time_ms": 45}"#.to_string()),
            },
            LogEntry {
                timestamp: self.current_timestamp(),
                level: "Error".to_string(),
                component: "web_ui".to_string(),
                message: "Failed to connect to gRPC server".to_string(),
                trace_id: None,
                test_run_id: None,
                data: None,
            },
        ];
        wasm_utils::console_log("📋 Fetched Elasticsearch logs");
    }

    fn fetch_active_traces(&mut self) {
        // Simulate fetching traces
        let trace_id = "trace_12345".to_string();
        let trace = TraceFlow {
            trace_id: trace_id.clone(),
            test_name: "Complete Pipeline Test".to_string(),
            status: "Completed".to_string(),
            start_time: self.current_timestamp(),
            total_duration_ms: Some(245),
            components: vec![
                TraceComponent {
                    component: "web_ui".to_string(),
                    event_type: "TestStart".to_string(),
                    timestamp: self.current_timestamp(),
                    duration_ms: Some(5),
                    data: Some("Test initiated".to_string()),
                    error: None,
                },
                TraceComponent {
                    component: "grpc_server".to_string(),
                    event_type: "RuleExecution".to_string(),
                    timestamp: self.current_timestamp(),
                    duration_ms: Some(45),
                    data: Some("Rule executed".to_string()),
                    error: None,
                },
            ],
        };
        self.active_traces.insert(trace_id, trace);
        wasm_utils::console_log("🔍 Fetched active traces");
    }

    fn fetch_error_logs(&mut self) {
        // Simulate fetching error logs
        self.error_log = vec![
            ErrorEntry {
                timestamp: self.current_timestamp(),
                component: "grpc_client".to_string(),
                error_type: "ConnectionError".to_string(),
                message: "Connection refused to localhost:50051".to_string(),
                trace_id: Some("trace_error_123".to_string()),
                stack_trace: Some("at grpc_client.rs:123\nat app.rs:456".to_string()),
            },
        ];
        wasm_utils::console_log("❌ Fetched error logs");
    }

    fn update_performance_metrics(&mut self) {
        // Simulate updating performance metrics
        self.performance_metrics = PerformanceMetrics {
            avg_request_time: 125.5,
            max_request_time: 450,
            total_requests: 234,
            failed_requests: 12,
            success_rate: 0.95,
            memory_usage_mb: 45.2,
            cpu_usage_percent: 23.1,
        };
        wasm_utils::console_log("📊 Updated performance metrics");
    }

    fn run_performance_benchmark(&mut self) {
        wasm_utils::console_log("📈 Running performance benchmark...");
        // Would trigger actual performance tests
    }

    fn current_timestamp(&self) -> String {
        js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default()
    }

    /// Add a new gRPC request to monitoring
    pub fn log_grpc_request(&mut self, method: &str, payload: &str, trace_id: Option<String>) -> String {
        let request_id = format!("req_{}", js_sys::Date::now());
        let request = GrpcRequest {
            request_id: request_id.clone(),
            method: method.to_string(),
            timestamp: self.current_timestamp(),
            payload: payload.to_string(),
            trace_id,
        };
        self.grpc_requests.push(request);
        request_id
    }

    /// Add a gRPC response to monitoring
    pub fn log_grpc_response(&mut self, request_id: String, status: &str, duration_ms: u64, payload: &str, error: Option<String>) {
        let response = GrpcResponse {
            request_id,
            status: status.to_string(),
            duration_ms,
            payload: payload.to_string(),
            error,
        };
        self.grpc_responses.push(response);
    }

    /// Add an error to tracking
    pub fn log_error(&mut self, component: &str, error_type: &str, message: &str, trace_id: Option<String>) {
        let error = ErrorEntry {
            timestamp: self.current_timestamp(),
            component: component.to_string(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            trace_id,
            stack_trace: None,
        };
        self.error_log.push(error);
    }
}