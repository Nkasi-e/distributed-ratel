use figment::{Figment, providers::Env, providers::Format, providers::Toml};
use serde::Deserialize;

use crate::application::policy::RateLimitPolicyConfig;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub rate_limit: RateLimitPolicyConfig,
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
