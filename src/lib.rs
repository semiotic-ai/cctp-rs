//! # cctp-rs
//!
//! A production-ready Rust SDK for Circle's Cross-Chain Transfer Protocol (CCTP).
//!
//! This library provides a safe, ergonomic interface for bridging USDC across
//! multiple blockchain networks using Circle's CCTP infrastructure.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use cctp_rs::{Cctp, CctpError};
//! use alloy_chains::NamedChain;
//! use alloy_primitives::FixedBytes;
//!
//! # async fn example() -> Result<(), CctpError> {
//! # use alloy_provider::ProviderBuilder;
//! // Set up providers and create bridge
//! let eth_provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
//! let arb_provider = ProviderBuilder::new().connect("http://localhost:8546").await?;
//!
//! let bridge = Cctp::builder()
//!     .source_chain(NamedChain::Mainnet)
//!     .destination_chain(NamedChain::Arbitrum)
//!     .source_provider(eth_provider)
//!     .destination_provider(arb_provider)
//!     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
//!     .build();
//!
//! // Get attestation for a bridge transaction
//! let message_hash: FixedBytes<32> = [0u8; 32].into();
//! let attestation = bridge.get_attestation_with_retry(message_hash, None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Type-safe contract interactions** using Alloy
//! - **Multi-chain support** for mainnet and testnet networks
//! - **Comprehensive error handling** with detailed error types
//! - **Builder pattern** for intuitive API usage
//! - **Extensive test coverage** ensuring reliability
//!
//! ## Modules
//!
//! - [`attestation`] - Types for Circle's Iris API attestation responses
//! - [`bridge`] - Core CCTP bridge implementation
//! - [`chain`] - Chain-specific configurations and the `CctpV1` trait
//! - [`error`] - Error types and result type alias
//! - [`domain_id`] - CCTP domain ID constants for supported chains
//! - [`message_transmitter`] - MessageTransmitter contract bindings
//! - [`token_messenger`] - TokenMessenger contract bindings

mod attestation;
mod bridge;
mod chain;
mod domain_id;
mod error;
mod message_transmitter;
mod spans;
mod token_messenger;

pub use attestation::*;
pub use bridge::*;
pub use chain::*;
pub use domain_id::*;
pub use error::*;
pub use message_transmitter::*;
pub use token_messenger::*;
