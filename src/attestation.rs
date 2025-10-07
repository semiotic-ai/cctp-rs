use alloy_primitives::hex;
use serde::Deserialize;
use std::fmt::Display;

/// To be passed to message transmitter to claim/mint
pub struct Attestation {
    pub attestation: Vec<u8>,
    pub message: Vec<u8>,
}

impl Display for Attestation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "attestation: {}, message: {}",
            hex::encode(&self.attestation),
            hex::encode(&self.message)
        )
    }
}

/// The bytes of the attestation.
pub type AttestationBytes = Vec<u8>;

/// Represents the response from the attestation service
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
    pub messages: Vec<AttestationMessage>,
}

/// Represents the one message from the attestation service
/// It contains the status of the attestation and optionally the attestation data itself.
/// The attestation data is a base64 encoded string that can be decoded into bytes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationMessage {
    pub status: AttestationStatus,
    #[serde(default)]
    pub attestation: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// Represents the status of the attestation.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttestationStatus {
    Complete,
    Pending,
    PendingConfirmations,
    Failed,
}
