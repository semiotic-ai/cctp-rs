//! OpenTelemetry span helpers for CCTP operations
//!
//! This module provides orthogonal span instrumentation following production
//! best practices: static span names, structured attributes, and separation
//! from business logic.
//!
//! # Usage
//!
//! These span helpers are used internally by the [`Cctp`](crate::Cctp) implementation
//! but are exposed publicly for advanced users who need custom instrumentation or
//! want to integrate with existing OpenTelemetry setups.
//!
//! # Example
//!
//! ```rust,no_run
//! use cctp_rs::spans;
//! use alloy_primitives::FixedBytes;
//! use alloy_chains::NamedChain;
//!
//! // Create a span for attestation polling
//! let message_hash = FixedBytes::from([0u8; 32]);
//! let span = spans::get_attestation_with_retry(
//!     &message_hash,
//!     &NamedChain::Mainnet,
//!     &NamedChain::Arbitrum,
//!     30,  // max attempts
//!     60,  // poll interval
//! );
//! let _guard = span.enter();
//! // Your custom attestation logic here
//! ```

use alloy_chains::NamedChain;
use alloy_primitives::{hex, FixedBytes, TxHash};
use tracing::Span;
use url::Url;

/// Create span for extracting MessageSent event from transaction receipt.
///
/// Parent: Top-level operation span (auto-attached by tracing)
/// Children: Provider RPC calls (from alloy instrumentation)
#[inline]
pub fn get_message_sent_event(
    tx_hash: TxHash,
    source_chain: &NamedChain,
    destination_chain: &NamedChain,
) -> Span {
    tracing::info_span!(
        "cctp_rs.get_message_sent_event",
        tx_hash = %tx_hash,
        source_chain = %source_chain,
        destination_chain = %destination_chain,
    )
}

/// Create span for polling attestation API with retry logic.
///
/// Parent: Top-level bridge operation span
/// Children: cctp_rs.get_attestation (multiple attempts)
#[inline]
pub fn get_attestation_with_retry(
    message_hash: &FixedBytes<32>,
    source_chain: &NamedChain,
    destination_chain: &NamedChain,
    max_attempts: u32,
    poll_interval_secs: u64,
) -> Span {
    tracing::info_span!(
        "cctp_rs.get_attestation_with_retry",
        message_hash = %hex::encode(message_hash),
        source_chain = %source_chain,
        destination_chain = %destination_chain,
        max_attempts = max_attempts,
        poll_interval_secs = poll_interval_secs,
    )
}

/// Create span for single attestation API request.
///
/// Parent: cctp_rs.get_attestation_with_retry
/// Children: HTTP client request spans (from reqwest instrumentation)
#[inline]
pub fn get_attestation(url: &Url, attempt: u32) -> Span {
    tracing::debug_span!(
        "cctp_rs.get_attestation",
        url = %url,
        attempt = attempt,
    )
}

/// Create span for attestation response processing.
///
/// Parent: cctp_rs.get_attestation
/// Children: None
#[inline]
pub fn process_attestation_response(status_code: u16, attempt: u32) -> Span {
    tracing::debug_span!(
        "cctp_rs.process_attestation_response",
        status_code = status_code,
        attempt = attempt,
    )
}
