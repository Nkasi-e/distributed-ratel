use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] crate::domain::error::DomainError),

    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}
