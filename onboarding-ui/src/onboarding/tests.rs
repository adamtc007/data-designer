#[cfg(test)]
mod tests {
    use crate::onboarding::{
        OnboardingManager, OnboardingAction, OnboardingStep, ClientData, DocumentType, Document, KycStatus
    };

    #[tokio::test]
    async fn test_manager_initialization() {
        let manager = OnboardingManager::new("http://localhost:8080");
        let state = manager.state();

        assert_eq!(state.current_step, OnboardingStep::ClientInfo);
        assert!(state.client_data.is_none());
        assert_eq!(state.kyc_status, KycStatus::NotStarted);
        assert!(!state.can_proceed);
        assert!(!state.can_go_back);
        assert_eq!(state.progress_percentage, 0); // Default state, no progress yet
    }

    #[tokio::test]
    async fn test_start_onboarding_flow() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Start onboarding
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_client_123".to_string(),
        });

        // Process the action
        manager.update().await;

        let state = manager.state();
        assert_eq!(state.current_step, OnboardingStep::ClientInfo);
        assert!(state.client_data.is_some());
        assert_eq!(state.client_id, Some("test_client_123".to_string()));

        if let Some(client_data) = &state.client_data {
            assert_eq!(client_data.name, "Client test_client_123");
            assert_eq!(client_data.email, "clienttest_client_123@example.com");
        }
    }

    #[tokio::test]
    async fn test_workflow_progression() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Start onboarding
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_client_456".to_string(),
        });
        manager.update().await;

        // Should be able to proceed after client info is loaded
        let state = manager.state();
        assert!(state.can_proceed);

        // Advance to KYC step
        manager.dispatch(OnboardingAction::AdvanceStep);
        manager.update().await;

        let state = manager.state();
        assert_eq!(state.current_step, OnboardingStep::KycDocuments);
        assert!(state.can_go_back);
        assert_eq!(state.progress_percentage, 33); // KycDocuments step
    }

    #[tokio::test]
    async fn test_document_upload() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Start onboarding first
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_client_789".to_string(),
        });
        manager.update().await;

        // Upload a document
        let document = Document {
            id: "doc_123".to_string(),
            name: "passport.pdf".to_string(),
            document_type: DocumentType::Identity,
            file_path: "/uploads/passport.pdf".to_string(),
            uploaded_at: Some("2023-01-01T00:00:00Z".to_string()),
            verified: false,
        };

        manager.dispatch(OnboardingAction::UploadDocument { document });
        manager.update().await;

        let state = manager.state();
        assert_eq!(state.kyc_documents.len(), 1);
        assert_eq!(state.kyc_documents[0].name, "passport.pdf");
    }

    #[tokio::test]
    async fn test_kyc_submission() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Start onboarding and add a document
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_client_kyc".to_string(),
        });
        manager.update().await;

        let document = Document {
            id: "doc_456".to_string(),
            name: "identity.pdf".to_string(),
            document_type: DocumentType::Identity,
            file_path: "/uploads/identity.pdf".to_string(),
            uploaded_at: Some("2023-01-01T00:00:00Z".to_string()),
            verified: false,
        };

        manager.dispatch(OnboardingAction::UploadDocument { document });
        manager.update().await;

        // Submit KYC
        manager.dispatch(OnboardingAction::SubmitKyc);
        manager.update().await;

        let state = manager.state();
        assert_eq!(state.kyc_status, KycStatus::Approved);
    }

    #[tokio::test]
    async fn test_risk_calculation() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Start onboarding
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_client_risk".to_string(),
        });
        manager.update().await;

        // Calculate risk
        manager.dispatch(OnboardingAction::CalculateRisk);
        manager.update().await;

        let state = manager.state();
        assert!(state.risk_score.is_some());
        assert!(state.risk_rating.is_some());
        assert_eq!(state.risk_score.unwrap(), 25);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Try to submit KYC without documents
        manager.dispatch(OnboardingAction::SubmitKyc);
        manager.update().await;

        let state = manager.state();
        assert!(state.has_errors());
        assert!(state.validation_errors.contains(&"No KYC documents to submit".to_string()));
    }

    #[tokio::test]
    async fn test_action_queuing() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Dispatch multiple actions
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_queue".to_string(),
        });
        manager.dispatch(OnboardingAction::CalculateRisk);
        manager.dispatch(OnboardingAction::ClearErrors);

        assert!(manager.has_pending_actions());

        // Process first action
        manager.update().await;
        assert!(manager.has_pending_actions()); // Still has 2 more

        // Process second action
        manager.update().await;
        assert!(manager.has_pending_actions()); // Still has 1 more

        // Process third action
        manager.update().await;
        assert!(!manager.has_pending_actions()); // All processed
    }

    #[tokio::test]
    async fn test_state_workflow_validation() {
        let mut manager = OnboardingManager::new("http://localhost:8080");

        // Initially cannot proceed or go back
        let state = manager.state();
        assert!(!state.can_proceed);
        assert!(!state.can_go_back);

        // After starting onboarding, can proceed
        manager.dispatch(OnboardingAction::StartOnboarding {
            client_id: "test_validation".to_string(),
        });
        manager.update().await;

        let state = manager.state();
        assert!(state.can_proceed);
        assert!(!state.can_go_back);

        // After advancing, can go back
        manager.dispatch(OnboardingAction::AdvanceStep);
        manager.update().await;

        let state = manager.state();
        assert!(state.can_go_back);
    }

    #[test]
    fn test_client_data_validation() {
        let client_data = ClientData {
            name: "Test Client".to_string(),
            email: "test@example.com".to_string(),
            lei_code: "LEI123456".to_string(),
            phone: Some("+1-555-0123".to_string()),
            address: Some("123 Test St".to_string()),
            company_name: Some("Test Corp".to_string()),
            business_type: Some("Testing".to_string()),
        };

        assert!(!client_data.name.is_empty());
        assert!(!client_data.email.is_empty());
        assert!(!client_data.lei_code.is_empty());
    }
}