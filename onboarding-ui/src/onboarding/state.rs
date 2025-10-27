use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OnboardingStep {
    ClientInfo,
    KycDocuments,
    RiskAssessment,
    AccountSetup,
    Review,
    Complete,
}

impl Default for OnboardingStep {
    fn default() -> Self {
        Self::ClientInfo
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KycStatus {
    NotStarted,
    InProgress,
    UnderReview,
    Approved,
    Rejected,
}

impl Default for KycStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskRating {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientData {
    pub name: String,
    pub email: String,
    pub lei_code: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company_name: Option<String>,
    pub business_type: Option<String>,
}

impl Default for ClientData {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: String::new(),
            lei_code: String::new(),
            phone: None,
            address: None,
            company_name: None,
            business_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub document_type: DocumentType,
    pub file_path: String,
    pub uploaded_at: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    Identity,
    AddressProof,
    BusinessRegistration,
    TaxDocument,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub account_type: String,
    pub investment_objectives: Vec<String>,
    pub trading_permissions: Vec<String>,
    pub funding_sources: Vec<String>,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            account_type: String::new(),
            investment_objectives: Vec::new(),
            trading_permissions: Vec::new(),
            funding_sources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslState {
    // Current workflow position
    pub current_step: OnboardingStep,

    // Client information
    pub client_data: Option<ClientData>,

    // KYC documents and status
    pub kyc_documents: Vec<Document>,
    pub kyc_status: KycStatus,

    // Risk assessment data
    pub risk_score: Option<u32>,
    pub risk_rating: Option<RiskRating>,

    // Account configuration
    pub account_config: Option<AccountConfig>,

    // Workflow control
    pub can_proceed: bool,
    pub can_go_back: bool,

    // Validation and errors
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,

    // UI state
    pub is_loading: bool,
    pub progress_percentage: u8,

    // Onboarding session
    pub session_id: Option<String>,
    pub client_id: Option<String>,
}

impl Default for DslState {
    fn default() -> Self {
        Self {
            current_step: OnboardingStep::default(),
            client_data: None,
            kyc_documents: Vec::new(),
            kyc_status: KycStatus::default(),
            risk_score: None,
            risk_rating: None,
            account_config: None,
            can_proceed: false,
            can_go_back: false,
            validation_errors: Vec::new(),
            warnings: Vec::new(),
            is_loading: false,
            progress_percentage: 0,
            session_id: None,
            client_id: None,
        }
    }
}

impl DslState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_errors(&mut self) {
        self.validation_errors.clear();
        self.warnings.clear();
    }

    pub fn add_error(&mut self, error: String) {
        self.validation_errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn update_progress(&mut self) {
        self.progress_percentage = match self.current_step {
            OnboardingStep::ClientInfo => 16,
            OnboardingStep::KycDocuments => 33,
            OnboardingStep::RiskAssessment => 50,
            OnboardingStep::AccountSetup => 66,
            OnboardingStep::Review => 83,
            OnboardingStep::Complete => 100,
        };
    }

    pub fn can_advance_from_current_step(&self) -> bool {
        match self.current_step {
            OnboardingStep::ClientInfo => {
                self.client_data.is_some()
                    && self.client_data.as_ref().unwrap().name.len() > 0
                    && self.client_data.as_ref().unwrap().email.len() > 0
            }
            OnboardingStep::KycDocuments => {
                self.kyc_status == KycStatus::Approved
                    && !self.kyc_documents.is_empty()
            }
            OnboardingStep::RiskAssessment => {
                self.risk_score.is_some() && self.risk_rating.is_some()
            }
            OnboardingStep::AccountSetup => {
                self.account_config.is_some()
                    && self.account_config.as_ref().unwrap().account_type.len() > 0
            }
            OnboardingStep::Review => {
                self.client_data.is_some()
                    && self.kyc_status == KycStatus::Approved
                    && self.risk_score.is_some()
                    && self.account_config.is_some()
            }
            OnboardingStep::Complete => false, // Cannot advance from completion
        }
    }

    pub fn can_go_back_from_current_step(&self) -> bool {
        self.current_step != OnboardingStep::ClientInfo
    }
}