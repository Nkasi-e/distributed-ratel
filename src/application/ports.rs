use async_trait::async_trait;

use crate::domain::key::RateLimitKey;

use super::error::AppError;

/// Inbound clock: monotonic elapsed time since process start (or similar).
pub trait MonotonicClock: Send + Sync {
    fn elapsed(&self) -> std::time::Duration;
}

#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError>;
}
