use crate::domain::key::KeyKind;
use crate::domain::token_bucket::TokenBucketConfig;
use crate::domain::sliding_window::SlidingWindowConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Loaded from config: default + optional overrides per `KeyKind`.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitPolicyConfig {
    pub default: RateLimitRuleSerde,
    #[serde(default)]
    pub by_kind: HashMap<KeyKindSerde, RateLimitRuleSerde>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBucketConfigSerde {
    pub capacity: u64,
    pub refill_per_second: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlidingWindowConfigSerde {
    pub window_secs: u64,
    pub max_cost_per_window: u64,
}



/// Exactly one nested block: `token_bucket` **or** `sliding_window_counter`.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitRuleSerde {
    #[serde(rename = "tocken_bucket")]
    pub token_bucket: Option<TokenBucketConfigSerde>,
    #[serde(rename = "sliding_window")]
    pub sliding_window_counter: Option<SlidingWindowConfigSerde>
}

/// Serde-friendly key kind (TOML keys are strings)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyKindSerde {
    UserId,
    Ip,
    ApiKey,
    Custom,
}

#[derive(Debug, Clone)]
pub enum ResolvedRateLimitPolicy {
    TokenBucket(TokenBucketConfig),
    SlidingWindow(SlidingWindowConfig),
}

impl From<&TokenBucketConfigSerde> for TokenBucketConfig {
    fn from(s: &TokenBucketConfigSerde) -> Self {
        TokenBucketConfig {
            capacity: s.capacity,
            refill_per_second: s.refill_per_second as u64,
        }
    }
}

impl From<KeyKindSerde> for KeyKind {
    fn from(k: KeyKindSerde) -> Self {
        match k {
            KeyKindSerde::UserId => KeyKind::UserId,
            KeyKindSerde::Ip => KeyKind::Ip,
            KeyKindSerde::Custom => KeyKind::Custom,
            KeyKindSerde::ApiKey => KeyKind::ApiKey,
        }
    }
}


#[derive(Debug, Error)]
pub enum PolicyConfigError {
    #[error("each rate limit rule must ser exactly one of `token_bucket` or `sliding_window_counter`")]
    AmbiguousRule,
}


fn resolve_rule(rule: &RateLimitRuleSerde) -> Result<ResolvedRateLimitPolicy, PolicyConfigError> {
    match (&rule.token_bucket, &rule.sliding_window_counter) {
        (Some(tb), None) => Ok(ResolvedRateLimitPolicy::TokenBucket(TokenBucketConfig::from(tb))),
        (None, Some(sw)) => Ok(ResolvedRateLimitPolicy::SlidingWindow(SlidingWindowConfig { window: Duration::from_secs(sw.window_secs), max_cost_per_window: sw.max_cost_per_window })),
        _ => Err(PolicyConfigError::AmbiguousRule),
    }
}


#[derive(Debug, Clone)]
pub struct PolicyTable {
    default: ResolvedRateLimitPolicy,
    by_kind: HashMap<KeyKind, ResolvedRateLimitPolicy>,
}

impl PolicyTable {
    pub fn try_from_config(cfg: RateLimitPolicyConfig) -> Result<Self, PolicyConfigError> {
        let default = resolve_rule(&cfg.default)?;
        let mut by_kind = HashMap::new();
        for (k, v) in cfg.by_kind {
            by_kind.insert(k.into(), resolve_rule(&v)?);
        }
        Ok(Self { default, by_kind })
    }

    pub fn resolve(&self, kind: KeyKind) -> ResolvedRateLimitPolicy {
        self.by_kind
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| self.default.clone())
    }
}
