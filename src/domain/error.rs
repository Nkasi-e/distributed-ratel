use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("cost must be positive")]
    InvalidCost,
}
