use eframe::egui;
use crate::wasm_utils;
use crate::onboarding::{OnboardingManager, OnboardingAction, OnboardingStep, DslState};

/// Clean Onboarding Application following the state manager pattern
pub struct OnboardingApp {
    // Single source of truth - the OnboardingManager
    manager: OnboardingManager,

    // Temporary UI state (not part of the business logic state)
    client_id_input: String,
}

impl OnboardingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        wasm_utils::set_panic_hook();
        wasm_utils::console_log("🚀 Starting Clean Onboarding Workflow Platform");

        // Initialize manager with backend endpoint
        let manager = OnboardingManager::new("http://localhost:8080");

        Self {
            manager,
            client_id_input: String::new(),
        }
    }
}

impl eframe::App for OnboardingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process pending actions asynchronously
        // Note: We can't easily move the manager into an async context in this architecture
        // For now, we'll process actions synchronously in the UI thread
        // In a real implementation, you'd use a different async strategy

        // For WASM, we need to handle async differently
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, we'll use a different approach - spawn the async operation
            // but don't move the manager. Instead, we'll poll it in the next frame.
            if self.manager.has_pending_actions() {
                // Schedule async update and request repaint
                ctx.request_repaint();

                // For now, we'll just ensure the UI keeps updating
                // The actual async processing would need a more sophisticated setup
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // For desktop, we can use a simpler blocking approach for now
            // In a real implementation, you'd use proper async runtime integration
            if self.manager.has_pending_actions() {
                // For this demo, we'll just request repaints
                ctx.request_repaint();
            }
        }

        // Render UI based on current state
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render(ui);
        });
    }
}

impl OnboardingApp {
    fn render(&mut self, ui: &mut egui::Ui) {
        // Get a copy of the state data we need for rendering
        let state = self.manager.state();
        let current_step = state.current_step;
        let is_loading = state.is_loading;
        let progress_percentage = state.progress_percentage;
        let can_go_back = state.can_go_back;
        let can_proceed = state.can_proceed;
        let has_errors = !state.validation_errors.is_empty();
        let has_warnings = !state.warnings.is_empty();

        // Main heading
        ui.heading("🚀 Onboarding Workflow Platform");
        ui.separator();

        // Show loading indicator if processing
        if is_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Processing...");
            });
            ui.add_space(10.0);
        }

        // Progress bar
        if progress_percentage > 0 {
            ui.horizontal(|ui| {
                ui.label("Progress:");
                ui.add(egui::ProgressBar::new(progress_percentage as f32 / 100.0)
                    .text(format!("{}%", progress_percentage)));
            });
            ui.add_space(10.0);
        }

        // Render based on current step
        match current_step {
            OnboardingStep::ClientInfo => {
                self.render_client_info_step(ui);
            }
            OnboardingStep::KycDocuments => {
                self.render_kyc_documents_step(ui);
            }
            OnboardingStep::RiskAssessment => {
                self.render_risk_assessment_step(ui);
            }
            OnboardingStep::AccountSetup => {
                self.render_account_setup_step(ui);
            }
            OnboardingStep::Review => {
                self.render_review_step(ui);
            }
            OnboardingStep::Complete => {
                self.render_complete_step(ui);
            }
        }

        ui.add_space(15.0);

        // Navigation buttons
        self.render_navigation(ui, can_go_back, can_proceed);

        ui.add_space(15.0);

        // Show errors and warnings at bottom
        if has_errors || has_warnings {
            self.render_errors_and_warnings(ui);
        }
    }

    fn render_client_info_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let client_data = state.client_data.clone();

        ui.group(|ui| {
            ui.heading("📋 Client Information");
            ui.separator();

            if client_data.is_none() {
                // Start onboarding
                ui.label("Enter a client ID to start the onboarding process:");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.client_id_input)
                            .hint_text("Enter client ID (e.g., client_123)")
                            .desired_width(200.0)
                    );

                    if ui.button("🚀 Start Onboarding").clicked() ||
                       (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        if !self.client_id_input.trim().is_empty() {
                            self.manager.dispatch(OnboardingAction::StartOnboarding {
                                client_id: self.client_id_input.trim().to_string(),
                            });
                            self.client_id_input.clear(); // Clear input after starting
                        }
                    }
                });

                ui.add_space(10.0);
                ui.label("💡 Try entering 'demo_client' for a demo experience");
            } else {
                // Show client data
                if let Some(client_data) = &client_data {
                    egui::Grid::new("client_info_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.label(&client_data.name);
                            ui.end_row();

                            ui.label("Email:");
                            ui.label(&client_data.email);
                            ui.end_row();

                            ui.label("LEI Code:");
                            ui.label(&client_data.lei_code);
                            ui.end_row();

                            if let Some(phone) = &client_data.phone {
                                ui.label("Phone:");
                                ui.label(phone);
                                ui.end_row();
                            }

                            if let Some(company) = &client_data.company_name {
                                ui.label("Company:");
                                ui.label(company);
                                ui.end_row();
                            }

                            if let Some(business_type) = &client_data.business_type {
                                ui.label("Business Type:");
                                ui.label(business_type);
                                ui.end_row();
                            }
                        });

                    ui.add_space(10.0);
                    ui.label("✅ Client information loaded successfully");
                }
            }
        });
    }

    fn render_kyc_documents_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let kyc_status = state.kyc_status.clone();
        let kyc_documents = state.kyc_documents.clone();

        ui.group(|ui| {
            ui.heading("📄 KYC Documents");
            ui.separator();

            ui.label(format!("KYC Status: {:?}", kyc_status));
            ui.add_space(10.0);

            // Show uploaded documents
            if !kyc_documents.is_empty() {
                ui.label("Uploaded Documents:");
                for (i, doc) in kyc_documents.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}. {} ({:?})", i + 1, doc.name, doc.document_type));
                        if ui.small_button("🗑").clicked() {
                            self.manager.dispatch(OnboardingAction::RemoveDocument {
                                doc_id: doc.id.clone(),
                            });
                        }
                    });
                }
                ui.add_space(10.0);

                if ui.button("📤 Submit KYC Documents").clicked() {
                    self.manager.dispatch(OnboardingAction::SubmitKyc);
                }
            } else {
                ui.label("No documents uploaded yet.");
                ui.add_space(10.0);

                if ui.button("📁 Upload Mock Document").clicked() {
                    use crate::onboarding::{Document, DocumentType};
                    let document = Document {
                        id: format!("doc_{}", uuid::Uuid::new_v4()),
                        name: "passport.pdf".to_string(),
                        document_type: DocumentType::Identity,
                        file_path: "/mock/passport.pdf".to_string(),
                        uploaded_at: Some("2023-01-01T00:00:00Z".to_string()),
                        verified: false,
                    };
                    self.manager.dispatch(OnboardingAction::UploadDocument { document });
                }
            }
        });
    }

    fn render_risk_assessment_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let risk_score = state.risk_score;
        let risk_rating = state.risk_rating;

        ui.group(|ui| {
            ui.heading("⚖️ Risk Assessment");
            ui.separator();

            if let (Some(score), Some(rating)) = (risk_score, risk_rating) {
                ui.horizontal(|ui| {
                    ui.label("Risk Score:");
                    ui.strong(&score.to_string());
                    ui.label(format!("({:?})", rating));
                });
                ui.add_space(10.0);

                // Override risk score
                ui.horizontal(|ui| {
                    let mut override_score = score.to_string();
                    ui.label("Override Score:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut override_score)
                            .desired_width(80.0)
                    );

                    if ui.button("Override").clicked() && response.lost_focus() {
                        if let Ok(new_score) = override_score.parse::<u32>() {
                            self.manager.dispatch(OnboardingAction::OverrideRiskScore {
                                score: new_score,
                                reason: "Manual override by user".to_string(),
                            });
                        }
                    }
                });
            } else {
                ui.label("Risk assessment not yet performed.");
                ui.add_space(10.0);

                if ui.button("📊 Calculate Risk").clicked() {
                    self.manager.dispatch(OnboardingAction::CalculateRisk);
                }
            }
        });
    }

    fn render_account_setup_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let account_config = state.account_config.clone();

        ui.group(|ui| {
            ui.heading("⚙️ Account Setup");
            ui.separator();

            if let Some(config) = &account_config {
                ui.label(format!("Account Type: {}", config.account_type));
                ui.label(format!("Investment Objectives: {}", config.investment_objectives.join(", ")));
                ui.label(format!("Trading Permissions: {}", config.trading_permissions.join(", ")));
                ui.label(format!("Funding Sources: {}", config.funding_sources.join(", ")));
            } else {
                ui.label("Configure account settings:");
                ui.add_space(10.0);

                if ui.button("⚙️ Configure Standard Account").clicked() {
                    use crate::onboarding::AccountConfig;
                    let config = AccountConfig {
                        account_type: "Standard Trading Account".to_string(),
                        investment_objectives: vec!["Growth".to_string(), "Income".to_string()],
                        trading_permissions: vec!["Equities".to_string(), "Fixed Income".to_string()],
                        funding_sources: vec!["Bank Transfer".to_string(), "Wire Transfer".to_string()],
                    };
                    self.manager.dispatch(OnboardingAction::ConfigureAccount { config });
                }
            }
        });
    }

    fn render_review_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let client_data = state.client_data.clone();
        let kyc_status = state.kyc_status.clone();
        let risk_score = state.risk_score;
        let account_config = state.account_config.clone();

        ui.group(|ui| {
            ui.heading("📋 Review & Finalize");
            ui.separator();

            ui.label("Please review all information before finalizing:");
            ui.add_space(10.0);

            // Summary
            if let Some(client_data) = &client_data {
                ui.label(format!("Client: {}", client_data.name));
            }
            ui.label(format!("KYC Status: {:?}", kyc_status));
            if let Some(score) = risk_score {
                ui.label(format!("Risk Score: {}", score));
            }
            if let Some(config) = &account_config {
                ui.label(format!("Account Type: {}", config.account_type));
            }

            ui.add_space(15.0);

            if ui.button("✅ Finalize Onboarding").clicked() {
                self.manager.dispatch(OnboardingAction::FinalizeOnboarding);
            }
        });
    }

    fn render_complete_step(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let session_id = state.session_id.clone();

        ui.group(|ui| {
            ui.heading("🎉 Onboarding Complete!");
            ui.separator();

            ui.label("✅ Onboarding has been completed successfully.");

            if let Some(session_id) = &session_id {
                ui.label(format!("Session ID: {}", session_id));
            }

            ui.add_space(15.0);

            if ui.button("🔄 Start New Onboarding").clicked() {
                self.manager.dispatch(OnboardingAction::Reset);
            }
        });
    }

    fn render_navigation(&mut self, ui: &mut egui::Ui, can_go_back: bool, can_proceed: bool) {
        ui.horizontal(|ui| {
            // Back button
            if ui.add_enabled(can_go_back, egui::Button::new("← Back")).clicked() {
                self.manager.dispatch(OnboardingAction::GoBackStep);
            }

            ui.add_space(10.0);

            // Next button
            if ui.add_enabled(can_proceed, egui::Button::new("Next →")).clicked() {
                self.manager.dispatch(OnboardingAction::AdvanceStep);
            }

            ui.add_space(20.0);

            // Reset button
            if ui.button("🔄 Reset").clicked() {
                self.manager.dispatch(OnboardingAction::Reset);
            }

            ui.add_space(10.0);

            // Refresh button
            if ui.button("↻ Refresh").clicked() {
                self.manager.dispatch(OnboardingAction::RefreshData);
            }
        });
    }

    fn render_errors_and_warnings(&mut self, ui: &mut egui::Ui) {
        let state = self.manager.state();
        let validation_errors = state.validation_errors.clone();
        let warnings = state.warnings.clone();

        // Show errors
        if !validation_errors.is_empty() {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::RED, "❌ Errors:");
                for error in &validation_errors {
                    ui.label(format!("• {}", error));
                }
                if ui.button("Clear Errors").clicked() {
                    self.manager.dispatch(OnboardingAction::ClearErrors);
                }
            });
            ui.add_space(10.0);
        }

        // Show warnings
        if !warnings.is_empty() {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Warnings:");
                for (i, warning) in warnings.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {}", warning));
                        if ui.small_button("✕").clicked() {
                            self.manager.dispatch(OnboardingAction::DismissWarning { index: i });
                        }
                    });
                }
            });
        }
    }
}