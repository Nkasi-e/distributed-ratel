use async_trait::async_trait;

use crate::domain::key::RateLimitKey;

use super::error::AppError;

/// Monotonic elasped time provider.
pub trait MonotonicClock: Send + Sync {
    fn elapsed(&self) -> std::time::Duration;
}

#[derive(Debug, Clone)]
pub struct RateLimitOutcome {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub reset_after_secs: u64,
}

/// Public contract consumed by handlers/services.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn allow_with_outcome(
        &self,
        key: &RateLimitKey,
        cost: u32,
    ) -> Result<RateLimitOutcome, AppError>;

    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        Ok(self.allow_with_outcome(key, cost).await?.allowed)
    }
}

/// Internal store contract implemented by memory/redis backend
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    async fn allow_with_outcome(
        &self,
        key: &RateLimitKey,
        cost: u32,
    ) -> Result<RateLimitOutcome, AppError>;
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        Ok(self.allow_with_outcome(key, cost).await?.allowed)
    }
}
