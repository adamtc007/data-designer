pub mod actions;
pub mod client;
pub mod manager;
pub mod state;

#[cfg(test)]
mod tests;

pub use actions::OnboardingAction;
pub use client::OnboardingClient;
pub use manager::OnboardingManager;
pub use state::{
    AccountConfig, ClientData, Document, DocumentType, DslState, KycStatus, OnboardingStep,
    RiskRating,
};

// Re-export for convenience
pub mod prelude {
    pub use super::actions::OnboardingAction;
    pub use super::client::OnboardingClient;
    pub use super::manager::OnboardingManager;
    pub use super::state::{
        AccountConfig, ClientData, Document, DocumentType, DslState, KycStatus, OnboardingStep,
        RiskRating,
    };
}