//! Core CCTP bridge implementation
//!
//! This module provides the primary types and functionality for bridging USDC across
//! chains using Circle's Cross-Chain Transfer Protocol (CCTP).

mod bridge_trait;
mod cctp;
mod config;
mod params;
mod v2;

pub use bridge_trait::CctpBridge;
pub use cctp::Cctp;
pub use params::BridgeParams;
pub use v2::CctpV2;
