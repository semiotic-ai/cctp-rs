use alloy_chains::NamedChain;
use alloy_primitives::Address;

use crate::error::{CctpError, Result};

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
    POLYGON_CCTP_V1_TOKEN_MESSENGER, POLYGON_DOMAIN_ID, UNICHAIN_CCTP_V1_MESSAGE_TRANSMITTER,
    UNICHAIN_CCTP_V1_TOKEN_MESSENGER, UNICHAIN_DOMAIN_ID,
};

/// Trait for chains that support CCTP bridging
pub trait CctpV1 {
    /// The average time to confirmation of the chain, according to the CCTP docs: <https://developers.circle.com/stablecoins/required-block-confirmations>
    fn confirmation_average_time_seconds(&self) -> Result<u64>;
    /// The domain ID of the chain - used to identify the chain when bridging: <https://developers.circle.com/stablecoins/evm-smart-contracts>
    fn cctp_domain_id(&self) -> Result<u32>;
    /// The address of the `TokenMessenger` contract on the chain
    fn token_messenger_address(&self) -> Result<Address>;
    /// The address of the `MessageTransmitter` contract on the chain
    fn message_transmitter_address(&self) -> Result<Address>;

    /// Check if the chain is supported for CCTP
    fn is_supported(&self) -> bool;
}

impl CctpV1 for NamedChain {
    fn confirmation_average_time_seconds(&self) -> Result<u64> {
        use NamedChain::*;

        match self {
            Mainnet | Arbitrum | Base | Optimism | Unichain => Ok(19 * 60),
            Avalanche => Ok(20),
            Polygon => Ok(8 * 60),
            // Testnets
            Sepolia => Ok(60),
            ArbitrumSepolia | AvalancheFuji | BaseSepolia | OptimismSepolia | PolygonAmoy => Ok(20),
            _ => Err(CctpError::ChainNotSupported {
                chain: self.to_string(),
            }),
        }
    }

    fn cctp_domain_id(&self) -> Result<u32> {
        use NamedChain::*;

        match self {
            Arbitrum | ArbitrumSepolia => Ok(ARBITRUM_DOMAIN_ID),
            Avalanche => Ok(AVALANCHE_DOMAIN_ID),
            Base | BaseSepolia => Ok(BASE_DOMAIN_ID),
            Mainnet | Sepolia => Ok(ETHEREUM_DOMAIN_ID),
            Optimism => Ok(OPTIMISM_DOMAIN_ID),
            Polygon => Ok(POLYGON_DOMAIN_ID),
            Unichain => Ok(UNICHAIN_DOMAIN_ID),
            _ => Err(CctpError::ChainNotSupported {
                chain: self.to_string(),
            }),
        }
    }

    fn token_messenger_address(&self) -> Result<Address> {
        use NamedChain::*;

        let address_str = match self {
            Arbitrum => ARBITRUM_TOKEN_MESSENGER_ADDRESS,
            ArbitrumSepolia => ARBITRUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Avalanche => AVALANCHE_TOKEN_MESSENGER_ADDRESS,
            Base => BASE_TOKEN_MESSENGER_ADDRESS,
            BaseSepolia => BASE_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Sepolia => ETHEREUM_SEPOLIA_TOKEN_MESSENGER_ADDRESS,
            Mainnet => ETHEREUM_TOKEN_MESSENGER_ADDRESS,
            Optimism => OPTIMISM_TOKEN_MESSENGER_ADDRESS,
            Polygon => POLYGON_CCTP_V1_TOKEN_MESSENGER,
            Unichain => UNICHAIN_CCTP_V1_TOKEN_MESSENGER,
            _ => {
                return Err(CctpError::ChainNotSupported {
                    chain: self.to_string(),
                })
            }
        };

        address_str.parse().map_err(|e| CctpError::InvalidAddress {
            address: address_str.to_string(),
            source: e,
        })
    }

    fn message_transmitter_address(&self) -> Result<Address> {
        use NamedChain::*;

        let address_str = match self {
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
            Unichain => UNICHAIN_CCTP_V1_MESSAGE_TRANSMITTER,
            _ => {
                return Err(CctpError::ChainNotSupported {
                    chain: self.to_string(),
                })
            }
        };

        address_str.parse().map_err(|e| CctpError::InvalidAddress {
            address: address_str.to_string(),
            source: e,
        })
    }

    fn is_supported(&self) -> bool {
        use NamedChain::*;

        matches!(
            self,
            Mainnet
                | Arbitrum
                | Base
                | Optimism
                | Unichain
                | Avalanche
                | Polygon
                | Sepolia
                | ArbitrumSepolia
                | AvalancheFuji
                | BaseSepolia
                | OptimismSepolia
                | PolygonAmoy
        )
    }
}
