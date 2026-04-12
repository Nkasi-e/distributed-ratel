use std::time::Duration;

use super::error::DomainError;

/// Static parameter for one bucket (per key or per key-kind).
#[derive(Debug, Clone)]
pub struct TokenBucketConfig {
    pub capacity: u64,
    /// Tokens added per wall second (Fractional allowed)
    pub refill_per_second: u64,
}

#[derive(Debug, Clone)]
pub struct TokenBucketState {
    /// Fractional token for smooth refill.
    tokens: f64,
    last_refill: Duration,
}

impl TokenBucketState {
    /// New bucket: start full at `now`.
    pub fn new_full_at(now: Duration, cfg: &TokenBucketConfig) -> Self {
        Self {
            tokens: cfg.capacity as f64,
            last_refill: now,
        }
    }

    /// Refill,  then consume `cost` if possible. Always advances refill time to `now`.
    pub fn try_allow(
        &mut self,
        cfg: &TokenBucketConfig,
        now: Duration,
        cost: u64,
    ) -> Result<bool, DomainError> {
        if cost == 0 {
            return Err(DomainError::InvalidCost);
        }

        let elapsed = now.saturating_sub(self.last_refill);
        let add = elapsed.as_secs_f64() * cfg.refill_per_second as f64;
        self.tokens = (self.tokens + add).min(cfg.capacity as f64);
        self.last_refill = now;

        let c = cost as f64;
        if self.tokens >= c {
            self.tokens -= c;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// exposed for tests / metrics
    pub fn token(&self) -> f64 {
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TokenBucketConfig {
        TokenBucketConfig {
            capacity: 10,
            refill_per_second: 1.0 as u64,
        }
    }

    #[test]
    fn burst_then_refill() {
        let c = cfg();
        let mut s = TokenBucketState::new_full_at(Duration::ZERO, &c);
        assert!(s.try_allow(&c, Duration::ZERO, 10).unwrap());
        assert!(!s.try_allow(&c, Duration::ZERO, 1).unwrap());

        assert!(s.try_allow(&c, Duration::from_secs(1), 1).unwrap());
    }
}
