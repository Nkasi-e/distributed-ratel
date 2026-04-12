use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::key::RateLimitKey;

use super::error::AppError;
use super::memory_limiter::MemoryRateLimiter;
use super::ports::RateLimiter;

/// Thin facade over the concrete store. - swap MemoryRateLimiterInner for RedisRateLimiterInner as needed (upon redis integration).
pub struct AllowService {
    inner: Arc<MemoryRateLimiter>,
}

impl AllowService {
    pub fn new(inner: Arc<MemoryRateLimiter>) -> Self {
        Self { inner }
    }

    pub fn memory(limit: MemoryRateLimiter) -> Self {
        Self {
            inner: Arc::new(limit),
        }
    }
}

#[async_trait]
impl RateLimiter for AllowService {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        self.inner.allow(key, cost).await
    }
}
