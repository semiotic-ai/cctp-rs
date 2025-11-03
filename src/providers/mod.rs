//! Production implementations of CCTP trait abstractions.
//!
//! This module provides the "real" implementations of the traits defined in
//! [`crate::traits`] that interact with actual blockchain networks, Circle's
//! Iris API, and the system clock.
//!
//! Users building applications will typically use these providers, while
//! test code will implement custom fakes.

mod alloy;
mod iris;
mod tokio_clock;

pub use self::alloy::AlloyProvider;
pub use self::iris::IrisAttestationProvider;
pub use self::tokio_clock::TokioClock;
