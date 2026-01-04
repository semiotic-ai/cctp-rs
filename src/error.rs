// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use alloy_json_rpc::RpcError;
use alloy_transport::TransportErrorKind;
use thiserror::Error;

/// Known revert reason patterns that indicate a message was already processed.
/// These are matched case-insensitively against error messages.
const ALREADY_RELAYED_PATTERNS: &[&str] = &[
    "nonce already used",
    "already received",
    "already processed",
    "message already received",
    "nonce used",
];

#[derive(Error, Debug)]
pub enum CctpError {
    #[error("Unsupported chain: {0:?}")]
    UnsupportedChain(alloy_chains::NamedChain),

    #[error("Message already relayed (transfer successful via third party): {original}")]
    AlreadyRelayed { original: String },

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Contract call failed: {0}")]
    ContractCall(String),

    #[error("Attestation failed: {reason}")]
    AttestationFailed { reason: String },

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout waiting for attestation")]
    AttestationTimeout,

    #[error("Invalid URL: {reason}")]
    InvalidUrl { reason: String },

    #[error("RPC error: {0}")]
    Rpc(#[from] alloy_json_rpc::RpcError<alloy_transport::TransportErrorKind>),

    #[error("ABI encoding/decoding error: {0}")]
    Abi(#[from] alloy_sol_types::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hex conversion error: {0}")]
    Hex(#[from] alloy_primitives::hex::FromHexError),
}

impl CctpError {
    /// Checks if this error indicates that a CCTP message was already relayed.
    ///
    /// This is common in CCTP v2 where third-party relayers may complete transfers
    /// before your application. When this returns `true`, the transfer was successful
    /// (just not by us).
    ///
    /// This method uses typed error inspection where possible and falls back to
    /// pattern matching on error messages for robustness across different RPC providers.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match bridge.mint(message, attestation, from).await {
    ///     Ok(tx_hash) => println!("Minted: {tx_hash}"),
    ///     Err(e) if e.is_already_relayed() => println!("Already relayed by third party"),
    ///     Err(e) => return Err(e),
    /// }
    /// ```
    pub fn is_already_relayed(&self) -> bool {
        match self {
            // Explicit AlreadyRelayed variant
            CctpError::AlreadyRelayed { .. } => true,

            // Check RPC errors for execution revert with known patterns
            CctpError::Rpc(rpc_error) => Self::rpc_error_is_already_relayed(rpc_error),

            // Check string-based errors (Provider, ContractCall, TransactionFailed)
            CctpError::Provider(msg)
            | CctpError::ContractCall(msg)
            | CctpError::TransactionFailed { reason: msg } => {
                Self::message_matches_already_relayed(msg)
            }

            // Other error types cannot indicate already relayed
            _ => false,
        }
    }

    /// Checks if an RPC error indicates the message was already relayed.
    fn rpc_error_is_already_relayed(error: &RpcError<TransportErrorKind>) -> bool {
        match error {
            // Check error response payload for revert reasons
            RpcError::ErrorResp(payload) => {
                Self::message_matches_already_relayed(&payload.message)
                    || payload
                        .data
                        .as_ref()
                        .is_some_and(|d| Self::message_matches_already_relayed(&d.to_string()))
            }
            // Local errors may contain the revert reason in their display
            RpcError::LocalUsageError(e) => Self::message_matches_already_relayed(&e.to_string()),
            // Transport or serialization errors don't indicate already relayed
            _ => false,
        }
    }

    /// Checks if a message string matches known "already relayed" patterns.
    fn message_matches_already_relayed(message: &str) -> bool {
        let lower = message.to_lowercase();
        ALREADY_RELAYED_PATTERNS
            .iter()
            .any(|pattern| lower.contains(pattern))
    }

    /// Checks if this is a timeout error.
    ///
    /// Useful for implementing retry logic for transient failures.
    pub fn is_timeout(&self) -> bool {
        if matches!(self, CctpError::AttestationTimeout) {
            return true;
        }

        // Check if network error indicates timeout
        if let CctpError::Network(e) = self {
            return e.is_timeout();
        }

        false
    }

    /// Checks if this is a rate limiting error.
    ///
    /// Useful for implementing backoff logic when hitting provider rate limits.
    pub fn is_rate_limited(&self) -> bool {
        match self {
            CctpError::Network(e) => e.status().is_some_and(|s| s.as_u16() == 429),
            CctpError::Rpc(RpcError::Transport(TransportErrorKind::HttpError(err))) => {
                err.status == 429
            }
            _ => false,
        }
    }

    /// Checks if this error is transient and the operation could be retried.
    ///
    /// Transient errors include timeouts, rate limiting, and temporary network issues.
    pub fn is_transient(&self) -> bool {
        self.is_timeout() || self.is_rate_limited() || self.is_network_error()
    }

    /// Checks if this is a network-level error.
    fn is_network_error(&self) -> bool {
        matches!(self, CctpError::Network(_))
            || matches!(
                self,
                CctpError::Rpc(RpcError::Transport(
                    TransportErrorKind::BackendGone | TransportErrorKind::HttpError(_)
                ))
            )
    }
}

pub type Result<T> = std::result::Result<T, CctpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_already_relayed_explicit_variant() {
        let err = CctpError::AlreadyRelayed {
            original: "test".to_string(),
        };
        assert!(err.is_already_relayed());
    }

    #[test]
    fn test_is_already_relayed_provider_error() {
        let err = CctpError::Provider("nonce already used".to_string());
        assert!(err.is_already_relayed());

        let err = CctpError::Provider("message already received".to_string());
        assert!(err.is_already_relayed());

        let err = CctpError::Provider("some other error".to_string());
        assert!(!err.is_already_relayed());
    }

    #[test]
    fn test_is_already_relayed_contract_call_error() {
        let err = CctpError::ContractCall("execution reverted: nonce used".to_string());
        assert!(err.is_already_relayed());

        let err = CctpError::ContractCall("already processed".to_string());
        assert!(err.is_already_relayed());

        let err = CctpError::ContractCall("insufficient funds".to_string());
        assert!(!err.is_already_relayed());
    }

    #[test]
    fn test_is_already_relayed_transaction_failed() {
        let err = CctpError::TransactionFailed {
            reason: "Already Received".to_string(),
        };
        assert!(err.is_already_relayed());
    }

    #[test]
    fn test_is_already_relayed_case_insensitive() {
        let err = CctpError::Provider("NONCE ALREADY USED".to_string());
        assert!(err.is_already_relayed());

        let err = CctpError::Provider("Nonce Already Used".to_string());
        assert!(err.is_already_relayed());
    }

    #[test]
    fn test_is_timeout() {
        let err = CctpError::AttestationTimeout;
        assert!(err.is_timeout());

        let err = CctpError::Provider("some error".to_string());
        assert!(!err.is_timeout());
    }

    #[test]
    fn test_unrelated_errors_not_already_relayed() {
        assert!(!CctpError::AttestationTimeout.is_already_relayed());
        assert!(!CctpError::InvalidConfig("test".to_string()).is_already_relayed());
        assert!(!CctpError::NotImplemented("test".to_string()).is_already_relayed());
    }
}
