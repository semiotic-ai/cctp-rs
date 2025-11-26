use crate::error::{CctpError, Result};
use crate::{spans, DomainId};
use crate::{AttestationBytes, AttestationResponse, AttestationStatus, CctpV1};
use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Address, FixedBytes, TxHash};
use alloy_provider::Provider;
use alloy_sol_types::SolEvent;
use async_trait::async_trait;
use bon::Builder;
use reqwest::{Client, Response};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info};
use url::Url;

use super::bridge_trait::CctpBridge;
use super::config::{PollingConfig, ATTESTATION_PATH_V1, IRIS_API, IRIS_API_SANDBOX};
use crate::contracts::message_transmitter::MessageTransmitter::MessageSent;
use crate::protocol::FinalityThreshold;

/// CCTP v1 bridge implementation
///
/// This struct provides the core functionality for bridging USDC across chains
/// using Circle's Cross-Chain Transfer Protocol (CCTP).
///
/// # Example
///
/// ```rust,no_run
/// # use cctp_rs::{Cctp, CctpError};
/// # use alloy_chains::NamedChain;
/// # use alloy_provider::ProviderBuilder;
/// # async fn example() -> Result<(), CctpError> {
/// let bridge = Cctp::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Arbitrum)
///     .source_provider(ProviderBuilder::new().on_builtin("http://localhost:8545").await?)
///     .destination_provider(ProviderBuilder::new().on_builtin("http://localhost:8546").await?)
///     .recipient("0x...".parse()?)
///     .build();
/// # Ok(())
/// # }
/// ```
#[derive(Builder, Clone, Debug)]
pub struct Cctp<P: Provider<Ethereum> + Clone> {
    source_provider: P,
    destination_provider: P,
    source_chain: NamedChain,
    destination_chain: NamedChain,
    recipient: Address,
}

impl<P: Provider<Ethereum> + Clone> Cctp<P> {
    /// Returns the CCTP API URL for the current environment
    pub fn api_url(&self) -> Url {
        if self.source_chain.is_testnet() {
            Url::parse(IRIS_API_SANDBOX).unwrap()
        } else {
            Url::parse(IRIS_API).unwrap()
        }
    }

    /// Returns the source chain
    pub fn source_chain(&self) -> &NamedChain {
        &self.source_chain
    }

    /// Returns the destination chain
    pub fn destination_chain(&self) -> &NamedChain {
        &self.destination_chain
    }

    /// Returns the destination domain id
    pub fn destination_domain_id(&self) -> Result<DomainId> {
        self.destination_chain.cctp_domain_id()
    }

    /// Returns the source provider
    pub fn source_provider(&self) -> &P {
        &self.source_provider
    }

    /// Returns the destination provider
    pub fn destination_provider(&self) -> &P {
        &self.destination_provider
    }

    /// Returns the CCTP token messenger contract, the address of the contract that handles the deposit and burn of USDC
    pub fn token_messenger_contract(&self) -> Result<Address> {
        self.source_chain.token_messenger_address()
    }

    /// Returns the CCTP message transmitter contract, the address of the contract that handles the receipt of messages
    pub fn message_transmitter_contract(&self) -> Result<Address> {
        self.destination_chain.message_transmitter_address()
    }

    /// Returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// Gets the `MessageSent` event data from a CCTP bridge transaction
    ///
    /// # Arguments
    ///
    /// * `tx_hash`: The hash of the transaction to get the `MessageSent` event for
    ///
    /// # Returns
    ///
    /// Returns the message bytes and its hash
    pub async fn get_message_sent_event(
        &self,
        tx_hash: TxHash,
    ) -> Result<(Vec<u8>, FixedBytes<32>)> {
        let span =
            spans::get_message_sent_event(tx_hash, &self.source_chain, &self.destination_chain);
        let _guard = span.enter();

        let tx_receipt = match self.source_provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => receipt,
            Err(e) => {
                let error_msg = format!("Failed to get transaction receipt: {}", e);
                spans::record_error_with_context(
                    "ReceiptRetrievalFailed",
                    &error_msg,
                    Some("RPC call to get_transaction_receipt failed"),
                );
                error!(
                    error = %e,
                    event = "transaction_receipt_retrieval_failed"
                );
                return Err(CctpError::TransactionFailed { reason: error_msg });
            }
        };

        if let Some(tx_receipt) = tx_receipt {
            // Calculate the event topic by hashing the event signature
            let message_sent_topic = alloy_primitives::keccak256(b"MessageSent(bytes)");

            let message_sent_log = tx_receipt
                .inner
                .logs()
                .iter()
                .find(|log| {
                    log.topics()
                        .first()
                        .is_some_and(|topic| topic.as_slice() == message_sent_topic)
                })
                .ok_or_else(|| {
                    spans::record_error_with_context(
                        "MessageSentEventNotFound",
                        "MessageSent event not found in transaction logs",
                        Some(&format!(
                            "Transaction contained {} logs but none matched MessageSent signature",
                            tx_receipt.inner.logs().len()
                        )),
                    );
                    error!(
                        available_logs = tx_receipt.inner.logs().len(),
                        event = "message_sent_event_not_found"
                    );
                    CctpError::TransactionFailed {
                        reason: "MessageSent event not found".to_string(),
                    }
                })?;

            // Decode the log data using the generated event bindings
            let decoded = MessageSent::abi_decode_data(&message_sent_log.data().data)?;

            let message_sent_event = decoded.0.to_vec();
            let message_hash = alloy_primitives::keccak256(&message_sent_event);

            info!(
                message_hash = %hex::encode(message_hash),
                message_length_bytes = message_sent_event.len(),
                event = "message_sent_event_extracted"
            );

            Ok((message_sent_event, message_hash))
        } else {
            spans::record_error_with_context(
                "TransactionNotFound",
                "Transaction receipt not found",
                Some("The transaction may not have been mined yet or the RPC node doesn't have it"),
            );
            error!(event = "transaction_not_found");
            Err(CctpError::TransactionFailed {
                reason: "Transaction not found".to_string(),
            })
        }
    }

    /// Gets the attestation for a message hash from Circle's Iris API
    ///
    /// This method polls the Iris API until the attestation is ready or times out.
    /// The message hash is typically obtained from `get_message_sent_event()`.
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The keccak256 hash of the MessageSent event bytes
    /// * `polling_config` - Configuration for polling behavior (attempts, intervals)
    ///
    /// # Returns
    ///
    /// The attestation bytes to submit to the destination chain's MessageTransmitter contract.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The attestation request fails
    /// - Circle's API returns a failed status
    /// - The maximum number of attempts is reached (timeout)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cctp_rs::PollingConfig;
    ///
    /// // Get the message from the burn transaction
    /// let (message, message_hash) = bridge.get_message_sent_event(burn_tx_hash).await?;
    ///
    /// // Poll for attestation with default settings (30 attempts, 60 seconds apart)
    /// let attestation = bridge.get_attestation(message_hash, PollingConfig::default()).await?;
    ///
    /// // Or with custom retry settings
    /// let attestation = bridge.get_attestation(
    ///     message_hash,
    ///     PollingConfig::default()
    ///         .with_max_attempts(20)
    ///         .with_poll_interval_secs(30),
    /// ).await?;
    /// ```
    pub async fn get_attestation(
        &self,
        message_hash: FixedBytes<32>,
        polling_config: PollingConfig,
    ) -> Result<AttestationBytes> {
        let max_attempts = polling_config.max_attempts;
        let poll_interval = polling_config.poll_interval_secs;

        let span = spans::get_attestation_with_retry(
            &message_hash,
            &self.source_chain,
            &self.destination_chain,
            max_attempts,
            poll_interval,
        );
        let _guard = span.enter();

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(CctpError::Network)?;
        let url = self.create_url(message_hash)?;

        info!(
            url = %url,
            event = "attestation_polling_started"
        );

        for attempt in 1..=max_attempts {
            let attempt_span = spans::get_attestation(&url, attempt);
            let _attempt_guard = attempt_span.enter();

            let response = match self.fetch_attestation_response(&client, &url).await {
                Ok(r) => r,
                Err(e) => {
                    spans::record_error_with_context(
                        "HttpRequestFailed",
                        &format!("Failed to fetch attestation: {}", e),
                        Some(&format!("Attempt {}/{}", attempt, max_attempts)),
                    );
                    error!(
                        error = %e,
                        attempt = attempt,
                        event = "attestation_http_request_failed"
                    );
                    return Err(e);
                }
            };

            let status_code = response.status().as_u16();
            let process_span = spans::process_attestation_response(status_code, attempt);
            let _process_guard = process_span.enter();

            // Handle rate limiting
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let secs = 5 * 60;
                debug!(sleep_secs = secs, event = "rate_limit_exceeded");
                sleep(Duration::from_secs(secs)).await;
                continue;
            }

            // Handle 404 status - treat as pending since the attestation likely doesn't exist yet
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                debug!(event = "attestation_not_found");
                sleep(Duration::from_secs(poll_interval)).await;
                continue;
            }

            // Ensure the response status is successful before trying to parse JSON
            response.error_for_status_ref()?;

            // Get response body as text first for better error logging
            let response_text = response.text().await?;

            let attestation: AttestationResponse = match serde_json::from_str(&response_text) {
                Ok(attestation) => attestation,
                Err(e) => {
                    error!(
                        error = %e,
                        response_body = %response_text,
                        message_hash = %hex::encode(message_hash),
                        attempt = attempt,
                        event = "attestation_decode_failed"
                    );
                    sleep(Duration::from_secs(poll_interval)).await;
                    continue;
                }
            };

            match attestation.status {
                AttestationStatus::Complete => {
                    let attestation_bytes = attestation
                        .attestation
                        .ok_or_else(|| {
                            spans::record_error_with_context(
                                "AttestationDataMissing",
                                "Attestation status is complete but attestation field is null",
                                Some("This indicates an unexpected API response format"),
                            );
                            error!(event = "attestation_data_missing");
                            CctpError::AttestationFailed {
                                reason: "Attestation missing".to_string(),
                            }
                        })?
                        .to_vec();

                    info!(
                        attestation_length_bytes = attestation_bytes.len(),
                        event = "attestation_complete"
                    );
                    return Ok(attestation_bytes);
                }
                AttestationStatus::Failed => {
                    spans::record_error_with_context(
                        "AttestationFailed",
                        "Circle API returned failed status for attestation",
                        Some(
                            "The message may be invalid or the source transaction may have failed",
                        ),
                    );
                    error!(event = "attestation_failed");
                    return Err(CctpError::AttestationFailed {
                        reason: "Attestation failed".to_string(),
                    });
                }
                AttestationStatus::Pending | AttestationStatus::PendingConfirmations => {
                    debug!(event = "attestation_pending");
                    sleep(Duration::from_secs(poll_interval)).await;
                }
            }
        }

        spans::record_error_with_context(
            "AttestationTimeout",
            &format!(
                "Attestation polling timed out after {} attempts",
                max_attempts
            ),
            Some(&format!(
                "Total duration: {} seconds",
                max_attempts as u64 * poll_interval
            )),
        );
        error!(
            total_duration_secs = max_attempts as u64 * poll_interval,
            event = "attestation_timeout"
        );
        Err(CctpError::AttestationTimeout)
    }

    /// Constructs the Iris API URL for attestation polling
    ///
    /// The message hash is formatted with the `0x` prefix as required by Circle's API.
    /// This uses the `Display` implementation of `FixedBytes<32>` which automatically
    /// includes the `0x` prefix.
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The keccak256 hash of the MessageSent event bytes
    ///
    /// # Returns
    ///
    /// The attestation endpoint URL
    ///
    /// # Errors
    ///
    /// Returns `CctpError::InvalidUrl` if URL construction fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use cctp_rs::Cctp;
    /// # use alloy_primitives::FixedBytes;
    /// # fn example(bridge: &Cctp<impl alloy_provider::Provider<alloy_network::Ethereum> + Clone>) {
    /// let hash = FixedBytes::from([0u8; 32]);
    /// let url = bridge.create_url(hash).unwrap();
    /// assert!(url.as_str().contains("/v1/attestations/0x"));
    /// # }
    /// ```
    ///
    /// See <https://developers.circle.com/stablecoins/cctp-apis>
    pub fn create_url(&self, message_hash: FixedBytes<32>) -> Result<Url> {
        self.api_url()
            .join(&format!("{ATTESTATION_PATH_V1}{message_hash}"))
            .map_err(|e| CctpError::InvalidUrl {
                reason: format!("Failed to construct attestation URL: {e}"),
            })
    }

    /// Fetches the attestation response from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `client`: The HTTP client to use
    /// * `url`: The URL to get the attestation from
    ///
    async fn fetch_attestation_response(&self, client: &Client, url: &Url) -> Result<Response> {
        client
            .get(url.as_str())
            .send()
            .await
            .map_err(CctpError::Network)
    }
}

// Implement CctpBridge trait for v1 Cctp struct
#[async_trait]
impl<P: Provider<Ethereum> + Clone> CctpBridge for Cctp<P> {
    fn source_chain(&self) -> NamedChain {
        self.source_chain
    }

    fn destination_chain(&self) -> NamedChain {
        self.destination_chain
    }

    fn recipient(&self) -> Address {
        self.recipient
    }

    async fn get_message_sent_event(&self, tx_hash: TxHash) -> Result<(Vec<u8>, FixedBytes<32>)> {
        self.get_message_sent_event(tx_hash).await
    }

    fn supports_fast_transfer(&self) -> bool {
        false
    }

    fn supports_hooks(&self) -> bool {
        false
    }

    fn finality_threshold(&self) -> Option<FinalityThreshold> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_chains::NamedChain;
    use alloy_primitives::{Address, FixedBytes};
    use alloy_provider::ProviderBuilder;
    use rstest::rstest;

    #[rstest]
    #[case(NamedChain::Mainnet, NamedChain::Arbitrum)]
    #[case(NamedChain::Arbitrum, NamedChain::Base)]
    #[case(NamedChain::Base, NamedChain::Polygon)]
    #[case(NamedChain::Sepolia, NamedChain::ArbitrumSepolia)]
    fn test_cross_chain_compatibility(#[case] source: NamedChain, #[case] destination: NamedChain) {
        // Test that chains are supported
        assert!(source.is_supported());
        assert!(destination.is_supported());

        // Test that we can get domain IDs for supported chains
        assert!(source.cctp_domain_id().is_ok());
        assert!(destination.cctp_domain_id().is_ok());
        assert!(source.token_messenger_address().is_ok());
        assert!(destination.message_transmitter_address().is_ok());
    }

    #[test]
    fn test_unsupported_chain_error() {
        let result = NamedChain::BinanceSmartChain.token_messenger_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CctpError::UnsupportedChain(_)
        ));
    }

    #[test]
    fn test_attestation_url_format_mainnet() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = Cctp::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api.circle.com/v1/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_attestation_url_format_sepolia() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = Cctp::builder()
            .source_chain(NamedChain::Sepolia)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api-sandbox.circle.com/v1/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_attestation_url_format_arbitrum() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = Cctp::builder()
            .source_chain(NamedChain::Arbitrum)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api.circle.com/v1/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_attestation_url_format_base() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = Cctp::builder()
            .source_chain(NamedChain::Base)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api.circle.com/v1/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_attestation_url_hash_format_edge_cases() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = Cctp::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // Test with all 0xff bytes
        let hash_ff = FixedBytes::from([0xff; 32]);
        let url_ff = bridge.create_url(hash_ff).unwrap();
        insta::assert_snapshot!(url_ff.as_str(), @"https://iris-api.circle.com/v1/attestations/0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

        // Test with all 0x00 bytes
        let hash_00 = FixedBytes::from([0x00; 32]);
        let url_00 = bridge.create_url(hash_00).unwrap();
        insta::assert_snapshot!(url_00.as_str(), @"https://iris-api.circle.com/v1/attestations/0x0000000000000000000000000000000000000000000000000000000000000000");

        // Test with mixed bytes
        let mut mixed_bytes = [0u8; 32];
        for (i, byte) in mixed_bytes.iter_mut().enumerate() {
            *byte = i as u8;
        }
        let hash_mixed = FixedBytes::from(mixed_bytes);
        let url_mixed = bridge.create_url(hash_mixed).unwrap();
        insta::assert_snapshot!(url_mixed.as_str(), @"https://iris-api.circle.com/v1/attestations/0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    }

    #[test]
    fn test_attestation_url_uses_correct_environment() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Mainnet should use production API
        let mainnet_bridge = Cctp::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Arbitrum)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .build();

        let mainnet_url = mainnet_bridge.create_url(FixedBytes::ZERO).unwrap();
        assert!(
            mainnet_url.as_str().starts_with(IRIS_API),
            "Mainnet should use production Iris API"
        );

        // Testnet should use sandbox API
        let testnet_bridge = Cctp::builder()
            .source_chain(NamedChain::Sepolia)
            .destination_chain(NamedChain::ArbitrumSepolia)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let testnet_url = testnet_bridge.create_url(FixedBytes::ZERO).unwrap();
        assert!(
            testnet_url.as_str().starts_with(IRIS_API_SANDBOX),
            "Testnet should use sandbox Iris API"
        );
    }
}
