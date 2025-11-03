//! Tokio-based clock implementation.

use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::traits::Clock;

/// Production clock implementation using Tokio's time functions.
///
/// This provider uses the real system clock and Tokio's async sleep,
/// making it suitable for production use.
///
/// For testing, you can implement a fake clock that allows fast-forwarding
/// time without actually waiting.
///
/// # Examples
///
/// ```rust
/// use cctp_rs::providers::TokioClock;
///
/// let clock = TokioClock::new();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TokioClock;

impl TokioClock {
    /// Creates a new Tokio clock instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TokioClock {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Clock for TokioClock {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    fn now(&self) -> Instant {
        Instant::now()
    }
}
