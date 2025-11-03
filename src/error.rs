use thiserror::Error;

#[derive(Error, Debug)]
pub enum CctpError {
    #[error("Chain not supported: {chain}")]
    ChainNotSupported { chain: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Contract call failed: {0}")]
    ContractCall(String),

    #[error("Attestation failed: {reason}")]
    AttestationFailed { reason: String },

    #[error("Rate limit exceeded, retry after {retry_after_seconds} seconds")]
    RateLimitExceeded { retry_after_seconds: u64 },

    #[error("Attestation not found (will retry)")]
    AttestationNotFound,

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout waiting for attestation")]
    AttestationTimeout,

    #[error("RPC error: {0}")]
    Rpc(#[from] alloy_json_rpc::RpcError<alloy_transport::TransportErrorKind>),

    #[error("ABI encoding/decoding error: {0}")]
    Abi(#[from] alloy_sol_types::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Hex conversion error: {0}")]
    Hex(#[from] alloy_primitives::hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, CctpError>;
