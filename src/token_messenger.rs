use std::marker::PhantomData;

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use TokenMessenger::{depositForBurnCall, TokenMessengerInstance};

// https://developers.circle.com/cctp/evm-smart-contracts
pub const TOKEN_MESSENGER_CONTRACT: &str = "0x28b5a0e9C621a5BadaA536219b3a228C8168cf5d";
pub const TOKEN_MESSENGER_CONTRACT_TESTNET: &str = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA";
pub const ARBITRUM_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT;
pub const ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const AVALANCHE_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT;
pub const BASE_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT;
pub const BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const ETHEREUM_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT;
pub const ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT_TESTNET;
pub const OPTIMISM_TOKEN_MESSENGER_ADDRESS: &str = TOKEN_MESSENGER_CONTRACT;
pub const POLYGON_CCTP_TOKEN_MESSENGER: &str = TOKEN_MESSENGER_CONTRACT;
pub const UNICHAIN_CCTP_TOKEN_MESSENGER: &str = TOKEN_MESSENGER_CONTRACT;

/// The CCTP v1 Token Messenger contract.
pub struct TokenMessengerContract<P: Provider<Ethereum>> {
    pub instance: TokenMessengerInstance<P>,
}

impl<P: Provider<Ethereum>> TokenMessengerContract<P> {
    /// Create a new TokenMessengerContract.
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: TokenMessengerInstance::new(address, provider),
        }
    }

    /// Create the call builder for the `depositForBurn` function.
    ///
    /// Most users will want to use the `deposit_for_burn_transaction` function instead.
    /// destination_caller: Address as bytes32 which can call receiveMessage on destination domain. If set to bytes32(0), any address can call receiveMessage
    /// max_fee: Max fee paid for fast burn, specified in units of burnToken
    /// min_finality_threshold: Minimum finality threshold at which burn will be attested
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn_call_builder(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
        destination_caller: Option<Address>,
        max_fee: Option<U256>,
        min_finality_threshold: Option<u32>,
    ) -> CallBuilder<&P, PhantomData<depositForBurnCall>> {
        self.instance
            .depositForBurn(
                amount,
                destination_domain,
                recipient.into_word(),
                token_address,
                destination_caller.unwrap_or(Address::ZERO).into_word(),
                max_fee.unwrap_or(U256::from(3)),
                min_finality_threshold.unwrap_or(0),
            )
            .from(from_address)
    }

    /// Create the transaction request for the `depositForBurn` function.
    ///
    /// Most users will want to use this function instead of the `deposit_for_burn_call_builder` function.
    /// destination_caller: Address as bytes32 which can call receiveMessage on destination domain. If set to bytes32(0), any address can call receiveMessage
    /// max_fee: Max fee paid for fast burn, specified in units of burnToken
    /// min_finality_threshold: Minimum finality threshold at which burn will be attested
    #[allow(clippy::too_many_arguments)]
    pub fn deposit_for_burn_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
        destination_caller: Option<Address>,
        max_fee: Option<U256>,
        min_finality_threshold: Option<u32>,
    ) -> TransactionRequest {
        self.deposit_for_burn_call_builder(
            from_address,
            recipient,
            destination_domain,
            token_address,
            amount,
            destination_caller,
            max_fee,
            min_finality_threshold,
        )
        .into_transaction_request()
    }
}

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    TokenMessenger,
    "abis/v2_token_messenger.json"
);
