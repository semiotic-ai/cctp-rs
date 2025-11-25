use crate::error::{CctpError, Result};
use crate::protocol::{AttestationBytes, FinalityThreshold};
use crate::{spans, AttestationStatus, CctpV2 as CctpV2Trait, DomainId, V2AttestationResponse};
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
use super::config::{IRIS_API, IRIS_API_SANDBOX, MESSAGES_PATH_V2};
use crate::contracts::erc20::Erc20Contract;
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
    /// **⚠️ WARNING**: For v2 transfers, the message extracted from transaction logs contains
    /// zeros in the nonce field (bytes 12-44). Circle's attestation service fills in the actual
    /// nonce before signing. If you need to mint tokens, use [`get_attestation_with_message`]
    /// instead, which returns the canonical message from Circle's API with the correct nonce.
    ///
    /// This function is useful for:
    /// - Computing the message hash for tracking/monitoring purposes
    /// - Debugging and inspecting the message structure
    ///
    /// For actual token minting, use [`get_attestation_with_message`] to get the correct message.
    ///
    /// # Arguments
    ///
    /// * `tx_hash`: The hash of the transaction to get the `MessageSent` event for
    ///
    /// # Returns
    ///
    /// Returns the message bytes (with zeros for nonce) and its hash
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

    /// Gets the attestation for a transaction from Circle's Iris API (v2)
    ///
    /// This method polls the Iris API until the attestation is ready or times out.
    /// Unlike CCTP v1 which uses message hashes, v2 uses the transaction hash directly.
    /// The source domain is automatically derived from the bridge's configured source chain.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the burn transaction on the source chain
    /// * `max_attempts` - Maximum number of polling attempts (default: 30)
    /// * `poll_interval` - Time between polls in seconds (default: 5 for fast transfer, 60 for standard)
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
    /// // Get attestation for a burn transaction (v2 uses tx hash, not message hash)
    /// let attestation = bridge.get_attestation(burn_tx_hash, None, None).await?;
    ///
    /// // Or with custom retry settings
    /// let attestation = bridge.get_attestation(
    ///     burn_tx_hash,
    ///     Some(60),  // max 60 attempts
    ///     Some(10),  // 10 seconds between polls
    /// ).await?;
    /// ```
    pub async fn get_attestation(
        &self,
        tx_hash: TxHash,
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

        let message_hash = FixedBytes::from([0u8; 32]); // Placeholder for span compatibility
        let span = spans::get_attestation_with_retry(
            &message_hash,
            &self.source_chain,
            &self.destination_chain,
            max_attempts,
            poll_interval,
        );
        let _guard = span.enter();

        let client = Client::new();
        let url = self.create_url(tx_hash)?;

        info!(
            url = %url,
            tx_hash = %tx_hash,
            version = "v2",
            fast_transfer = self.fast_transfer,
            finality_threshold = %self.finality_threshold(),
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
                        &format!("Failed to fetch attestation: {e}"),
                        Some(&format!("Attempt {attempt}/{max_attempts}")),
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

            // Parse v2 response format (array of messages)
            let v2_response: V2AttestationResponse = match serde_json::from_str(&response_text) {
                Ok(response) => response,
                Err(e) => {
                    error!(
                        error = %e,
                        response_body = %response_text,
                        tx_hash = %tx_hash,
                        attempt = attempt,
                        event = "attestation_decode_failed"
                    );
                    sleep(Duration::from_secs(poll_interval)).await;
                    continue;
                }
            };

            // V2 returns an array of messages - get the first one
            let message = match v2_response.messages.first() {
                Some(msg) => msg,
                None => {
                    debug!(event = "no_messages_in_response");
                    sleep(Duration::from_secs(poll_interval)).await;
                    continue;
                }
            };

            match message.status {
                AttestationStatus::Complete => {
                    let attestation_bytes = message
                        .attestation
                        .as_ref()
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
            &format!("Attestation polling timed out after {max_attempts} attempts"),
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

    /// Gets the attestation AND message for a transaction from Circle's Iris API (v2)
    ///
    /// **IMPORTANT**: This is the recommended method for v2 transfers. Unlike v1, the MessageSent
    /// event log contains a "template" message with zeros in the nonce field. Circle's attestation
    /// service fills in the actual nonce before signing. You MUST use the message returned by this
    /// function (from Circle's API), not the message extracted from transaction logs.
    ///
    /// This method polls the Iris API until the attestation is ready or times out.
    /// Unlike CCTP v1 which uses message hashes, v2 uses the transaction hash directly.
    /// The source domain is automatically derived from the bridge's configured source chain.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the burn transaction on the source chain
    /// * `max_attempts` - Maximum number of polling attempts (default: 30)
    /// * `poll_interval` - Time between polls in seconds (default: 5 for fast transfer, 60 for standard)
    ///
    /// # Returns
    ///
    /// A tuple of `(message_bytes, attestation_bytes)` where:
    /// - `message_bytes`: The canonical message from Circle's API (with nonce filled in)
    /// - `attestation_bytes`: The signed attestation to submit to the destination chain
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
    /// // Get attestation and message for a burn transaction
    /// let (message, attestation) = bridge.get_attestation_with_message(burn_tx_hash, None, None).await?;
    ///
    /// // Use this message (NOT the one from get_message_sent_event) for minting
    /// let mint_tx = bridge.mint(message, attestation, recipient).await?;
    /// ```
    pub async fn get_attestation_with_message(
        &self,
        tx_hash: TxHash,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<(Vec<u8>, AttestationBytes)> {
        // Adjust defaults based on fast transfer mode
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(if self.fast_transfer {
            5 // Fast transfers poll more frequently (5 seconds)
        } else {
            60 // Standard transfers poll every minute
        });

        let message_hash = FixedBytes::from([0u8; 32]); // Placeholder for span compatibility
        let span = spans::get_attestation_with_retry(
            &message_hash,
            &self.source_chain,
            &self.destination_chain,
            max_attempts,
            poll_interval,
        );
        let _guard = span.enter();

        let client = Client::new();
        let url = self.create_url(tx_hash)?;

        info!(
            url = %url,
            tx_hash = %tx_hash,
            version = "v2",
            fast_transfer = self.fast_transfer,
            finality_threshold = %self.finality_threshold(),
            event = "attestation_with_message_polling_started"
        );

        for attempt in 1..=max_attempts {
            let attempt_span = spans::get_attestation(&url, attempt);
            let _attempt_guard = attempt_span.enter();

            let response = match self.fetch_attestation_response(&client, &url).await {
                Ok(r) => r,
                Err(e) => {
                    spans::record_error_with_context(
                        "HttpRequestFailed",
                        &format!("Failed to fetch attestation: {e}"),
                        Some(&format!("Attempt {attempt}/{max_attempts}")),
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

            // Parse v2 response format (array of messages)
            let v2_response: V2AttestationResponse = match serde_json::from_str(&response_text) {
                Ok(response) => response,
                Err(e) => {
                    error!(
                        error = %e,
                        response_body = %response_text,
                        tx_hash = %tx_hash,
                        attempt = attempt,
                        event = "attestation_decode_failed"
                    );
                    sleep(Duration::from_secs(poll_interval)).await;
                    continue;
                }
            };

            // V2 returns an array of messages - get the first one
            let message = match v2_response.messages.first() {
                Some(msg) => msg,
                None => {
                    debug!(event = "no_messages_in_response");
                    sleep(Duration::from_secs(poll_interval)).await;
                    continue;
                }
            };

            match message.status {
                AttestationStatus::Complete => {
                    let attestation_bytes = message
                        .attestation
                        .as_ref()
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

                    let message_bytes = message
                        .message
                        .as_ref()
                        .ok_or_else(|| {
                            spans::record_error_with_context(
                                "MessageDataMissing",
                                "Attestation status is complete but message field is null",
                                Some("This indicates an unexpected API response format"),
                            );
                            error!(event = "message_data_missing");
                            CctpError::AttestationFailed {
                                reason: "Message missing".to_string(),
                            }
                        })?
                        .to_vec();

                    info!(
                        message_length_bytes = message_bytes.len(),
                        attestation_length_bytes = attestation_bytes.len(),
                        version = "v2",
                        fast_transfer = self.fast_transfer,
                        event = "attestation_with_message_complete"
                    );
                    return Ok((message_bytes, attestation_bytes));
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
            &format!("Attestation polling timed out after {max_attempts} attempts"),
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

    /// Get the current ERC20 allowance for the TokenMessenger contract
    ///
    /// Use this to check if approval is needed before calling `burn`.
    ///
    /// # Arguments
    ///
    /// * `token_address` - The ERC20 token contract address (e.g., USDC)
    /// * `owner` - The address that owns the tokens
    ///
    /// # Returns
    ///
    /// The current allowance amount
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::{Address, U256};
    /// # async fn example<P>(bridge: CctpV2Bridge<P>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    /// let owner = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    ///
    /// let allowance = bridge.get_allowance(usdc, owner).await?;
    /// if allowance < U256::from(1_000_000) {
    ///     // Need to approve first
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_allowance(&self, token_address: Address, owner: Address) -> Result<U256> {
        let spender = self.token_messenger_v2_contract()?;
        let erc20 = Erc20Contract::new(token_address, self.source_provider.clone());

        erc20
            .allowance(owner, spender)
            .await
            .map_err(|e| CctpError::ContractCall(format!("Failed to get allowance: {e}")))
    }

    /// Approve the TokenMessenger contract to spend tokens
    ///
    /// This must be called before `burn` if the TokenMessenger doesn't have
    /// sufficient allowance to transfer the desired amount.
    ///
    /// # Arguments
    ///
    /// * `token_address` - The ERC20 token contract address (e.g., USDC)
    /// * `owner` - The address that owns the tokens and will sign the transaction
    /// * `amount` - The amount to approve
    ///
    /// # Returns
    ///
    /// The transaction hash of the approval transaction
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::{Address, U256};
    /// # async fn example<P>(bridge: CctpV2Bridge<P>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    /// let owner = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    /// let amount = U256::from(1_000_000); // 1 USDC
    ///
    /// // Check allowance first
    /// let allowance = bridge.get_allowance(usdc, owner).await?;
    /// if allowance < amount {
    ///     let tx_hash = bridge.approve(usdc, owner, amount).await?;
    ///     println!("Approved: {}", tx_hash);
    /// }
    ///
    /// // Now burn is safe to call
    /// let burn_tx = bridge.burn(amount, owner, usdc).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn approve(
        &self,
        token_address: Address,
        owner: Address,
        amount: U256,
    ) -> Result<TxHash> {
        let spender = self.token_messenger_v2_contract()?;
        let erc20 = Erc20Contract::new(token_address, self.source_provider.clone());

        let tx_request = erc20.approve_transaction(owner, spender, amount);

        info!(
            owner = %owner,
            spender = %spender,
            amount = %amount,
            token_address = %token_address,
            version = "v2",
            event = "approval_transaction_initiated"
        );

        let pending_tx = self.source_provider.send_transaction(tx_request).await?;
        let tx_hash = *pending_tx.tx_hash();

        info!(
            tx_hash = %tx_hash,
            version = "v2",
            event = "approval_transaction_sent"
        );

        Ok(tx_hash)
    }

    /// Check if approval is needed and approve if necessary
    ///
    /// This is a convenience method that combines `get_allowance` and `approve`.
    /// It only sends an approval transaction if the current allowance is less than
    /// the requested amount.
    ///
    /// # Arguments
    ///
    /// * `token_address` - The ERC20 token contract address (e.g., USDC)
    /// * `owner` - The address that owns the tokens
    /// * `amount` - The amount that needs to be approved
    ///
    /// # Returns
    ///
    /// `Some(tx_hash)` if an approval was sent, `None` if approval was already sufficient
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::{Address, U256};
    /// # async fn example<P>(bridge: CctpV2Bridge<P>) -> Result<(), Box<dyn std::error::Error>>
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    /// let owner = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?;
    /// let amount = U256::from(1_000_000);
    ///
    /// // Approve if needed, then burn
    /// if let Some(approval_tx) = bridge.ensure_approval(usdc, owner, amount).await? {
    ///     println!("Approval sent: {}", approval_tx);
    /// }
    /// let burn_tx = bridge.burn(amount, owner, usdc).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ensure_approval(
        &self,
        token_address: Address,
        owner: Address,
        amount: U256,
    ) -> Result<Option<TxHash>> {
        let current_allowance = self.get_allowance(token_address, owner).await?;

        if current_allowance >= amount {
            info!(
                owner = %owner,
                current_allowance = %current_allowance,
                required_amount = %amount,
                token_address = %token_address,
                version = "v2",
                event = "approval_not_needed"
            );
            return Ok(None);
        }

        info!(
            owner = %owner,
            current_allowance = %current_allowance,
            required_amount = %amount,
            token_address = %token_address,
            version = "v2",
            event = "approval_needed"
        );

        let tx_hash = self.approve(token_address, owner, amount).await?;
        Ok(Some(tx_hash))
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

        // Step 2: Poll for attestation and get canonical message from Circle's API
        // Note: We use get_attestation_with_message because the MessageSent event log
        // contains zeros in the nonce field. Circle fills in the actual nonce before signing.
        let (message_bytes, attestation) = self
            .get_attestation_with_message(burn_tx_hash, None, None)
            .await?;

        info!(
            burn_tx_hash = %burn_tx_hash,
            message_len = message_bytes.len(),
            attestation_len = attestation.len(),
            event = "attestation_with_message_received"
        );

        // Step 3: Mint tokens on destination chain
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
    /// The v2 API uses a different endpoint format than v1:
    /// - V1: `/v1/attestations/{messageHash}`
    /// - V2: `/v2/messages/{sourceDomain}?transactionHash={txHash}`
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The transaction hash of the burn transaction on the source chain
    ///
    /// # Returns
    ///
    /// The v2 messages endpoint URL with source domain and transaction hash
    ///
    /// # Errors
    ///
    /// Returns `CctpError::InvalidUrl` if URL construction fails, or
    /// `CctpError::ChainNotSupported` if the source chain doesn't have a v2 domain ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cctp_rs::CctpV2Bridge;
    /// # use alloy_primitives::TxHash;
    /// # fn example<P>(bridge: &CctpV2Bridge<P>)
    /// # where P: alloy_provider::Provider<alloy_network::Ethereum> + Clone
    /// # {
    /// let tx_hash: TxHash = "0x123...".parse().unwrap();
    /// let url = bridge.create_url(tx_hash).unwrap();
    /// // URL format: https://iris-api.circle.com/v2/messages/0?transactionHash=0x123...
    /// assert!(url.as_str().contains("/v2/messages/"));
    /// assert!(url.as_str().contains("transactionHash="));
    /// # }
    /// ```
    ///
    /// See <https://developers.circle.com/cctp/transfer-usdc-on-testnet-from-ethereum-to-avalanche>
    pub fn create_url(&self, tx_hash: TxHash) -> Result<Url> {
        let source_domain = self.source_chain.cctp_v2_domain_id()?.as_u32();
        self.api_url()
            .join(&format!(
                "{MESSAGES_PATH_V2}{source_domain}?transactionHash={tx_hash}"
            ))
            .map_err(|e| CctpError::InvalidUrl {
                reason: format!("Failed to construct v2 messages URL: {e}"),
            })
    }

    /// Fetches the attestation response from the CCTP v2 API
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
    fn test_v2_messages_url_format_mainnet() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // V2 uses transaction hash, not message hash
        let test_tx_hash: TxHash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_tx_hash).unwrap();
        // Format: /v2/messages/{domain}?transactionHash={txHash}
        // Ethereum mainnet domain = 0
        insta::assert_snapshot!(url.as_str(), @"https://iris-api.circle.com/v2/messages/0?transactionHash=0x1212121212121212121212121212121212121212121212121212121212121212");
    }

    #[test]
    fn test_v2_messages_url_format_sepolia() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Sepolia)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // V2 uses transaction hash, not message hash
        let test_tx_hash: TxHash = FixedBytes::from([0x12; 32]);
        let url = bridge.create_url(test_tx_hash).unwrap();
        // Format: /v2/messages/{domain}?transactionHash={txHash}
        // Sepolia domain = 0 (same as mainnet Ethereum)
        insta::assert_snapshot!(url.as_str(), @"https://iris-api-sandbox.circle.com/v2/messages/0?transactionHash=0x1212121212121212121212121212121212121212121212121212121212121212");
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

    // Integration tests for transfer flow logic

    #[test]
    fn test_v2_contract_method_selection_standard() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Standard transfer should use basic depositForBurn
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // Verify configuration for standard transfer
        assert!(!bridge.is_fast_transfer());
        assert!(bridge.hook_data().is_none());
        assert_eq!(bridge.finality_threshold(), FinalityThreshold::Standard);
        assert_eq!(bridge.finality_threshold().as_u32(), 2000);
    }

    #[test]
    fn test_v2_contract_method_selection_fast() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Fast transfer should use depositForBurnFast
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .max_fee(U256::from(1000))
            .build();

        // Verify configuration for fast transfer
        assert!(bridge.is_fast_transfer());
        assert!(bridge.hook_data().is_none());
        assert_eq!(bridge.finality_threshold(), FinalityThreshold::Fast);
        assert_eq!(bridge.finality_threshold().as_u32(), 1000);
        assert_eq!(bridge.max_fee(), Some(U256::from(1000)));
    }

    #[test]
    fn test_v2_contract_method_selection_hooks() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let hook_data = Bytes::from(vec![1, 2, 3, 4]);

        // With hooks should use depositForBurnWithHook
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .hook_data(hook_data.clone())
            .build();

        // Verify configuration for hooks transfer
        assert!(!bridge.is_fast_transfer());
        assert_eq!(bridge.hook_data(), Some(&hook_data));
        assert_eq!(bridge.finality_threshold(), FinalityThreshold::Standard);
    }

    #[test]
    fn test_v2_contract_method_selection_priority() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let hook_data = Bytes::from(vec![1, 2, 3, 4]);

        // Hooks should take priority over fast transfer
        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .max_fee(U256::from(1000))
            .hook_data(hook_data.clone())
            .build();

        // Verify hooks take priority
        assert!(bridge.is_fast_transfer());
        assert_eq!(bridge.hook_data(), Some(&hook_data));
        assert_eq!(bridge.finality_threshold(), FinalityThreshold::Fast);
    }

    #[rstest]
    #[case(NamedChain::Mainnet, NamedChain::Linea)]
    #[case(NamedChain::Arbitrum, NamedChain::Sonic)]
    #[case(NamedChain::Base, NamedChain::Sei)]
    #[case(NamedChain::Sepolia, NamedChain::BaseSepolia)]
    fn test_v2_fast_transfer_chain_support(
        #[case] source: NamedChain,
        #[case] destination: NamedChain,
    ) {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // All v2 chains support fast transfer
        let bridge = CctpV2::builder()
            .source_chain(source)
            .destination_chain(destination)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .build();

        assert!(bridge.supports_fast_transfer());
        assert_eq!(bridge.finality_threshold(), FinalityThreshold::Fast);
    }

    #[test]
    fn test_v2_domain_id_resolution() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // Verify domain IDs are correctly resolved
        let source_domain = bridge.source_chain().cctp_v2_domain_id().unwrap();
        let dest_domain = bridge.destination_domain_id().unwrap();

        assert_eq!(source_domain, DomainId::Ethereum);
        assert_eq!(dest_domain, DomainId::Linea);
        assert_eq!(source_domain.as_u32(), 0);
        assert_eq!(dest_domain.as_u32(), 11);
    }

    #[test]
    fn test_v2_contract_address_resolution() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // Verify contract addresses are correctly resolved
        let token_messenger = bridge.token_messenger_v2_contract().unwrap();
        let message_transmitter = bridge.message_transmitter_v2_contract().unwrap();

        // Mainnet v2 addresses (unified across all v2 chains)
        assert_eq!(
            token_messenger,
            "0x28b5a0e9C621a5BadaA536219b3a228C8168cf5d"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            message_transmitter,
            "0x81D40F21F12A8F0E3252Bccb954D722d4c464B64"
                .parse::<Address>()
                .unwrap()
        );
    }

    #[test]
    fn test_v2_api_url_construction() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Mainnet should use production API
        let mainnet_bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .build();

        let test_tx_hash: TxHash = FixedBytes::from([0xab; 32]);
        let mainnet_url = mainnet_bridge.create_url(test_tx_hash).unwrap();
        assert!(mainnet_url.as_str().contains("iris-api.circle.com"));
        assert!(mainnet_url.as_str().contains("/v2/messages/"));
        assert!(mainnet_url.as_str().contains("transactionHash="));

        // Testnet should use sandbox API
        let testnet_bridge = CctpV2::builder()
            .source_chain(NamedChain::Sepolia)
            .destination_chain(NamedChain::BaseSepolia)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        let testnet_url = testnet_bridge.create_url(test_tx_hash).unwrap();
        assert!(testnet_url.as_str().contains("iris-api-sandbox.circle.com"));
        assert!(testnet_url.as_str().contains("/v2/messages/"));
        assert!(testnet_url.as_str().contains("transactionHash="));
    }

    #[test]
    fn test_v2_finality_threshold_mapping() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Standard transfer
        let standard = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .build();

        assert_eq!(standard.finality_threshold(), FinalityThreshold::Standard);
        assert_eq!(standard.finality_threshold().as_u32(), 2000);
        assert!(standard.finality_threshold().is_standard());
        assert!(!standard.finality_threshold().is_fast());

        // Fast transfer
        let fast = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .build();

        assert_eq!(fast.finality_threshold(), FinalityThreshold::Fast);
        assert_eq!(fast.finality_threshold().as_u32(), 1000);
        assert!(!fast.finality_threshold().is_standard());
        assert!(fast.finality_threshold().is_fast());
    }

    #[rstest]
    #[case(NamedChain::Mainnet, NamedChain::Linea)]
    #[case(NamedChain::Arbitrum, NamedChain::Sonic)]
    #[case(NamedChain::Base, NamedChain::Sei)]
    #[case(NamedChain::Optimism, NamedChain::Polygon)]
    fn test_v2_cross_chain_integration(
        #[case] source: NamedChain,
        #[case] destination: NamedChain,
    ) {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Verify we can create a bridge for any valid v2 chain pair
        let bridge = CctpV2::builder()
            .source_chain(source)
            .destination_chain(destination)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .build();

        // All should resolve successfully
        assert!(bridge.source_chain().cctp_v2_domain_id().is_ok());
        assert!(bridge.destination_domain_id().is_ok());
        assert!(bridge.token_messenger_v2_contract().is_ok());
        assert!(bridge.message_transmitter_v2_contract().is_ok());

        // All mainnet chains should have the same v2 contract addresses
        if !source.is_testnet() && !destination.is_testnet() {
            let token_messenger = bridge.token_messenger_v2_contract().unwrap();
            let message_transmitter = bridge.message_transmitter_v2_contract().unwrap();

            assert_eq!(
                token_messenger,
                "0x28b5a0e9C621a5BadaA536219b3a228C8168cf5d"
                    .parse::<Address>()
                    .unwrap()
            );
            assert_eq!(
                message_transmitter,
                "0x81D40F21F12A8F0E3252Bccb954D722d4c464B64"
                    .parse::<Address>()
                    .unwrap()
            );
        }
    }

    #[test]
    fn test_v2_error_handling_unsupported_chain() {
        // Try to get v2 addresses for a chain that doesn't support v2
        let result = NamedChain::Moonbeam.token_messenger_v2_address();
        assert!(result.is_err());

        let result = NamedChain::Moonbeam.message_transmitter_v2_address();
        assert!(result.is_err());

        let result = NamedChain::Moonbeam.cctp_v2_domain_id();
        assert!(result.is_err());
    }

    #[test]
    fn test_v2_recipient_address_validation() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d"
            .parse::<Address>()
            .unwrap();

        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(recipient)
            .build();

        assert_eq!(bridge.recipient(), &recipient);
    }

    #[test]
    fn test_v2_max_fee_defaults() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());

        // Without max_fee specified
        let no_fee = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider.clone())
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .build();

        assert_eq!(no_fee.max_fee(), None);

        // With max_fee specified
        let with_fee = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .fast_transfer(true)
            .max_fee(U256::from(500))
            .build();

        assert_eq!(with_fee.max_fee(), Some(U256::from(500)));
    }

    #[test]
    fn test_v2_hooks_data_validation() {
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let hook_data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);

        let bridge = CctpV2::builder()
            .source_chain(NamedChain::Mainnet)
            .destination_chain(NamedChain::Linea)
            .source_provider(provider.clone())
            .destination_provider(provider)
            .recipient(Address::ZERO)
            .hook_data(hook_data.clone())
            .build();

        assert_eq!(bridge.hook_data(), Some(&hook_data));
        assert_eq!(bridge.hook_data().unwrap().len(), 4);
        assert_eq!(bridge.hook_data().unwrap()[0], 0xde);
    }
}
