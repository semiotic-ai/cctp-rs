//! Core CCTP bridge implementation
//!
//! This module provides the primary types and functionality for bridging USDC across
//! chains using Circle's Cross-Chain Transfer Protocol (CCTP).

mod cctp;
mod config;
mod params;

pub use cctp::Cctp;
pub use config::{
    get_chain_confirmation_config, ATTESTATION_PATH_V1, CHAIN_CONFIRMATION_CONFIG,
    DEFAULT_CONFIRMATION_TIMEOUT, IRIS_API, IRIS_API_SANDBOX,
};
pub use params::BridgeParams;
