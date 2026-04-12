use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::application::ports::MonotonicClock;

/// Elapsed since first call to `start` (lazy) or since construction.
pub struct SystemClock {
    start: Mutex<Option<Instant>>,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: Mutex::new(None),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock for SystemClock {
    fn elapsed(&self) -> Duration {
        let mut g = self.start.lock().unwrap();
        let now = Instant::now();
        if let Some(t0) = *g {
            now.duration_since(t0)
        } else {
            *g = Some(now);
            Duration::ZERO
        }
    }
}

// Use this simpler version for production semantics. If you want to use `Instant::now()` directly, you can uncomment the following code in the `SystemClock` struct.

// pub struct SystemClock {
//     start: Instant,
// }
// impl SystemClock {
//     pub fn new() -> Self {
//         Self { start: Instant::now() }
//     }
// }
// impl MonotonicClock for SystemClock {
//     fn elapsed(&self) -> Duration {
//         self.start.elapsed()
//     }
// }
