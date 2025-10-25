use thiserror::Error;

#[derive(Error, Debug)]
pub enum OnboardingError {
    #[error("not found: {0}")]
    NotFound(String),
}
