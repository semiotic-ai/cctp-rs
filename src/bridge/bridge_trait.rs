// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use crate::error::Result;
use crate::protocol::FinalityThreshold;
use alloy_chains::NamedChain;
use alloy_primitives::{Address, FixedBytes, TxHash};
use async_trait::async_trait;

/// Common trait interface for CCTP bridge implementations (v1 and v2)
///
/// This trait provides shared functionality for bridging USDC across chains using
/// Circle's Cross-Chain Transfer Protocol. It covers chain configuration and
/// message extraction that are common to both protocol versions.
///
/// # Attestation Fetching
///
/// Attestation fetching is **not** part of this trait because V1 and V2 use
/// fundamentally different APIs:
///
/// - **V1**: `Cctp::get_attestation(message_hash)` - looks up by message hash
/// - **V2**: `CctpV2::get_attestation(tx_hash)` - looks up by transaction hash
///
/// This design ensures compile-time type safety: you can't accidentally pass
/// the wrong query type to the wrong bridge version.
///
/// # Example
///
/// ```rust,ignore
/// use cctp_rs::{Cctp, CctpV2Bridge};
///
/// // V1 bridge - get attestation by message hash
/// let (message, message_hash) = v1_bridge.get_message_sent_event(tx_hash).await?;
/// let attestation = v1_bridge.get_attestation(message_hash, None, None).await?;
///
/// // V2 bridge - get attestation by transaction hash
/// let attestation = v2_bridge.get_attestation(tx_hash, None, None).await?;
/// ```
#[async_trait]
pub trait CctpBridge: Send + Sync {
    /// Returns the source chain for the bridge
    ///
    /// This is the chain where the USDC burn transaction originates.
    fn source_chain(&self) -> NamedChain;

    /// Returns the destination chain for the bridge
    ///
    /// This is the chain where USDC will be minted after attestation.
    fn destination_chain(&self) -> NamedChain;

    /// Returns the recipient address on the destination chain
    ///
    /// This is the address that will receive the bridged USDC.
    fn recipient(&self) -> Address;

    /// Gets the `MessageSent` event data from a CCTP bridge transaction
    ///
    /// Extracts the message bytes and computes their hash from the transaction receipt.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the burn transaction on the source chain
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(message_bytes, message_hash)`:
    /// - `message_bytes`: The raw message data from the `MessageSent` event
    /// - `message_hash`: The keccak256 hash of the message bytes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction receipt cannot be retrieved
    /// - The transaction doesn't contain a `MessageSent` event
    /// - The event data cannot be decoded
    async fn get_message_sent_event(&self, tx_hash: TxHash) -> Result<(Vec<u8>, FixedBytes<32>)>;

    /// Returns whether this bridge supports fast transfers (v2 feature)
    ///
    /// Fast transfers enable <30 second settlement times with optional fees (0-14 bps).
    /// This uses a lower finality threshold (1000 = "confirmed" level) compared to
    /// standard transfers (2000 = "finalized" level).
    ///
    /// # Default Implementation
    ///
    /// Returns `false` for v1 implementations which only support standard settlement.
    fn supports_fast_transfer(&self) -> bool {
        false
    }

    /// Returns whether this bridge supports programmable hooks (v2 feature)
    ///
    /// Hooks enable automated post-transfer actions executed either pre-mint or post-mint.
    /// Common use cases include automatic swaps, lending, staking, or notifications.
    ///
    /// # Security Note
    ///
    /// Hook execution is separate from the core CCTP protocol - integrators control
    /// the execution logic and security model.
    ///
    /// # Default Implementation
    ///
    /// Returns `false` for v1 implementations which don't support hooks.
    fn supports_hooks(&self) -> bool {
        false
    }

    /// Returns the finality threshold for this bridge (v2 feature)
    ///
    /// The finality threshold determines how quickly attestations are issued:
    ///
    /// - **Fast** (1000): "confirmed" finality level, ~30 second settlement
    /// - **Standard** (2000): "finalized" finality level, ~15 minute settlement
    ///
    /// # Default Implementation
    ///
    /// Returns `None` for v1 implementations, which implicitly use standard finality.
    /// V2 implementations should return `Some(threshold)` based on their configuration.
    fn finality_threshold(&self) -> Option<FinalityThreshold> {
        None
    }
}
