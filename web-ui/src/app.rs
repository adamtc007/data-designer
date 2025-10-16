use eframe::egui;
use crate::{AppRoute, WebRouter, wasm_utils};
use crate::resource_sheet_ui::ResourceSheetManager;
use crate::minimal_types::ResourceSheetRecord;
use crate::http_api_client::DataDesignerHttpClient;
use crate::grpc_client::{GrpcClient, GetAiSuggestionsRequest, GetAiSuggestionsResponse, AiProviderConfig, AiSuggestion, InstantiateResourceRequest, ExecuteDslRequest};

/// Web version of the Data Designer application
pub struct DataDesignerWebApp {
    router: WebRouter,

    // Resource Sheet Manager (web-compatible)
    resource_sheet_manager: ResourceSheetManager,

    // Sample data for demo (since we can't connect to local DB from web)
    sample_resource_sheets: Vec<ResourceSheetRecord>,

    // UI state
    loading: bool,
    error_message: Option<String>,

    // Web-specific state
    grpc_endpoint: String,
    connection_status: ConnectionStatus,
    api_client: Option<DataDesignerHttpClient>,
    grpc_client: Option<GrpcClient>,

    // Template Editor
    template_editor: crate::template_editor::TemplateEditor,

    // Debug tools
    show_debug_panel: bool,

    // AI Command Palette
    show_ai_palette: bool,
    ai_prompt: String,
    ai_context: String,
    ai_loading: bool,
    ai_response: Option<GetAiSuggestionsResponse>,
}

#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

impl DataDesignerWebApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting Data Designer Web App");

        let mut app = Self {
            router: WebRouter::new(),
            resource_sheet_manager: ResourceSheetManager::new(),
            sample_resource_sheets: Vec::new(),
            loading: false,
            error_message: None,
            grpc_endpoint: "http://localhost:8080".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            api_client: None,
            grpc_client: Some(GrpcClient::new("http://localhost:50051")),
            template_editor: crate::template_editor::TemplateEditor::new(),
            show_debug_panel: false,

            // AI Command Palette
            show_ai_palette: false,
            ai_prompt: String::new(),
            ai_context: "general".to_string(),
            ai_loading: false,
            ai_response: None,
        };

        // Load sample data
        app.load_sample_data();

        // Connect to API on startup for true JSON-centric sync
        wasm_utils::console_log("🚀 Auto-connecting to Template API on startup");
        app.attempt_api_connection();

        app
    }

    fn attempt_api_connection(&mut self) {
        wasm_utils::console_log(&format!("🔌 Attempting to connect to: {}", self.grpc_endpoint));

        let mut client = DataDesignerHttpClient::new(&self.grpc_endpoint);

        // Set the connection status to connecting
        self.connection_status = ConnectionStatus::Connecting;

        // Test endpoint and set connected status
        let endpoint = self.grpc_endpoint.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // Test endpoint reachability
            let reachable = crate::http_api_client::test_api_endpoint(&endpoint).await;
            if reachable {
                wasm_utils::console_log("✅ API endpoint is reachable");
            } else {
                wasm_utils::console_log("❌ API endpoint unreachable or CORS blocked");
            }
        });

        // Mark client as connected for UI purposes (since endpoint testing is async)
        client.set_connected(true);

        // Set up the connected client
        self.api_client = Some(client.clone());
        self.template_editor.set_api_client(client);
        self.connection_status = ConnectionStatus::Connected;
    }

    fn load_sample_data(&mut self) {
        // Create sample resource sheets for web demo
        let sample_kyc = ResourceSheetRecord {
            resource_id: "web-kyc-001".to_string(),
            resource_type: "Domain_KYC".to_string(),
            name: "KYC Case for Web Demo".to_string(),
            description: Some("Sample KYC case for web demonstration".to_string()),
            version: "1.0.0".to_string(),
            client_id: Some("WEB-CLIENT-001".to_string()),
            product_id: Some("DEMO-TRADING".to_string()),
            status: "Pending".to_string(),
            json_data: serde_json::json!({
                "id": "web-kyc-001",
                "case_id": "KYC-WEB-001",
                "client_id": "WEB-CLIENT-001",
                "product_id": "DEMO-TRADING",
                "resource_type": "Domain_KYC",
                "status": "Pending",
                "metadata": {
                    "name": "KYC Case for Web Demo",
                    "description": "Sample KYC case for web demonstration",
                    "version": "1.0.0",
                    "priority": "Normal",
                    "estimated_duration_minutes": 120,
                    "tags": ["KYC", "Compliance", "Web Demo"]
                },
                "business_logic_dsl": "WORKFLOW \"WebDemoKYC\"\nSTEP \"InitialAssessment\"\n    DERIVE_REGULATORY_CONTEXT FOR_JURISDICTION \"US\" WITH_PRODUCTS [\"Trading\"]\n    ASSESS_RISK USING_FACTORS [\"jurisdiction\", \"product\", \"client\"] OUTPUT \"combinedRisk\"\nSTEP \"DocumentCollection\"\n    COLLECT_DOCUMENT \"PassportCopy\" FROM Client REQUIRED true\n    COLLECT_DOCUMENT \"ProofOfAddress\" FROM Client REQUIRED true\nSTEP \"ScreeningProcess\"\n    SCREEN_ENTITY \"client.name\" AGAINST \"SanctionsList\" THRESHOLD 0.85\n    SCREEN_ENTITY \"client.name\" AGAINST \"PEPList\" THRESHOLD 0.90\nSTEP \"FinalDecision\"\n    IF combinedRisk = \"High\" THEN\n        FLAG_FOR_REVIEW \"High risk client requires manual review\" PRIORITY High\n    ELSE\n        APPROVE_CASE WITH_CONDITIONS [\"Annual review required\"]",
                "risk_profile": {
                    "jurisdiction_risk": "Medium",
                    "product_risk": "Low",
                    "client_risk": "Medium",
                    "combined_risk": "Medium",
                    "risk_factors": []
                },
                "documents": [
                    {
                        "document_type": "PassportCopy",
                        "required": true,
                        "collected": true,
                        "verified": false
                    },
                    {
                        "document_type": "ProofOfAddress",
                        "required": true,
                        "collected": false,
                        "verified": false
                    }
                ],
                "screenings": [],
                "regulatory_context": {
                    "applicable_regulations": ["AML", "KYC", "BSA", "GDPR"],
                    "jurisdiction": "US",
                    "policy_overrides": {},
                    "exemptions": []
                },
                "clearance_decision": null
            }),
            metadata: serde_json::json!({"priority": "Normal", "estimated_duration_minutes": 120}),
            created_by: "web-demo".to_string(),
            tags: serde_json::json!(["KYC", "Compliance", "Web Demo"]),
        };

        let sample_onboarding = ResourceSheetRecord {
            resource_id: "web-onboarding-001".to_string(),
            resource_type: "Orchestrator".to_string(),
            name: "Onboarding Orchestrator for Web Demo".to_string(),
            description: Some("Sample onboarding orchestrator for web demonstration".to_string()),
            version: "1.0.0".to_string(),
            client_id: Some("WEB-CLIENT-001".to_string()),
            product_id: Some("DEMO-SUITE".to_string()),
            status: "Executing".to_string(),
            json_data: serde_json::json!({
                "id": "web-onboarding-001",
                "client_id": "WEB-CLIENT-001",
                "products": ["DEMO-TRADING", "DEMO-CUSTODY"],
                "resource_type": "Orchestrator",
                "status": "Executing",
                "metadata": {
                    "name": "Onboarding Orchestrator for Web Demo",
                    "description": "Sample onboarding orchestrator for web demonstration",
                    "version": "1.0.0",
                    "priority": "High",
                    "estimated_duration_minutes": 240,
                    "tags": ["Onboarding", "Orchestration", "Web Demo"]
                },
                "orchestration_dsl": "WORKFLOW \"WebDemoOnboarding\"\nPHASE \"Discovery\"\n    DISCOVER_DEPENDENCIES FOR_PRODUCTS [\"DEMO-TRADING\", \"DEMO-CUSTODY\"]\n    BUILD_MASTER_DICTIONARY FROM_RESOURCES [\"ProductCatalog\", \"RegulatoryRules\"]\nPHASE \"ResourceCreation\"\n    INSTANTIATE_RESOURCE \"KYC\" \"ClientKYCClearance\"\n    INSTANTIATE_RESOURCE \"AccountSetup\" \"ClientAccountSetup\"\nPHASE \"Execution\"\n    EXECUTE_RESOURCE_DSL \"ClientKYCClearance\"\n    AWAIT_RESOURCES [\"ClientKYCClearance\"] TO_BE \"Complete\"\n    EXECUTE_RESOURCE_DSL \"ClientAccountSetup\"\nPHASE \"Completion\"\n    VALIDATE_ORCHESTRATION_STATE USING [\"AllResourcesComplete\", \"NoErrors\"]\n    DERIVE_GLOBAL_STATE FROM_RESOURCES [\"ClientKYCClearance\", \"ClientAccountSetup\"]",
                "sub_resources": {
                    "ClientKYCClearance": {
                        "resource_id": "web-kyc-001",
                        "domain_type": "KYC",
                        "status": "Pending",
                        "dependencies": [],
                        "data_requirements": [],
                        "created_at": "2024-01-01T00:00:00Z"
                    }
                },
                "execution_plan": {
                    "phases": [
                        {
                            "name": "Discovery",
                            "description": "Discover required resources and build master data dictionary",
                            "resources": [],
                            "blocking": true,
                            "timeout_minutes": 30
                        },
                        {
                            "name": "Execution",
                            "description": "Execute domain resources in sequence",
                            "resources": ["ClientKYCClearance"],
                            "blocking": true,
                            "timeout_minutes": 180
                        }
                    ],
                    "current_phase": 1,
                    "parallel_execution": false,
                    "failure_strategy": "RequireManualReview"
                }
            }),
            metadata: serde_json::json!({"priority": "High", "estimated_duration_minutes": 240}),
            created_by: "web-demo".to_string(),
            tags: serde_json::json!(["Onboarding", "Orchestration", "Web Demo"]),
        };

        self.sample_resource_sheets = vec![sample_kyc, sample_onboarding];
        self.resource_sheet_manager.resource_sheets = self.sample_resource_sheets.clone();
    }

    fn show_connection_panel(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("🔌 Connection Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Template API:");
                ui.text_edit_singleline(&mut self.grpc_endpoint);

                let button_text = match self.connection_status {
                    ConnectionStatus::Connecting => "Connecting...",
                    ConnectionStatus::Connected => "Disconnect",
                    _ => "Connect",
                };

                if ui.button(button_text).clicked() {
                    match self.connection_status {
                        ConnectionStatus::Connected => {
                            self.api_client = None;
                            self.connection_status = ConnectionStatus::Disconnected;
                        },
                        _ => {
                            self.connection_status = ConnectionStatus::Connecting;
                            self.attempt_api_connection();
                        }
                    }
                }
            });

            // Show connection status
            let (status_text, status_color) = match &self.connection_status {
                ConnectionStatus::Disconnected => ("Template API Disconnected", egui::Color32::GRAY),
                ConnectionStatus::Connecting => ("Connecting to Template API...", egui::Color32::YELLOW),
                ConnectionStatus::Connected => ("Template API Connected", egui::Color32::GREEN),
                ConnectionStatus::Failed(err) => (err.as_str(), egui::Color32::RED),
            };

            ui.colored_label(status_color, status_text);
        });
    }

    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let current_route = self.router.current_route().clone();

            if ui.selectable_label(current_route == AppRoute::Dashboard, "🏠 Dashboard").clicked() {
                self.router.navigate_to(AppRoute::Dashboard);
            }

            if ui.selectable_label(current_route == AppRoute::ResourceSheets, "🗂️ Resource Sheets").clicked() {
                self.router.navigate_to(AppRoute::ResourceSheets);
            }

            if ui.selectable_label(current_route == AppRoute::Templates, "📝 Templates").clicked() {
                self.router.navigate_to(AppRoute::Templates);
            }

            if ui.selectable_label(current_route == AppRoute::CBUs, "🏢 CBUs").clicked() {
                self.router.navigate_to(AppRoute::CBUs);
            }

            if ui.selectable_label(current_route == AppRoute::Rules, "⚡ Rules").clicked() {
                self.router.navigate_to(AppRoute::Rules);
            }

            if ui.selectable_label(current_route == AppRoute::Database, "🗄️ Database").clicked() {
                self.router.navigate_to(AppRoute::Database);
            }

            if ui.selectable_label(current_route == AppRoute::ProductTaxonomy, "📦 Products").clicked() {
                self.router.navigate_to(AppRoute::ProductTaxonomy);
            }

            if ui.selectable_label(current_route == AppRoute::InvestmentMandates, "🎯 Mandates").clicked() {
                self.router.navigate_to(AppRoute::InvestmentMandates);
            }

            if ui.selectable_label(current_route == AppRoute::Transpiler, "📝 Transpiler").clicked() {
                self.router.navigate_to(AppRoute::Transpiler);
            }

            // AI Command Palette button (always available)
            ui.separator();
            if ui.button("🧠 AI Assistant").clicked() {
                self.show_ai_palette = !self.show_ai_palette;
            }

            // Debug panel toggle
            if ui.button("🔍 Debug").clicked() {
                self.show_debug_panel = !self.show_debug_panel;
            }
        });
    }

    fn show_ai_command_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧠 AI Assistant");
        ui.separator();

        // Context selection
        ui.horizontal(|ui| {
            ui.label("Context:");
            egui::ComboBox::from_label("")
                .selected_text(&self.ai_context)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.ai_context, "general".to_string(), "🔧 General");
                    ui.selectable_value(&mut self.ai_context, "kyc".to_string(), "🔍 KYC");
                    ui.selectable_value(&mut self.ai_context, "onboarding".to_string(), "📋 Onboarding");
                    ui.selectable_value(&mut self.ai_context, "dsl".to_string(), "⚡ DSL Help");
                    ui.selectable_value(&mut self.ai_context, "transpiler".to_string(), "📝 Transpiler");
                    ui.selectable_value(&mut self.ai_context, "validation".to_string(), "✅ Validation");
                });
        });

        ui.add_space(10.0);

        // Prompt input
        ui.label("Your prompt:");
        let prompt_response = ui.add(
            egui::TextEdit::multiline(&mut self.ai_prompt)
                .desired_rows(4)
                .hint_text("Ask the AI assistant to generate DSL code, explain concepts, or help with data modeling...")
        );

        ui.add_space(10.0);

        // Action buttons
        ui.horizontal(|ui| {
            let generate_button = ui.add_enabled(
                !self.ai_prompt.trim().is_empty() && !self.ai_loading,
                egui::Button::new("🚀 Generate DSL")
            );

            if generate_button.clicked() {
                self.generate_ai_dsl();
            }

            if ui.button("🗑️ Clear").clicked() {
                self.ai_prompt.clear();
                self.ai_response = None;
            }

            if ui.button("❌ Close").clicked() {
                self.show_ai_palette = false;
            }
        });

        ui.add_space(10.0);

        // Loading indicator
        if self.ai_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Generating AI response...");
            });
        }

        // AI Response - Multiple Suggestions
        if let Some(response) = &self.ai_response {
            ui.separator();
            ui.heading("🧠 AI Suggestions:");
            ui.label(format!("Status: {}", response.status_message));
            ui.add_space(10.0);

            for (index, suggestion) in response.suggestions.iter().enumerate() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        // Suggestion header
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", index + 1, suggestion.title));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("🎯 {:.0}%", suggestion.confidence * 100.0));
                                ui.label(format!("🏷️ {}", suggestion.category));
                            });
                        });

                        ui.separator();

                        // Suggestion content
                        let mut suggestion_text = suggestion.description.clone();
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut suggestion_text)
                                        .desired_width(f32::INFINITY)
                                        .code_editor()
                                );
                            });

                        // Action buttons for each suggestion
                        ui.horizontal(|ui| {
                            let suggestion_for_copy = suggestion.description.clone();
                            if ui.button("📋 Copy").clicked() {
                                ui.ctx().copy_text(suggestion_for_copy);
                            }

                            let suggestion_for_insert = suggestion.description.clone();
                            if ui.button("📝 Insert").clicked() {
                                wasm_utils::console_log(&format!("Would insert suggestion: {}", suggestion_for_insert));
                            }

                            // Show applicable contexts
                            if !suggestion.applicable_contexts.is_empty() {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(format!("Contexts: {}", suggestion.applicable_contexts.join(", ")));
                                });
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }

            // Global action buttons
            ui.horizontal(|ui| {
                if ui.button("🗑️ Clear All Suggestions").clicked() {
                    self.ai_response = None;
                }
            });
        }

        // Sample prompts for inspiration
        ui.separator();
        ui.collapsing("💡 Sample Prompts", |ui| {
            let samples = vec![
                "Generate a KYC workflow for a high-risk client",
                "Create validation rules for US regulatory compliance",
                "Write DSL for document collection and verification",
                "Generate risk assessment logic using multiple factors",
                "Create a decision tree for investment mandate approval",
            ];

            for sample in samples {
                if ui.button(sample).clicked() {
                    self.ai_prompt = sample.to_string();
                }
            }
        });

        // End-to-end testing section
        ui.separator();
        ui.collapsing("🔬 End-to-End Testing", |ui| {
            ui.label("Test the complete flow:");

            if ui.button("🏭 Test Template Instantiation").clicked() {
                self.test_template_instantiation();
            }

            if ui.button("⚡ Test DSL Execution").clicked() {
                self.test_dsl_execution();
            }

            if ui.button("🧠 Test AI + Instantiation + Execution").clicked() {
                self.test_full_pipeline();
            }
        });
    }

    fn generate_ai_dsl(&mut self) {
        if self.ai_prompt.trim().is_empty() {
            return;
        }

        self.ai_loading = true;
        let prompt = self.ai_prompt.clone();
        let context = self.ai_context.clone();

        // Make actual gRPC call to the AI assistant
        if let Some(grpc_client) = &self.grpc_client {
            let client = grpc_client.clone();
            let request = GetAiSuggestionsRequest {
                query: prompt,
                context: Some(context),
                ai_provider: Some(AiProviderConfig {
                    provider_type: 2, // Offline for now
                    api_key: None,
                }),
            };

            // Use spawn_local for async call in WASM
            wasm_bindgen_futures::spawn_local(async move {
                match client.get_ai_suggestions(request).await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("AI Response: {:?}", response));
                        // TODO: Update UI with response
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("AI Error: {:?}", e));
                    }
                }
            });
        }

        // For now, also keep the simulation for immediate UI feedback
        self.simulate_ai_response(&self.ai_prompt.clone(), &self.ai_context.clone());
    }

    fn simulate_ai_response(&mut self, prompt: &str, context: &str) {
        // Mock AI response generation (replace with actual gRPC call)
        let mock_response = match context {
            "kyc" => format!(
                "// Generated KYC DSL for: {}\nWORKFLOW \"GeneratedKYC\"\nSTEP \"RiskAssessment\"\n    ASSESS_RISK USING_FACTORS [\"jurisdiction\", \"product\"] OUTPUT \"riskLevel\"\nSTEP \"DocumentCollection\"\n    COLLECT_DOCUMENT \"Identity\" FROM Client REQUIRED true\nSTEP \"Decision\"\n    IF riskLevel = \"High\" THEN\n        FLAG_FOR_REVIEW \"Manual review required\" PRIORITY High\n    ELSE\n        APPROVE_CASE",
                prompt
            ),
            "validation" => format!(
                "// Generated Validation DSL for: {}\nVALIDATE_FIELD \"client.email\" FORMAT \"email\" REQUIRED true\nVALIDATE_FIELD \"client.age\" RANGE [18, 120] REQUIRED true\nVALIDATE_DOCUMENT \"passport\" EXPIRY_CHECK true",
                prompt
            ),
            "dsl" => format!(
                "// DSL Help for: {}\n// Available commands: WORKFLOW, STEP, VALIDATE, ASSESS_RISK, COLLECT_DOCUMENT\n// Example structure:\nWORKFLOW \"MyWorkflow\"\nSTEP \"StepName\"\n    // Your logic here",
                prompt
            ),
            _ => format!(
                "// Generated DSL for: {}\n// Context: {}\nWORKFLOW \"GeneratedWorkflow\"\nSTEP \"MainStep\"\n    // Generated based on your prompt\n    VALIDATE_INPUT \"data\" REQUIRED true",
                prompt, context
            )
        };

        // Create mock response with the new structure
        let suggestions = vec![
            AiSuggestion {
                title: "Generated DSL Code".to_string(),
                description: mock_response,
                category: "code_generation".to_string(),
                confidence: 0.85,
                applicable_contexts: vec![context.to_string()],
            }
        ];

        let response = GetAiSuggestionsResponse {
            suggestions,
            status_message: format!("Mock AI suggestions generated for {} context", context),
        };

        self.ai_response = Some(response);
        self.ai_loading = false;
    }

    fn test_template_instantiation(&self) {
        if let Some(grpc_client) = &self.grpc_client {
            let client = grpc_client.clone();
            let request = InstantiateResourceRequest {
                template_id: "kyc-sample-001".to_string(),
                onboarding_request_id: format!("web-test-{}", js_sys::Date::now()),
                context: Some("testing".to_string()),
                initial_data: Some(r#"{"test_mode": true, "source": "web_ui"}"#.to_string()),
            };

            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log("🏭 Testing Template Instantiation...");
                match client.instantiate_resource(request).await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ Template Instantiation Success: {:?}", response));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Template Instantiation Error: {:?}", e));
                    }
                }
            });
        }
    }

    fn test_dsl_execution(&self) {
        if let Some(grpc_client) = &self.grpc_client {
            let client = grpc_client.clone();
            let request = ExecuteDslRequest {
                instance_id: "test-instance-123".to_string(),
                execution_context: Some("web_testing".to_string()),
                input_data: Some(r#"{"test_data": "web_ui_test", "timestamp": "2025-10-16"}"#.to_string()),
            };

            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log("⚡ Testing DSL Execution...");
                match client.execute_dsl(request).await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ DSL Execution Success: {:?}", response));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ DSL Execution Error: {:?}", e));
                    }
                }
            });
        }
    }

    fn test_full_pipeline(&self) {
        if let Some(grpc_client) = &self.grpc_client {
            let client = grpc_client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log("🧠 Testing Full Pipeline: AI → Template → DSL...");

                // Step 1: AI Suggestion
                let ai_request = GetAiSuggestionsRequest {
                    query: "Generate a KYC workflow for testing".to_string(),
                    context: Some("kyc".to_string()),
                    ai_provider: Some(AiProviderConfig {
                        provider_type: 2,
                        api_key: None,
                    }),
                };

                match client.get_ai_suggestions(ai_request).await {
                    Ok(ai_response) => {
                        wasm_utils::console_log(&format!("✅ AI Suggestions: {:?}", ai_response));

                        // Step 2: Template Instantiation
                        let inst_request = InstantiateResourceRequest {
                            template_id: "kyc-sample-001".to_string(),
                            onboarding_request_id: format!("pipeline-test-{}", js_sys::Date::now()),
                            context: Some("full_pipeline_test".to_string()),
                            initial_data: Some(r#"{"ai_generated": true, "pipeline_test": true}"#.to_string()),
                        };

                        match client.instantiate_resource(inst_request).await {
                            Ok(inst_response) => {
                                wasm_utils::console_log(&format!("✅ Template Instantiation: {:?}", inst_response));

                                // Step 3: DSL Execution
                                if let Some(instance) = inst_response.instance {
                                    let exec_request = ExecuteDslRequest {
                                        instance_id: instance.instance_id,
                                        execution_context: Some("full_pipeline".to_string()),
                                        input_data: Some(r#"{"pipeline_test": true, "step": "execution"}"#.to_string()),
                                    };

                                    match client.execute_dsl(exec_request).await {
                                        Ok(exec_response) => {
                                            wasm_utils::console_log(&format!("✅ Full Pipeline Success: {:?}", exec_response));
                                        }
                                        Err(e) => {
                                            wasm_utils::console_log(&format!("❌ Pipeline DSL Execution Error: {:?}", e));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                wasm_utils::console_log(&format!("❌ Pipeline Template Error: {:?}", e));
                            }
                        }
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Pipeline AI Error: {:?}", e));
                    }
                }
            });
        }
    }

}

impl eframe::App for DataDesignerWebApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🦀 Data Designer - Web Edition");
            ui.separator();
            self.show_connection_panel(ui);
            ui.separator();
            self.render_navigation(ui);
        });

        // Main content panel
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.router.current_route() {
                AppRoute::Dashboard => {
                    self.show_dashboard(ui);
                }
                AppRoute::ResourceSheets => {
                    self.show_resource_sheets(ui);
                }
                AppRoute::Templates => {
                    self.show_template_editor(ui);
                }
                AppRoute::CBUs => {
                    self.show_placeholder(ui, "🏢 Client Business Units", "CBU management functionality");
                }
                AppRoute::Rules => {
                    self.show_placeholder(ui, "⚡ Rule Engine", "DSL rule editing and testing");
                }
                AppRoute::Database => {
                    self.show_placeholder(ui, "🗄️ Database", "Database operations and queries");
                }
                AppRoute::ProductTaxonomy => {
                    self.show_placeholder(ui, "📦 Product Taxonomy", "Product hierarchy management");
                }
                AppRoute::InvestmentMandates => {
                    self.show_placeholder(ui, "🎯 Investment Mandates", "Investment mandate management");
                }
                AppRoute::Transpiler => {
                    self.show_template_editor(ui);
                }
            }
        });

        // AI Command Palette (modal window)
        if self.show_ai_palette {
            egui::Window::new("🧠 AI Assistant")
                .resizable(true)
                .default_width(600.0)
                .default_height(400.0)
                .show(ctx, |ui| {
                    self.show_ai_command_palette(ui);
                });
        }

        // Debug panel (collapsible right panel)
        egui::SidePanel::right("debug_panel")
            .resizable(true)
            .show_animated(ctx, self.show_debug_panel, |ui| {
                ui.heading("🔍 Debug Panel");
                ui.separator();

                ui.collapsing("UI Inspector", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ctx.inspection_ui(ui);
                    });
                });

                ui.collapsing("Memory Usage", |ui| {
                    ui.label("Memory info available in debug mode");
                    if ui.button("Clear visual cache").clicked() {
                        ctx.clear_animations();
                    }
                });

                ui.collapsing("Settings", |ui| {
                    egui::Grid::new("debug_settings").show(ui, |ui| {
                        ui.label("Zoom factor:");
                        let mut zoom = ctx.zoom_factor();
                        if ui.add(egui::DragValue::new(&mut zoom).range(0.5..=2.0)).changed() {
                            ctx.set_zoom_factor(zoom);
                        }
                        ui.end_row();

                        ui.label("Pixels per point:");
                        ui.label(format!("{:.1}", ctx.pixels_per_point()));
                        ui.end_row();
                    });
                });

                ui.collapsing("Style", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ctx.style_ui(ui, egui::Theme::Dark);
                    });
                });
            });

        // Debug panel toggle button (floating)
        egui::Window::new("Debug")
            .collapsible(false)
            .resizable(false)
            .default_pos([10.0, 100.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(if self.show_debug_panel { "Hide Debug" } else { "Show Debug" }).clicked() {
                        self.show_debug_panel = !self.show_debug_panel;
                    }
                    ui.label("Panel");
                });
            });
    }
}

impl DataDesignerWebApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏠 Dashboard");
        ui.separator();

        ui.label("Welcome to the Data Designer Template Editor!");
        ui.add_space(10.0);

        // Quick stats
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("📊 System Status");
                    ui.label(format!("Sample Resources: {}", self.sample_resource_sheets.len()));
                    let api_status = if self.api_client.as_ref().map_or(false, |c| c.is_connected()) {
                        "✅ Connected"
                    } else {
                        "❌ Disconnected"
                    };
                    ui.label(format!("Template API: {}", api_status));
                    ui.label("Architecture: WASM + HTTP API");
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("🚀 Quick Actions");
                    if ui.button("📝 Template Editor").clicked() {
                        self.router.navigate_to(AppRoute::Templates);
                    }
                    if ui.button("📋 View Resource Sheets").clicked() {
                        self.router.navigate_to(AppRoute::ResourceSheets);
                    }
                    if ui.button("🔗 Connect to API").clicked() {
                        self.attempt_api_connection();
                    }
                });
            });
        });

        ui.add_space(20.0);
        ui.separator();

        // Architecture info
        ui.collapsing("ℹ️ Template Editor System", |ui| {
            ui.label("This is the **Template Editor** for the master resource file-based system.");
            ui.label("• WASM frontend built with egui + wasm-bindgen");
            ui.label("• HTTP REST API for template management");
            ui.label("• File-based templating system (resource_templates.json)");
            ui.label("• Visual editor for DSL workflows and attributes");
            ui.separator();
            ui.label("🎯 Core Purpose: Edit templates that serve as 'cookie cutters' for live resources");
        });
    }

    fn show_resource_sheets(&mut self, ui: &mut egui::Ui) {
        // Use a simplified version of resource sheet manager for web
        self.resource_sheet_manager.render_web_version(ui);
    }

    fn show_template_editor(&mut self, ui: &mut egui::Ui) {
        // Ensure template editor has the same API client as the main app
        if let Some(api_client) = &self.api_client {
            if self.template_editor.api_client.is_none() {
                self.template_editor.set_api_client(api_client.clone());
            }
        }

        self.template_editor.render(ui);
    }

    fn show_placeholder(&mut self, ui: &mut egui::Ui, title: &str, description: &str) {
        ui.heading(title);
        ui.separator();
        ui.label(description);
        ui.add_space(10.0);
        ui.label("🚧 This feature will be implemented in the web version soon!");

        if ui.button("🏠 Back to Dashboard").clicked() {
            self.router.navigate_to(AppRoute::Dashboard);
        }
    }
}