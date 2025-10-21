use eframe::egui;
use crate::{AppRoute, WebRouter, wasm_utils};
use crate::resource_sheet_ui::ResourceSheetManager;
use crate::minimal_types::ResourceSheetRecord;
use crate::http_api_client::DataDesignerHttpClient;
use crate::grpc_client::{GrpcClient, GetAiSuggestionsRequest, GetAiSuggestionsResponse, AiProviderConfig, AiSuggestion, InstantiateResourceRequest, ExecuteDslRequest};
use crate::debug_ui::DebugTestInterface;
use crate::template_designer::TemplateDesignerIDE;
use crate::data_designer::DataDesignerIDE;
use crate::entity_management::EntityManagementUI;
use crate::cbu_dsl_ide::CbuDslIDE;

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
    click_counter: u32,

    // Template detail view state
    selected_template_id: Option<String>,
    template_details: Option<String>,

    // Dynamic template list
    available_templates: Vec<(String, String, String)>, // (id, name, description)

    // Web-specific state
    grpc_endpoint: String,
    connection_status: ConnectionStatus,
    api_client: Option<DataDesignerHttpClient>,
    grpc_client: Option<GrpcClient>,


    // Debug tools
    show_debug_panel: bool,
    debug_interface: DebugTestInterface,

    // Template Designer IDE
    template_designer: TemplateDesignerIDE,

    // Data Designer IDE
    data_designer: DataDesignerIDE,

    // Entity Management UI
    entity_management: EntityManagementUI,


    // CBU DSL IDE
    cbu_dsl_ide: CbuDslIDE,

    // AI Command Palette
    show_ai_palette: bool,
    ai_prompt: String,
    ai_context: String,
    ai_loading: bool,
    ai_response: Option<GetAiSuggestionsResponse>,

    // Onboarding Requests state
    workflow_type: String,
    jurisdiction: String,
    initial_data_json: String,
    onboarding_instance_id: Option<String>,
    workflow_status: Option<serde_json::Value>,
    pending_solicitations: Vec<serde_json::Value>,
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
            click_counter: 0,
            selected_template_id: None,
            template_details: None,
            available_templates: Vec::new(),
            grpc_endpoint: "http://localhost:3030".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            api_client: None,
            grpc_client: Some(GrpcClient::new("http://localhost:50051")),
            show_debug_panel: false,
            debug_interface: DebugTestInterface::new(),
            template_designer: TemplateDesignerIDE::new(),
            data_designer: DataDesignerIDE::new(),
            entity_management: EntityManagementUI::new(),
            cbu_dsl_ide: CbuDslIDE::new(),

            // AI Command Palette
            show_ai_palette: false,
            ai_prompt: String::new(),
            ai_context: "general".to_string(),
            ai_loading: false,
            ai_response: None,

            // Onboarding Requests state
            workflow_type: "client_onboarding".to_string(),
            jurisdiction: "US".to_string(),
            initial_data_json: "{}".to_string(),
            onboarding_instance_id: None,
            workflow_status: None,
            pending_solicitations: Vec::new(),
        };

        // Load sample data
        app.load_sample_data();

        // Connect to API on startup for true JSON-centric sync
        wasm_utils::console_log("🚀 Auto-connecting to Template API on startup");
        app.attempt_api_connection();

        // Load templates dynamically from API
        app.load_templates_from_api();

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
        self.connection_status = ConnectionStatus::Connected;
    }

    fn load_templates_from_api(&mut self) {
        wasm_utils::console_log("📋 Loading templates from API...");

        // Fetch templates from the templates list endpoint
        wasm_bindgen_futures::spawn_local(async move {
            let url = "http://localhost:3030/api/templates";
            match reqwest::get(url).await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => {
                                wasm_utils::console_log(&format!("✅ Got templates list: {}", text));
                                // Parse the templates response
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(templates) = json.get("templates").and_then(|t| t.as_object()) {
                                        let template_list: Vec<(String, String, String)> = templates
                                            .iter()
                                            .map(|(id, template)| {
                                                let name = template.get("description")
                                                    .and_then(|d| d.as_str())
                                                    .unwrap_or(id)
                                                    .to_string();
                                                let description = template.get("description")
                                                    .and_then(|d| d.as_str())
                                                    .unwrap_or("No description")
                                                    .to_string();
                                                (id.clone(), name, description)
                                            })
                                            .collect();

                                        wasm_utils::console_log(&format!("📋 Loaded {} templates dynamically", template_list.len()));
                                        // Note: We can't update the app state from async context
                                        // This will be used for console verification for now
                                    }
                                }
                            }
                            Err(e) => wasm_utils::console_log(&format!("❌ Failed to read templates response: {:?}", e)),
                        }
                    } else {
                        wasm_utils::console_log(&format!("❌ Templates API returned status: {}", response.status()));
                    }
                }
                Err(e) => wasm_utils::console_log(&format!("❌ Failed to fetch templates: {:?}", e)),
            }
        });

        // For now, use a fallback list that matches what's in the API
        // This ensures the UI works immediately while we load from API
        self.available_templates = vec![
            ("account_setup_trading_v1".to_string(), "Account Setup Trading".to_string(), "Trading account setup workflow".to_string()),
            ("baseline_template".to_string(), "Baseline Template".to_string(), "Basic template for simple processes".to_string()),
            ("kyc_clearance_v1".to_string(), "KYC Clearance".to_string(), "Client KYC clearance workflow".to_string()),
            ("onboarding_orchestrator_v1".to_string(), "Onboarding Orchestrator".to_string(), "Complete client onboarding".to_string()),
            ("regulatory_reporting_v1".to_string(), "Regulatory Reporting".to_string(), "Regulatory compliance reporting".to_string()),
        ];
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

            // Dashboard
            if ui.selectable_label(current_route == AppRoute::Dashboard, "🏠 Dashboard").clicked() {
                self.router.navigate_to(AppRoute::Dashboard);
            }

            ui.separator();

            // Main functional areas with clear labels
            if ui.selectable_label(current_route == AppRoute::ResourceTemplates, "📋 Resource Templates").clicked() {
                self.router.navigate_to(AppRoute::ResourceTemplates);
            }

            if ui.selectable_label(current_route == AppRoute::PrivateData, "🔒 Private Data").clicked() {
                self.router.navigate_to(AppRoute::PrivateData);
            }

            if ui.selectable_label(current_route == AppRoute::OnboardingRequests, "🚀 Onboarding Requests").clicked() {
                self.router.navigate_to(AppRoute::OnboardingRequests);
            }

            ui.separator();


            ui.separator();

            // Entity Management
            if ui.selectable_label(current_route == AppRoute::CbuDslIde, "🏢 CBU DSL IDE").clicked() {
                self.router.navigate_to(AppRoute::CbuDslIde);
            }

            if ui.selectable_label(current_route == AppRoute::ProductManagement, "📦 Products").clicked() {
                self.router.navigate_to(AppRoute::ProductManagement);
            }

            if ui.selectable_label(current_route == AppRoute::ServiceManagement, "⚙️ Services").clicked() {
                self.router.navigate_to(AppRoute::ServiceManagement);
            }

            if ui.selectable_label(current_route == AppRoute::ResourceManagement, "🔧 Resources").clicked() {
                self.router.navigate_to(AppRoute::ResourceManagement);
            }

            if ui.selectable_label(current_route == AppRoute::WorkflowManagement, "📋 Workflows").clicked() {
                self.router.navigate_to(AppRoute::WorkflowManagement);
            }

            ui.separator();

            // Supporting tools

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
                    ui.selectable_value(&mut self.ai_context, "rag_capabilities".to_string(), "⚡ Capability-Aware RAG");
                    ui.selectable_value(&mut self.ai_context, "kyc".to_string(), "🔍 KYC");
                    ui.selectable_value(&mut self.ai_context, "onboarding".to_string(), "📋 Onboarding");
                    ui.selectable_value(&mut self.ai_context, "dsl".to_string(), "⚡ DSL Help");
                    ui.selectable_value(&mut self.ai_context, "transpiler".to_string(), "📝 Transpiler");
                    ui.selectable_value(&mut self.ai_context, "validation".to_string(), "✅ Validation");
                    ui.selectable_value(&mut self.ai_context, "code_completion".to_string(), "🤖 Code Completion");
                    ui.selectable_value(&mut self.ai_context, "error_analysis".to_string(), "🔍 Error Analysis");
                });
        });

        ui.add_space(10.0);

        // Prompt input
        ui.label("Your prompt:");
        let _prompt_response = ui.add(
            egui::TextEdit::multiline(&mut self.ai_prompt)
                .desired_rows(4)
                .hint_text("Ask about capabilities, generate DSL code, analyze errors, or get contextual suggestions. Try: 'Set up account', 'Configure trade feed', 'Validate client data'...")
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
                    provider_type: 2, // Offline but enhanced with capability-aware features
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
        // Force continuous repainting to ensure responsiveness
        ctx.request_repaint();
        // Top panel with navigation
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🦀 Data Designer - Web Edition");
            ui.colored_label(egui::Color32::GREEN, format!("DEBUG: Current route = {:?}", self.router.current_route()));
            ui.separator();
            self.show_connection_panel(ui);
            ui.separator();
            self.render_navigation(ui);
        });

        // Main content panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.colored_label(egui::Color32::YELLOW, "DEBUG: Main content panel is rendering");
            ui.separator();

            match self.router.current_route() {
                AppRoute::Dashboard => {
                    self.show_dashboard(ui);
                }
                AppRoute::ResourceTemplates => {
                    self.show_resource_templates(ui);
                }
                AppRoute::PrivateData => {
                    self.show_private_data(ui);
                }
                AppRoute::OnboardingRequests => {
                    self.show_onboarding_requests(ui);
                }
                AppRoute::TemplateDesigner => {
                    self.template_designer.render(ui);
                }
                AppRoute::DataDesigner => {
                    self.data_designer.show(ui, self.api_client.as_ref());
                }
                AppRoute::CbuDslIde => {
                    self.cbu_dsl_ide.render(ui, self.grpc_client.as_ref());
                }
                AppRoute::ProductManagement => {
                    self.entity_management.show_product_management(ui, self.grpc_client.as_ref());
                }
                AppRoute::ServiceManagement => {
                    self.entity_management.show_service_management(ui, self.grpc_client.as_ref());
                }
                AppRoute::ResourceManagement => {
                    self.entity_management.show_resource_management(ui, self.grpc_client.as_ref());
                }
                AppRoute::WorkflowManagement => {
                    self.show_placeholder(ui, "📋 Workflow Management", "Onboarding workflow CRUD operations");
                }
                AppRoute::Transpiler => {
                    self.show_ai_command_palette(ui);
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
            .default_width(400.0)
            .show_animated(ctx, self.show_debug_panel, |ui| {
                // Comprehensive debug and testing interface
                self.debug_interface.render(ui);

                ui.separator();

                // Keep some of the original debug tools
                ui.collapsing("⚙️ System Debug", |ui| {
                    ui.collapsing("UI Inspector", |ui| {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            ctx.inspection_ui(ui);
                        });
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

                        if ui.button("Clear visual cache").clicked() {
                            ctx.clear_animations();
                        }
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

    fn show_resource_sheets(&mut self, ui: &mut egui::Ui) {
        // Use a simplified version of resource sheet manager for web
        self.resource_sheet_manager.render_web_version(ui);
    }


    fn show_simple_templates(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Resource Templates");
        ui.separator();

        // Check if we're showing template details
        if let Some(template_id) = &self.selected_template_id.clone() {
            self.show_template_details(ui, template_id);
            return;
        }

        ui.label("Available resource templates from the API:");
        ui.add_space(10.0);

        // Connection status
        let connected = self.api_client.as_ref().is_some_and(|c| c.is_connected());
        if !connected {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Not connected to API server");
            if ui.button("🔄 Connect").clicked() {
                self.attempt_api_connection();
            }
            return;
        }

        // Simple template list without complex layout
        ui.label("🏭 Available Templates:");
        ui.separator();

        // Use dynamic template list
        if self.available_templates.is_empty() {
            ui.label("Loading templates from API...");
            return;
        }

        // Clone the templates to avoid borrowing issues
        let templates = self.available_templates.clone();
        for (i, (id, name, description)) in templates.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{}.", i + 1));
                ui.vertical(|ui| {
                    ui.strong(name);
                    ui.small(description);
                    ui.small(format!("Template ID: {}", id));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔍 View Details").clicked() {
                        self.view_template_details(id);
                    }
                    if ui.button("🎨 Edit in Designer").clicked() {
                        self.launch_template_designer_for_edit(id);
                    }
                    if ui.button("🧪 Test").clicked() {
                        self.test_template_instantiation_with_id(id);
                    }
                });
            });
            ui.separator();
        }

        ui.add_space(20.0);

        // Template creation section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("📝 Template Management:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ Create New Template").clicked() {
                        self.template_designer.start_new_template();
                        self.router.navigate_to(AppRoute::TemplateDesigner);
                    }
                });
            });
            ui.label("Create custom workflow templates with domain-specific syntax highlighting");
        });

        ui.add_space(20.0);

        // Testing section
        ui.label("🛠️ Development Testing:");
        ui.horizontal(|ui| {
            if ui.button("🧪 Test Instantiation").clicked() {
                self.test_template_instantiation();
            }
            if ui.button("⚡ Test DSL Execution").clicked() {
                self.test_dsl_execution();
            }
            if ui.button("🔄 Full Pipeline").clicked() {
                self.test_full_pipeline();
            }
        });
        ui.small("Check browser console for test results");
    }

    fn view_template_details(&mut self, template_id: &str) {
        self.selected_template_id = Some(template_id.to_string());

        // Instead of async loading, provide immediate template details
        // We know the templates exist and what they contain
        let template_details = match template_id {
            "account_setup_trading_v1" => {
                serde_json::json!({
                    "id": "account_setup_trading_v1",
                    "description": "Sets up trading account infrastructure and permissions for a client.",
                    "workflow": "TradingAccountSetup",
                    "steps": [
                        "Prerequisites - Check KYC clearance",
                        "CreateAccount - Generate account number",
                        "SetupPermissions - Assign trading permissions",
                        "SystemIntegration - Integrate with platforms",
                        "Testing - Validate trading access"
                    ],
                    "attributes": [
                        {"name": "case_id", "type": "String", "required": true},
                        {"name": "client_id", "type": "String", "required": true},
                        {"name": "account_number", "type": "String", "required": false},
                        {"name": "trading_permissions", "type": "Array", "required": false},
                        {"name": "setup_status", "type": "String", "required": true}
                    ],
                    "dsl": "WORKFLOW \"TradingAccountSetup\"\n\nSTEP \"Prerequisites\"\n    LOG \"Setting up trading account for client: \" + client_id\n    REQUIRE_RESOURCE \"ClientKYCClearance_\" + client_id STATUS \"Approved\"\n    SET setup_status TO \"AccountCreation\"\nPROCEED_TO STEP \"CreateAccount\"\n\nSTEP \"CreateAccount\"\n    GENERATE_ACCOUNT_NUMBER PREFIX \"TRD\" FOR client_id\n    STORE_RESULT AS \"account_number\"\n    CREATE_TRADING_ACCOUNT account_number FOR client_id\n    SET setup_status TO \"PermissionsSetup\"\nPROCEED_TO STEP \"SetupPermissions\"\n\nSTEP \"SetupPermissions\"\n    ASSIGN_TRADING_PERMISSIONS [\"Equities\", \"FixedIncome\", \"FX\"] TO account_number\n    STORE_PERMISSIONS AS \"trading_permissions\"\n    SET setup_status TO \"Integration\"\nPROCEED_TO STEP \"SystemIntegration\"\n\nSTEP \"SystemIntegration\"\n    INTEGRATE_WITH_SYSTEM \"TradingPlatform\" ACCOUNT account_number\n    INTEGRATE_WITH_SYSTEM \"RiskManagement\" ACCOUNT account_number\n    INTEGRATE_WITH_SYSTEM \"Settlement\" ACCOUNT account_number\n    SET setup_status TO \"Testing\"\nPROCEED_TO STEP \"Testing\"\n\nSTEP \"Testing\"\n    RUN_SYSTEM_TESTS FOR account_number\n    VALIDATE_TRADING_ACCESS FOR client_id\n    SET setup_status TO \"Complete\"\n    LOG \"Trading account setup completed for client: \" + client_id + \", account: \" + account_number"
                }).to_string()
            }
            "kyc_clearance_v1" => {
                serde_json::json!({
                    "id": "kyc_clearance_v1",
                    "description": "Performs standard KYC due diligence on a client entity.",
                    "workflow": "StandardClientKYC",
                    "steps": [
                        "InitialAssessment - Derive regulatory context and assess risk",
                        "Screening - Screen entity against sanctions and PEP lists",
                        "DocumentCollection - Collect required documents",
                        "Decision - Make approval/rejection decision based on risk"
                    ],
                    "attributes": [
                        {"name": "case_id", "type": "String", "required": true},
                        {"name": "status", "type": "String", "required": true},
                        {"name": "client_legal_name", "type": "String", "required": true},
                        {"name": "client_jurisdiction", "type": "String", "required": true},
                        {"name": "risk_rating", "type": "String", "required": false},
                        {"name": "screening_results", "type": "Object", "required": false}
                    ],
                    "dsl": "WORKFLOW \"StandardClientKYC\"\n\nSTEP \"InitialAssessment\"\n    LOG \"Starting KYC for client: \" + client_legal_name\n    DERIVE_REGULATORY_CONTEXT FOR_JURISDICTION client_jurisdiction WITH_PRODUCTS [\"Trading\"]\n    ASSESS_RISK USING_FACTORS [\"jurisdiction\", \"product\", \"client\"] OUTPUT \"risk_rating\"\nPROCEED_TO STEP \"Screening\"\n\nSTEP \"Screening\"\n    SCREEN_ENTITY client_legal_name AGAINST \"SanctionsList\" THRESHOLD 0.85\n    SCREEN_ENTITY client_legal_name AGAINST \"PEPList\" THRESHOLD 0.90\n    STORE_RESULTS AS \"screening_results\"\nPROCEED_TO STEP \"DocumentCollection\"\n\nSTEP \"DocumentCollection\"\n    COLLECT_DOCUMENT \"PassportCopy\" FROM Client REQUIRED true\n    COLLECT_DOCUMENT \"ProofOfAddress\" FROM Client REQUIRED true\n    COLLECT_DOCUMENT \"FinancialStatements\" FROM Client REQUIRED false\nPROCEED_TO STEP \"Decision\"\n\nSTEP \"Decision\"\n    IF risk_rating == \"High\" THEN\n        SET status TO \"Review\"\n        FLAG_FOR_REVIEW \"High risk client requires manual review\" PRIORITY High\n    ELSE IF screening_results.sanctions_match > 0.85 THEN\n        SET status TO \"Rejected\"\n        REJECT_CASE \"Client found on sanctions list\"\n    ELSE\n        SET status TO \"Approved\"\n        APPROVE_CASE WITH_CONDITIONS [\"Annual review required\"]\n    END_IF\n    LOG \"KYC completed for \" + client_legal_name + \" with status: \" + status"
                }).to_string()
            }
            "onboarding_orchestrator_v1" => {
                serde_json::json!({
                    "id": "onboarding_orchestrator_v1",
                    "description": "Orchestrates the complete client onboarding process across multiple domains and products.",
                    "workflow": "ClientOnboardingOrchestrator",
                    "phases": [
                        "Discovery - Discover dependencies and build master dictionary",
                        "ResourceCreation - Instantiate required resources",
                        "Execution - Execute KYC and account setup in sequence",
                        "Completion - Validate orchestration state"
                    ],
                    "attributes": [
                        {"name": "client_id", "type": "String", "required": true},
                        {"name": "products", "type": "Array", "required": true},
                        {"name": "orchestration_status", "type": "String", "required": true},
                        {"name": "current_phase", "type": "Number", "required": false},
                        {"name": "sub_resources", "type": "Object", "required": false},
                        {"name": "master_dictionary", "type": "Object", "required": false}
                    ],
                    "dsl": "WORKFLOW \"ClientOnboardingOrchestrator\"\n\nPHASE \"Discovery\"\n    LOG \"Starting onboarding orchestration for client: \" + client_id\n    SET orchestration_status TO \"Discovery\"\n    DISCOVER_DEPENDENCIES FOR_PRODUCTS products\n    BUILD_MASTER_DICTIONARY FROM_RESOURCES [\"ProductCatalog\", \"RegulatoryRules\", \"ClientRequirements\"]\n    STORE_DICTIONARY AS \"master_dictionary\"\nPROCEED_TO PHASE \"ResourceCreation\"\n\nPHASE \"ResourceCreation\"\n    SET orchestration_status TO \"ResourceCreation\"\n    FOR_EACH product IN products DO\n        IF product == \"Trading\" THEN\n            INSTANTIATE_RESOURCE \"KYC\" \"ClientKYCClearance_\" + client_id\n            INSTANTIATE_RESOURCE \"AccountSetup\" \"TradingAccountSetup_\" + client_id\n        ELSE_IF product == \"Custody\" THEN\n            INSTANTIATE_RESOURCE \"KYC\" \"ClientKYCClearance_\" + client_id\n            INSTANTIATE_RESOURCE \"AccountSetup\" \"CustodyAccountSetup_\" + client_id\n        END_IF\n    END_FOR\nPROCEED_TO PHASE \"Execution\"\n\nPHASE \"Execution\"\n    SET orchestration_status TO \"Execution\"\n    # Execute KYC first (blocking)\n    EXECUTE_RESOURCE_DSL \"ClientKYCClearance_\" + client_id\n    AWAIT_RESOURCES [\"ClientKYCClearance_\" + client_id] TO_BE \"Complete\"\n    \n    # Then execute account setup in parallel\n    FOR_EACH product IN products DO\n        IF product == \"Trading\" THEN\n            EXECUTE_RESOURCE_DSL \"TradingAccountSetup_\" + client_id ASYNC\n        ELSE_IF product == \"Custody\" THEN\n            EXECUTE_RESOURCE_DSL \"CustodyAccountSetup_\" + client_id ASYNC\n        END_IF\n    END_FOR\n    \n    AWAIT_ALL_RESOURCES TO_BE \"Complete\"\nPROCEED_TO PHASE \"Completion\"\n\nPHASE \"Completion\"\n    SET orchestration_status TO \"Complete\"\n    VALIDATE_ORCHESTRATION_STATE USING [\"AllResourcesComplete\", \"NoErrors\"]\n    DERIVE_GLOBAL_STATE FROM_ALL_RESOURCES\n    LOG \"Onboarding orchestration completed successfully for client: \" + client_id\n    NOTIFY_STAKEHOLDERS \"Client onboarding complete\" FOR client_id"
                }).to_string()
            }
            "regulatory_reporting_v1" => {
                serde_json::json!({
                    "id": "regulatory_reporting_v1",
                    "description": "Handles regulatory reporting requirements for client activities.",
                    "workflow": "RegulatoryReporting",
                    "steps": [
                        "DetermineRequirements - Lookup regulations and derive requirements",
                        "PrepareFilings - Prepare and validate filing data",
                        "Review - Review all filings for compliance",
                        "Submit - Submit filings to jurisdiction and await acknowledgments"
                    ],
                    "attributes": [
                        {"name": "case_id", "type": "String", "required": true},
                        {"name": "client_id", "type": "String", "required": true},
                        {"name": "jurisdiction", "type": "String", "required": true},
                        {"name": "reporting_requirements", "type": "Array", "required": false},
                        {"name": "filing_status", "type": "String", "required": true}
                    ],
                    "dsl": "WORKFLOW \"RegulatoryReporting\"\n\nSTEP \"DetermineRequirements\"\n    LOG \"Determining regulatory requirements for \" + client_id + \" in \" + jurisdiction\n    LOOKUP_REGULATIONS FOR jurisdiction\n    DERIVE_REQUIREMENTS BASED_ON [\"client_type\", \"products\", \"jurisdiction\"]\n    STORE_REQUIREMENTS AS \"reporting_requirements\"\n    SET filing_status TO \"Preparing\"\nPROCEED_TO STEP \"PrepareFilings\"\n\nSTEP \"PrepareFilings\"\n    FOR_EACH requirement IN reporting_requirements DO\n        PREPARE_FILING requirement FOR client_id\n        VALIDATE_FILING_DATA requirement\n    END_FOR\n    SET filing_status TO \"Review\"\nPROCEED_TO STEP \"Review\"\n\nSTEP \"Review\"\n    REVIEW_ALL_FILINGS FOR client_id\n    IF review_passed THEN\n        SET filing_status TO \"Filed\"\n        PROCEED_TO STEP \"Submit\"\n    ELSE\n        SET filing_status TO \"Preparing\"\n        PROCEED_TO STEP \"PrepareFilings\"\n    END_IF\n\nSTEP \"Submit\"\n    FOR_EACH requirement IN reporting_requirements DO\n        SUBMIT_FILING requirement TO jurisdiction\n    END_FOR\n    AWAIT_ACKNOWLEDGMENTS\n    SET filing_status TO \"Acknowledged\"\n    LOG \"Regulatory reporting completed for client: \" + client_id"
                }).to_string()
            }
            "baseline_template" => {
                serde_json::json!({
                    "id": "baseline_template",
                    "description": "A baseline template for all new resources. Includes common attributes and a default DSL.",
                    "workflow": "DefaultWorkflow",
                    "steps": [
                        "Start - Initialize workflow and log case ID",
                        "Middle - Execute middle phase logic",
                        "End - Complete workflow and finalize"
                    ],
                    "attributes": [
                        {"name": "case_id", "type": "String", "required": true},
                        {"name": "status", "type": "String", "required": true},
                        {"name": "created_by", "type": "String", "required": true},
                        {"name": "priority", "type": "String", "required": false}
                    ],
                    "dsl": "WORKFLOW \"DefaultWorkflow\"\n\nSTEP \"Start\"\n    # Add your logic here\n    LOG \"Starting workflow for case: \" + case_id\nSTEP \"Middle\"\n\tLOG \"Middle phase\"\nPROCEED_TO STEP \"End\"\n\nSTEP \"End\"\n    LOG \"Workflow complete for case: \" + case_id\n    # Workflow complete"
                }).to_string()
            }
            _ => {
                serde_json::json!({
                    "id": template_id,
                    "description": "Template details loading...",
                    "note": "This template exists but details are being fetched from API"
                }).to_string()
            }
        };

        self.template_details = Some(template_details);
        wasm_utils::console_log(&format!("📄 Template details loaded for: {}", template_id));
    }

    fn launch_template_designer_for_edit(&mut self, template_id: &str) {
        // Fetch the template JSON from API and launch designer
        wasm_utils::console_log(&format!("🎨 Launching Template Designer for: {}", template_id));

        // For now, simulate with the hardcoded template data
        let template_json = match template_id {
            "account_setup_trading_v1" => {
                serde_json::json!({
                    "id": "account_setup_trading_v1",
                    "description": "Sets up trading account infrastructure and permissions for a client.",
                    "attributes": [
                        {"name": "case_id", "dataType": "String", "ui": {"required": true, "label": "Case ID"}},
                        {"name": "client_id", "dataType": "String", "ui": {"required": true, "label": "Client ID"}},
                        {"name": "account_number", "dataType": "String", "ui": {"required": false, "label": "Account Number"}},
                        {"name": "trading_permissions", "dataType": "Array", "ui": {"required": false, "label": "Trading Permissions"}},
                        {"name": "setup_status", "dataType": "String", "ui": {"required": true, "label": "Setup Status"}}
                    ],
                    "dsl": "WORKFLOW \"TradingAccountSetup\"\n\nSTEP \"Prerequisites\"\n    LOG \"Setting up trading account for client: \" + client_id\n    REQUIRE_RESOURCE \"ClientKYCClearance_\" + client_id STATUS \"Approved\"\n    SET setup_status TO \"AccountCreation\"\nPROCEED_TO STEP \"CreateAccount\"\n\nSTEP \"CreateAccount\"\n    GENERATE_ACCOUNT_NUMBER PREFIX \"TRD\" FOR client_id\n    STORE_RESULT AS \"account_number\"\n    CREATE_TRADING_ACCOUNT account_number FOR client_id\n    SET setup_status TO \"PermissionsSetup\"\nPROCEED_TO STEP \"SetupPermissions\""
                }).to_string()
            }
            _ => {
                serde_json::json!({
                    "id": template_id,
                    "description": "Template for editing",
                    "attributes": [],
                    "dsl": "WORKFLOW \"EditableWorkflow\"\n\nSTEP \"Start\"\n    LOG \"Starting workflow\""
                }).to_string()
            }
        };

        // Launch the template designer with this template
        self.template_designer.start_edit_template(&template_json);
        self.router.navigate_to(AppRoute::TemplateDesigner);
    }

    fn show_template_details(&mut self, ui: &mut egui::Ui, template_id: &str) {
        ui.heading(format!("📄 Template Details: {}", template_id));
        ui.separator();

        // Back button
        if ui.button("⬅️ Back to Templates").clicked() {
            self.selected_template_id = None;
            self.template_details = None;
            return;
        }

        ui.add_space(10.0);

        if let Some(details) = &self.template_details {
            // Parse and display the template details in a structured way
            match serde_json::from_str::<serde_json::Value>(details) {
                Ok(json) => {
                    // Show key information first
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            if let Some(description) = json.get("description") {
                                ui.strong("Description:");
                                ui.label(description.as_str().unwrap_or("N/A"));
                            }

                            if let Some(workflow) = json.get("workflow") {
                                ui.strong("Workflow:");
                                ui.label(workflow.as_str().unwrap_or("N/A"));
                            }

                            // Show steps or phases
                            if let Some(steps) = json.get("steps").and_then(|s| s.as_array()) {
                                ui.strong("Workflow Steps:");
                                for (i, step) in steps.iter().enumerate() {
                                    ui.label(format!("{}. {}", i + 1, step.as_str().unwrap_or("N/A")));
                                }
                            } else if let Some(phases) = json.get("phases").and_then(|p| p.as_array()) {
                                ui.strong("Workflow Phases:");
                                for (i, phase) in phases.iter().enumerate() {
                                    ui.label(format!("{}. {}", i + 1, phase.as_str().unwrap_or("N/A")));
                                }
                            }

                            // Show attributes
                            if let Some(attributes) = json.get("attributes").and_then(|a| a.as_array()) {
                                ui.strong("Template Attributes:");
                                for attr in attributes {
                                    if let (Some(name), Some(attr_type), Some(required)) = (
                                        attr.get("name").and_then(|n| n.as_str()),
                                        attr.get("type").and_then(|t| t.as_str()),
                                        attr.get("required").and_then(|r| r.as_bool())
                                    ) {
                                        let req_text = if required { " (required)" } else { " (optional)" };
                                        ui.label(format!("• {} : {}{}", name, attr_type, req_text));
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Show DSL code if available
                    if let Some(dsl) = json.get("dsl").and_then(|d| d.as_str()) {
                        ui.strong("DSL Code:");
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                self.render_dsl_with_syntax_highlighting(ui, dsl);
                            });
                    }

                    ui.add_space(10.0);

                    // Raw JSON view (collapsible)
                    ui.collapsing("🔍 Raw JSON", |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.code(serde_json::to_string_pretty(&json).unwrap_or_else(|_| details.clone()));
                            });
                    });
                }
                Err(_) => {
                    // Fallback to showing raw details
                    ui.label("Template Content:");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.code(details);
                        });
                }
            }
        } else {
            ui.spinner();
            ui.label("Loading template details...");

            // Try to fetch and immediately display template details
            // Since we know the API is working, let's make a synchronous-style display
            if let Some(api_client) = &self.api_client {
                if api_client.is_connected() {
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.strong("Template Information");
                            ui.label(format!("ID: {}", template_id));
                            ui.label("Status: Connected to API on port 3030");
                            ui.label("Note: Check browser console for full template details");
                            ui.separator();

                            // Show template data inline since we know it exists
                            match template_id {
                                "account_setup_trading_v1" => {
                                    ui.label("Description: Sets up trading account infrastructure and permissions");
                                    ui.label("Workflow: TradingAccountSetup");
                                    ui.label("Steps: Prerequisites → CreateAccount → SetupPermissions → SystemIntegration → Testing");
                                }
                                "kyc_clearance_v1" => {
                                    ui.label("Description: Performs standard KYC due diligence on a client entity");
                                    ui.label("Workflow: StandardClientKYC");
                                    ui.label("Steps: InitialAssessment → Screening → DocumentCollection → Decision");
                                }
                                "onboarding_orchestrator_v1" => {
                                    ui.label("Description: Orchestrates the complete client onboarding process");
                                    ui.label("Workflow: ClientOnboardingOrchestrator");
                                    ui.label("Phases: Discovery → ResourceCreation → Execution → Completion");
                                }
                                _ => {
                                    ui.label("Template data is being fetched from API...");
                                }
                            }
                        });
                    });
                }
            }
        }
    }

    fn test_template_instantiation_with_id(&self, template_id: &str) {
        if let Some(grpc_client) = &self.grpc_client {
            let client = grpc_client.clone();
            let template_id_owned = template_id.to_string();
            let request = InstantiateResourceRequest {
                template_id: template_id_owned.clone(),
                onboarding_request_id: format!("web-test-{}", js_sys::Date::now()),
                context: Some("ui_testing".to_string()),
                initial_data: Some(format!(r#"{{"test_mode": true, "source": "web_ui", "template": "{}"}}"#, template_id_owned)),
            };

            wasm_bindgen_futures::spawn_local(async move {
                wasm_utils::console_log(&format!("🏭 Testing Template Instantiation for {}...", template_id_owned));
                match client.instantiate_resource(request).await {
                    Ok(response) => {
                        wasm_utils::console_log(&format!("✅ Template {} Instantiation Success: {:?}", template_id_owned, response));
                    }
                    Err(e) => {
                        wasm_utils::console_log(&format!("❌ Template {} Instantiation Error: {:?}", template_id_owned, e));
                    }
                }
            });
        }
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

    fn render_dsl_with_syntax_highlighting(&self, ui: &mut egui::Ui, dsl: &str) {
        // Basic syntax highlighting for DSL based on workflow_dsl.ebnf
        let lines = dsl.lines();

        for line in lines {
            ui.horizontal(|ui| {
                let trimmed = line.trim();

                // Keywords - workflow structure
                if trimmed.starts_with("WORKFLOW") {
                    ui.colored_label(egui::Color32::from_rgb(86, 156, 214), "WORKFLOW");
                    if let Some(rest) = line.strip_prefix("WORKFLOW") {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("STEP") || trimmed.starts_with("PHASE") {
                    let keyword = if trimmed.starts_with("STEP") { "STEP" } else { "PHASE" };
                    ui.colored_label(egui::Color32::from_rgb(86, 156, 214), keyword);
                    if let Some(rest) = line.strip_prefix(keyword) {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("PROCEED_TO") {
                    ui.colored_label(egui::Color32::from_rgb(86, 156, 214), "PROCEED_TO");
                    if let Some(rest) = line.strip_prefix("PROCEED_TO") {
                        ui.label(rest);
                    }
                }
                // Control flow keywords
                else if trimmed.starts_with("IF") {
                    ui.colored_label(egui::Color32::from_rgb(197, 134, 192), "IF");
                    if let Some(rest) = line.strip_prefix("IF") {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("THEN") {
                    ui.colored_label(egui::Color32::from_rgb(197, 134, 192), "THEN");
                    if let Some(rest) = line.strip_prefix("THEN") {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("ELSE") {
                    ui.colored_label(egui::Color32::from_rgb(197, 134, 192), "ELSE");
                    if let Some(rest) = line.strip_prefix("ELSE") {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("END_IF") {
                    ui.colored_label(egui::Color32::from_rgb(197, 134, 192), "END_IF");
                    if let Some(rest) = line.strip_prefix("END_IF") {
                        ui.label(rest);
                    }
                }
                // Domain-specific commands
                else if trimmed.starts_with("LOG") {
                    ui.colored_label(egui::Color32::from_rgb(220, 220, 170), "LOG");
                    if let Some(rest) = line.strip_prefix("LOG") {
                        ui.label(rest);
                    }
                } else if trimmed.starts_with("SET") {
                    ui.colored_label(egui::Color32::from_rgb(220, 220, 170), "SET");
                    if let Some(rest) = line.strip_prefix("SET") {
                        ui.label(rest);
                    }
                }
                // KYC/Financial domain commands
                else if trimmed.contains("DERIVE_REGULATORY_CONTEXT") {
                    self.highlight_line_with_command(ui, line, "DERIVE_REGULATORY_CONTEXT");
                } else if trimmed.contains("ASSESS_RISK") {
                    self.highlight_line_with_command(ui, line, "ASSESS_RISK");
                } else if trimmed.contains("SCREEN_ENTITY") {
                    self.highlight_line_with_command(ui, line, "SCREEN_ENTITY");
                } else if trimmed.contains("COLLECT_DOCUMENT") {
                    self.highlight_line_with_command(ui, line, "COLLECT_DOCUMENT");
                } else if trimmed.contains("GENERATE_ACCOUNT_NUMBER") {
                    self.highlight_line_with_command(ui, line, "GENERATE_ACCOUNT_NUMBER");
                } else if trimmed.contains("INTEGRATE_WITH_SYSTEM") {
                    self.highlight_line_with_command(ui, line, "INTEGRATE_WITH_SYSTEM");
                } else if trimmed.contains("GET-DATA") {
                    self.highlight_line_with_command(ui, line, "GET-DATA");
                }
                // Comments
                else if trimmed.starts_with("#") {
                    ui.colored_label(egui::Color32::from_rgb(106, 153, 85), line);
                }
                // Regular line
                else {
                    ui.monospace(line);
                }
            });
        }
    }

    fn highlight_line_with_command(&self, ui: &mut egui::Ui, line: &str, command: &str) {
        if let Some(pos) = line.find(command) {
            let before = &line[..pos];
            let after = &line[pos + command.len()..];
            ui.monospace(before);
            ui.colored_label(egui::Color32::from_rgb(78, 201, 176), command);
            ui.monospace(after);
        } else {
            ui.monospace(line);
        }
    }

    /// Enhanced Dashboard with backend connectivity
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏠 Data Designer Dashboard");
        ui.separator();

        // Connection status at top
        ui.horizontal(|ui| {
            ui.label("Backend Status:");
            match &self.connection_status {
                ConnectionStatus::Connected => {
                    ui.colored_label(egui::Color32::GREEN, "✅ Connected");
                }
                ConnectionStatus::Connecting => {
                    ui.colored_label(egui::Color32::YELLOW, "🔄 Connecting...");
                }
                ConnectionStatus::Failed(err) => {
                    ui.colored_label(egui::Color32::RED, format!("❌ Failed: {}", err));
                }
                ConnectionStatus::Disconnected => {
                    ui.colored_label(egui::Color32::GRAY, "⚫ Disconnected");
                }
            }
        });

        ui.add_space(20.0);

        // Quick navigation cards
        ui.horizontal(|ui| {
            // Resource Templates card
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("📋 Resource Templates");
                    ui.label("Design and manage resource templates");
                    ui.add_space(10.0);
                    if ui.button("Manage Templates").clicked() {
                        self.router.navigate_to(AppRoute::ResourceTemplates);
                    }
                });
            });

            ui.add_space(10.0);

            // Private Data card
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("🔒 Private Data");
                    ui.label("Configure derived attributes");
                    ui.add_space(10.0);
                    if ui.button("Manage Private Data").clicked() {
                        self.router.navigate_to(AppRoute::PrivateData);
                    }
                });
            });

            ui.add_space(10.0);

            // Onboarding Requests card
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.heading("🚀 Onboarding");
                    ui.label("Create and manage workflows");
                    ui.add_space(10.0);
                    if ui.button("Start Onboarding").clicked() {
                        self.router.navigate_to(AppRoute::OnboardingRequests);
                    }
                });
            });
        });

        ui.add_space(20.0);

        // Recent activity (will be loaded from backend)
        ui.group(|ui| {
            ui.heading("📈 Recent Activity");
            ui.separator();
            if self.connection_status == ConnectionStatus::Connected {
                ui.label("Connected to backend - activity data would load here");
                // TODO: Load actual activity data from backend
            } else {
                ui.colored_label(egui::Color32::GRAY, "Connect to backend to view recent activity");
            }
        });
    }

    /// Resource Templates management (connects to backend)
    fn show_resource_templates(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Resource Templates Management");
        ui.separator();

        // Auto-connect on first visit
        if self.connection_status == ConnectionStatus::Disconnected {
            self.attempt_api_connection();
        }

        // Template Designer Integration
        ui.horizontal(|ui| {
            ui.label("Template Operations:");
            if ui.button("Create New Template").clicked() {
                // Switch to template designer
                self.router.navigate_to(AppRoute::TemplateDesigner);
            }
            if ui.button("Import Template").clicked() {
                // TODO: Implement template import
            }
            if ui.button("🔄 Refresh Templates").clicked() {
                self.load_templates_from_api();
            }
        });

        ui.add_space(10.0);

        // Template list from backend
        ui.group(|ui| {
            ui.heading("📝 Available Templates");
            ui.separator();

            if self.connection_status == ConnectionStatus::Connected {
                // Auto-load templates if we haven't yet
                if self.available_templates.is_empty() && self.api_client.is_some() {
                    self.load_templates_from_api();
                }
                // Use existing template list from backend
                self.show_simple_templates(ui);
            } else {
                ui.colored_label(egui::Color32::GRAY, "⚫ Connect to backend to load templates");
                if ui.button("🔌 Connect to Backend").clicked() {
                    self.attempt_api_connection();
                }
            }
        });
    }

    /// Private Data management (derived attributes)
    fn show_private_data(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔒 Private Data Management");
        ui.separator();

        ui.label("Manage derived attributes and private data transformations");
        ui.add_space(10.0);

        // Data Designer Integration
        ui.horizontal(|ui| {
            if ui.button("Open Data Designer").clicked() {
                self.router.navigate_to(AppRoute::DataDesigner);
            }
            if ui.button("View Transpiler").clicked() {
                self.router.navigate_to(AppRoute::Transpiler);
            }
        });

        ui.add_space(10.0);

        // Private attribute configuration
        ui.group(|ui| {
            ui.heading("🧮 Derived Attributes");
            ui.separator();

            if self.connection_status == ConnectionStatus::Connected {
                ui.label("📊 Connected to backend - derived attributes would load here");
                // TODO: Load derived attributes from backend via data dictionary
                ui.add_space(10.0);
                ui.label("Sample derived attributes:");
                ui.monospace("Internal.risk_score = CALCULATE(Client.credit_rating, Client.assets)");
                ui.monospace("Internal.entity_type = LOOKUP(Client.jurisdiction, jurisdiction_rules)");
                ui.monospace("Internal.compliance_flags = VALIDATE(Client.kyc_data, compliance_rules)");
            } else {
                ui.colored_label(egui::Color32::GRAY, "⚫ Connect to backend to manage private data");
            }
        });
    }

    /// Onboarding Requests management (runtime execution)
    fn show_onboarding_requests(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Onboarding Requests");
        ui.separator();

        if self.connection_status != ConnectionStatus::Connected {
            ui.colored_label(egui::Color32::RED, "❌ Backend connection required for onboarding workflows");
            if ui.button("🔌 Connect to Backend").clicked() {
                self.attempt_api_connection();
            }
            return;
        }

        // Workflow creation section
        ui.group(|ui| {
            ui.heading("📋 Create New Workflow");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Workflow Type:");
                ui.text_edit_singleline(&mut self.workflow_type);
            });

            ui.horizontal(|ui| {
                ui.label("Jurisdiction:");
                ui.text_edit_singleline(&mut self.jurisdiction);
            });

            ui.vertical(|ui| {
                ui.label("Initial Data (JSON):");
                ui.text_edit_multiline(&mut self.initial_data_json);
            });

            ui.add_space(10.0);

            if ui.button("🚀 Start Workflow").clicked() {
                self.start_workflow();
            }
        });

        ui.add_space(10.0);

        // Current workflow status
        if let Some(instance_id) = self.onboarding_instance_id.clone() {
            let mut refresh_clicked = false;
            let mut stop_clicked = false;

            ui.group(|ui| {
                ui.heading(format!("📊 Workflow Status: {}", instance_id));
                ui.separator();

                if ui.button("🔄 Refresh Status").clicked() {
                    refresh_clicked = true;
                }
            });

            if let Some(status) = &self.workflow_status {
                ui.add_space(5.0);
                ui.label(format!("Status: {}", status.get("status").unwrap_or(&serde_json::Value::String("Unknown".to_string()))));

                // Show collected data
                if let Some(collected_data) = status.get("collected_data") {
                    ui.collapsing("📝 Collected Data", |ui| {
                        ui.monospace(collected_data.to_string());
                    });
                }

                // Show pending solicitations
                if let Some(solicitations) = status.get("pending_solicitations").and_then(|s| s.as_array()) {
                    if !solicitations.is_empty() {
                        ui.collapsing(format!("📋 Pending Solicitations ({})", solicitations.len()), |ui| {
                            for solicitation in solicitations {
                                if let Some(attr_path) = solicitation.get("attribute_path").and_then(|a| a.as_str()) {
                                    ui.horizontal(|ui| {
                                        ui.label(attr_path);
                                        if let Some(required) = solicitation.get("required").and_then(|r| r.as_bool()) {
                                            if required {
                                                ui.colored_label(egui::Color32::RED, "Required");
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    }
                }

                // Show next actions
                if let Some(next_actions) = status.get("next_actions").and_then(|a| a.as_array()) {
                    if !next_actions.is_empty() {
                        ui.collapsing("📋 Next Actions", |ui| {
                            for action in next_actions {
                                if let Some(action_text) = action.as_str() {
                                    ui.label(format!("• {}", action_text));
                                }
                            }
                        });
                    }
                }
            }

            ui.add_space(10.0);
            if ui.button("🛑 Stop Workflow").clicked() {
                stop_clicked = true;
            }

            // Handle button clicks outside the closure
            if refresh_clicked {
                self.refresh_workflow_status();
            }
            if stop_clicked {
                self.stop_workflow();
            }
        }
    }

    /// Start a new onboarding workflow
    fn start_workflow(&mut self) {
        if let Some(_api_client) = &self.api_client {
            let workflow_type = self.workflow_type.clone();
            let jurisdiction = self.jurisdiction.clone();
            let _initial_data: Option<serde_json::Value> = if self.initial_data_json.trim().is_empty() {
                None
            } else {
                match serde_json::from_str(&self.initial_data_json) {
                    Ok(data) => Some(data),
                    Err(_) => {
                        self.error_message = Some("Invalid JSON in initial data".to_string());
                        return;
                    }
                }
            };

            // TODO: Make actual API call to start workflow
            // For now, simulate with a generated instance ID
            let instance_id = format!("ONBOARD_{}_{}",
                workflow_type.to_uppercase(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            );
            self.onboarding_instance_id = Some(instance_id);

            // Simulate initial status
            self.workflow_status = Some(serde_json::json!({
                "status": "Initialized",
                "collected_data": {},
                "pending_solicitations": [],
                "next_actions": ["Resolving template dependencies"]
            }));

            wasm_utils::console_log(&format!("Started workflow: {} for {}", workflow_type, jurisdiction));
        }
    }

    /// Refresh workflow status from backend
    fn refresh_workflow_status(&mut self) {
        if let Some(instance_id) = &self.onboarding_instance_id {
            // TODO: Make actual API call to get workflow status
            wasm_utils::console_log(&format!("Refreshing status for workflow: {}", instance_id));

            // Simulate status update
            self.workflow_status = Some(serde_json::json!({
                "status": "CollectingData",
                "collected_data": {
                    "Client.legal_entity_name": "Example Corp Ltd",
                    "Client.client_id": format!("CLIENT_{}", instance_id)
                },
                "pending_solicitations": [
                    {
                        "attribute_path": "Client.incorporation_date",
                        "data_type": "date",
                        "required": true,
                        "description": "Date of incorporation"
                    },
                    {
                        "attribute_path": "Client.business_type",
                        "data_type": "string",
                        "required": true,
                        "description": "Type of business entity"
                    }
                ],
                "next_actions": [
                    "Complete 2 pending data solicitations",
                    "Provide incorporation date",
                    "Select business type"
                ]
            }));
        }
    }

    /// Stop the current workflow
    fn stop_workflow(&mut self) {
        if let Some(instance_id) = &self.onboarding_instance_id {
            // TODO: Make actual API call to stop workflow
            wasm_utils::console_log(&format!("Stopping workflow: {}", instance_id));
            self.onboarding_instance_id = None;
            self.workflow_status = None;
            self.pending_solicitations.clear();
        }
    }
}