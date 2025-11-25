//! # cctp-rs
//!
//! A production-ready Rust SDK for Circle's Cross-Chain Transfer Protocol (CCTP).
//!
//! This library provides a safe, ergonomic interface for bridging USDC across
//! multiple blockchain networks using Circle's CCTP infrastructure.
//!
//! ## Quick Start (V1)
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
//! // Get message from burn transaction, then fetch attestation
//! let burn_tx_hash = FixedBytes::from([0u8; 32]);
//! let (message, message_hash) = bridge.get_message_sent_event(burn_tx_hash).await?;
//! let attestation = bridge.get_attestation(message_hash, None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Quick Start (V2)
//!
//! ```rust,no_run
//! use cctp_rs::{CctpV2Bridge, CctpError};
//! use alloy_chains::NamedChain;
//! use alloy_primitives::FixedBytes;
//!
//! # async fn example() -> Result<(), CctpError> {
//! # use alloy_provider::ProviderBuilder;
//! // V2 bridge with fast transfer support
//! let eth_provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
//! let linea_provider = ProviderBuilder::new().connect("http://localhost:8546").await?;
//!
//! let bridge = CctpV2Bridge::builder()
//!     .source_chain(NamedChain::Mainnet)
//!     .destination_chain(NamedChain::Linea)
//!     .source_provider(eth_provider)
//!     .destination_provider(linea_provider)
//!     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
//!     .fast_transfer(true)  // Enable sub-30 second settlement
//!     .build();
//!
//! // V2 uses transaction hash directly and returns both message and attestation
//! let burn_tx_hash = FixedBytes::from([0u8; 32]);
//! let (message, attestation) = bridge.get_attestation(burn_tx_hash, None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Direct Contract Access
//!
//! For advanced use cases, you can use the contract wrappers directly:
//!
//! ```rust,no_run
//! use cctp_rs::{TokenMessengerV2Contract, MessageTransmitterV2Contract};
//! use alloy_primitives::address;
//! use alloy_provider::ProviderBuilder;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
//! let contract_address = address!("9f3B8679c73C2Fef8b59B4f3444d4e156fb70AA5");
//!
//! // Create contract wrapper
//! let token_messenger = TokenMessengerV2Contract::new(contract_address, provider);
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
//! ## Public API
//!
//! - [`AttestationResponse`] and [`AttestationStatus`] - Circle's Iris API attestation types
//! - [`Cctp`] and [`CctpV2Bridge`] - Core CCTP bridge implementations for v1 and v2
//! - [`CctpV1`] and [`CctpV2`] - Traits for chain-specific configurations
//! - [`CctpError`] and [`Result`] - Error types for error handling
//! - Contract wrappers for direct contract interaction:
//!   - v1: [`TokenMessengerContract`], [`MessageTransmitterContract`]
//!   - v2: [`TokenMessengerV2Contract`], [`MessageTransmitterV2Contract`]

mod bridge;
mod chain;
mod contracts;
mod error;
mod protocol;

// Public API - minimal surface for 1.0.0 stability
pub use bridge::{Cctp, CctpBridge, CctpV2 as CctpV2Bridge};
pub use chain::addresses::{
    CCTP_V2_MESSAGE_TRANSMITTER_MAINNET, CCTP_V2_MESSAGE_TRANSMITTER_TESTNET,
    CCTP_V2_TOKEN_MESSENGER_MAINNET, CCTP_V2_TOKEN_MESSENGER_TESTNET,
};
pub use chain::{CctpV1, CctpV2};
pub use contracts::{
    erc20::Erc20Contract,
    message_transmitter::MessageTransmitterContract,
    token_messenger::TokenMessengerContract,
    v2::{MessageTransmitterV2Contract, TokenMessengerV2Contract},
};
pub use error::{CctpError, Result};
pub use protocol::{
    AttestationBytes, AttestationResponse, AttestationStatus, BurnMessageV2, DomainId,
    FinalityThreshold, MessageHeader, V2AttestationResponse, V2Message,
};

// Public module for advanced users who need custom instrumentation
pub mod spans;
