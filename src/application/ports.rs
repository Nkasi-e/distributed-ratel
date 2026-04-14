use async_trait::async_trait;

use crate::domain::key::RateLimitKey;

use super::error::AppError;

/// Monotonic elasped time provider.
pub trait MonotonicClock: Send + Sync {
    fn elapsed(&self) -> std::time::Duration;
}

/// Public contract consumed by handlers/services.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError>;
}

/// Internal store contract implemented by memory/redis backend
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError>;
}
