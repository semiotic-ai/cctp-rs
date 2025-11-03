use serde::Deserialize;

/// The bytes of the attestation.
pub type AttestationBytes = Vec<u8>;

/// Represents the response from the attestation service
///
/// It contains the status of the attestation and optionally the attestation data itself.
/// The attestation data is a base64 encoded string that can be decoded into bytes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
    pub status: AttestationStatus,
    #[serde(default)]
    pub attestation: Option<String>,
}

/// Represents the status of the attestation.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttestationStatus {
    Complete,
    Pending,
    PendingConfirmations,
    Failed,
}
