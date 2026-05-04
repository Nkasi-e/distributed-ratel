use async_trait::async_trait;
use redis::Script;
use redis::aio::ConnectionManager;
use tokio::sync::Mutex;

use crate::application::error::AppError;
use crate::application::policy::{PolicyTable, ResolvedRateLimitPolicy};
use crate::application::ports::RateLimitStore;
use crate::domain::key::RateLimitKey;

use super::config::{FallbackStrategy, RedisConfig};

const TOKEN_BUCKET_LUA: &str = include_str!("../../lua/token_bucket.lua");
const SLIDING_WINDOW_LUA: &str = include_str!("../../lua/sliding_window.lua");

pub struct RedisRateLimiter {
    conn: Mutex<ConnectionManager>,
    script_tb: Script,
    script_sw: Script,
    key_prefix: String,
    fallback_strategy: FallbackStrategy,
    policy: PolicyTable,
}

impl RedisRateLimiter {
    pub async fn new(
        redis_cfg: RedisConfig,
        fallback_strategy: FallbackStrategy,
        policy: PolicyTable,
    ) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_cfg.url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn: Mutex::new(conn),
            script_tb: Script::new(TOKEN_BUCKET_LUA),
            script_sw: Script::new(SLIDING_WINDOW_LUA),
            key_prefix: redis_cfg.key_prefix,
            fallback_strategy,
            policy,
        })
    }

    fn redis_key_tb(&self, key: &RateLimitKey) -> String {
        format!("{}:{}", self.key_prefix, serialize_key(key))
    }

    fn redis_keys_sw(&self, key: &RateLimitKey) -> (String, String) {
        let base = format!("{}:{}:sw", self.key_prefix, serialize_key(key));
        (format!("{base}:z"), format!("{base}:seq"))
    }

    fn ttl_ms_tb(capacity: u64, refill_per_sec: u64) -> u64 {
        if refill_per_sec == 0 {
            return 86_400_000; // 24 hours
        }
        let refill_seconds = ((capacity as f64) / (refill_per_sec as f64)).ceil() as u64;
        let ttl_seconds = refill_seconds.saturating_mul(2).max(1);
        ttl_seconds.saturating_mul(1000)
    }

    fn ttl_ms_sw(window_ms: u64) -> u64 {
        window_ms.saturating_mul(2).max(1).saturating_add(1000)
    }

    fn apply_fallback(&self, err: redis::RedisError) -> Result<bool, AppError> {
        match self.fallback_strategy {
            FallbackStrategy::FailOpen => {
                tracing::warn!(error = %err, "redis unavailable, fail_open => allowing request");
                Ok(true)
            }
            FallbackStrategy::FailClose => {
                tracing::warn!(error = %err, "redis unavailable, fail_close => rejecting request");
                Ok(false)
            }
        }
    }
}

#[async_trait]
impl RateLimitStore for RedisRateLimiter {
    async fn allow(&self, key: &RateLimitKey, cost: u32) -> Result<bool, AppError> {
        let policy = self.policy.resolve(key.kind());
        let mut conn = self.conn.lock().await;

        let result = match policy {
            ResolvedRateLimitPolicy::TokenBucket(cfg) => {
                let redis_key = self.redis_key_tb(key);
                let ttl_ms = Self::ttl_ms_tb(cfg.capacity, cfg.refill_per_second);
                self.script_tb
                    .key(redis_key)
                    .arg(cfg.capacity as i64)
                    .arg(cfg.refill_per_second as i64)
                    .arg(cost as i64)
                    .arg(ttl_ms as i64)
                    .invoke_async::<i32>(&mut *conn)
                    .await
            }
            ResolvedRateLimitPolicy::SlidingWindow(cfg) => {
                let window_ms = cfg.window.as_millis() as u64;
                let (z_key, seq_key) = self.redis_keys_sw(key);
                let ttl_ms = Self::ttl_ms_sw(window_ms);
                self.script_sw
                    .key(z_key)
                    .key(seq_key)
                    .arg(window_ms as i64)
                    .arg(cfg.max_cost_per_window as i64)
                    .arg(cost as i64)
                    .arg(ttl_ms as i64)
                    .invoke_async::<i32>(&mut *conn)
                    .await
            }
        };
        match result {
            Ok(v) => Ok(v == 1),
            Err(err) => self.apply_fallback(err),
        }
    }
}

fn serialize_key(key: &RateLimitKey) -> String {
    match key {
        RateLimitKey::UserId(v) => format!("user_id:{v}"),
        RateLimitKey::Ip(v) => format!("ip:{v}"),
        RateLimitKey::ApiKey(v) => format!("api_key:{v}"),
        RateLimitKey::Custom(v) => format!("custom:{v}"),
    }
}
