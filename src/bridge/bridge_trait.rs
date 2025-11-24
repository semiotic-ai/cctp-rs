use crate::error::Result;
use crate::protocol::{AttestationBytes, FinalityThreshold};
use alloy_chains::NamedChain;
use alloy_primitives::{Address, FixedBytes, TxHash};
use async_trait::async_trait;

/// Common trait interface for CCTP bridge implementations (v1 and v2)
///
/// This trait provides a unified API for bridging USDC across chains using
/// Circle's Cross-Chain Transfer Protocol. It abstracts over protocol versions,
/// allowing both v1 and v2 implementations to be used interchangeably.
///
/// # Protocol Version Support
///
/// The trait is designed to work seamlessly with both CCTP v1 and v2:
///
/// - **Core methods**: Required by all implementations (v1 and v2)
/// - **V2-specific methods**: Have default implementations that return conservative values for v1
///
/// # Dynamic Dispatch
///
/// This trait is object-safe, enabling use as trait objects:
///
/// ```rust,no_run
/// # use cctp_rs::{Cctp, CctpBridge};
/// # use alloy_chains::NamedChain;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # use alloy_provider::ProviderBuilder;
/// # let provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
/// let bridge = Cctp::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Arbitrum)
///     .source_provider(provider.clone())
///     .destination_provider(provider)
///     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
///     .build();
///
/// // Use as trait object for dynamic dispatch
/// let bridge_trait: &dyn CctpBridge = &bridge;
/// let source = bridge_trait.source_chain();
/// # Ok(())
/// # }
/// ```
///
/// # Implementation Guide
///
/// When implementing this trait:
///
/// 1. **V1 implementations**: Only implement core methods, use default impls for v2 features
/// 2. **V2 implementations**: Override v2-specific methods as needed
/// 3. **Error handling**: Use the provided `Result` type for consistent error propagation
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
    /// The message hash is used to poll for attestations from Circle's Iris API.
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

    /// Gets the attestation for a message hash from Circle's Iris API
    ///
    /// This method polls the Iris API until the attestation is ready or times out.
    /// The attestation is required to complete the bridge transfer on the destination chain.
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The keccak256 hash of the message from `get_message_sent_event`
    /// * `max_attempts` - Maximum number of polling attempts (default: 30)
    /// * `poll_interval` - Time to wait between polling attempts in seconds (default: 60)
    ///
    /// # Returns
    ///
    /// The attestation bytes that must be submitted to the destination chain's
    /// `MessageTransmitter` contract to complete the transfer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The attestation request fails
    /// - Circle's API returns a failed status
    /// - The maximum number of attempts is reached (timeout)
    ///
    /// # Example Timing
    ///
    /// With default parameters (30 attempts × 60 seconds):
    /// - Total timeout: 30 minutes
    /// - Typical v1 attestation time: 13-19 minutes
    /// - Typical v2 fast transfer time: <30 seconds
    async fn get_attestation_with_retry(
        &self,
        message_hash: FixedBytes<32>,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<AttestationBytes>;

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
