use std::net::IpAddr;
use std::str::FromStr;


use serde::{Deserialize, Serialize};

use crate::domain::key::RateLimitKey;
use crate::application::policy::ResolvedRateLimitPolicy;

/// `POST /allow` body
#[derive(Debug, Deserialize)]
pub struct AllowRequest {
    pub key: KeyDto,
    pub cost: u32,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KeyDto {
    UserId {id: String},
    Ip {addr: String},
    ApiKey {key: String},
    Custom {id: String},
}

impl TryFrom<KeyDto> for RateLimitKey {
    type Error = String;

    fn try_from(value: KeyDto) -> Result<Self, Self::Error> {
        match value {
            KeyDto::UserId { id } => Ok(RateLimitKey::UserId(id)),
            KeyDto::Ip { addr } => {
                let ip = IpAddr::from_str(&addr).map_err(|e| e.to_string())?;
                Ok(RateLimitKey::Ip(ip))
            }
            KeyDto::ApiKey { key } => Ok(RateLimitKey::ApiKey(key)),
            KeyDto::Custom { id } => Ok(RateLimitKey::Custom(id)),
        }
    }
}



/// `POST /allow` JSON response (extend when store returns real remaining/reset).
#[derive(Debug, Serialize)]
pub struct AllowResponse {
    pub allowed: bool,
    pub cost: u64,
/// TODO: real remaining from bucket/window state, not heuristic.
    pub remaining: u64,
    /// Unix seconds — TODO align with algorithm (window end / bucket refill). 
    pub reset_unix: u64,
}


/// Map policy to a single “limit” number for `X-RateLimit-Limit`.
pub fn limit_from_header(policy: &ResolvedRateLimitPolicy) -> u64 {
    match policy {
        ResolvedRateLimitPolicy::TokenBucket(tb) => tb.capacity,
        ResolvedRateLimitPolicy::SlidingWindow(sw) => sw.max_cost_per_window,
    }
}


/// Rough reset horizon in seconds from “now” for `X-RateLimit-Reset` stub.
pub fn reset_offset_secs(policy: &ResolvedRateLimitPolicy) -> u64 {
    match policy {
        ResolvedRateLimitPolicy::SlidingWindow(sw) => sw.window.as_secs().max(1),
        ResolvedRateLimitPolicy::TokenBucket(tb) => {
            let rps = tb.refill_per_second.max(1);
            tb.capacity.div_ceil(rps as u64)
        }
    }
}