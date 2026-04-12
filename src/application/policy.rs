use crate::domain::key::KeyKind;
use crate::domain::token_bucket::TokenBucketConfig;
use serde::Deserialize;
use std::collections::HashMap;

/// Loaded from config: default + optional overrides per `KeyKind`.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitPolicyConfig {
    pub default: TokenBucketConfigSerde,
    #[serde(default)]
    pub by_kind: HashMap<KeyKindSerde, TokenBucketConfigSerde>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBucketConfigSerde {
    pub capacity: u64,
    pub refill_per_second: f64,
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

#[derive(Debug, Clone)]
pub struct PolicyTable {
    default: TokenBucketConfig,
    by_kind: HashMap<KeyKind, TokenBucketConfig>,
}

impl PolicyTable {
    pub fn from_config(cfg: RateLimitPolicyConfig) -> Self {
        let default = TokenBucketConfig::from(&cfg.default);
        let mut by_kind = HashMap::new();
        for (k, v) in cfg.by_kind {
            by_kind.insert(k.into(), TokenBucketConfig::from(&v));
        }
        Self { default, by_kind }
    }

    pub fn resolve(&self, kind: KeyKind) -> TokenBucketConfig {
        self.by_kind
            .get(&kind)
            .cloned()
            .unwrap_or_else(|| self.default.clone())
    }
}
