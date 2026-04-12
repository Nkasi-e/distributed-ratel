use std::net::IpAddr;

/// Classifies a key for policy lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyKind {
    UserId,
    Ip,
    ApiKey,
    Custom,
}

/// Rate-Limit identity, Extend with new variants as products need them
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    UserId(String),
    Ip(IpAddr),
    ApiKey(String),
    Custom(String),
}

impl RateLimitKey {
    pub fn kind(&self) -> KeyKind {
        match self {
            RateLimitKey::UserId(_) => KeyKind::UserId,
            RateLimitKey::Ip(_) => KeyKind::Ip,
            RateLimitKey::ApiKey(_) => KeyKind::ApiKey,
            RateLimitKey::Custom(_) => KeyKind::Custom,
        }
    }
}
