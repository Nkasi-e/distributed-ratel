use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::key::RateLimitKey;

use super::error::AppError;
use super::ports::{RateLimitStore, RateLimiter};

/// Thin facade over the concrete store. - swap MemoryRateLimiterInner for RedisRateLimiterInner as needed (upon redis integration).
pub struct AllowService {
    store: Arc<dyn RateLimitStore>,
}

impl AllowService {
    pub fn new(store: Arc<dyn RateLimitStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl RateLimiter for AllowService {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        self.store.allow(key, cost).await
    }
}
