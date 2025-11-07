use crate::error::{CctpError, Result};
use crate::spans;
use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Address, FixedBytes, TxHash, U256};
use alloy_provider::Provider;
use alloy_sol_types::SolEvent;
use bon::Builder;
use reqwest::{Client, Response};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info};
use url::Url;

use crate::{AttestationBytes, AttestationResponse, AttestationStatus, CctpV1};

use super::MessageTransmitter::MessageSent;

/// Circle Iris API environment URLs
///
/// See <https://developers.circle.com/stablecoins/cctp-apis>
///
pub const IRIS_API: &str = "https://iris-api.circle.com";
pub const IRIS_API_SANDBOX: &str = "https://iris-api-sandbox.circle.com";

/// CCTP v1 attestation API path
pub const ATTESTATION_PATH_V1: &str = "/v1/attestations/";

/// Default confirmation requirements and timeouts for different chains
pub const DEFAULT_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes default
pub const CHAIN_CONFIRMATION_CONFIG: &[(NamedChain, u64, Duration)] = &[
    // (Chain, Required Confirmations, Timeout)
    (NamedChain::Mainnet, 2, Duration::from_secs(300)), // 5 mins for Ethereum
    (NamedChain::Arbitrum, 1, Duration::from_secs(120)), // 2 mins for Arbitrum
    (NamedChain::Optimism, 1, Duration::from_secs(120)), // 2 mins for Optimism
    (NamedChain::Polygon, 15, Duration::from_secs(180)), // More confirmations for Polygon
    (NamedChain::Avalanche, 3, Duration::from_secs(120)), // 2 mins for Avalanche
    (NamedChain::BinanceSmartChain, 2, Duration::from_secs(120)), // 2 mins for BNB Chain
    (NamedChain::Base, 1, Duration::from_secs(120)),    // 2 mins for Base
    (NamedChain::Unichain, 1, Duration::from_secs(120)), // 2 mins for Unichain
];

/// Gets the chain-specific confirmation configuration
pub fn get_chain_confirmation_config(chain: &NamedChain) -> (u64, Duration) {
    CHAIN_CONFIRMATION_CONFIG
        .iter()
        .find(|(ch, _, _)| ch == chain)
        .map(|(_, confirmations, timeout)| (*confirmations, *timeout))
        .unwrap_or((1, DEFAULT_CONFIRMATION_TIMEOUT))
}

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
    pub fn destination_domain_id(&self) -> Result<u32> {
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

        let tx_receipt = self
            .source_provider
            .get_transaction_receipt(tx_hash)
            .await?;

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
            error!(event = "transaction_not_found");
            Err(CctpError::TransactionFailed {
                reason: "Transaction not found".to_string(),
            })
        }
    }

    /// Gets the attestation for a message hash from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `message_hash`: The hash of the message to get the attestation for
    /// * `max_attempts`: Maximum number of polling attempts (default: 30)
    /// * `poll_interval`: Time to wait between polling attempts in seconds (default: 60)
    ///
    /// # Returns
    ///
    /// The attestation bytes if successful
    pub async fn get_attestation_with_retry(
        &self,
        message_hash: FixedBytes<32>,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<AttestationBytes> {
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(60);

        let span = spans::get_attestation_with_retry(
            &message_hash,
            &self.source_chain,
            &self.destination_chain,
            max_attempts,
            poll_interval,
        );
        let _guard = span.enter();

        let client = Client::new();
        let url = self.create_url(message_hash)?;

        info!(
            url = %url,
            event = "attestation_polling_started"
        );

        for attempt in 1..=max_attempts {
            let attempt_span = spans::get_attestation(&url, attempt);
            let _attempt_guard = attempt_span.enter();

            let response = self.get_attestation(&client, &url).await?;

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

            let attestation: AttestationResponse = match response.json::<serde_json::Value>().await
            {
                Ok(attestation) => serde_json::from_value(attestation)?,
                Err(e) => {
                    error!(
                        error = %e,
                        event = "attestation_decode_failed"
                    );
                    continue;
                }
            };

            match attestation.status {
                AttestationStatus::Complete => {
                    let attestation_bytes = attestation
                        .attestation
                        .ok_or_else(|| CctpError::AttestationFailed {
                            reason: "Attestation missing".to_string(),
                        })?
                        .to_vec();

                    info!(
                        attestation_length_bytes = attestation_bytes.len(),
                        event = "attestation_complete"
                    );
                    return Ok(attestation_bytes);
                }
                AttestationStatus::Failed => {
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

    /// Gets the attestation for a message hash from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `client`: The HTTP client to use
    /// * `url`: The URL to get the attestation from
    ///
    pub async fn get_attestation(&self, client: &Client, url: &Url) -> Result<Response> {
        client
            .get(url.as_str())
            .send()
            .await
            .map_err(CctpError::Network)
    }
}

/// Parameters for bridging USDC
#[derive(Builder, Debug, Clone)]
pub struct BridgeParams {
    pub from_address: Address,
    pub recipient: Address,
    pub token_address: Address,
    pub amount: U256,
}

impl BridgeParams {
    pub fn from_address(&self) -> Address {
        self.from_address
    }

    pub fn recipient(&self) -> Address {
        self.recipient
    }

    pub fn token_address(&self) -> Address {
        self.token_address
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_chains::NamedChain;
    use alloy_primitives::{Address, U256};
    use rstest::rstest;

    #[test]
    fn test_bridge_params_builder() {
        let params = BridgeParams::builder()
            .from_address(Address::ZERO)
            .recipient(Address::ZERO)
            .token_address(Address::ZERO)
            .amount(U256::from(1000))
            .build();

        assert_eq!(params.from_address(), Address::ZERO);
        assert_eq!(params.recipient(), Address::ZERO);
        assert_eq!(params.token_address(), Address::ZERO);
        assert_eq!(params.amount(), U256::from(1000));
    }

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
            CctpError::ChainNotSupported { .. }
        ));
    }

    #[test]
    fn test_attestation_url_format_mainnet() {
        use alloy_provider::ProviderBuilder;

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
        use alloy_provider::ProviderBuilder;

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
        use alloy_provider::ProviderBuilder;

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
        use alloy_provider::ProviderBuilder;

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
        use alloy_provider::ProviderBuilder;

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
        use alloy_provider::ProviderBuilder;

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
