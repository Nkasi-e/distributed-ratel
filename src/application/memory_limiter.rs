use async_trait::async_trait;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::domain::key::RateLimitKey;
use crate::domain::sliding_window::SlidingWindowState;
use crate::domain::token_bucket::{TokenBucketConfig, TokenBucketState};

use super::error::AppError;
use super::policy::{PolicyTable, ResolvedRateLimitPolicy};
use super::ports::{MonotonicClock, RateLimitStore};

pub struct MemoryRateLimiter {
    policy: PolicyTable,
    clock: Arc<dyn MonotonicClock>,
    buckets: DashMap<RateLimitKey, Mutex<LimiterState>>,
}

enum LimiterState {
    Tb(TokenBucketState),
    Sw(SlidingWindowState),
}

impl MemoryRateLimiter {
    pub fn new(policy: PolicyTable, clock: Arc<dyn MonotonicClock>) -> Self {
        Self {
            policy,
            clock,
            buckets: DashMap::new(),
        }
    }

    fn reset_state(policy: &ResolvedRateLimitPolicy, now: std::time::Duration) -> LimiterState {
        match policy {
            ResolvedRateLimitPolicy::TokenBucket(cfg) => {
                LimiterState::Tb(TokenBucketState::new_full_at(now, cfg))
            }
            ResolvedRateLimitPolicy::SlidingWindow(_) => LimiterState::Sw(SlidingWindowState::new()),
        }
    }

    fn policy_matches(state: &LimiterState, policy: &ResolvedRateLimitPolicy) -> bool {
        matches!((state, policy), (LimiterState::Tb(_), ResolvedRateLimitPolicy::TokenBucket(_)) | (LimiterState::Sw(_), ResolvedRateLimitPolicy::SlidingWindow(_)))
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
}

#[async_trait]
impl RateLimitStore for MemoryRateLimiter {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        let now = self.clock.elapsed();
        let policy = self.policy.resolve(key.kind());
        let cost_u = cost as u64;

        match self.buckets.entry(key.clone()) {
            Entry::Occupied(o) => {
                let mut guard = o.get().lock();
                if Self::policy_matches(&guard, &policy) {
                    *guard = Self::reset_state(&policy, now)
                }
                Ok(match (&policy, &mut *guard) {
                    (ResolvedRateLimitPolicy::TokenBucket(cfg), LimiterState::Tb(tb)) => {
                        tb.try_allow(cfg, now, cost_u)?
                    }
                    (ResolvedRateLimitPolicy::SlidingWindow(cfg), LimiterState::Sw(sw)) => {
                        sw.try_allow(cfg, now, cost_u)?
                    }
                    _ => unreachable!("Policy_matches ensures alignment"),
                })
            }

            Entry::Vacant(v) => {
                let mut guard = Self::reset_state(&policy, now);
                let ok = match(&policy, &mut guard) {
                    (ResolvedRateLimitPolicy::TokenBucket(cfg), LimiterState::Tb(tb)) => {
                        tb.try_allow(cfg, now, cost_u)?
                    }
                    (ResolvedRateLimitPolicy::SlidingWindow(cfg), LimiterState::Sw(sw)) => {
                        sw.try_allow(cfg, now, cost_u)?
                    }
                    _ => unreachable!("reset_state matches policy"),
                };
                v.insert(Mutex::new(guard));
                Ok(ok)
            }
        }
    }
}
