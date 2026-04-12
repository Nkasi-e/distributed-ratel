#![forbid(unsafe_code)]

//! Layered rate limiter:

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub mod prelude {
    pub use crate::application::ports::RateLimiter;
    pub use crate::domain::key::{KeyKind, RateLimitKey};
}
