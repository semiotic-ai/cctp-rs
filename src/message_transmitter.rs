use alloy_primitives::{address, Address};
use alloy_sol_types::sol;

/// <https://arbiscan.io/address/0xC30362313FBBA5cf9163F0bb16a0e01f01a896ca>
pub const ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("c30362313fbba5cf9163f0bb16a0e01f01a896ca");

/// <https://sepolia.arbiscan.io/address/0xacf1ceef35caac005e15888ddb8a3515c41b4872>
pub const ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("acf1ceef35caac005e15888ddb8a3515c41b4872");

/// <https://snowtrace.io/address/0x8186359af5f57fbb40c6b14a588d2a59c0c29880>
pub const AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("8186359af5f57fbb40c6b14a588d2a59c0c29880");

/// <https://basescan.org/address/0xAD09780d193884d503182aD4588450C416D6F9D4>
pub const BASE_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("ad09780d193884d503182ad4588450c416d6f9d4");

/// <https://base-sepolia.blockscout.com/address/0x7865fAfC2db2093669d92c0F33AeEF291086BEFD>
pub const BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("7865fAfC2db2093669d92c0F33AeEF291086BEFD");

/// <https://etherscan.io/address/0x0a992d191DEeC32aFe36203Ad87D7d289a738F81>
pub const ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("0a992d191DEeC32aFe36203Ad87D7d289a738F81");

/// <https://sepolia.etherscan.io/address/0x7865fAfC2db2093669d92c0F33AeEF291086BEFD>
pub const ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("7865fAfC2db2093669d92c0F33AeEF291086BEFD");

/// <https://optimistic.etherscan.io/address/0x4D41f22c5a0e5c74090899E5a8Fb597a8842b3e8>
pub const OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS: Address =
    address!("4D41f22c5a0e5c74090899E5a8Fb597a8842b3e8");

/// <https://polygonscan.com/address/0x9daF8c91AEFAE50b9c0E69629D3F6Ca40cA3B3FE>
pub const POLYGON_CCTP_V1_MESSAGE_TRANSMITTER: Address =
    address!("F3be9355363857F3e001be68856A2f96b4C39Ba9");

/// <https://uniscan.xyz/address/0x353bE9E2E38AB1D19104534e4edC21c643Df86f4>
pub const UNICHAIN_CCTP_V1_MESSAGE_TRANSMITTER: Address =
    address!("353bE9E2E38AB1D19104534e4edC21c643Df86f4");

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MessageTransmitter,
    "abis/v1_message_transmitter.json"
);
