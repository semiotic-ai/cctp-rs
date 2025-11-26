/// Circle Iris API environment URLs
///
/// See <https://developers.circle.com/stablecoins/cctp-apis>
///
pub const IRIS_API: &str = "https://iris-api.circle.com";
pub const IRIS_API_SANDBOX: &str = "https://iris-api-sandbox.circle.com";

/// CCTP v1 attestation API path
pub const ATTESTATION_PATH_V1: &str = "/v1/attestations/";

/// CCTP v2 messages API path
///
/// V2 uses a different endpoint format than v1:
/// - V1: `/v1/attestations/{messageHash}`
/// - V2: `/v2/messages/{sourceDomain}?transactionHash={txHash}`
pub const MESSAGES_PATH_V2: &str = "/v2/messages/";
