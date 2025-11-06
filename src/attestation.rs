use alloy_primitives::Bytes;
use serde::Deserialize;

/// The bytes of the attestation.
pub type AttestationBytes = Vec<u8>;

/// Represents the response from the attestation service
///
/// It contains the status of the attestation and optionally the attestation data itself.
/// The attestation data is a hex-encoded string (with or without "0x" prefix) that is
/// automatically deserialized into bytes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
    pub status: AttestationStatus,
    #[serde(default)]
    pub attestation: Option<Bytes>,
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
