//! TokenMessengerV2 contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP v2
//! TokenMessenger contract, which manages USDC burn and mint operations with
//! Fast Transfer and hooks support.

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};

use crate::spans;
// TODO: Uncomment once we have actual v2 ABI with these methods
// use TokenMessengerV2::{depositForBurnCall, TokenMessengerV2Instance};
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
    /// # TODO
    ///
    /// This is a placeholder implementation. Once we have the actual v2 ABI,
    /// this will call the real depositForBurn method with v2 parameters.
    #[allow(dead_code)]
    fn deposit_for_burn_internal(
        &self,
        _from_address: Address,
        _recipient: Address,
        _destination_domain: u32,
        _token_address: Address,
        _amount: U256,
    ) -> TransactionRequest {
        // TODO: Implement once we have actual v2 ABI
        // For now, return an empty transaction request
        TransactionRequest::default()
    }

    /// Create the transaction request for the `depositForBurn` function (v2 standard)
    #[allow(dead_code)]
    pub fn deposit_for_burn_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
    ) -> TransactionRequest {
        let span = spans::deposit_for_burn(
            &from_address,
            &recipient,
            destination_domain,
            &token_address,
            &amount,
        );
        let _guard = span.enter();

        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = destination_domain,
            token_address = %token_address,
            amount = %amount,
            contract_address = %self.instance.address(),
            version = "v2",
            event = "deposit_for_burn_v2_transaction_created"
        );

        self.deposit_for_burn_internal(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
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
    /// of "finalized" level (~15 minutes).
    #[allow(dead_code)]
    pub fn deposit_for_burn_fast_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
        max_fee: U256,
    ) -> TransactionRequest {
        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = destination_domain,
            token_address = %token_address,
            amount = %amount,
            max_fee = %max_fee,
            contract_address = %self.instance.address(),
            version = "v2",
            transfer_type = "fast",
            event = "deposit_for_burn_fast_transaction_created"
        );

        // TODO: Update this once we have the actual v2 ABI with fast transfer parameters
        // For now, use standard deposit_for_burn
        // The real v2 contract will have additional parameters for minFinalityThreshold and maxFee
        self.deposit_for_burn_internal(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
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
        destination_domain: u32,
        token_address: Address,
        amount: U256,
        hook_data: Bytes,
    ) -> TransactionRequest {
        info!(
            from_address = %from_address,
            recipient = %recipient,
            destination_domain = destination_domain,
            token_address = %token_address,
            amount = %amount,
            hook_data_len = hook_data.len(),
            contract_address = %self.instance.address(),
            version = "v2",
            has_hooks = true,
            event = "deposit_for_burn_hooks_transaction_created"
        );

        // TODO: Update this once we have the actual v2 ABI with hooks parameter
        // For now, use standard deposit_for_burn
        // The real v2 contract will have hookData parameter
        self.deposit_for_burn_internal(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
        )
    }

    /// Get the minimum fee required for fast transfer on this chain
    ///
    /// Only available on chains where fast transfer fee is enabled.
    /// Returns 0 on chains without fast transfer fees.
    #[allow(dead_code)]
    pub async fn get_min_fee_amount(
        &self,
        _burn_amount: U256,
    ) -> Result<U256, alloy_contract::Error> {
        // TODO: Implement once we have the actual v2 ABI
        // For now, return 0 (no fee)
        Ok(U256::ZERO)
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
    TokenMessengerV2,
    "abis/v2/token_messenger_v2.json"
);
