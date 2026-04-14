use async_trait::async_trait;
use redis::Script;
use redis::aio::ConnectionManager;
use tokio::sync::Mutex;

use crate::application::error::AppError;
use crate::application::policy::PolicyTable;
use crate::application::ports::RateLimitStore;
use crate::domain::key::RateLimitKey;

use super::config::{FallbackStrategy, RedisConfig};

const TOKEN_BUCKET_LUA: &str = r#"
local capacity = tonumber(ARGV[1])
local refill_per_sec = tonumber(ARGV[2])
local cost = tonumber(ARGV[3])
local ttl_ms = tonumber(ARGV[4])

if cost <= 0 then
    retun redis.error_reply("invalid_cost")
end

local t = redis.call("TIME")
local now_ms = t[1] * 1000 + math.floor(t[2] / 1000)

local vals = redis.call("HMGET", KEYS[1], "tokens", "ts")
local tokens = tonumber(vals[1])
local ts = tonumber(vals[2])

if tokens == nil then
    tokens = capacity
    ts = now_ms
end

local elapsed_ms = math.max(0, now_ms - ts)
tokens = math.min(capacity, tokens + (elapsed_ms / 1000.0) * refill_per_sec)

local allowed = 0
if tokens >= cost then
    tokens = tokens - cost
    allowed = 1
end

redis.call("HMSET", KEYS[1], "tokens", tokens, "ts", now_ms)
redis.call("PEXPIRE", KEYS[1], ttl_ms)

return allowed
"#;

pub struct RedisRateLimiter {
    conn: Mutex<ConnectionManager>,
    script: Script,
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
            script: Script::new(TOKEN_BUCKET_LUA),
            key_prefix: redis_cfg.key_prefix,
            fallback_strategy,
            policy,
        })
    }

    fn redis_key(&self, key: &RateLimitKey) -> String {
        format!("{}:{}", self.key_prefix, serialize_key(key))
    }

    fn ttl_ms(capacity: u64, refill_per_sec: u64) -> u64 {
        if refill_per_sec == 0 {
            return 86_400_000; // 24 hours
        }
        let refill_seconds = ((capacity as f64) / (refill_per_sec as f64)).ceil() as u64;
        let ttl_seconds = refill_seconds.saturating_mul(2).max(1);
        ttl_seconds.saturating_mul(1000)
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
        let cfg = self.policy.resolve(key.kind());
        let redis_key = self.redis_key(key);
        let ttl_ms = Self::ttl_ms(cfg.capacity, cfg.refill_per_second);

        let mut conn = self.conn.lock().await;
        let result = self
            .script
            .key(redis_key)
            .arg(cfg.capacity as i64)
            .arg(cfg.refill_per_second as i64)
            .arg(cost as i64)
            .arg(ttl_ms as i64)
            .invoke_async::<i32>(&mut *conn)
            .await;

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
