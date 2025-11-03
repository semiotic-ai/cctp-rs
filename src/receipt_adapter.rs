//! Universal receipt handling adapter for all Alloy network types.
//!
//! This module provides a trait and universal implementation for accessing logs from
//! transaction receipts across all blockchain networks that follow Alloy's receipt conventions.

use alloy_network::Network;
use alloy_rpc_types::{Log, TransactionReceipt};

/// Trait for network-agnostic receipt log access.
///
/// This trait abstracts over different network types to provide a uniform interface
/// for extracting transaction logs from receipts. It works with any Alloy `Network`
/// type whose `ReceiptResponse` is a `TransactionReceipt`.
pub trait ReceiptAdapter<N: Network> {
    /// Extract logs from a receipt response
    fn logs<'a>(&self, receipt: &'a N::ReceiptResponse) -> &'a [Log];
}

/// Universal receipt adapter that works with all Alloy network types.
///
/// This adapter provides a single implementation that works across all network types
/// (Ethereum, Optimism, Arbitrum, etc.) that use Alloy's standard `TransactionReceipt`
/// structure with an inner `ReceiptEnvelope`.
///
/// # Type Parameters
///
/// The adapter is generic over the network type `N`, but the implementation is constrained
/// to only work when `N::ReceiptResponse` is `TransactionReceipt`. This ensures type safety
/// while allowing a single universal implementation.
///
/// # Examples
///
/// ```rust
/// use cctp_rs::UniversalReceiptAdapter;
/// use alloy_network::Ethereum;
///
/// let adapter = UniversalReceiptAdapter;
/// // Can be used with any Network type: Ethereum, Optimism, etc.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct UniversalReceiptAdapter;

impl<N> ReceiptAdapter<N> for UniversalReceiptAdapter
where
    N: Network<ReceiptResponse = TransactionReceipt>,
{
    fn logs<'a>(&self, receipt: &'a N::ReceiptResponse) -> &'a [Log] {
        // All Alloy network types use TransactionReceipt with an inner ReceiptEnvelope
        // This works for Ethereum, Optimism (OP Stack), and other EVM-compatible chains
        match &receipt.inner {
            alloy_rpc_types::ReceiptEnvelope::Eip1559(r) => &r.receipt.logs,
            alloy_rpc_types::ReceiptEnvelope::Eip2930(r) => &r.receipt.logs,
            alloy_rpc_types::ReceiptEnvelope::Legacy(r) => &r.receipt.logs,
            alloy_rpc_types::ReceiptEnvelope::Eip4844(r) => &r.receipt.logs,
            alloy_rpc_types::ReceiptEnvelope::Eip7702(r) => &r.receipt.logs,
        }
    }
}
