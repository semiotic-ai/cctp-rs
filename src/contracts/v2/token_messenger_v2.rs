// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! TokenMessengerV2 contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP v2
//! TokenMessenger contract, which manages USDC burn and mint operations with
//! Fast Transfer and hooks support.

#![allow(dead_code)] // Public API methods not used internally

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};

use crate::protocol::DomainId;
use crate::spans;
use TokenMessengerV2::TokenMessengerV2Instance;

/// The CCTP v2 Token Messenger contract wrapper
///
/// Supports v2 features including Fast Transfer (with fees) and programmable hooks.
#[allow(dead_code)]
pub struct TokenMessengerV2Contract<P: Provider<Ethereum>> {
    instance: TokenMessengerV2Instance<P>,
}

impl<P: Provider<Ethereum>> TokenMessengerV2Contract<P> {
    /// Create a new TokenMessengerV2Contract
    #[allow(dead_code)]
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "token_messenger_v2_contract_initialized"
        );
        Self {
            instance: TokenMessengerV2Instance::new(address, provider),
        }
    }

    /// Create the transaction request for the `depositForBurn` function (v2 standard transfer)
    ///
    /// For standard transfers without fast transfer or hooks.
    ///
    /// # Arguments
    ///
    /// * `from_address` - Address initiating the burn
    /// * `recipient` - Recipient address on destination chain
    /// * `destination_domain` - CCTP domain ID for destination
    /// * `token_address` - USDC token contract address
    /// * `amount` - Amount to burn
    /// * `max_fee` - Maximum fee for fast transfer (0 for standard)
    /// * `min_finality_threshold` - 1000 (fast) or 2000 (standard)
    /// * `destination_caller` - Authorized caller on destination (0x0 = anyone)
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn deposit_for_burn_internal(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: DomainId,
        token_address: Address,
        amount: U256,
        max_fee: U256,
        min_finality_threshold: u32,
        destination_caller: Address,
    ) -> TransactionRequest {
        self.instance
            .depositForBurn(
                amount,
                destination_domain.as_u32(),
                recipient.into_word(),
                token_address,
                destination_caller.into_word(),
                max_fee,
                min_finality_threshold,
            )
            .from(from_address)
            .into_transaction_request()
    }

    /// Create the transaction request for the `depositForBurn` function (v2 standard)
    ///
    /// Standard transfer with 2000 (finalized) threshold and no fees.
    #[allow(dead_code)]
    pub fn deposit_for_burn_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: DomainId,
        token_address: Address,
        amount: U256,
    ) -> TransactionRequest {
        let span = spans::deposit_for_burn(
            &from_address,
            &recipient,
            destination_domain.as_u32(),
            &token_address,
            &amount,
        );
        let _guard = span.enter();

        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = %destination_domain,
            token_address = %token_address,
            amount = %amount,
            contract_address = %self.instance.address(),
            version = "v2",
            finality_threshold = 2000,
            event = "deposit_for_burn_v2_transaction_created"
        );

        self.deposit_for_burn_internal(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
            U256::ZERO,    // max_fee: 0 for standard transfers
            2000,          // min_finality_threshold: 2000 = finalized
            Address::ZERO, // destination_caller: 0x0 = anyone
        )
    }

    /// Create transaction for depositForBurn with Fast Transfer enabled
    ///
    /// # Arguments
    ///
    /// * `from_address` - Sender address
    /// * `recipient` - Recipient address on destination chain
    /// * `destination_domain` - CCTP domain ID for destination chain
    /// * `token_address` - USDC token contract address
    /// * `amount` - Amount to transfer
    /// * `max_fee` - Maximum fee willing to pay for fast transfer
    ///
    /// # Fast Transfer
    ///
    /// When max_fee >= minimum fast transfer fee for the chain, the transfer
    /// will be attested at the "confirmed" finality level (~30 seconds) instead
    /// of "finalized" level (~15 minutes). Uses finality threshold 1000.
    #[allow(dead_code)]
    pub fn deposit_for_burn_fast_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: DomainId,
        token_address: Address,
        amount: U256,
        max_fee: U256,
    ) -> TransactionRequest {
        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = %destination_domain,
            token_address = %token_address,
            amount = %amount,
            max_fee = %max_fee,
            contract_address = %self.instance.address(),
            version = "v2",
            transfer_type = "fast",
            finality_threshold = 1000,
            event = "deposit_for_burn_fast_transaction_created"
        );

        self.deposit_for_burn_internal(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
            max_fee,       // max_fee: provided by caller
            1000,          // min_finality_threshold: 1000 = confirmed (fast)
            Address::ZERO, // destination_caller: 0x0 = anyone
        )
    }

    /// Create transaction for depositForBurn with hooks
    ///
    /// # Arguments
    ///
    /// * `from_address` - Sender address
    /// * `recipient` - Recipient address on destination chain
    /// * `destination_domain` - CCTP domain ID for destination chain
    /// * `token_address` - USDC token contract address
    /// * `amount` - Amount to transfer
    /// * `hook_data` - Arbitrary bytes to pass to destination chain for programmable actions
    ///
    /// # Hooks
    ///
    /// Hook data is opaque to CCTP but can be used by integrators to trigger
    /// actions on the destination chain (e.g., swap, lend, stake).
    #[allow(dead_code)]
    pub fn deposit_for_burn_with_hooks_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: DomainId,
        token_address: Address,
        amount: U256,
        hook_data: Bytes,
    ) -> TransactionRequest {
        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = %destination_domain,
            token_address = %token_address,
            amount = %amount,
            hook_data_len = hook_data.len(),
            contract_address = %self.instance.address(),
            version = "v2",
            has_hooks = true,
            finality_threshold = 2000,
            event = "deposit_for_burn_hooks_transaction_created"
        );

        self.instance
            .depositForBurnWithHook(
                amount,
                destination_domain.as_u32(),
                recipient.into_word(),
                token_address,
                Address::ZERO.into_word(), // destination_caller: 0x0 = anyone
                U256::ZERO,                // max_fee: 0 for standard transfers
                2000,                      // min_finality_threshold: 2000 = finalized
                hook_data,
            )
            .from(from_address)
            .into_transaction_request()
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
    TokenMessengerV2,
    "abis/v2/token_messenger_v2.json"
);
