//! Circle Iris API attestation provider implementation.

use alloy_primitives::{hex, FixedBytes};
use async_trait::async_trait;
use reqwest::Client;
use tracing::{debug, instrument, trace};

use crate::attestation::AttestationResponse;
use crate::error::{CctpError, Result};
use crate::traits::AttestationProvider;

/// Production attestation provider using Circle's Iris API.
///
/// This provider fetches CCTP attestations from Circle's official API endpoints,
/// handling both production and sandbox environments.
///
/// # Examples
///
/// ```rust,no_run
/// use cctp_rs::providers::IrisAttestationProvider;
/// use cctp_rs::traits::AttestationProvider;
/// use alloy_primitives::FixedBytes;
///
/// # async fn example() -> Result<(), cctp_rs::CctpError> {
/// let provider = IrisAttestationProvider::production();
/// let message_hash: FixedBytes<32> = [0u8; 32].into();
/// let response = provider.get_attestation(message_hash).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct IrisAttestationProvider {
    base_url: String,
    client: Client,
}

impl IrisAttestationProvider {
    /// Creates a new Iris attestation provider.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL for the Iris API (e.g., <https://iris-api.circle.com>)
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Creates a provider for Circle's production environment.
    pub fn production() -> Self {
        Self::new("https://iris-api.circle.com")
    }

    /// Creates a provider for Circle's sandbox (testnet) environment.
    pub fn sandbox() -> Self {
        Self::new("https://iris-api-sandbox.circle.com")
    }

    /// Constructs the full API URL for a given message hash.
    fn attestation_url(&self, message_hash: FixedBytes<32>) -> String {
        format!(
            "{}/v1/attestations/{}",
            self.base_url,
            hex::encode(message_hash)
        )
    }
}

#[async_trait]
impl AttestationProvider for IrisAttestationProvider {
    #[instrument(skip(self), fields(message_hash = %hex::encode(message_hash)))]
    async fn get_attestation(&self, message_hash: FixedBytes<32>) -> Result<AttestationResponse> {
        let url = self.attestation_url(message_hash);
        trace!(url = %url, "Requesting attestation from Iris API");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(CctpError::Network)?;

        let status_code = response.status();
        trace!(status_code = %status_code, "Received response from Iris API");

        // Handle rate limiting - extract Retry-After header if present
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(300);

            debug!(retry_after_seconds = retry_after, "Rate limit exceeded");
            return Err(CctpError::RateLimitExceeded {
                retry_after_seconds: retry_after,
            });
        }

        // Handle 404 - attestation not found yet (should be retried)
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!("Attestation not found");
            return Err(CctpError::AttestationNotFound);
        }

        // Check for HTTP errors
        response.error_for_status_ref()?;

        // Parse JSON response
        let json_value = response
            .json::<serde_json::Value>()
            .await
            .map_err(CctpError::Network)?;

        let attestation: AttestationResponse = serde_json::from_value(json_value)?;
        debug!(status = ?attestation.status, "Attestation response parsed");

        Ok(attestation)
    }
}
