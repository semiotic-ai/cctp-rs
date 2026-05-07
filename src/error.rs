// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use alloy_json_rpc::RpcError;
use alloy_primitives::TxHash;
use alloy_transport::TransportErrorKind;
use std::fmt;
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

/// Categorical reasons an attestation poll can fail.
///
/// Carried by [`CctpError::AttestationFailed`] so callers can react
/// to specific failure modes without substring-matching on a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttestationFailureKind {
    /// The Iris API returned `AttestationStatus::Failed`.
    ApiReportedFailed,
    /// Status was `Complete` but the `attestation` field was null.
    AttestationMissing,
    /// Status was `Complete` but the `message` field was null (v2 only).
    MessageMissing,
}

impl fmt::Display for AttestationFailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::ApiReportedFailed => "Iris API reported failed status",
            Self::AttestationMissing => "attestation field missing in complete response",
            Self::MessageMissing => "message field missing in complete response",
        };
        f.write_str(msg)
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CctpError {
    #[error("Unsupported chain: {0:?}")]
    UnsupportedChain(alloy_chains::NamedChain),

    #[error("Message already relayed (transfer successful via third party): {original}")]
    AlreadyRelayed { original: String },

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// A typed contract-call error from `alloy_contract`. Preserves alloy's
    /// structured introspection so callers can use [`alloy_contract::Error::as_revert_data`]
    /// and [`alloy_contract::Error::as_decoded_interface_error`] for revert decoding.
    #[error(transparent)]
    Contract(#[from] alloy_contract::Error),

    #[error("Attestation failed: {0}")]
    AttestationFailed(AttestationFailureKind),

    /// The source-chain RPC returned no receipt for the given transaction
    /// hash. The transaction may not have been mined yet, may have been
    /// dropped, or the queried node may not have indexed it.
    #[error("Transaction not found: {tx_hash}")]
    TransactionNotFound { tx_hash: TxHash },

    /// The transaction receipt was found but did not contain a `MessageSent`
    /// log. This indicates the transaction did not emit the CCTP bridge
    /// event — typically because the call hit the wrong contract or
    /// reverted before reaching the burn step.
    #[error("MessageSent event not found in transaction logs: {tx_hash}")]
    MessageSentEventMissing { tx_hash: TxHash },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout waiting for attestation")]
    AttestationTimeout,

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

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
    /// Detects the explicit [`AlreadyRelayed`](Self::AlreadyRelayed) variant
    /// directly; for [`Rpc`](Self::Rpc) and the transport-error subset of
    /// [`Contract`](Self::Contract), inspects the RPC error payload's message
    /// and data fields against known revert phrases for robustness across
    /// different RPC providers.
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

            // Only `TransportError` carries chain-level revert data — see
            // `alloy_contract::Error::as_revert_data`.
            CctpError::Contract(alloy_contract::Error::TransportError(rpc_error)) => {
                Self::rpc_error_is_already_relayed(rpc_error)
            }

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

    fn error_payload(message: &'static str) -> alloy_json_rpc::ErrorPayload {
        alloy_json_rpc::ErrorPayload {
            code: 3,
            message: message.into(),
            data: None,
        }
    }

    #[test]
    fn test_is_already_relayed_explicit_variant() {
        let err = CctpError::AlreadyRelayed {
            original: "test".to_string(),
        };
        assert!(err.is_already_relayed());
    }

    #[test]
    fn test_is_already_relayed_rpc_payload() {
        let check = |message: &'static str| {
            CctpError::Rpc(RpcError::ErrorResp(error_payload(message))).is_already_relayed()
        };

        assert!(check("nonce already used"));
        assert!(check("message already received"));
        assert!(check("Already Received"));
        assert!(check("NONCE ALREADY USED"));
        assert!(check("Nonce Already Used"));
        assert!(!check("some other error"));
    }

    #[test]
    fn test_is_timeout() {
        let err = CctpError::AttestationTimeout;
        assert!(err.is_timeout());

        let err = CctpError::InvalidConfig("some error".to_string());
        assert!(!err.is_timeout());
    }

    #[test]
    fn test_unrelated_errors_not_already_relayed() {
        assert!(!CctpError::AttestationTimeout.is_already_relayed());
        assert!(!CctpError::InvalidConfig("test".to_string()).is_already_relayed());
        assert!(!CctpError::NotImplemented("test".to_string()).is_already_relayed());
        assert!(!CctpError::TransactionNotFound {
            tx_hash: TxHash::ZERO,
        }
        .is_already_relayed());
        assert!(!CctpError::MessageSentEventMissing {
            tx_hash: TxHash::ZERO,
        }
        .is_already_relayed());
    }

    #[test]
    fn test_contract_variant_routes_through_is_already_relayed() {
        // `ContractNotDeployed` renders without any "already relayed" pattern.
        let err: CctpError = alloy_contract::Error::ContractNotDeployed.into();
        assert!(matches!(err, CctpError::Contract(_)));
        assert!(!err.is_already_relayed());
    }

    #[test]
    fn test_attestation_failure_kind_renders_prose() {
        let render = |kind: AttestationFailureKind| CctpError::AttestationFailed(kind).to_string();

        assert_eq!(
            render(AttestationFailureKind::ApiReportedFailed),
            "Attestation failed: Iris API reported failed status",
        );
        assert_eq!(
            render(AttestationFailureKind::AttestationMissing),
            "Attestation failed: attestation field missing in complete response",
        );
        assert_eq!(
            render(AttestationFailureKind::MessageMissing),
            "Attestation failed: message field missing in complete response",
        );
    }

    #[test]
    fn test_invalid_url_preserves_typed_parse_error() {
        let parse_err = url::Url::parse("not a url").unwrap_err();
        let err: CctpError = parse_err.into();
        assert!(matches!(err, CctpError::InvalidUrl(_)));
    }

    #[test]
    fn test_contract_transport_error_inspects_rpc_payload() {
        let contract_err_with_message = |message: &'static str| -> CctpError {
            alloy_contract::Error::TransportError(alloy_transport::TransportError::ErrorResp(
                error_payload(message),
            ))
            .into()
        };

        assert!(
            contract_err_with_message("execution reverted: nonce already used")
                .is_already_relayed()
        );
        assert!(
            !contract_err_with_message("execution reverted: insufficient allowance")
                .is_already_relayed()
        );
    }
}
