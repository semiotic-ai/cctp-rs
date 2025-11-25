//! Chain configuration and contract addresses for CCTP
//!
//! This module contains chain-specific configuration including contract addresses,
//! confirmation times, and domain ID mappings for all supported CCTP chains.
//!
//! - `CctpV1`: Original 7-chain support
//! - `CctpV2`: Enhanced 26+ chain support with Fast Transfer

pub mod addresses;
mod config;
mod v2;

pub use config::CctpV1;
pub use v2::CctpV2;
