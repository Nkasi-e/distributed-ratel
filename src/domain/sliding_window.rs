//! Sliding-window counter: sum of cost in `[now - window, now` must stay ≤ `max_cost_per_window`.

use std::collections::VecDeque;
use std::time::Duration;

use super::error::DomainError;

#[derive(Debug, Clone)]
pub struct SlidingWindowConfig {
    pub window: Duration,
    pub max_cost_per_window: u64,
}

#[derive(Debug, Clone)]
pub struct SlidingWindowState {
    entries: VecDeque<(Duration, u64)>,
}

impl SlidingWindowState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    pub fn try_allow(
        &mut self,
        cfg: &SlidingWindowConfig,
        now: Duration,
        cost: u64,
    ) -> Result<bool, DomainError> {
        if cost == 0 {
            return Err(DomainError::InvalidCost);
        }
        if cost > cfg.max_cost_per_window {
            return Ok(false);
        }

        let horizon = now.saturating_sub(cfg.window);
        while let Some(&(t, _)) = self.entries.front() {
            if t < horizon {
                self.entries.pop_front();
            } else {
                break;
            }
        }

        let sum: u64 = self.entries.iter().map(|(_, c)| *c).sum();
        if sum + cost <= cfg.max_cost_per_window {
            self.entries.push_back((now, cost));
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for SlidingWindowState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_when_cost_exceeds_cap() {
        let c = SlidingWindowConfig {
            window: Duration::from_secs(10),
            max_cost_per_window: 10,
        };

        let mut s = SlidingWindowState::new();
        assert!(!s.try_allow(&c, Duration::ZERO, 11).unwrap());
    }

    #[test]
    fn window_slides() {
        let c = SlidingWindowConfig {
            window: Duration::from_secs(10),
            max_cost_per_window: 5,
        };

        let mut s = SlidingWindowState::new();
        assert!(s.try_allow(&c, Duration::ZERO, 5).unwrap());
        assert!(s.try_allow(&c, Duration::ZERO, 1).unwrap());
        assert!(s.try_allow(&c, Duration::from_secs(11), 5).unwrap());
    }
}
