//! TokenMessenger contract bindings and wrapper
//!
//! This module contains the Alloy-generated contract bindings for the CCTP
//! TokenMessenger contract, which manages USDC burn and mint operations for
//! cross-chain transfers.

use std::marker::PhantomData;

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};
use TokenMessenger::{depositForBurnCall, TokenMessengerInstance};

use crate::spans;

/// The CCTP v1 Token Messenger contract wrapper
#[allow(dead_code)]
pub struct TokenMessengerContract<P: Provider<Ethereum>> {
    instance: TokenMessengerInstance<P>,
}

impl<P: Provider<Ethereum>> TokenMessengerContract<P> {
    /// Create a new TokenMessengerContract.
    #[allow(dead_code)]
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "token_messenger_contract_initialized"
        );
        Self {
            instance: TokenMessengerInstance::new(address, provider),
        }
    }

    /// Create the call builder for the `depositForBurn` function.
    ///
    /// Most users will want to use the `deposit_for_burn_transaction` function instead.
    #[allow(dead_code)]
    pub fn deposit_for_burn_call_builder(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
    ) -> CallBuilder<&P, PhantomData<depositForBurnCall>> {
        self.instance
            .depositForBurn(
                amount,
                destination_domain,
                recipient.into_word(),
                token_address,
            )
            .from(from_address)
    }

    /// Create the transaction request for the `depositForBurn` function.
    ///
    /// Most users will want to use this function instead of the `deposit_for_burn_call_builder` function.
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
            event = "deposit_for_burn_transaction_created"
        );

        self.deposit_for_burn_call_builder(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
        )
        .into_transaction_request()
    }
}

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    TokenMessenger,
    "abis/v1_token_messenger.json"
);
