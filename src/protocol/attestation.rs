use alloy_primitives::{hex::FromHex, Bytes};
use serde::{Deserialize, Deserializer};

/// The bytes of the attestation.
pub type AttestationBytes = Vec<u8>;

// ============================================================================
// V2 Attestation Response Types
// ============================================================================

/// Represents the response from the CCTP v2 attestation API
///
/// The v2 API uses a different endpoint format (`/v2/messages/{domain}?transactionHash={tx}`)
/// and returns a wrapper containing an array of messages, since a single transaction
/// can emit multiple `MessageSent` events.
///
/// # Example Response
///
/// ```json
/// {
///   "messages": [
///     {
///       "status": "complete",
///       "message": "0x...",
///       "attestation": "0x..."
///     }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct V2AttestationResponse {
    /// Array of messages from the transaction
    pub messages: Vec<V2Message>,
}

/// Represents a single message in the v2 attestation response
///
/// Each message contains the attestation status, the original message bytes,
/// and the signed attestation (when complete).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V2Message {
    /// Status of the attestation
    pub status: AttestationStatus,

    /// The original message bytes from the MessageSent event
    #[serde(default, deserialize_with = "deserialize_optional_bytes_or_pending")]
    pub message: Option<Bytes>,

    /// The signed attestation bytes (null/PENDING until complete)
    #[serde(default, deserialize_with = "deserialize_optional_bytes_or_pending")]
    pub attestation: Option<Bytes>,
}

// ============================================================================
// V1 Attestation Response Types
// ============================================================================

/// Represents the response from the attestation service
///
/// It contains the status of the attestation and optionally the attestation data itself.
/// The attestation data is a hex-encoded string (with or without "0x" prefix) that is
/// automatically deserialized into bytes.
///
/// **API Quirk**: Circle's Iris API sometimes returns the string `"PENDING"` for the
/// attestation field instead of `null` when the attestation is not yet ready. This
/// deserializer handles that case gracefully by treating "PENDING" as `None`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
    pub status: AttestationStatus,
    #[serde(default, deserialize_with = "deserialize_optional_bytes_or_pending")]
    pub attestation: Option<Bytes>,
}

/// Custom deserializer that handles Circle API quirk where attestation field
/// may be the string "PENDING" instead of null
///
/// Handles the following cases:
/// - Valid hex string (with or without "0x") → deserializes to `Some(Bytes)`
/// - "PENDING" or "pending" → returns `None`
/// - null or missing field → returns `None`
/// - Empty string → returns `None`
/// - Invalid hex → returns error
fn deserialize_optional_bytes_or_pending<'de, D>(deserializer: D) -> Result<Option<Bytes>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;

    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) if s.eq_ignore_ascii_case("pending") => Ok(None),
        Some(s) => {
            let bytes = Bytes::from_hex(s).map_err(serde::de::Error::custom)?;
            Ok(Some(bytes))
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_attestation_with_valid_hex() {
        let json = r#"{"status":"complete","attestation":"0x1234abcd"}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Complete);
        assert!(response.attestation.is_some());
        assert_eq!(
            response.attestation.unwrap().to_vec(),
            vec![0x12, 0x34, 0xab, 0xcd]
        );
    }

    #[test]
    fn test_deserialize_attestation_with_pending_string() {
        let json = r#"{"status":"pending","attestation":"PENDING"}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Pending);
        assert!(response.attestation.is_none());
    }

    #[test]
    fn test_deserialize_attestation_with_pending_lowercase() {
        let json = r#"{"status":"pending","attestation":"pending"}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Pending);
        assert!(response.attestation.is_none());
    }

    #[test]
    fn test_deserialize_attestation_with_null() {
        let json = r#"{"status":"pending","attestation":null}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Pending);
        assert!(response.attestation.is_none());
    }

    #[test]
    fn test_deserialize_attestation_missing_field() {
        let json = r#"{"status":"pending"}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Pending);
        assert!(response.attestation.is_none());
    }

    #[test]
    fn test_deserialize_attestation_with_empty_string() {
        let json = r#"{"status":"pending","attestation":""}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Pending);
        assert!(response.attestation.is_none());
    }

    #[test]
    fn test_deserialize_attestation_with_hex_no_prefix() {
        let json = r#"{"status":"complete","attestation":"deadbeef"}"#;
        let response: AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.status, AttestationStatus::Complete);
        assert!(response.attestation.is_some());
        assert_eq!(
            response.attestation.unwrap().to_vec(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_deserialize_attestation_with_invalid_hex_fails() {
        let json = r#"{"status":"complete","attestation":"not_valid_hex"}"#;
        let result = serde_json::from_str::<AttestationResponse>(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_all_status_variants() {
        let complete = r#"{"status":"complete"}"#;
        let pending = r#"{"status":"pending"}"#;
        let pending_confirmations = r#"{"status":"pending_confirmations"}"#;
        let failed = r#"{"status":"failed"}"#;

        assert_eq!(
            serde_json::from_str::<AttestationResponse>(complete)
                .unwrap()
                .status,
            AttestationStatus::Complete
        );
        assert_eq!(
            serde_json::from_str::<AttestationResponse>(pending)
                .unwrap()
                .status,
            AttestationStatus::Pending
        );
        assert_eq!(
            serde_json::from_str::<AttestationResponse>(pending_confirmations)
                .unwrap()
                .status,
            AttestationStatus::PendingConfirmations
        );
        assert_eq!(
            serde_json::from_str::<AttestationResponse>(failed)
                .unwrap()
                .status,
            AttestationStatus::Failed
        );
    }

    // ========================================================================
    // V2 Response Tests
    // ========================================================================

    #[test]
    fn test_v2_deserialize_complete_response() {
        let json = r#"{
            "messages": [
                {
                    "status": "complete",
                    "message": "0xdeadbeef",
                    "attestation": "0x1234abcd"
                }
            ]
        }"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.messages[0].status, AttestationStatus::Complete);
        assert!(response.messages[0].message.is_some());
        assert!(response.messages[0].attestation.is_some());
        assert_eq!(
            response.messages[0].attestation.as_ref().unwrap().to_vec(),
            vec![0x12, 0x34, 0xab, 0xcd]
        );
        assert_eq!(
            response.messages[0].message.as_ref().unwrap().to_vec(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_v2_deserialize_pending_response() {
        let json = r#"{
            "messages": [
                {
                    "status": "pending",
                    "message": null,
                    "attestation": null
                }
            ]
        }"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.messages[0].status, AttestationStatus::Pending);
        assert!(response.messages[0].message.is_none());
        assert!(response.messages[0].attestation.is_none());
    }

    #[test]
    fn test_v2_deserialize_pending_with_string() {
        let json = r#"{
            "messages": [
                {
                    "status": "pending",
                    "message": "PENDING",
                    "attestation": "PENDING"
                }
            ]
        }"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.messages[0].status, AttestationStatus::Pending);
        assert!(response.messages[0].message.is_none());
        assert!(response.messages[0].attestation.is_none());
    }

    #[test]
    fn test_v2_deserialize_multiple_messages() {
        let json = r#"{
            "messages": [
                {
                    "status": "complete",
                    "message": "0xaa",
                    "attestation": "0xbb"
                },
                {
                    "status": "complete",
                    "message": "0xcc",
                    "attestation": "0xdd"
                }
            ]
        }"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 2);
        assert_eq!(response.messages[0].status, AttestationStatus::Complete);
        assert_eq!(response.messages[1].status, AttestationStatus::Complete);
    }

    #[test]
    fn test_v2_deserialize_empty_messages() {
        let json = r#"{"messages": []}"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 0);
    }

    #[test]
    fn test_v2_deserialize_pending_confirmations() {
        let json = r#"{
            "messages": [
                {
                    "status": "pending_confirmations",
                    "message": "0xdeadbeef",
                    "attestation": null
                }
            ]
        }"#;
        let response: V2AttestationResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages[0].status,
            AttestationStatus::PendingConfirmations
        );
        assert!(response.messages[0].message.is_some());
        assert!(response.messages[0].attestation.is_none());
    }
}
