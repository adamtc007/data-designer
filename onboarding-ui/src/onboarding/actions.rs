use super::state::{AccountConfig, ClientData, Document, OnboardingStep};

#[derive(Debug, Clone)]
pub enum OnboardingAction {
    // Workflow control
    StartOnboarding { client_id: String },
    AdvanceStep,
    GoBackStep,
    JumpToStep(OnboardingStep),
    Reset,

    // Data updates
    UpdateClientInfo { data: ClientData },
    UploadDocument { document: Document },
    RemoveDocument { doc_id: String },
    SubmitKyc,

    // Risk assessment
    CalculateRisk,
    OverrideRiskScore { score: u32, reason: String },

    // Account setup
    ConfigureAccount { config: AccountConfig },
    FinalizeOnboarding,

    // Manual refresh
    RefreshData,

    // Error handling
    ClearErrors,
    DismissWarning { index: usize },

    // UI state management
    SetLoading { loading: bool },
}

impl OnboardingAction {
    pub fn description(&self) -> &'static str {
        match self {
            OnboardingAction::StartOnboarding { .. } => "Starting onboarding process",
            OnboardingAction::AdvanceStep => "Advancing to next step",
            OnboardingAction::GoBackStep => "Going back to previous step",
            OnboardingAction::JumpToStep(_) => "Jumping to specific step",
            OnboardingAction::Reset => "Resetting onboarding",
            OnboardingAction::UpdateClientInfo { .. } => "Updating client information",
            OnboardingAction::UploadDocument { .. } => "Uploading document",
            OnboardingAction::RemoveDocument { .. } => "Removing document",
            OnboardingAction::SubmitKyc => "Submitting KYC documents",
            OnboardingAction::CalculateRisk => "Calculating risk assessment",
            OnboardingAction::OverrideRiskScore { .. } => "Overriding risk score",
            OnboardingAction::ConfigureAccount { .. } => "Configuring account settings",
            OnboardingAction::FinalizeOnboarding => "Finalizing onboarding",
            OnboardingAction::RefreshData => "Refreshing data from server",
            OnboardingAction::ClearErrors => "Clearing validation errors",
            OnboardingAction::DismissWarning { .. } => "Dismissing warning",
            OnboardingAction::SetLoading { .. } => "Updating loading state",
        }
    }

    pub fn is_async(&self) -> bool {
        match self {
            // These require backend calls
            OnboardingAction::StartOnboarding { .. } |
            OnboardingAction::SubmitKyc |
            OnboardingAction::CalculateRisk |
            OnboardingAction::OverrideRiskScore { .. } |
            OnboardingAction::FinalizeOnboarding |
            OnboardingAction::RefreshData => true,

            // These are local state changes
            OnboardingAction::AdvanceStep |
            OnboardingAction::GoBackStep |
            OnboardingAction::JumpToStep(_) |
            OnboardingAction::Reset |
            OnboardingAction::UpdateClientInfo { .. } |
            OnboardingAction::UploadDocument { .. } |
            OnboardingAction::RemoveDocument { .. } |
            OnboardingAction::ConfigureAccount { .. } |
            OnboardingAction::ClearErrors |
            OnboardingAction::DismissWarning { .. } |
            OnboardingAction::SetLoading { .. } => false,
        }
    }

    pub fn requires_network(&self) -> bool {
        self.is_async()
    }
}