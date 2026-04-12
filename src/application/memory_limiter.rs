use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::domain::key::RateLimitKey;
use crate::domain::token_bucket::{TokenBucketConfig, TokenBucketState};

use super::error::AppError;
use super::policy::PolicyTable;
use super::ports::MonotonicClock;

pub struct MemoryRateLimiter {
    policy: PolicyTable,
    clock: Arc<dyn MonotonicClock>,
    buckets: DashMap<RateLimitKey, Mutex<TokenBucketState>>,
}

impl MemoryRateLimiter {
    pub fn new(policy: PolicyTable, clock: Arc<dyn MonotonicClock>) -> Self {
        Self {
            policy,
            clock,
            buckets: DashMap::new(),
        }
    }

    // pub async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
    //     let now = self.clock.elapsed();
    //     let cfg: TokenBucketConfig = self.policy.resolve(key.kind());
    //     let cost_u = cost as u64;

    //     let allowed = self.buckets.entry(key.clone()).or_insert_with(|| {
    //         Mutex::new(TokenBucketState::new_full_at(now, &cfg))
    //     }).with_mut(|mtx| {
    //         let mut state = mtx.lock();
    //         state.try_allow(&cfg, now, cost_u)
    //     })?;

    //     Ok(allowed)
    // }

    pub async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        let now = self.clock.elapsed();
        let cfg: TokenBucketConfig = self.policy.resolve(key.kind());

        match self.buckets.entry(key.clone()) {
            Entry::Occupied(o) => {
                let mut state = o.get().lock();
                Ok(state.try_allow(&cfg, now, cost as u64)?)
            }
            Entry::Vacant(v) => {
                let mut state = TokenBucketState::new_full_at(now, &cfg);
                let ok = state.try_allow(&cfg, now, cost as u64)?;
                v.insert(Mutex::new(state));
                Ok(ok)
            }
        }
    }
}
