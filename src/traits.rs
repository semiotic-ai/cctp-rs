//! Core trait abstractions for CCTP bridge operations.
//!
//! This module defines the fundamental traits that enable dependency injection
//! and testing of the CCTP bridge functionality. By abstracting blockchain
//! operations, attestation retrieval, and time control behind traits, users
//! can implement fake/mock versions for comprehensive testing including
//! adversarial scenarios.
//!
//! # Example: Implementing a Test Fake
//!
//! ```rust,ignore
//! use cctp::{BlockchainProvider, AttestationProvider, Clock};
//! use std::collections::HashMap;
//!
//! struct FakeBlockchainProvider {
//!     receipts: HashMap<TxHash, TransactionReceipt>,
//! }
//!
//! #[async_trait::async_trait]
//! impl BlockchainProvider<Ethereum> for FakeBlockchainProvider {
//!     async fn get_transaction_receipt(&self, tx_hash: TxHash)
//!         -> Result<Option<TransactionReceipt>> {
//!         Ok(self.receipts.get(&tx_hash).cloned())
//!     }
//!
//!     async fn get_block_number(&self) -> Result<u64> {
//!         Ok(12345)
//!     }
//! }
//! ```

use alloy_network::Network;
use alloy_primitives::{FixedBytes, TxHash};
use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::attestation::AttestationResponse;
use crate::error::Result;

/// Trait for blockchain RPC operations.
///
/// This trait abstracts all blockchain interactions required by the CCTP bridge,
/// allowing for testing with fake implementations that can simulate various
/// failure modes, network conditions, and edge cases.
///
/// The trait is generic over `N: Network` to support different blockchain networks
/// beyond Ethereum (e.g., Optimism via `op-alloy`).
///
/// # Test Scenarios
///
/// Implementing this trait with fakes enables testing:
/// - Transaction receipt not found
/// - Malformed transaction data
/// - Network timeouts
/// - Reorg scenarios
/// - Slow block confirmations
#[async_trait]
pub trait BlockchainProvider<N: Network>: Send + Sync {
    /// Fetches the transaction receipt for a given transaction hash.
    ///
    /// Returns `None` if the transaction is not found or not yet mined.
    ///
    /// The return type is the network's `ReceiptResponse` which contains the receipt data.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response cannot be parsed.
    async fn get_transaction_receipt(&self, tx_hash: TxHash) -> Result<Option<N::ReceiptResponse>>;

    /// Gets the current block number.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails.
    async fn get_block_number(&self) -> Result<u64>;
}

/// Trait for attestation retrieval from Circle's Iris API.
///
/// This trait abstracts the HTTP operations required to fetch CCTP attestations,
/// enabling testing with fake implementations that can simulate various API
/// behaviors and failure modes.
///
/// # Test Scenarios
///
/// Implementing this trait with fakes enables testing:
/// - Rate limiting (429 responses)
/// - API timeouts
/// - Malformed JSON responses
/// - State transitions (Pending → PendingConfirmations → Complete)
/// - Failed attestations
/// - Slow/flaky API responses
#[async_trait]
pub trait AttestationProvider: Send + Sync {
    /// Fetches attestation status and data for a message hash.
    ///
    /// This is typically called repeatedly (polling) until the attestation
    /// status becomes `Complete` or `Failed`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The response cannot be parsed
    /// - The API returns an error status code
    async fn get_attestation(&self, message_hash: FixedBytes<32>) -> Result<AttestationResponse>;
}

/// Trait for time-based operations.
///
/// This trait abstracts sleep and time queries, enabling fast-forward testing
/// where tests can instantly advance through polling loops and timeouts without
/// actually waiting.
///
/// # Test Scenarios
///
/// Implementing this trait with fakes enables testing:
/// - Timeout behavior without waiting
/// - Polling interval correctness
/// - Rate limit backoff periods
/// - Time-dependent state transitions
#[async_trait]
pub trait Clock: Send + Sync {
    /// Asynchronously sleeps for the given duration.
    async fn sleep(&self, duration: Duration);

    /// Returns the current instant in time.
    ///
    /// Used for calculating timeouts and measuring elapsed time.
    fn now(&self) -> Instant;
}
