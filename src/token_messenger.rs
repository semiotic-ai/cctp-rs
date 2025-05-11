use std::marker::PhantomData;

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use TokenMessenger::{depositForBurnCall, TokenMessengerInstance};

/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const ARBITRUM_TOKEN_MESSENGER_ADDRESS: &str = "0x19330d10D9Cc8751218eaf51E8885D058642E08A";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str =
    "0x9f3B8679c73C2Fef8b59B4f3444d4e156fb70AA5";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const AVALANCHE_TOKEN_MESSENGER_ADDRESS: &str = "0x6b25532e1060ce10cc3b0a99e5683b91bfde6982";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const BASE_TOKEN_MESSENGER_ADDRESS: &str = "0x1682ae6375c4e4a97e4b583bc394c861a46d8962";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str = "0x9f3B8679c73C2Fef8b59B4f3444d4e156fb70AA5";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const ETHEREUM_TOKEN_MESSENGER_ADDRESS: &str = "0xbd3fa81b58ba92a82136038b25adec7066af3155";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS: &str =
    "0x9f3B8679c73C2Fef8b59B4f3444d4e156fb70AA5";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const OPTIMISM_TOKEN_MESSENGER_ADDRESS: &str = "0x2B4069517957735bE00ceE0fadAE88a26365528f";
/// <https://developers.circle.com/stablecoins/evm-smart-contracts>
pub const POLYGON_CCTP_V1_TOKEN_MESSENGER: &str = "0x9daF8c91AEFAE50b9c0E69629D3F6Ca40cA3B3FE";

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
    pub fn deposit_for_burn_transaction(
        &self,
        from_address: Address,
        recipient: Address,
        destination_domain: u32,
        token_address: Address,
        amount: U256,
    ) -> TransactionRequest {
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
