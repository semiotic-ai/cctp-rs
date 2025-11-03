//! # cctp-rs
//!
//! A production-ready Rust SDK for Circle's Cross-Chain Transfer Protocol (CCTP) v2.0.
//!
//! This library provides a safe, ergonomic, and **fully testable** interface for bridging
//! USDC across multiple blockchain networks using Circle's CCTP infrastructure.
//!
//! ## What's New in v2.0
//!
//! Version 2.0 introduces a **trait-based architecture** that enables comprehensive testing
//! through dependency injection. All external I/O operations (blockchain RPC calls,
//! attestation API requests, time operations) are abstracted behind traits, allowing you
//! to implement test fakes for adversarial testing scenarios.
//!
//! ### Breaking Changes
//!
//! - `Cctp` now has 6 type parameters instead of 1
//! - Builder requires explicit provider, attestation provider, and clock injection
//! - See the migration guide below for details
//!
//! ## Quick Start
//!
//! ### Production Usage
//!
//! ```rust,no_run
//! use cctp_rs::{Cctp, CctpError, UniversalReceiptAdapter};
//! use cctp_rs::providers::{AlloyProvider, IrisAttestationProvider, TokioClock};
//! use alloy_chains::NamedChain;
//! use alloy_network::Ethereum;
//! use alloy_primitives::FixedBytes;
//! use alloy_provider::ProviderBuilder;
//!
//! # async fn example() -> Result<(), CctpError> {
//! // Set up providers
//! let eth_provider = ProviderBuilder::new().on_builtin("http://localhost:8545").await?;
//! let arb_provider = ProviderBuilder::new().on_builtin("http://localhost:8546").await?;
//!
//! // Create bridge with production providers
//! let bridge = Cctp::builder()
//!     .source_chain(NamedChain::Mainnet)
//!     .destination_chain(NamedChain::Arbitrum)
//!     .source_provider(AlloyProvider::new(eth_provider))
//!     .destination_provider(AlloyProvider::new(arb_provider))
//!     .attestation_provider(IrisAttestationProvider::production())
//!     .clock(TokioClock::new())
//!     .receipt_adapter(UniversalReceiptAdapter)
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
//! ### Testing with Fakes
//!
//! See the [`examples/test_fakes.rs`](https://github.com/semiotic-ai/cctp-rs/blob/main/examples/test_fakes.rs)
//! file for comprehensive examples of implementing test fakes for adversarial testing.
//!
//! ## Migration Guide (v1.x → v2.0)
//!
//! **v1.x code:**
//! ```rust,ignore
//! let bridge = Cctp::builder()
//!     .source_provider(eth_provider)
//!     .destination_provider(arb_provider)
//!     // ...
//!     .build();
//! ```
//!
//! **v2.0 code:**
//! ```rust,ignore
//! use cctp_rs::providers::{AlloyProvider, IrisAttestationProvider, TokioClock};
//!
//! let bridge = Cctp::builder()
//!     .source_provider(AlloyProvider::new(eth_provider))
//!     .destination_provider(AlloyProvider::new(arb_provider))
//!     .attestation_provider(IrisAttestationProvider::production())
//!     .clock(TokioClock::new())
//!     // ...
//!     .build();
//! ```
//!
//! ## Features
//!
//! - **Trait-based abstraction** enabling comprehensive testing
//! - **Multi-network support** including Optimism via op-alloy
//! - **Type-safe contract interactions** using Alloy
//! - **Multi-chain support** for mainnet and testnet networks
//! - **Comprehensive error handling** with detailed error types
//! - **Builder pattern** for intuitive API usage
//!
//! ## Key Types
//!
//! - [`Cctp`] - Main bridge struct with trait-based architecture
//! - [`CctpError`] - Comprehensive error types
//! - [`AttestationStatus`] - Attestation state tracking
//! - [`CctpV1`] - Trait for chain-specific configuration
//! - [`BlockchainProvider`] - Trait for blockchain RPC operations
//! - [`AttestationProvider`] - Trait for attestation API operations
//! - [`Clock`] - Trait for time operations
//!
//! ## Modules
//!
//! - [`traits`] - Core trait abstractions for blockchain, attestation, and time operations
//! - [`providers`] - Production implementations of the trait abstractions

mod attestation;
mod bridge;
mod chain;
mod domain_id;
mod error;
mod message_transmitter;
mod receipt_adapter;
mod token_messenger;

pub mod providers;
pub mod traits;

// Testing utilities are always available so users can test their own code
pub mod testing;

pub use receipt_adapter::{ReceiptAdapter, UniversalReceiptAdapter};

/// Adapter for extracting logs from transaction receipts on all EVM-compatible networks.
///
/// This is an alias for [`UniversalReceiptAdapter`]. Both names work identically.
pub type EthereumReceiptAdapter = UniversalReceiptAdapter;

pub use attestation::*;
pub use bridge::*;
pub use chain::*;
pub use domain_id::*;
pub use error::*;
pub use message_transmitter::*;
pub use token_messenger::*;
pub use traits::{AttestationProvider, BlockchainProvider, Clock};

// Type aliases for common configurations
use alloy_network::Ethereum;
use providers::{AlloyProvider, IrisAttestationProvider, TokioClock};

/// CCTP bridge configured for Ethereum-compatible networks using production providers.
///
/// This type alias simplifies the common case of using CCTP with Ethereum-compatible
/// networks. It uses:
/// - [`providers::AlloyProvider`] for blockchain RPC calls
/// - [`providers::IrisAttestationProvider`] for Circle's Iris API
/// - [`providers::TokioClock`] for time operations
/// - [`UniversalReceiptAdapter`] for extracting transaction logs
///
/// # Example
///
/// ```rust,no_run
/// # use cctp_rs::{EthereumCctp, CctpError};
/// # use alloy_chains::NamedChain;
/// # use alloy_provider::ProviderBuilder;
/// # use alloy_primitives::Address;
/// # use cctp_rs::providers::{AlloyProvider, IrisAttestationProvider, TokioClock};
/// # use cctp_rs::UniversalReceiptAdapter;
/// # async fn example() -> Result<(), CctpError> {
/// let eth_provider = ProviderBuilder::new().on_builtin("http://localhost:8545").await?;
/// let arb_provider = ProviderBuilder::new().on_builtin("http://localhost:8546").await?;
///
/// let bridge: EthereumCctp<_, _> = cctp_rs::Cctp::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Arbitrum)
///     .source_provider(AlloyProvider::new(eth_provider))
///     .destination_provider(AlloyProvider::new(arb_provider))
///     .attestation_provider(IrisAttestationProvider::production())
///     .clock(TokioClock::new())
///     .receipt_adapter(UniversalReceiptAdapter)
///     .recipient(Address::ZERO)
///     .build();
/// # Ok(())
/// # }
/// ```
pub type EthereumCctp<SP, DP> = Cctp<
    Ethereum,
    Ethereum,
    AlloyProvider<Ethereum, SP>,
    AlloyProvider<Ethereum, DP>,
    IrisAttestationProvider,
    TokioClock,
    UniversalReceiptAdapter,
>;
