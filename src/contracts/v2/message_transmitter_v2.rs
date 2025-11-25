//! MessageTransmitterV2 contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP v2
//! MessageTransmitter contract, which handles cross-chain message verification
//! and reception with finality-aware processing.

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};

use crate::protocol::DomainId;
use MessageTransmitterV2::MessageTransmitterV2Instance;

/// The CCTP v2 Message Transmitter contract wrapper
///
/// Handles message verification and reception with support for different
/// finality levels (Fast Transfer vs Standard).
pub struct MessageTransmitterV2Contract<P: Provider<Ethereum>> {
    instance: MessageTransmitterV2Instance<P>,
}

impl<P: Provider<Ethereum>> MessageTransmitterV2Contract<P> {
    /// Create a new MessageTransmitterV2Contract
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "message_transmitter_v2_contract_initialized"
        );
        Self {
            instance: MessageTransmitterV2Instance::<P>::new(address, provider),
        }
    }

    /// Create transaction request for receiving a cross-chain message with attestation
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes from the source chain
    /// * `attestation` - Circle's attestation signature for the message
    /// * `from_address` - Address that will submit the transaction
    ///
    /// # Finality Handling
    ///
    /// v2 contracts handle different finality levels:
    /// - Fast Transfer messages (threshold 1000) trigger `handleReceiveUnfinalizedMessage`
    /// - Standard messages (threshold 2000) trigger `handleReceiveFinalizedMessage`
    ///
    /// The receiving contract must implement the appropriate handler interface.
    pub fn receive_message_transaction(
        &self,
        message: Bytes,
        attestation: Bytes,
        from_address: Address,
    ) -> TransactionRequest {
        info!(
            message_len = message.len(),
            attestation_len = attestation.len(),
            from_address = %from_address,
            contract_address = %self.instance.address(),
            version = "v2",
            event = "receive_message_v2_transaction_created"
        );

        self.instance
            .receiveMessage(message, attestation)
            .from(from_address)
            .into_transaction_request()
    }

    /// Create transaction request for sending a generic cross-chain message
    ///
    /// v2 adds support for sending arbitrary messages, not just token burns.
    ///
    /// # Arguments
    ///
    /// * `from_address` - Address initiating the message send
    /// * `destination_domain` - CCTP domain ID for destination chain
    /// * `recipient` - Recipient address on destination chain
    /// * `message_body` - Arbitrary message data
    /// * `destination_caller` - Optional authorized caller on destination (0x0 = anyone)
    /// * `min_finality_threshold` - 1000 (fast) or 2000 (standard)
    pub fn send_message_transaction(
        &self,
        from_address: Address,
        destination_domain: DomainId,
        recipient: Address,
        message_body: Bytes,
        destination_caller: Address,
        min_finality_threshold: u32,
    ) -> TransactionRequest {
        info!(
            from_address = %from_address,
            destination_domain = %destination_domain,
            recipient = %recipient,
            message_len = message_body.len(),
            destination_caller = %destination_caller,
            finality_threshold = min_finality_threshold,
            contract_address = %self.instance.address(),
            version = "v2",
            event = "send_message_v2_transaction_created"
        );

        self.instance
            .sendMessage(
                destination_domain.as_u32(),
                recipient.into_word(),
                destination_caller.into_word(),
                min_finality_threshold,
                message_body,
            )
            .from(from_address)
            .into_transaction_request()
    }

    /// Check if a message has been received (anti-replay protection)
    ///
    /// Queries the `usedNonces` mapping to determine if a message has already
    /// been processed. A non-zero value indicates the message was received.
    ///
    /// This is useful for checking replay protection before attempting to
    /// receive a message on the destination chain.
    pub async fn is_message_received(
        &self,
        message_hash: [u8; 32],
    ) -> Result<bool, alloy_contract::Error> {
        let nonce_status = self.instance.usedNonces(message_hash.into()).call().await?;

        debug!(
            message_hash = ?message_hash,
            nonce_status = %nonce_status,
            is_received = !nonce_status.is_zero(),
            event = "is_message_received_checked"
        );

        // NONCE_USED constant is non-zero, so any non-zero value means received
        Ok(!nonce_status.is_zero())
    }

    /// Returns the contract address
    pub fn address(&self) -> Address {
        *self.instance.address()
    }
}

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MessageTransmitterV2,
    "abis/v2/message_transmitter_v2.json"
);
