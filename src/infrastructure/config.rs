use figment::{Figment, providers::Env, providers::Format, providers::Toml};
use serde::Deserialize;

use crate::application::policy::RateLimitPolicyConfig;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub rate_limit: RateLimitPolicyConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub backend: BackendKind,
    #[serde(default)]
    pub fallback_strategy: FallbackStrategy,
    #[serde(default)]
    pub redis: RedisConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: BackendKind::Memory,
            fallback_strategy: FallbackStrategy::FailClose,
            redis: RedisConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    #[default]
    Memory,
    Redis,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    FailOpen,
    #[default]
    FailClose,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            key_prefix: default_key_prefix(),
        }
    }
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379/".to_string()
}

fn default_key_prefix() -> String {
    "rate_limit".to_string()
}

impl AppConfig {
    /// Merge `config/default.toml` (optional) + `RATEL__` env overrides.
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Env::prefixed("RATEL__").split("__"))
            .extract()
    }
}
