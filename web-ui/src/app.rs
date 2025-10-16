use eframe::egui;
use crate::{AppRoute, WebRouter, wasm_utils};
use crate::resource_sheet_ui::ResourceSheetManager;
use crate::minimal_types::ResourceSheetRecord;
use crate::http_api_client::DataDesignerHttpClient;

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

    // Template Editor
    template_editor: crate::template_editor::TemplateEditor,

    // Debug tools
    show_debug_panel: bool,
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
            grpc_endpoint: "http://localhost:3030".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            api_client: None,
            template_editor: crate::template_editor::TemplateEditor::new(),
            show_debug_panel: false,
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
        });
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