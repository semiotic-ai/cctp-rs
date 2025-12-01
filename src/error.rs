// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

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

pub type Result<T> = std::result::Result<T, CctpError>;
