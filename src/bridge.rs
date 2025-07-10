use crate::error::{CctpError, Result};
use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Address, FixedBytes, TxHash, U256};
use alloy_provider::Provider;
use alloy_sol_types::SolEvent;
use bon::Builder;
use reqwest::{Client, Response};
use std::{thread::sleep, time::Duration};
use tracing::{debug, info, instrument, Level};

use crate::{AttestationBytes, AttestationResponse, AttestationStatus, CctpV1};

use super::MessageTransmitter::MessageSent;

/// Circle Iris API environment URLs
///
/// See <https://developers.circle.com/stablecoins/cctp-apis>
///
pub const IRIS_API: &str = "https://iris-api.circle.com";
pub const IRIS_API_SANDBOX: &str = "https://iris-api-sandbox.circle.com";

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

/// CCTP v1 bridge
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
    pub fn api_url(&self) -> &'static str {
        if self.source_chain.is_testnet() {
            IRIS_API_SANDBOX
        } else {
            IRIS_API
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
    #[instrument(skip(self), level = Level::INFO)]
    pub async fn get_message_sent_event(
        &self,
        tx_hash: TxHash,
    ) -> Result<(Vec<u8>, FixedBytes<32>)> {
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
                .ok_or_else(|| CctpError::TransactionFailed {
                    reason: "MessageSent event not found".to_string(),
                })?;

            // Decode the log data using the generated event bindings
            let decoded = MessageSent::abi_decode_data(&message_sent_log.data().data)?;

            let message_sent_event = decoded.0.to_vec();
            let message_hash = alloy_primitives::keccak256(&message_sent_event);

            Ok((message_sent_event, message_hash))
        } else {
            return Err(CctpError::TransactionFailed {
                reason: "Transaction not found".to_string(),
            });
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
        let client = Client::new();
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(60);

        info!(message_hash = ?message_hash, "Polling for attestation ...");

        let url = self.create_url(message_hash);

        info!(url = ?url, "Attestation URL");

        for attempt in 1..=max_attempts {
            info!(
                attempt = ?attempt,
                max_attempts = ?max_attempts,
                "Getting attestation ..."
            );
            let response = self.get_attestation(&client, &url).await?;
            debug!(response = ?response);

            info!(attestation_status = ?response.status());

            // Handle rate limiting
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let secs = 5 * 60;
                info!(sleep_secs = ?secs, "Rate limit exceeded, waiting before retrying");
                sleep(Duration::from_secs(secs));
                continue;
            }

            // Handle 404 status - treat as pending since the attestation likely doesn't exist yet
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                info!(
                    attempt = ?attempt,
                    max_attempts = ?max_attempts,
                    poll_interval = ?poll_interval,
                    "Attestation not found (404), waiting before retrying"
                );
                sleep(Duration::from_secs(poll_interval));
                continue;
            }

            // Ensure the response status is successful before trying to parse JSON
            response.error_for_status_ref()?;

            info!("Decoding attestation response");

            let attestation: AttestationResponse = match response.json::<serde_json::Value>().await
            {
                Ok(attestation) => {
                    info!(attestation = ?attestation, "Attestation response");
                    serde_json::from_value(attestation)?
                }
                Err(e) => {
                    info!(error = ?e, "Error decoding attestation response");
                    continue;
                }
            };

            match attestation.status {
                AttestationStatus::Complete => {
                    let attestation_bytes =
                        attestation
                            .attestation
                            .ok_or_else(|| CctpError::AttestationFailed {
                                reason: "Attestation missing".to_string(),
                            })?;

                    // Remove '0x' prefix if present and decode hex
                    let attestation_bytes =
                        if let Some(stripped) = attestation_bytes.strip_prefix("0x") {
                            hex::decode(stripped)
                        } else {
                            hex::decode(&attestation_bytes)
                        }?;

                    info!("Attestation received successfully");
                    return Ok(attestation_bytes);
                }
                AttestationStatus::Failed => {
                    return Err(CctpError::AttestationFailed {
                        reason: "Attestation failed".to_string(),
                    });
                }
                AttestationStatus::Pending | AttestationStatus::PendingConfirmations => {
                    info!(
                        attempt = ?attempt,
                        max_attempts = ?max_attempts,
                        poll_interval = ?poll_interval,
                        "Attestation pending, waiting before retrying"
                    );
                    sleep(Duration::from_secs(poll_interval));
                }
            }
        }

        Err(CctpError::AttestationTimeout)
    }

    /// See <https://developers.circle.com/stablecoins/cctp-apis>
    pub fn create_url(&self, message_hash: FixedBytes<32>) -> String {
        let base_url = self.api_url();
        format!("{base_url}/v1/attestations/{message_hash}")
    }

    /// Gets the attestation for a message hash from the CCTP API
    ///
    /// # Arguments
    ///
    /// * `client`: The HTTP client to use
    /// * `url`: The URL to get the attestation from
    ///
    pub async fn get_attestation(&self, client: &Client, url: &str) -> Result<Response> {
        client.get(url).send().await.map_err(CctpError::Network)
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
}
