// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! MessageTransmitter contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP v1
//! MessageTransmitter contract, which handles cross-chain message verification
//! and processing.

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};

use MessageTransmitter::MessageTransmitterInstance;

/// The CCTP v1 Message Transmitter contract wrapper
///
/// Handles message verification and reception for cross-chain transfers.
pub struct MessageTransmitterContract<P: Provider<Ethereum>> {
    instance: MessageTransmitterInstance<P>,
}

impl<P: Provider<Ethereum>> MessageTransmitterContract<P> {
    /// Create a new MessageTransmitterContract
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "message_transmitter_contract_initialized"
        );
        Self {
            instance: MessageTransmitterInstance::<P>::new(address, provider),
        }
    }

    /// Create transaction request for receiving a cross-chain message with attestation
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes from the source chain
    /// * `attestation` - Circle's attestation signature for the message
    /// * `from_address` - Address that will submit the transaction
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
            version = "v1",
            event = "receive_message_transaction_created"
        );

        self.instance
            .receiveMessage(message, attestation)
            .from(from_address)
            .into_transaction_request()
    }

    /// Check if a message nonce has been used (anti-replay protection)
    ///
    /// Queries the `usedNonces` mapping to determine if a nonce has already
    /// been processed.
    ///
    /// # Arguments
    ///
    /// * `nonce_hash` - The hash of the nonce to check (keccak256 of source domain + nonce)
    pub async fn is_nonce_used(&self, nonce_hash: [u8; 32]) -> Result<bool, alloy_contract::Error> {
        let nonce_status = self.instance.usedNonces(nonce_hash.into()).call().await?;

        debug!(
            nonce_hash = ?nonce_hash,
            nonce_status = %nonce_status,
            is_used = nonce_status == 1,
            event = "is_nonce_used_checked"
        );

        // 1 = used, 0 = not used
        Ok(nonce_status == 1)
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
    MessageTransmitter,
    "abis/v1_message_transmitter.json"
);
