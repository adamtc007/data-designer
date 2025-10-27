use super::state::{ClientData, Document, KycStatus, RiskRating, AccountConfig};
use crate::http_client::{GrpcClient, Result};
use serde::{Deserialize, Serialize};

/// Onboarding-specific client wrapper around the generic GrpcClient
pub struct OnboardingClient {
    grpc_client: GrpcClient,
}

impl OnboardingClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            grpc_client: GrpcClient::new(endpoint),
        }
    }

    pub async fn start_onboarding(&self, client_id: &str) -> Result<OnboardingResponse> {
        let request = StartOnboardingRequest {
            client_id: client_id.to_string(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/StartOnboarding", &request)
            .await
    }

    pub async fn submit_kyc(&self, documents: &[Document]) -> Result<KycResponse> {
        let request = SubmitKycRequest {
            documents: documents.to_vec(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/SubmitKyc", &request)
            .await
    }

    pub async fn calculate_risk(&self, client_data: &ClientData) -> Result<RiskResponse> {
        let request = CalculateRiskRequest {
            client_data: client_data.clone(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/CalculateRisk", &request)
            .await
    }

    pub async fn override_risk_score(
        &self,
        score: u32,
        reason: &str,
        client_id: &str,
    ) -> Result<RiskResponse> {
        let request = OverrideRiskRequest {
            client_id: client_id.to_string(),
            new_score: score,
            reason: reason.to_string(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/OverrideRisk", &request)
            .await
    }

    pub async fn finalize_onboarding(
        &self,
        client_id: &str,
        account_config: &AccountConfig,
    ) -> Result<FinalizeResponse> {
        let request = FinalizeOnboardingRequest {
            client_id: client_id.to_string(),
            account_config: account_config.clone(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/FinalizeOnboarding", &request)
            .await
    }

    pub async fn upload_document(&self, document: &Document) -> Result<UploadResponse> {
        let request = UploadDocumentRequest {
            document: document.clone(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/UploadDocument", &request)
            .await
    }

    pub async fn get_onboarding_status(&self, client_id: &str) -> Result<OnboardingStatusResponse> {
        let request = GetStatusRequest {
            client_id: client_id.to_string(),
        };

        self.grpc_client
            .grpc_call("onboarding.OnboardingService/GetStatus", &request)
            .await
    }
}

// Request/Response types for the onboarding API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartOnboardingRequest {
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResponse {
    pub success: bool,
    pub message: String,
    pub client_data: Option<ClientData>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitKycRequest {
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycResponse {
    pub success: bool,
    pub message: String,
    pub status: KycStatus,
    pub verification_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateRiskRequest {
    pub client_data: ClientData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResponse {
    pub success: bool,
    pub message: String,
    pub risk_score: u32,
    pub risk_rating: RiskRating,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideRiskRequest {
    pub client_id: String,
    pub new_score: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeOnboardingRequest {
    pub client_id: String,
    pub account_config: AccountConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeResponse {
    pub success: bool,
    pub message: String,
    pub session_id: String,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub document: Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStatusRequest {
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatusResponse {
    pub success: bool,
    pub message: String,
    pub current_step: String,
    pub progress_percentage: u8,
    pub client_data: Option<ClientData>,
    pub kyc_status: Option<KycStatus>,
    pub risk_score: Option<u32>,
    pub risk_rating: Option<RiskRating>,
    pub account_config: Option<AccountConfig>,
}