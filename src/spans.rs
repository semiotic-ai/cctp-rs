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
//!     60,  // poll intervali
//! );
//! let _guard = span.enter();
//! // Your custom attestation logic here
//! ```

use alloy_chains::NamedChain;
use alloy_primitives::{hex, Address, FixedBytes, TxHash, U256};
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
        error.type = tracing::field::Empty,
        error.message = tracing::field::Empty,
        error.source = tracing::field::Empty,
        otel.status_code = "OK",
    )
}

/// Create span for polling attestation API with retry logic (v1).
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
        error.type = tracing::field::Empty,
        error.message = tracing::field::Empty,
        error.source = tracing::field::Empty,
        otel.status_code = "OK",
    )
}

/// Create span for polling attestation API with retry logic (v2).
///
/// V2 uses transaction hash instead of message hash for attestation lookup.
///
/// Parent: Top-level bridge operation span
/// Children: cctp_rs.get_attestation (multiple attempts)
#[inline]
pub fn get_v2_attestation_with_retry(
    tx_hash: TxHash,
    source_chain: &NamedChain,
    destination_chain: &NamedChain,
    max_attempts: u32,
    poll_interval_secs: u64,
) -> Span {
    tracing::info_span!(
        "cctp_rs.get_attestation_with_retry",
        tx_hash = %tx_hash,
        source_chain = %source_chain,
        destination_chain = %destination_chain,
        max_attempts = max_attempts,
        poll_interval_secs = poll_interval_secs,
        version = "v2",
        error.type = tracing::field::Empty,
        error.message = tracing::field::Empty,
        error.source = tracing::field::Empty,
        otel.status_code = "OK",
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

/// Create span for USDC deposit and burn transaction creation.
///
/// Parent: Top-level bridge operation span
/// Children: Contract call preparation spans
#[inline]
pub fn deposit_for_burn(
    from_address: &Address,
    recipient: &Address,
    destination_domain: u32,
    token_address: &Address,
    amount: &U256,
) -> Span {
    tracing::info_span!(
        "cctp_rs.deposit_for_burn",
        from_address = %from_address,
        recipient = %recipient,
        destination_domain = destination_domain,
        token_address = %token_address,
        amount = %amount,
        error.type = tracing::field::Empty,
        error.message = tracing::field::Empty,
        error.context = tracing::field::Empty,
        otel.status_code = "OK",
    )
}

/// Create span for transaction submission to blockchain.
///
/// Parent: Operation span (e.g., deposit_for_burn)
/// Children: Provider RPC calls
#[inline]
pub fn send_transaction(tx_hash: &str, source_chain: &NamedChain) -> Span {
    tracing::debug_span!(
        "cctp_rs.send_transaction",
        tx_hash = tx_hash,
        source_chain = %source_chain,
    )
}

/// Create span for waiting for transaction confirmation.
///
/// Parent: send_transaction or top-level operation
/// Children: Provider RPC calls (polling)
#[inline]
pub fn wait_for_confirmation(
    tx_hash: TxHash,
    chain: &NamedChain,
    required_confirmations: u64,
) -> Span {
    tracing::debug_span!(
        "cctp_rs.wait_for_confirmation",
        tx_hash = %tx_hash,
        chain = %chain,
        required_confirmations = required_confirmations,
    )
}

/// Create span for receiving message on destination chain.
///
/// Parent: Top-level bridge operation span
/// Children: Contract interaction spans, RPC calls
#[inline]
pub fn receive_message(
    message_hash: &FixedBytes<32>,
    destination_chain: &NamedChain,
    attestation_length: usize,
) -> Span {
    tracing::info_span!(
        "cctp_rs.receive_message",
        message_hash = %hex::encode(message_hash),
        destination_chain = %destination_chain,
        attestation_length_bytes = attestation_length,
    )
}

/// Create span for HTTP request to Circle API.
///
/// Parent: get_attestation or other API operation
/// Children: None (HTTP client handles internal spans)
#[inline]
pub fn http_request(method: &str, url: &Url, request_id: Option<&str>) -> Span {
    tracing::trace_span!(
        "cctp_rs.http_request",
        http.method = method,
        http.url = %url,
        http.request_id = request_id,
    )
}

/// Create span for RPC call to blockchain provider.
///
/// Parent: Operation span (get_message_sent_event, wait_for_confirmation, etc.)
/// Children: None (provider handles internal spans)
#[inline]
pub fn rpc_call(method: &str, chain: &NamedChain, params_summary: &str) -> Span {
    tracing::trace_span!(
        "cctp_rs.rpc_call",
        rpc.method = method,
        rpc.chain = %chain,
        rpc.params = params_summary,
    )
}

/// Create span for transaction receipt retrieval.
///
/// Parent: get_message_sent_event or other receipt operations
/// Children: RPC calls
#[inline]
pub fn get_transaction_receipt(tx_hash: TxHash, chain: &NamedChain) -> Span {
    tracing::debug_span!(
        "cctp_rs.get_transaction_receipt",
        tx_hash = %tx_hash,
        chain = %chain,
    )
}

/// Record error attributes on the current span.
///
/// Follows OpenTelemetry semantic conventions for error tracking:
/// - error.type: The error type/variant
/// - error.message: Human-readable error message
/// - error.stack: Optional stack trace or additional context
///
/// # Example
///
/// ```rust,no_run
/// use cctp_rs::spans;
/// use cctp_rs::CctpError;
///
/// # fn example() -> Result<(), CctpError> {
/// let span = tracing::info_span!("cctp_rs.operation");
/// let _guard = span.enter();
///
/// let result = some_operation();
/// if let Err(ref e) = result {
///     spans::record_error(e);
/// }
/// result
/// # }
/// # fn some_operation() -> Result<(), CctpError> { Ok(()) }
/// ```
pub fn record_error<E: std::error::Error>(error: &E) {
    let current_span = tracing::Span::current();
    current_span.record(
        "error.type",
        error.to_string().split(':').next().unwrap_or("Unknown"),
    );
    current_span.record("error.message", error.to_string());
    current_span.record("otel.status_code", "ERROR");

    // Record error chain if available
    if let Some(source) = error.source() {
        current_span.record("error.source", source.to_string());
    }
}

/// Record error attributes with custom context on the current span.
///
/// This variant allows adding additional context fields to the error.
///
/// # Example
///
/// ```rust,no_run
/// use cctp_rs::spans;
///
/// # fn example() {
/// let span = tracing::info_span!("cctp_rs.operation");
/// let _guard = span.enter();
///
/// if let Err(e) = some_operation() {
///     spans::record_error_with_context(
///         "TransactionFailed",
///         &format!("Failed to submit transaction: {}", e),
///         Some("Transaction may have been dropped from mempool"),
///     );
/// }
/// # }
/// # fn some_operation() -> Result<(), String> { Ok(()) }
/// ```
pub fn record_error_with_context(
    error_type: &str,
    error_message: &str,
    additional_context: Option<&str>,
) {
    let current_span = tracing::Span::current();
    current_span.record("error.type", error_type);
    current_span.record("error.message", error_message);
    current_span.record("otel.status_code", "ERROR");

    if let Some(context) = additional_context {
        current_span.record("error.context", context);
    }
}
