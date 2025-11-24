use crate::error::{CctpError, Result};
use crate::protocol::{AttestationBytes, FinalityThreshold};
use crate::{spans, AttestationResponse, AttestationStatus, CctpV2 as CctpV2Trait, DomainId};
use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Address, Bytes, FixedBytes, TxHash, U256};
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
use super::config::{ATTESTATION_PATH_V2, IRIS_API, IRIS_API_SANDBOX};
use crate::contracts::message_transmitter::MessageTransmitter::MessageSent;
use crate::contracts::v2::{MessageTransmitterV2Contract, TokenMessengerV2Contract};

/// CCTP v2 bridge implementation
///
/// This struct provides the core functionality for bridging USDC across chains
/// using Circle's Cross-Chain Transfer Protocol v2 with support for Fast Transfer,
/// programmable hooks, and expanded network coverage.
///
/// # V2 Features
///
/// - **Fast Transfer**: Sub-30 second settlement times (vs 13-19 minutes in v1)
/// - **Programmable Hooks**: Execute custom logic post-transfer (swaps, lending, etc.)
/// - **Expanded Networks**: 26+ chains supported (vs 7 in v1)
/// - **Unified Addresses**: Same contract addresses across all chains in each environment
///
/// # Example
///
/// ```rust,no_run
/// # use cctp_rs::CctpV2Bridge;
/// # use alloy_chains::NamedChain;
/// # use alloy_provider::ProviderBuilder;
/// # use alloy_primitives::{Address, U256, Bytes};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Standard transfer
/// let provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
/// let bridge = CctpV2Bridge::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Linea)
///     .source_provider(provider.clone())
///     .destination_provider(provider)
///     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
///     .build();
///
/// // Fast transfer with hooks
/// let provider2 = ProviderBuilder::new().connect("http://localhost:8545").await?;
/// let fast_bridge = CctpV2Bridge::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Linea)
///     .source_provider(provider2.clone())
///     .destination_provider(provider2)
///     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
///     .fast_transfer(true)
///     .max_fee(U256::from(100))
///     .build();
/// # Ok(())
/// # }
/// ```
#[derive(Builder, Clone, Debug)]
pub struct CctpV2<P: Provider<Ethereum> + Clone> {
    source_provider: P,
    destination_provider: P,
    source_chain: NamedChain,
    destination_chain: NamedChain,
    recipient: Address,

    /// Enable fast transfer (sub-30 second settlement)
    #[builder(default)]
    fast_transfer: bool,

    /// Optional hook data for programmable actions on destination chain
    hook_data: Option<Bytes>,

    /// Maximum fee willing to pay for fast transfer (in USDC atomic units)
    max_fee: Option<U256>,
}

impl<P: Provider<Ethereum> + Clone> CctpV2<P> {
    /// Returns the CCTP v2 API URL for the current environment
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
        self.destination_chain.cctp_v2_domain_id()
    }

    /// Returns the source provider
    pub fn source_provider(&self) -> &P {
        &self.source_provider
    }

    /// Returns the destination provider
    pub fn destination_provider(&self) -> &P {
        &self.destination_provider
    }

    /// Returns the CCTP v2 token messenger contract address
    pub fn token_messenger_v2_contract(&self) -> Result<Address> {
        self.source_chain.token_messenger_v2_address()
    }

    /// Returns the CCTP v2 message transmitter contract address
    pub fn message_transmitter_v2_contract(&self) -> Result<Address> {
        self.destination_chain.message_transmitter_v2_address()
    }

    /// Returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// Returns whether fast transfer is enabled
    pub fn is_fast_transfer(&self) -> bool {
        self.fast_transfer
    }

    /// Returns the hook data if set
    pub fn hook_data(&self) -> Option<&Bytes> {
        self.hook_data.as_ref()
    }

    /// Returns the max fee if set
    pub fn max_fee(&self) -> Option<U256> {
        self.max_fee
    }

    /// Returns the finality threshold based on configuration
    pub fn finality_threshold(&self) -> FinalityThreshold {
        if self.fast_transfer {
            FinalityThreshold::Fast
        } else {
            FinalityThreshold::Standard
        }
    }

    /// Gets the `MessageSent` event data from a CCTP v2 bridge transaction
    ///
    /// Note: V2 uses the same MessageSent event format as v1 for now.
    /// Future versions may introduce v2-specific event structures.
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
                version = "v2",
                fast_transfer = self.fast_transfer,
                has_hooks = self.hook_data.is_some(),
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

    /// Gets the attestation for a message hash from the CCTP v2 API
    ///
    /// Uses the v2 attestation endpoint which supports fast transfer finality.
    ///
    /// # Arguments
    ///
    /// * `message_hash`: The hash of the message to get the attestation for
    /// * `max_attempts`: Maximum number of polling attempts (default: 30)
    /// * `poll_interval`: Time to wait between polling attempts in seconds (default: 60 for standard, 5 for fast)
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
        // Adjust defaults based on fast transfer mode
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(if self.fast_transfer {
            5 // Fast transfers poll more frequently (5 seconds)
        } else {
            60 // Standard transfers poll every minute
        });

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
            version = "v2",
            fast_transfer = self.fast_transfer,
            finality_threshold = %self.finality_threshold(),
            event = "attestation_polling_started"
        );

        for attempt in 1..=max_attempts {
            let attempt_span = spans::get_attestation(&url, attempt);
            let _attempt_guard = attempt_span.enter();

            let response = match self.get_attestation(&client, &url).await {
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
                        version = "v2",
                        fast_transfer = self.fast_transfer,
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

    /// Initiate a USDC burn on the source chain
    ///
    /// This creates and sends the depositForBurn transaction which locks USDC on the source
    /// chain and emits a MessageSent event.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of USDC to transfer (in atomic units, e.g., 1 USDC = 1_000_000)
    /// * `from` - Address that will send the transaction (must have USDC balance and gas)
    /// * `token_address` - USDC token contract address on source chain
    ///
    /// # Returns
    ///
    /// The transaction hash of the burn transaction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::{Address, U256};
    /// # async fn example<P>(bridge: CctpV2Bridge<P>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let amount = U256::from(1_000_000); // 1 USDC
    /// let from_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    /// let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    ///
    /// let tx_hash = bridge.burn(amount, from_address, usdc).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn burn(
        &self,
        amount: U256,
        from: Address,
        token_address: Address,
    ) -> Result<TxHash> {
        let token_messenger_address = self.token_messenger_v2_contract()?;
        let destination_domain = self.destination_domain_id()?;

        let token_messenger =
            TokenMessengerV2Contract::new(token_messenger_address, self.source_provider.clone());

        let tx_request = if let Some(hook_data) = &self.hook_data {
            // Use depositForBurnWithHook if hooks are configured
            token_messenger.deposit_for_burn_with_hooks_transaction(
                from,
                self.recipient,
                destination_domain,
                token_address,
                amount,
                hook_data.clone(),
            )
        } else if self.fast_transfer {
            // Use fast transfer variant
            let max_fee = self.max_fee.unwrap_or(U256::ZERO);
            token_messenger.deposit_for_burn_fast_transaction(
                from,
                self.recipient,
                destination_domain,
                token_address,
                amount,
                max_fee,
            )
        } else {
            // Standard transfer
            token_messenger.deposit_for_burn_transaction(
                from,
                self.recipient,
                destination_domain,
                token_address,
                amount,
            )
        };

        info!(
            from = %from,
            amount = %amount,
            token_address = %token_address,
            destination_domain = %destination_domain,
            fast_transfer = self.fast_transfer,
            has_hooks = self.hook_data.is_some(),
            version = "v2",
            event = "burn_transaction_initiated"
        );

        let pending_tx = self.source_provider.send_transaction(tx_request).await?;
        let tx_hash = *pending_tx.tx_hash();

        info!(
            tx_hash = %tx_hash,
            version = "v2",
            event = "burn_transaction_sent"
        );

        Ok(tx_hash)
    }

    /// Complete a transfer by minting USDC on the destination chain
    ///
    /// This submits the receiveMessage transaction with the attestation to mint USDC
    /// on the destination chain.
    ///
    /// # Arguments
    ///
    /// * `message_bytes` - The message bytes from the MessageSent event
    /// * `attestation` - Circle's attestation signature for the message
    /// * `from` - Address that will submit the transaction (needs gas on destination chain)
    ///
    /// # Returns
    ///
    /// The transaction hash of the mint transaction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::Address;
    /// # async fn example<P>(
    /// #     bridge: CctpV2Bridge<P>,
    /// #     message: Vec<u8>,
    /// #     attestation: Vec<u8>
    /// # ) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let from_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    /// let tx_hash = bridge.mint(message, attestation, from_address).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mint(
        &self,
        message_bytes: Vec<u8>,
        attestation: AttestationBytes,
        from: Address,
    ) -> Result<TxHash> {
        let message_transmitter_address = self.message_transmitter_v2_contract()?;

        let message_transmitter = MessageTransmitterV2Contract::new(
            message_transmitter_address,
            self.destination_provider.clone(),
        );

        let tx_request = message_transmitter.receive_message_transaction(
            Bytes::from(message_bytes.clone()),
            Bytes::from(attestation.clone()),
            from,
        );

        info!(
            from = %from,
            message_len = message_bytes.len(),
            attestation_len = attestation.len(),
            version = "v2",
            event = "mint_transaction_initiated"
        );

        let pending_tx = self
            .destination_provider
            .send_transaction(tx_request)
            .await?;
        let tx_hash = *pending_tx.tx_hash();

        info!(
            tx_hash = %tx_hash,
            version = "v2",
            event = "mint_transaction_sent"
        );

        Ok(tx_hash)
    }

    /// Execute a full cross-chain transfer: burn + wait for attestation + mint
    ///
    /// This is a convenience method that orchestrates the complete transfer flow:
    /// 1. Burns USDC on source chain
    /// 2. Extracts MessageSent event from burn transaction
    /// 3. Polls Circle's Iris API for attestation
    /// 4. Mints USDC on destination chain
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of USDC to transfer (in atomic units)
    /// * `from` - Address initiating the transfer (needs USDC + gas on source, gas on destination)
    /// * `token_address` - USDC token contract address on source chain
    ///
    /// # Returns
    ///
    /// Tuple of (burn_tx_hash, mint_tx_hash)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::{Address, U256};
    /// # async fn example<P>(bridge: CctpV2Bridge<P>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let amount = U256::from(1_000_000); // 1 USDC
    /// let from_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    /// let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    ///
    /// let (burn_tx, mint_tx) = bridge.transfer(amount, from_address, usdc).await?;
    /// println!("Transfer complete! Burn: {}, Mint: {}", burn_tx, mint_tx);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transfer(
        &self,
        amount: U256,
        from: Address,
        token_address: Address,
    ) -> Result<(TxHash, TxHash)> {
        info!(
            amount = %amount,
            from = %from,
            token_address = %token_address,
            source_chain = ?self.source_chain,
            destination_chain = ?self.destination_chain,
            fast_transfer = self.fast_transfer,
            has_hooks = self.hook_data.is_some(),
            version = "v2",
            event = "full_transfer_initiated"
        );

        // Step 1: Burn tokens on source chain
        let burn_tx_hash = self.burn(amount, from, token_address).await?;

        info!(
            burn_tx_hash = %burn_tx_hash,
            event = "waiting_for_message_sent_event"
        );

        // Step 2: Get MessageSent event from burn transaction
        let (message_bytes, message_hash) = self.get_message_sent_event(burn_tx_hash).await?;

        info!(
            message_hash = %hex::encode(message_hash),
            event = "message_sent_event_retrieved"
        );

        // Step 3: Poll for attestation
        let attestation = self
            .get_attestation_with_retry(message_hash, None, None)
            .await?;

        info!(
            message_hash = %hex::encode(message_hash),
            attestation_len = attestation.len(),
            event = "attestation_received"
        );

        // Step 4: Mint tokens on destination chain
        let mint_tx_hash = self.mint(message_bytes, attestation, from).await?;

        info!(
            burn_tx_hash = %burn_tx_hash,
            mint_tx_hash = %mint_tx_hash,
            version = "v2",
            event = "full_transfer_completed"
        );

        Ok((burn_tx_hash, mint_tx_hash))
    }

    /// Constructs the Iris API v2 URL for attestation polling
    ///
    /// The message hash is formatted with the `0x` prefix as required by Circle's API.
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The keccak256 hash of the MessageSent event bytes
    ///
    /// # Returns
    ///
    /// The v2 attestation endpoint URL
    ///
    /// # Errors
    ///
    /// Returns `CctpError::InvalidUrl` if URL construction fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::FixedBytes;
    /// # fn example<P>(bridge: &CctpV2Bridge<P>)
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let hash = FixedBytes::from([0u8; 32]);
    /// let url = bridge.create_url(hash).unwrap();
    /// assert!(url.as_str().contains("/v2/attestations/0x"));
    /// # }
    /// ```
    ///
    /// See <https://developers.circle.com/cctp/v2-apis>
    pub fn create_url(&self, message_hash: FixedBytes<32>) -> Result<Url> {
        self.api_url()
            .join(&format!("{ATTESTATION_PATH_V2}{message_hash}"))
            .map_err(|e| CctpError::InvalidUrl {
                reason: format!("Failed to construct v2 attestation URL: {e}"),
            })
    }

    /// Gets the attestation for a message hash from the CCTP v2 API
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

// Implement CctpBridge trait for v2 CctpV2 struct
#[async_trait]
impl<P: Provider<Ethereum> + Clone> CctpBridge for CctpV2<P> {
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

    async fn get_attestation_with_retry(
        &self,
        message_hash: FixedBytes<32>,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<AttestationBytes> {
        self.get_attestation_with_retry(message_hash, max_attempts, poll_interval)
            .await
    }

    fn supports_fast_transfer(&self) -> bool {
        self.fast_transfer
    }

    fn supports_hooks(&self) -> bool {
        self.hook_data.is_some()
    }

    fn finality_threshold(&self) -> Option<FinalityThreshold> {
        Some(self.finality_threshold())
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
    #[case(NamedChain::Mainnet, NamedChain::Linea)]
    #[case(NamedChain::Arbitrum, NamedChain::Sonic)]
    #[case(NamedChain::Base, NamedChain::Sei)]
    #[case(NamedChain::Sepolia, NamedChain::BaseSepolia)]
    fn test_v2_cross_chain_compatibility(
        #[case] source: NamedChain,
        #[case] destination: NamedChain,
    ) {
        // Test that chains support v2
        assert!(source.supports_cctp_v2());
        assert!(destination.supports_cctp_v2());

        // Test that we can get domain IDs for supported chains
        assert!(source.cctp_v2_domain_id().is_ok());
        assert!(destination.cctp_v2_domain_id().is_ok());
        assert!(source.token_messenger_v2_address().is_ok());
        assert!(destination.message_transmitter_v2_address().is_ok());
    }

    #[test]
    fn test_v2_unsupported_chain_error() {
        let result = NamedChain::Moonbeam.token_messenger_v2_address();
        assert!(result.is_err());
    }

    #[test]
    fn test_v2_attestation_url_format_mainnet() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api.circle.com/v2/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_v2_attestation_url_format_sepolia() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Sepolia)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let test_hash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_hash).unwrap();
        insta::assert_snapshot!(url.as_str(), @"https://iris-api-sandbox.circle.com/v2/attestations/0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_v2_fast_transfer_flag() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Standard transfer (default)
        let standard = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .build();

        assert!(!standard.is_fast_transfer());
        assert_eq!(standard.finality_threshold(), FinalityThreshold::Standard);
        assert!(!standard.supports_fast_transfer());

        // Fast transfer
        let fast = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .build();

        assert!(fast.is_fast_transfer());
        assert_eq!(fast.finality_threshold(), FinalityThreshold::Fast);
        assert!(fast.supports_fast_transfer());
    }

    #[test]
    fn test_v2_hooks_support() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Without hooks
        let no_hooks = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .build();

        assert!(!no_hooks.supports_hooks());
        assert!(no_hooks.hook_data().is_none());

        // With hooks
        let hook_data = Bytes::from(vec![1, 2, 3, 4]);
        let with_hooks = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .hook_data(hook_data.clone())
            .build();

        assert!(with_hooks.supports_hooks());
        assert_eq!(with_hooks.hook_data(), Some(&hook_data));
    }

    #[test]
    fn test_v2_max_fee() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let max_fee = U256::from(1000);

        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .max_fee(max_fee)
            .build();

        assert_eq!(bridge.max_fee(), Some(max_fee));
    }

    #[test]
    fn test_v2_unified_addresses() {
        // All v2 mainnet chains should have the same addresses
        let linea_tm = NamedChain::Linea.token_messenger_v2_address().unwrap();
        let sonic_tm = NamedChain::Sonic.token_messenger_v2_address().unwrap();
        let mainnet_tm = NamedChain::Mainnet.token_messenger_v2_address().unwrap();

        assert_eq!(linea_tm, sonic_tm);
        assert_eq!(linea_tm, mainnet_tm);

        let linea_mt = NamedChain::Linea.message_transmitter_v2_address().unwrap();
        let sonic_mt = NamedChain::Sonic.message_transmitter_v2_address().unwrap();
        let mainnet_mt = NamedChain::Mainnet
            .message_transmitter_v2_address()
            .unwrap();

        assert_eq!(linea_mt, sonic_mt);
        assert_eq!(linea_mt, mainnet_mt);
    }

    #[test]
    fn test_v2_builder_pattern() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Build with all options
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .max_fee(U256::from(500))
            .hook_data(Bytes::from(vec![1, 2, 3]))
            .build();

        assert_eq!(bridge.source_chain(), &NamedChain::Mainnet);
        assert_eq!(bridge.destination_chain(), &NamedChain::Linea);
        assert_eq!(bridge.recipient(), &Address::ZERO);
        assert!(bridge.is_fast_transfer());
        assert_eq!(bridge.max_fee(), Some(U256::from(500)));
        assert!(bridge.hook_data().is_some());
    }
}
