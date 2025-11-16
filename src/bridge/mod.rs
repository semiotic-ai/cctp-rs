//! Core CCTP bridge implementation
//!
//! This module provides the primary types and functionality for bridging USDC across
//! chains using Circle's Cross-Chain Transfer Protocol (CCTP).

mod cctp;
mod config;
mod params;

pub use cctp::Cctp;
pub use params::BridgeParams;
