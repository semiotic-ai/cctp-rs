use alloy_chains::NamedChain;
use alloy_primitives::Address;
use anyhow::bail;

use crate::{
    ARBITRUM_DOMAIN_ID, ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS,
    ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS, ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
    ARBITRUM_TOKEN_MESSENGER_ADDRESS, AVALANCHE_DOMAIN_ID, AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS,
    AVALANCHE_TOKEN_MESSENGER_ADDRESS, BASE_DOMAIN_ID, BASE_MESSAGE_TRANSMITTER_ADDRESS,
    BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS, BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
    BASE_TOKEN_MESSENGER_ADDRESS, ETHEREUM_DOMAIN_ID, ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS,
    ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS, ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
    ETHEREUM_TOKEN_MESSENGER_ADDRESS, OPTIMISM_DOMAIN_ID, OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS,
    OPTIMISM_TOKEN_MESSENGER_ADDRESS, POLYGON_CCTP_V1_MESSAGE_TRANSMITTER,
    POLYGON_CCTP_V1_TOKEN_MESSENGER, POLYGON_DOMAIN_ID, UNICHAIN_DOMAIN_ID,
};

/// Trait for chains that support CCTP bridging
pub trait CctpV1 {
    /// The average time to confirmation of the chain, according to the CCTP docs: <https://developers.circle.com/stablecoins/required-block-confirmations>
    fn confirmation_average_time_seconds(&self) -> anyhow::Result<u64>;
    /// The domain ID of the chain - used to identify the chain when bridging: <https://developers.circle.com/stablecoins/evm-smart-contracts>
    fn cctp_domain_id(&self) -> u32;
    /// The address of the `TokenMessenger` contract on the chain
    fn token_messenger_address(&self) -> Address;
    /// The address of the `MessageTransmitter` contract on the chain
    fn message_transmitter_address(&self) -> Address;
}

impl CctpV1 for NamedChain {
    fn confirmation_average_time_seconds(&self) -> anyhow::Result<u64> {
        use NamedChain::*;

        match self {
            Mainnet | Arbitrum | Base | Optimism | Unichain => Ok(19 * 60),
            Avalanche => Ok(20),
            Polygon => Ok(8 * 60),
            // Testnets
            Sepolia => Ok(60),
            ArbitrumSepolia | AvalancheFuji | BaseSepolia | OptimismSepolia | PolygonAmoy => Ok(20),
            _ => bail!("Unsupported chain for CCTP v1: {self}"),
        }
    }

    fn cctp_domain_id(&self) -> u32 {
        use NamedChain::*;

        match self {
            Arbitrum | ArbitrumSepolia => ARBITRUM_DOMAIN_ID,
            Avalanche => AVALANCHE_DOMAIN_ID,
            Base | BaseSepolia => BASE_DOMAIN_ID,
            Mainnet | Sepolia => ETHEREUM_DOMAIN_ID,
            Optimism => OPTIMISM_DOMAIN_ID,
            Polygon => POLYGON_DOMAIN_ID,
            Unichain => UNICHAIN_DOMAIN_ID,
            _ => panic!("Can't get domain ID for unsupported chain: {self}"),
        }
    }

    fn token_messenger_address(&self) -> Address {
        use NamedChain::*;

        match self {
            Arbitrum => ARBITRUM_TOKEN_MESSENGER_ADDRESS,
            ArbitrumSepolia => ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Avalanche => AVALANCHE_TOKEN_MESSENGER_ADDRESS,
            Base => BASE_TOKEN_MESSENGER_ADDRESS,
            BaseSepolia => BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Sepolia => ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Mainnet => ETHEREUM_TOKEN_MESSENGER_ADDRESS,
            Optimism => OPTIMISM_TOKEN_MESSENGER_ADDRESS,
            Polygon => POLYGON_CCTP_V1_TOKEN_MESSENGER,
            _ => panic!("Can't get token messenger address for unsupported chain: {self}"),
        }
        .parse()
        .unwrap()
    }

    fn message_transmitter_address(&self) -> Address {
        use NamedChain::*;

        match self {
            Arbitrum => ARBITRUM_MESSAGE_TRANSMITTER_ADDRESS,
            Avalanche => AVALANCHE_MESSAGE_TRANSMITTER_ADDRESS,
            Base => BASE_MESSAGE_TRANSMITTER_ADDRESS,
            Mainnet => ETHEREUM_MESSAGE_TRANSMITTER_ADDRESS,
            Optimism => OPTIMISM_MESSAGE_TRANSMITTER_ADDRESS,
            Polygon => POLYGON_CCTP_V1_MESSAGE_TRANSMITTER,
            // Testnets
            ArbitrumSepolia => ARBITRUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            BaseSepolia => BASE_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            Sepolia => ETHEREUM_SEPOLIA_MESSAGE_TRANSMITTER_ADDRESS,
            _ => panic!("Can't get message transmitter address for unsupported chain: {self}"),
        }
        .parse()
        .unwrap()
    }
}
