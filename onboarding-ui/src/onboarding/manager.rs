use std::collections::VecDeque;
use super::{OnboardingAction, DslState, OnboardingStep, ClientData, Document, AccountConfig, KycStatus, RiskRating};
use super::client::OnboardingClient;

pub struct OnboardingManager {
    // Current state - single source of truth
    state: DslState,

    // Onboarding-specific client for API calls
    onboarding_client: OnboardingClient,

    // Action queue for sequential processing
    pending_actions: VecDeque<OnboardingAction>,

    // Internal state management
    is_processing: bool,
}

impl OnboardingManager {
    pub fn new(endpoint: &str) -> Self {
        Self {
            state: DslState::default(),
            onboarding_client: OnboardingClient::new(endpoint),
            pending_actions: VecDeque::new(),
            is_processing: false,
        }
    }

    pub fn new_with_client(onboarding_client: OnboardingClient) -> Self {
        Self {
            state: DslState::default(),
            onboarding_client,
            pending_actions: VecDeque::new(),
            is_processing: false,
        }
    }

    /// UI calls this - synchronous, just queues the action
    pub fn dispatch(&mut self, action: OnboardingAction) {
        log::debug!("Dispatching action: {:?}", action.description());
        self.pending_actions.push_back(action);
    }

    /// Call this each frame - processes one action from queue
    pub async fn update(&mut self) {
        if self.is_processing {
            return; // Prevent recursive processing
        }

        if let Some(action) = self.pending_actions.pop_front() {
            self.is_processing = true;
            log::debug!("Processing action: {}", action.description());

            self.handle_action(action).await;

            self.is_processing = false;
        }
    }

    /// UI reads this - immutable reference
    pub fn state(&self) -> &DslState {
        &self.state
    }

    /// Check if there are pending actions (for UI repaint requests)
    pub fn has_pending_actions(&self) -> bool {
        !self.pending_actions.is_empty() || self.is_processing
    }

    /// Clear all pending actions (useful for reset scenarios)
    pub fn clear_pending_actions(&mut self) {
        self.pending_actions.clear();
    }

    /// Internal - processes actions and updates state
    async fn handle_action(&mut self, action: OnboardingAction) {
        // Clear previous errors unless it's an error-handling action
        if !matches!(action, OnboardingAction::ClearErrors | OnboardingAction::DismissWarning { .. }) {
            self.state.clear_errors();
        }

        match action {
            // Workflow control actions
            OnboardingAction::StartOnboarding { client_id } => {
                self.handle_start_onboarding(client_id).await;
            }
            OnboardingAction::AdvanceStep => {
                self.handle_advance_step();
            }
            OnboardingAction::GoBackStep => {
                self.handle_go_back_step();
            }
            OnboardingAction::JumpToStep(step) => {
                self.handle_jump_to_step(step);
            }
            OnboardingAction::Reset => {
                self.handle_reset();
            }

            // Data update actions
            OnboardingAction::UpdateClientInfo { data } => {
                self.handle_update_client_info(data);
            }
            OnboardingAction::UploadDocument { document } => {
                self.handle_upload_document(document).await;
            }
            OnboardingAction::RemoveDocument { doc_id } => {
                self.handle_remove_document(doc_id);
            }
            OnboardingAction::SubmitKyc => {
                self.handle_submit_kyc().await;
            }

            // Risk assessment actions
            OnboardingAction::CalculateRisk => {
                self.handle_calculate_risk().await;
            }
            OnboardingAction::OverrideRiskScore { score, reason } => {
                self.handle_override_risk_score(score, reason).await;
            }

            // Account setup actions
            OnboardingAction::ConfigureAccount { config } => {
                self.handle_configure_account(config);
            }
            OnboardingAction::FinalizeOnboarding => {
                self.handle_finalize_onboarding().await;
            }

            // Utility actions
            OnboardingAction::RefreshData => {
                self.handle_refresh_data().await;
            }
            OnboardingAction::ClearErrors => {
                self.state.clear_errors();
            }
            OnboardingAction::DismissWarning { index } => {
                self.handle_dismiss_warning(index);
            }
            OnboardingAction::SetLoading { loading } => {
                self.state.set_loading(loading);
            }
        }

        // Update workflow state after each action
        self.update_workflow_state();
    }

    /// Update can_proceed and can_go_back flags based on current state
    fn update_workflow_state(&mut self) {
        self.state.can_proceed = self.state.can_advance_from_current_step();
        self.state.can_go_back = self.state.can_go_back_from_current_step();
        self.state.update_progress();
    }
}

// Action handler implementations
impl OnboardingManager {
    async fn handle_start_onboarding(&mut self, client_id: String) {
        self.state.set_loading(true);
        self.state.client_id = Some(client_id.clone());

        // TODO: Replace with actual gRPC call
        // For now, create mock client data
        match self.mock_start_onboarding(&client_id).await {
            Ok(client_data) => {
                self.state.client_data = Some(client_data);
                self.state.current_step = OnboardingStep::ClientInfo;
                log::info!("Onboarding started for client: {}", client_id);
            }
            Err(error) => {
                self.state.add_error(format!("Failed to start onboarding: {}", error));
                log::error!("Failed to start onboarding: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    fn handle_advance_step(&mut self) {
        if !self.state.can_proceed {
            self.state.add_error("Cannot advance: current step requirements not met".to_string());
            return;
        }

        self.state.current_step = match self.state.current_step {
            OnboardingStep::ClientInfo => OnboardingStep::KycDocuments,
            OnboardingStep::KycDocuments => OnboardingStep::RiskAssessment,
            OnboardingStep::RiskAssessment => OnboardingStep::AccountSetup,
            OnboardingStep::AccountSetup => OnboardingStep::Review,
            OnboardingStep::Review => OnboardingStep::Complete,
            OnboardingStep::Complete => OnboardingStep::Complete, // Stay at complete
        };

        log::info!("Advanced to step: {:?}", self.state.current_step);
    }

    fn handle_go_back_step(&mut self) {
        if !self.state.can_go_back {
            self.state.add_error("Cannot go back from the first step".to_string());
            return;
        }

        self.state.current_step = match self.state.current_step {
            OnboardingStep::ClientInfo => OnboardingStep::ClientInfo, // Stay at first step
            OnboardingStep::KycDocuments => OnboardingStep::ClientInfo,
            OnboardingStep::RiskAssessment => OnboardingStep::KycDocuments,
            OnboardingStep::AccountSetup => OnboardingStep::RiskAssessment,
            OnboardingStep::Review => OnboardingStep::AccountSetup,
            OnboardingStep::Complete => OnboardingStep::Review,
        };

        log::info!("Went back to step: {:?}", self.state.current_step);
    }

    fn handle_jump_to_step(&mut self, step: OnboardingStep) {
        // TODO: Add validation for whether jump is allowed
        self.state.current_step = step;
        log::info!("Jumped to step: {:?}", step);
    }

    fn handle_reset(&mut self) {
        self.state = DslState::default();
        self.pending_actions.clear();
        log::info!("Onboarding state reset");
    }

    fn handle_update_client_info(&mut self, data: ClientData) {
        self.state.client_data = Some(data);
        log::info!("Client information updated");
    }

    async fn handle_upload_document(&mut self, document: Document) {
        self.state.set_loading(true);

        // TODO: Replace with actual document upload API call
        match self.mock_upload_document(&document).await {
            Ok(_) => {
                self.state.kyc_documents.push(document);
                log::info!("Document uploaded successfully");
            }
            Err(error) => {
                self.state.add_error(format!("Failed to upload document: {}", error));
                log::error!("Failed to upload document: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    fn handle_remove_document(&mut self, doc_id: String) {
        self.state.kyc_documents.retain(|doc| doc.id != doc_id);
        log::info!("Document removed: {}", doc_id);
    }

    async fn handle_submit_kyc(&mut self) {
        self.state.set_loading(true);

        if self.state.kyc_documents.is_empty() {
            self.state.add_error("No KYC documents to submit".to_string());
            self.state.set_loading(false);
            return;
        }

        // TODO: Replace with actual KYC submission API call
        match self.mock_submit_kyc().await {
            Ok(status) => {
                self.state.kyc_status = status;
                log::info!("KYC submitted successfully");
            }
            Err(error) => {
                self.state.add_error(format!("Failed to submit KYC: {}", error));
                log::error!("Failed to submit KYC: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    async fn handle_calculate_risk(&mut self) {
        self.state.set_loading(true);

        // TODO: Replace with actual risk calculation API call
        match self.mock_calculate_risk().await {
            Ok((score, rating)) => {
                self.state.risk_score = Some(score);
                self.state.risk_rating = Some(rating);
                log::info!("Risk calculated: score={}, rating={:?}", score, rating);
            }
            Err(error) => {
                self.state.add_error(format!("Failed to calculate risk: {}", error));
                log::error!("Failed to calculate risk: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    async fn handle_override_risk_score(&mut self, score: u32, reason: String) {
        self.state.set_loading(true);

        // TODO: Replace with actual risk override API call
        match self.mock_override_risk_score(score, &reason).await {
            Ok(rating) => {
                self.state.risk_score = Some(score);
                self.state.risk_rating = Some(rating);
                self.state.add_warning(format!("Risk score overridden: {}", reason));
                log::info!("Risk score overridden: score={}, reason={}", score, reason);
            }
            Err(error) => {
                self.state.add_error(format!("Failed to override risk score: {}", error));
                log::error!("Failed to override risk score: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    fn handle_configure_account(&mut self, config: AccountConfig) {
        self.state.account_config = Some(config);
        log::info!("Account configuration updated");
    }

    async fn handle_finalize_onboarding(&mut self) {
        self.state.set_loading(true);

        // TODO: Replace with actual finalization API call
        match self.mock_finalize_onboarding().await {
            Ok(session_id) => {
                self.state.session_id = Some(session_id);
                self.state.current_step = OnboardingStep::Complete;
                log::info!("Onboarding finalized successfully");
            }
            Err(error) => {
                self.state.add_error(format!("Failed to finalize onboarding: {}", error));
                log::error!("Failed to finalize onboarding: {}", error);
            }
        }

        self.state.set_loading(false);
    }

    async fn handle_refresh_data(&mut self) {
        self.state.set_loading(true);

        // TODO: Replace with actual data refresh API calls
        log::info!("Refreshing data from server");

        self.state.set_loading(false);
    }

    fn handle_dismiss_warning(&mut self, index: usize) {
        if index < self.state.warnings.len() {
            self.state.warnings.remove(index);
            log::debug!("Warning dismissed at index: {}", index);
        }
    }
}

// Mock implementations for testing - will be replaced with real gRPC calls
impl OnboardingManager {
    async fn mock_start_onboarding(&self, client_id: &str) -> Result<ClientData, String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(ClientData {
            name: format!("Client {}", client_id),
            email: format!("client{}@example.com", client_id),
            lei_code: format!("LEI{}", client_id),
            phone: Some("+1-555-0100".to_string()),
            address: Some("123 Business Ave, Suite 100".to_string()),
            company_name: Some("Example Corp".to_string()),
            business_type: Some("Financial Services".to_string()),
        })
    }

    async fn mock_upload_document(&self, _document: &Document) -> Result<(), String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        Ok(())
    }

    async fn mock_submit_kyc(&self) -> Result<KycStatus, String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        Ok(KycStatus::Approved)
    }

    async fn mock_calculate_risk(&self) -> Result<(u32, RiskRating), String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        Ok((25, RiskRating::Low))
    }

    async fn mock_override_risk_score(&self, score: u32, _reason: &str) -> Result<RiskRating, String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        let rating = match score {
            0..=20 => RiskRating::Low,
            21..=50 => RiskRating::Medium,
            51..=80 => RiskRating::High,
            _ => RiskRating::Critical,
        };

        Ok(rating)
    }

    async fn mock_finalize_onboarding(&self) -> Result<String, String> {
        // Simulate network delay
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        Ok(format!("session_{}", uuid::Uuid::new_v4()))
    }
}