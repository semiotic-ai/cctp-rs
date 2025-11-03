//! CCTP v1 bridge implementation with trait-based abstraction.

use crate::error::{CctpError, Result};
use crate::receipt_adapter::ReceiptAdapter;
use crate::traits::{AttestationProvider, BlockchainProvider, Clock};
use crate::{AttestationBytes, AttestationResponse, AttestationStatus, CctpV1};
use alloy_chains::NamedChain;
use alloy_network::Network;
use alloy_primitives::{hex, keccak256, Address, FixedBytes, TxHash, U256};
use alloy_rpc_types::Log;
use alloy_sol_types::SolEvent;
use bon::Builder;
use std::time::Duration;
use tracing::{debug, error, info, instrument, trace, Level};

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

/// CCTP v1 bridge implementation with trait-based abstraction.
///
/// This struct provides the core functionality for bridging USDC across chains
/// using Circle's Cross-Chain Transfer Protocol (CCTP). It is generic over:
///
/// - `SN`: Source network type (e.g., `Ethereum`, `Optimism`)
/// - `DN`: Destination network type
/// - `SP`: Source blockchain provider
/// - `DP`: Destination blockchain provider
/// - `A`: Attestation provider
/// - `C`: Clock for time operations
///
/// This design enables comprehensive testing by allowing fake implementations
/// of all external I/O operations.
///
/// # Examples
///
/// ## Production Usage
///
/// ```rust,no_run
/// # use cctp_rs::{Cctp, CctpError, UniversalReceiptAdapter};
/// # use cctp_rs::providers::{AlloyProvider, IrisAttestationProvider, TokioClock};
/// # use alloy_chains::NamedChain;
/// # use alloy_network::Ethereum;
/// # use alloy_provider::ProviderBuilder;
/// # async fn example() -> Result<(), CctpError> {
/// let eth_provider = ProviderBuilder::new().on_builtin("http://localhost:8545").await?;
/// let arb_provider = ProviderBuilder::new().on_builtin("http://localhost:8546").await?;
///
/// let bridge = Cctp::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Arbitrum)
///     .source_provider(AlloyProvider::new(eth_provider))
///     .destination_provider(AlloyProvider::new(arb_provider))
///     .attestation_provider(IrisAttestationProvider::production())
///     .clock(TokioClock::new())
///     .receipt_adapter(UniversalReceiptAdapter)
///     .recipient("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d".parse()?)
///     .build();
/// # Ok(())
/// # }
/// ```
///
/// ## Testing with Fakes
///
/// ```rust,ignore
/// let fake_blockchain = FakeBlockchainProvider::new();
/// let fake_attestation = FakeAttestationProvider::new();
/// let fake_clock = FakeClock::new();
///
/// let bridge = Cctp::builder()
///     .source_chain(NamedChain::Mainnet)
///     .destination_chain(NamedChain::Arbitrum)
///     .source_provider(fake_blockchain.clone())
///     .destination_provider(fake_blockchain)
///     .attestation_provider(fake_attestation)
///     .clock(fake_clock)
///     .recipient(Address::ZERO)
///     .build();
/// ```
#[derive(Builder, Clone, Debug)]
pub struct Cctp<SN, DN, SP, DP, A, C, RA>
where
    SN: Network,
    DN: Network,
    SP: BlockchainProvider<SN>,
    DP: BlockchainProvider<DN>,
    A: AttestationProvider,
    C: Clock,
    RA: ReceiptAdapter<SN>,
{
    source_provider: SP,
    destination_provider: DP,
    attestation_provider: A,
    clock: C,
    receipt_adapter: RA,
    source_chain: NamedChain,
    destination_chain: NamedChain,
    recipient: Address,
    #[builder(skip)]
    _source_network: std::marker::PhantomData<SN>,
    #[builder(skip)]
    _destination_network: std::marker::PhantomData<DN>,
}

impl<SN, DN, SP, DP, A, C, RA> Cctp<SN, DN, SP, DP, A, C, RA>
where
    SN: Network,
    DN: Network,
    SP: BlockchainProvider<SN>,
    DP: BlockchainProvider<DN>,
    A: AttestationProvider,
    C: Clock,
    RA: ReceiptAdapter<SN>,
{
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
    pub fn source_provider(&self) -> &SP {
        &self.source_provider
    }

    /// Returns the destination provider
    pub fn destination_provider(&self) -> &DP {
        &self.destination_provider
    }

    /// Returns the attestation provider
    pub fn attestation_provider(&self) -> &A {
        &self.attestation_provider
    }

    /// Returns the clock
    pub fn clock(&self) -> &C {
        &self.clock
    }

    /// Returns the CCTP token messenger contract address
    ///
    /// This is the address of the contract that handles the deposit and burn of USDC
    pub fn token_messenger_contract(&self) -> Result<Address> {
        self.source_chain.token_messenger_address()
    }

    /// Returns the CCTP message transmitter contract address
    ///
    /// This is the address of the contract that handles the receipt of messages
    pub fn message_transmitter_contract(&self) -> Result<Address> {
        self.destination_chain.message_transmitter_address()
    }

    /// Returns the recipient address
    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    /// Constructs the Iris API URL for a given message hash
    ///
    /// # Arguments
    ///
    /// * `message_hash` - The message hash to query
    ///
    /// # Returns
    ///
    /// The full URL to query the attestation status
    pub fn iris_api_url(&self, message_hash: &FixedBytes<32>) -> String {
        format!(
            "{}/attestations/{}",
            self.api_url(),
            hex::encode(message_hash)
        )
    }

    /// Constructs the Iris API URL for a given message hash (v1 endpoint)
    ///
    /// See <https://developers.circle.com/stablecoins/cctp-apis>
    pub fn create_url(&self, message_hash: FixedBytes<32>) -> String {
        let base_url = self.api_url();
        format!("{base_url}/v1/attestations/{message_hash}")
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
    #[instrument(skip(self), level = Level::INFO, fields(
        source_chain = %self.source_chain,
        destination_chain = %self.destination_chain,
    ))]
    pub async fn get_message_sent_event(
        &self,
        tx_hash: TxHash,
    ) -> Result<(Vec<u8>, FixedBytes<32>)> {
        let tx_receipt = self
            .source_provider
            .get_transaction_receipt(tx_hash)
            .await?;

        if let Some(tx_receipt) = tx_receipt {
            let message_sent_log = self.find_message_sent_log(&tx_receipt, tx_hash)?;

            // Decode the log data using the generated event bindings
            let decoded = MessageSent::abi_decode_data(&message_sent_log.data().data)?;

            let message_sent_event = decoded.0.to_vec();
            let message_hash = keccak256(&message_sent_event);

            info!(
                tx_hash = %tx_hash,
                message_hash = %hex::encode(message_hash),
                message_length_bytes = message_sent_event.len(),
                event = "message_sent_event_extracted"
            );

            Ok((message_sent_event, message_hash))
        } else {
            error!(
                tx_hash = %tx_hash,
                source_chain = %self.source_chain,
                event = "transaction_not_found"
            );
            Err(CctpError::TransactionFailed {
                reason: "Transaction not found".to_string(),
            })
        }
    }

    /// Finds the MessageSent log in a transaction receipt using the receipt adapter
    fn find_message_sent_log(
        &self,
        tx_receipt: &SN::ReceiptResponse,
        tx_hash: TxHash,
    ) -> Result<Log> {
        // Calculate the event topic by hashing the event signature
        let message_sent_topic = keccak256(b"MessageSent(bytes)");

        // Use the receipt adapter to get logs in a network-agnostic way
        let logs = self.receipt_adapter.logs(tx_receipt);

        logs.iter()
            .find(|log| {
                log.topics()
                    .first()
                    .is_some_and(|topic| topic.as_slice() == message_sent_topic)
            })
            .cloned()
            .ok_or_else(|| {
                error!(
                    tx_hash = %tx_hash,
                    source_chain = %self.source_chain,
                    available_logs = logs.len(),
                    event = "message_sent_event_not_found"
                );
                CctpError::TransactionFailed {
                    reason: "MessageSent event not found".to_string(),
                }
            })
    }

    /// Gets the attestation for a message hash from the CCTP API with retry logic
    ///
    /// This method polls the attestation provider until the attestation is complete,
    /// failed, or the maximum number of attempts is reached.
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The attestation fails
    /// - The maximum number of attempts is reached (timeout)
    /// - The attestation provider returns an error
    pub async fn get_attestation_with_retry(
        &self,
        message_hash: FixedBytes<32>,
        max_attempts: Option<u32>,
        poll_interval: Option<u64>,
    ) -> Result<AttestationBytes> {
        let max_attempts = max_attempts.unwrap_or(30);
        let poll_interval = poll_interval.unwrap_or(60);
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 5;

        info!(
            source_chain = %self.source_chain,
            destination_chain = %self.destination_chain,
            message_hash = %hex::encode(message_hash),
            max_attempts = max_attempts,
            poll_interval_secs = poll_interval,
            event = "attestation_polling_started"
        );

        for attempt in 1..=max_attempts {
            trace!(
                attempt = attempt,
                max_attempts = max_attempts,
                event = "attestation_attempt"
            );

            match self
                .attestation_provider
                .get_attestation(message_hash)
                .await
            {
                Ok(attestation) => {
                    consecutive_errors = 0;
                    match attestation.status {
                        AttestationStatus::Complete => {
                            return self.handle_complete_attestation(
                                attestation,
                                attempt,
                                message_hash,
                            );
                        }
                        AttestationStatus::Failed => {
                            return self.handle_failed_attestation(attempt, message_hash);
                        }
                        AttestationStatus::Pending | AttestationStatus::PendingConfirmations => {
                            self.handle_pending_attestation(attempt, max_attempts, poll_interval)
                                .await;
                        }
                    }
                }
                Err(e) => {
                    match e {
                        CctpError::RateLimitExceeded {
                            retry_after_seconds,
                        } => {
                            consecutive_errors = 0;
                            debug!(
                                source_chain = %self.source_chain,
                                destination_chain = %self.destination_chain,
                                retry_after_seconds = retry_after_seconds,
                                event = "rate_limit_exceeded"
                            );
                            self.clock
                                .sleep(Duration::from_secs(retry_after_seconds))
                                .await;
                            continue;
                        }
                        CctpError::AttestationNotFound => {
                            consecutive_errors = 0;
                            debug!(
                                source_chain = %self.source_chain,
                                destination_chain = %self.destination_chain,
                                attempt = attempt,
                                max_attempts = max_attempts,
                                poll_interval_secs = poll_interval,
                                event = "attestation_not_found"
                            );
                            self.clock.sleep(Duration::from_secs(poll_interval)).await;
                            continue;
                        }
                        _ => {}
                    }

                    // For other errors, increment consecutive error counter
                    consecutive_errors += 1;
                    error!(
                        source_chain = %self.source_chain,
                        destination_chain = %self.destination_chain,
                        error = %e,
                        attempt = attempt,
                        consecutive_errors = consecutive_errors,
                        event = "attestation_request_failed"
                    );

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        error!(
                            source_chain = %self.source_chain,
                            destination_chain = %self.destination_chain,
                            consecutive_errors = consecutive_errors,
                            event = "circuit_breaker_triggered"
                        );
                        return Err(CctpError::AttestationFailed {
                            reason: format!(
                                "Circuit breaker triggered after {} consecutive errors: {}",
                                consecutive_errors, e
                            ),
                        });
                    }

                    self.clock.sleep(Duration::from_secs(poll_interval)).await;
                }
            }
        }

        error!(
            source_chain = %self.source_chain,
            destination_chain = %self.destination_chain,
            message_hash = %hex::encode(message_hash),
            max_attempts = max_attempts,
            total_duration_secs = max_attempts as u64 * poll_interval,
            event = "attestation_timeout"
        );
        Err(CctpError::AttestationTimeout)
    }

    fn handle_complete_attestation(
        &self,
        attestation: AttestationResponse,
        attempt: u32,
        _message_hash: FixedBytes<32>,
    ) -> Result<AttestationBytes> {
        let attestation_bytes =
            attestation
                .attestation
                .ok_or_else(|| CctpError::AttestationFailed {
                    reason: "Attestation missing".to_string(),
                })?;

        // Remove '0x' prefix if present and decode hex
        let attestation_bytes = if let Some(stripped) = attestation_bytes.strip_prefix("0x") {
            hex::decode(stripped)
        } else {
            hex::decode(&attestation_bytes)
        }?;

        info!(
            source_chain = %self.source_chain,
            destination_chain = %self.destination_chain,
            attempt = attempt,
            attestation_length_bytes = attestation_bytes.len(),
            event = "attestation_complete"
        );
        Ok(attestation_bytes)
    }

    fn handle_failed_attestation(
        &self,
        attempt: u32,
        message_hash: FixedBytes<32>,
    ) -> Result<AttestationBytes> {
        error!(
            source_chain = %self.source_chain,
            destination_chain = %self.destination_chain,
            message_hash = %hex::encode(message_hash),
            attempt = attempt,
            event = "attestation_failed"
        );
        Err(CctpError::AttestationFailed {
            reason: "Attestation failed".to_string(),
        })
    }

    async fn handle_pending_attestation(
        &self,
        attempt: u32,
        max_attempts: u32,
        poll_interval: u64,
    ) {
        debug!(
            source_chain = %self.source_chain,
            destination_chain = %self.destination_chain,
            attempt = attempt,
            max_attempts = max_attempts,
            poll_interval_secs = poll_interval,
            event = "attestation_pending"
        );
        self.clock.sleep(Duration::from_secs(poll_interval)).await;
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
