//! MessageTransmitterV2 contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP v2
//! MessageTransmitter contract, which handles cross-chain message verification
//! and reception with finality-aware processing.

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes};
use alloy_provider::Provider;
use alloy_sol_types::sol;
use tracing::debug;

use MessageTransmitterV2::MessageTransmitterV2Instance;

/// The CCTP v2 Message Transmitter contract wrapper
///
/// Handles message verification and reception with support for different
/// finality levels (Fast Transfer vs Standard).
#[allow(dead_code)]
pub struct MessageTransmitterV2Contract<P: Provider<Ethereum>> {
    instance: MessageTransmitterV2Instance<P>,
}

impl<P: Provider<Ethereum>> MessageTransmitterV2Contract<P> {
    /// Create a new MessageTransmitterV2Contract
    #[allow(dead_code)]
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "message_transmitter_v2_contract_initialized"
        );
        Self {
            instance: MessageTransmitterV2Instance::<P>::new(address, provider),
        }
    }

    /// Receive a cross-chain message with attestation
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes from the source chain
    /// * `attestation` - Circle's attestation signature for the message
    ///
    /// # Finality Handling
    ///
    /// v2 contracts handle different finality levels:
    /// - Fast Transfer messages (threshold 1000) trigger `handleReceiveUnfinalizedMessage`
    /// - Standard messages (threshold 2000) trigger `handleReceiveFinalizedMessage`
    ///
    /// The receiving contract must implement the appropriate handler interface.
    #[allow(dead_code)]
    pub async fn receive_message(
        &self,
        message: Bytes,
        attestation: Bytes,
    ) -> Result<(), alloy_contract::Error> {
        // TODO: Implement once we have the actual v2 ABI
        // This will call the receiveMessage function on the contract
        debug!(
            message_len = message.len(),
            attestation_len = attestation.len(),
            event = "receive_message_v2_called"
        );
        Ok(())
    }

    /// Send a generic cross-chain message
    ///
    /// v2 adds support for sending arbitrary messages, not just token burns.
    ///
    /// # Arguments
    ///
    /// * `destination_domain` - CCTP domain ID for destination chain
    /// * `recipient` - Recipient address on destination chain
    /// * `message_body` - Arbitrary message data
    /// * `destination_caller` - Optional authorized caller on destination (0x0 = anyone)
    #[allow(dead_code)]
    pub async fn send_message(
        &self,
        destination_domain: u32,
        recipient: Address,
        message_body: Bytes,
        destination_caller: Address,
    ) -> Result<(), alloy_contract::Error> {
        // TODO: Implement once we have the actual v2 ABI
        debug!(
            destination_domain = destination_domain,
            recipient = %recipient,
            message_len = message_body.len(),
            destination_caller = %destination_caller,
            event = "send_message_v2_called"
        );
        Ok(())
    }

    /// Query the finality threshold executed for a specific message
    ///
    /// Returns the actual finality level at which the message was attested.
    #[allow(dead_code)]
    pub async fn get_finality_threshold_executed(
        &self,
        message_hash: [u8; 32],
    ) -> Result<u32, alloy_contract::Error> {
        // TODO: Implement once we have the actual v2 ABI
        debug!(
            message_hash = ?message_hash,
            event = "get_finality_threshold_executed_called"
        );
        Ok(2000) // Default to standard finality
    }

    /// Check if a message has been received (anti-replay protection)
    #[allow(dead_code)]
    pub async fn is_message_received(
        &self,
        message_hash: [u8; 32],
    ) -> Result<bool, alloy_contract::Error> {
        // TODO: Implement once we have the actual v2 ABI
        debug!(
            message_hash = ?message_hash,
            event = "is_message_received_called"
        );
        Ok(false)
    }

    /// Returns the contract address
    #[allow(dead_code)]
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
