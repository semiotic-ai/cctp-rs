//! CCTP v2 chain configuration trait
//!
//! This module defines the `CctpV2` trait which provides v2-specific
//! chain capabilities including Fast Transfer support, hooks, and
//! v2 contract addresses.

use alloy_chains::NamedChain;
use alloy_primitives::Address;

use super::addresses::{
    CCTP_V2_MESSAGE_TRANSMITTER_MAINNET, CCTP_V2_MESSAGE_TRANSMITTER_TESTNET,
    CCTP_V2_TOKEN_MESSENGER_MAINNET, CCTP_V2_TOKEN_MESSENGER_TESTNET,
};
use crate::{CctpError, DomainId, Result};

/// CCTP v2 chain configuration trait
///
/// Implemented on `alloy_chains::NamedChain` to provide v2-specific
/// configuration for each supported blockchain network.
///
/// # v2 Features
///
/// - **Fast Transfer**: Chains that support fast transfer (finality threshold 1000)
/// - **Dynamic Fees**: Some chains charge fees for fast transfer (0-14 bps)
/// - **v2 Contracts**: Updated contract addresses for TokenMessengerV2 and MessageTransmitterV2
/// - **Expanded Chains**: Support for 26+ networks vs v1's 7
///
/// # Example
///
/// ```rust
/// use cctp_rs::CctpV2;
/// use alloy_chains::NamedChain;
///
/// let chain = NamedChain::Mainnet;
/// assert!(chain.supports_cctp_v2());
/// assert!(chain.supports_fast_transfer().unwrap());
/// ```
pub trait CctpV2 {
    /// Returns true if this chain supports CCTP v2
    ///
    /// All v1 chains support v2, plus 19 additional v2-only chains.
    fn supports_cctp_v2(&self) -> bool;

    /// Returns true if this chain supports Fast Transfer
    ///
    /// Fast Transfer enables ~30 second settlement times vs 13-19 minutes.
    fn supports_fast_transfer(&self) -> Result<bool>;

    /// Returns the fast transfer fee in basis points (0-14 bps)
    ///
    /// Returns `None` for chains that don't charge fast transfer fees,
    /// or if the chain doesn't support fast transfer.
    ///
    /// Fee ranges:
    /// - 0 bps: Free fast transfer (most chains)
    /// - 1-14 bps: Small fee for fast settlement
    fn fast_transfer_fee_bps(&self) -> Result<Option<u32>>;

    /// Returns the TokenMessengerV2 contract address for this chain
    ///
    /// Returns an error if the chain doesn't support CCTP v2 or if
    /// contracts haven't been deployed yet.
    fn token_messenger_v2_address(&self) -> Result<Address>;

    /// Returns the MessageTransmitterV2 contract address for this chain
    ///
    /// Returns an error if the chain doesn't support CCTP v2 or if
    /// contracts haven't been deployed yet.
    fn message_transmitter_v2_address(&self) -> Result<Address>;

    /// Returns the CCTP domain ID for this chain
    ///
    /// Note: Domain IDs are the same in v1 and v2 for chains that
    /// existed in v1. New v2-only chains have domain IDs >= 11.
    fn cctp_v2_domain_id(&self) -> Result<DomainId>;

    /// Returns the average Fast Transfer attestation time in seconds
    ///
    /// Fast Transfer uses a lower finality threshold (≤1000) to achieve
    /// rapid attestations at the cost of a small fee on some chains.
    ///
    /// Typical times:
    /// - Ethereum: ~20 seconds (2 block confirmations)
    /// - Most L2s and alt-L1s: ~8 seconds (1 block confirmation)
    /// - High-performance chains (Sonic, Sei): ~5 seconds
    ///
    /// See: <https://developers.circle.com/stablecoins/required-block-confirmations>
    fn fast_transfer_confirmation_time_seconds(&self) -> Result<u64>;

    /// Returns the average Standard Transfer attestation time in seconds
    ///
    /// Standard Transfer waits for full chain finality before Circle's Iris
    /// service provides an attestation. This is the default behavior.
    ///
    /// Typical times:
    /// - Ethereum + L2s settling to Ethereum: 13-19 minutes (~65 ETH blocks)
    /// - Avalanche, Polygon: 5-20 seconds (native finality)
    /// - Sei, Sonic: ~5 seconds (high-performance chains)
    /// - Linea: 6-32 hours (zkEVM proof generation)
    ///
    /// See: <https://developers.circle.com/stablecoins/required-block-confirmations>
    fn standard_transfer_confirmation_time_seconds(&self) -> Result<u64>;
}

impl CctpV2 for NamedChain {
    fn supports_cctp_v2(&self) -> bool {
        matches!(
            self,
            // v1 chains (all support v2)
            Self::Mainnet
                | Self::Sepolia
                | Self::Arbitrum
                | Self::ArbitrumSepolia
                | Self::Base
                | Self::BaseSepolia
                | Self::Optimism
                | Self::OptimismSepolia
                | Self::Avalanche
                | Self::AvalancheFuji
                | Self::Polygon
                | Self::PolygonAmoy
                | Self::Unichain
                // v2-only priority chains
                | Self::Linea
                | Self::Sonic
                // TODO: Add BNB Smart Chain once available in alloy_chains
                // | Self::Bsc
                | Self::Sei
        )
    }

    fn supports_fast_transfer(&self) -> Result<bool> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // All v2 chains support fast transfer
        Ok(true)
    }

    fn fast_transfer_fee_bps(&self) -> Result<Option<u32>> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // Most chains have 0 bps fees (free fast transfer!)
        // Based on Circle's documentation, fees range from 0-14 bps
        // TODO: Update with actual fee data per chain when available
        Ok(Some(0))
    }

    fn token_messenger_v2_address(&self) -> Result<Address> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // V2 uses unified addresses across all chains within each environment
        Ok(if self.is_testnet() {
            CCTP_V2_TOKEN_MESSENGER_TESTNET
        } else {
            CCTP_V2_TOKEN_MESSENGER_MAINNET
        })
    }

    fn message_transmitter_v2_address(&self) -> Result<Address> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // V2 uses unified addresses across all chains within each environment
        Ok(if self.is_testnet() {
            CCTP_V2_MESSAGE_TRANSMITTER_TESTNET
        } else {
            CCTP_V2_MESSAGE_TRANSMITTER_MAINNET
        })
    }

    fn cctp_v2_domain_id(&self) -> Result<DomainId> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        Ok(match self {
            // v1 and v2 chains
            Self::Mainnet | Self::Sepolia => DomainId::Ethereum,
            Self::Avalanche | Self::AvalancheFuji => DomainId::Avalanche,
            Self::Optimism | Self::OptimismSepolia => DomainId::Optimism,
            Self::Arbitrum | Self::ArbitrumSepolia => DomainId::Arbitrum,
            Self::Base | Self::BaseSepolia => DomainId::Base,
            Self::Polygon | Self::PolygonAmoy => DomainId::Polygon,
            Self::Unichain => DomainId::Unichain,
            // v2-only priority chains
            Self::Linea => DomainId::Linea,
            Self::Sonic => DomainId::Sonic,
            // TODO: Add BNB Smart Chain once available in alloy_chains
            // Self::Bsc => DomainId::BnbSmartChain,
            Self::Sei => DomainId::Sei,
            // This is unreachable due to supports_cctp_v2() check above
            _ => return Err(CctpError::UnsupportedChain(*self)),
        })
    }

    fn fast_transfer_confirmation_time_seconds(&self) -> Result<u64> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // Fast Transfer attestation times (1-2 block confirmations)
        // Based on Circle docs: https://developers.circle.com/stablecoins/required-block-confirmations
        Ok(match self {
            // Ethereum: ~20 seconds (2 block confirmations)
            Self::Mainnet | Self::Sepolia => 20,
            // Arbitrum: ~8 seconds (1 block confirmation)
            Self::Arbitrum | Self::ArbitrumSepolia => 8,
            // Base: ~8 seconds (1 block confirmation)
            Self::Base | Self::BaseSepolia => 8,
            // Optimism: ~8 seconds (1 block confirmation)
            Self::Optimism | Self::OptimismSepolia => 8,
            // Avalanche: ~8 seconds (1 block confirmation)
            Self::Avalanche | Self::AvalancheFuji => 8,
            // Polygon: ~8 seconds (1 block confirmation)
            Self::Polygon | Self::PolygonAmoy => 8,
            // Unichain: ~8 seconds (1 block confirmation)
            Self::Unichain => 8,
            // Linea: ~8 seconds (vs 6-32 hours for Standard!)
            Self::Linea => 8,
            // Sonic: ~5 seconds (high-performance chain)
            Self::Sonic => 5,
            // Sei: ~5 seconds (parallel EVM)
            Self::Sei => 5,
            _ => return Err(CctpError::UnsupportedChain(*self)),
        })
    }

    fn standard_transfer_confirmation_time_seconds(&self) -> Result<u64> {
        if !self.supports_cctp_v2() {
            return Err(CctpError::UnsupportedChain(*self));
        }

        // Standard Transfer attestation times (full finality)
        // Based on Circle docs: https://developers.circle.com/stablecoins/required-block-confirmations
        Ok(match self {
            // Ethereum L1 + L2s settling to Ethereum: 13-19 minutes (~65 ETH blocks)
            Self::Mainnet | Self::Sepolia => 19 * 60,
            Self::Arbitrum | Self::ArbitrumSepolia => 19 * 60,
            Self::Base | Self::BaseSepolia => 19 * 60,
            Self::Optimism | Self::OptimismSepolia => 19 * 60,
            Self::Unichain => 19 * 60,
            // Avalanche: ~20 seconds (native finality)
            Self::Avalanche | Self::AvalancheFuji => 20,
            // Polygon: ~8 minutes (PoS finality)
            Self::Polygon | Self::PolygonAmoy => 8 * 60,
            // Linea: 6-32 hours (zkEVM proof generation) - use conservative 8 hours
            Self::Linea => 8 * 60 * 60,
            // Sonic: ~5 seconds (high-performance chain, native finality)
            Self::Sonic => 5,
            // Sei: ~5 seconds (parallel EVM, native finality)
            Self::Sei => 5,
            _ => return Err(CctpError::UnsupportedChain(*self)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_chain_support() {
        // v1 chains should support v2
        assert!(NamedChain::Mainnet.supports_cctp_v2());
        assert!(NamedChain::Arbitrum.supports_cctp_v2());
        assert!(NamedChain::Base.supports_cctp_v2());

        // v2-only priority chains
        assert!(NamedChain::Linea.supports_cctp_v2());
        assert!(NamedChain::Sonic.supports_cctp_v2());
        // TODO: Add BNB Smart Chain once available in alloy_chains
        // assert!(NamedChain::Bsc.supports_cctp_v2());
        assert!(NamedChain::Sei.supports_cctp_v2());

        // Unsupported chain
        assert!(!NamedChain::Moonbeam.supports_cctp_v2());
    }

    #[test]
    fn test_fast_transfer_support() {
        // All v2 chains support fast transfer
        assert!(NamedChain::Mainnet.supports_fast_transfer().unwrap());
        assert!(NamedChain::Linea.supports_fast_transfer().unwrap());
        assert!(NamedChain::Sonic.supports_fast_transfer().unwrap());

        // Unsupported chain returns error
        assert!(NamedChain::Moonbeam.supports_fast_transfer().is_err());
    }

    #[test]
    fn test_fast_transfer_fees() {
        // Currently all chains return 0 bps (placeholder)
        assert_eq!(
            NamedChain::Mainnet.fast_transfer_fee_bps().unwrap(),
            Some(0)
        );
        assert_eq!(NamedChain::Linea.fast_transfer_fee_bps().unwrap(), Some(0));
    }

    #[test]
    fn test_domain_id_mapping() {
        // v1 chains
        assert_eq!(
            NamedChain::Mainnet.cctp_v2_domain_id().unwrap(),
            DomainId::Ethereum
        );
        assert_eq!(
            NamedChain::Arbitrum.cctp_v2_domain_id().unwrap(),
            DomainId::Arbitrum
        );

        // v2-only chains
        assert_eq!(
            NamedChain::Linea.cctp_v2_domain_id().unwrap(),
            DomainId::Linea
        );
        assert_eq!(
            NamedChain::Sonic.cctp_v2_domain_id().unwrap(),
            DomainId::Sonic
        );
        // TODO: Add BNB Smart Chain once available in alloy_chains
        // assert_eq!(
        //     NamedChain::Bsc.cctp_v2_domain_id().unwrap(),
        //     DomainId::BnbSmartChain
        // );
        assert_eq!(NamedChain::Sei.cctp_v2_domain_id().unwrap(), DomainId::Sei);
    }

    #[test]
    fn test_contract_addresses() {
        // Mainnet chains should return mainnet addresses
        let linea_tm = NamedChain::Linea.token_messenger_v2_address().unwrap();
        let linea_mt = NamedChain::Linea.message_transmitter_v2_address().unwrap();
        assert_eq!(linea_tm, CCTP_V2_TOKEN_MESSENGER_MAINNET);
        assert_eq!(linea_mt, CCTP_V2_MESSAGE_TRANSMITTER_MAINNET);

        let sonic_tm = NamedChain::Sonic.token_messenger_v2_address().unwrap();
        let sonic_mt = NamedChain::Sonic.message_transmitter_v2_address().unwrap();
        assert_eq!(sonic_tm, CCTP_V2_TOKEN_MESSENGER_MAINNET);
        assert_eq!(sonic_mt, CCTP_V2_MESSAGE_TRANSMITTER_MAINNET);

        // All mainnet chains should have the same v2 addresses
        assert_eq!(linea_tm, sonic_tm);
        assert_eq!(linea_mt, sonic_mt);
    }

    #[test]
    fn test_fast_transfer_confirmation_times() {
        // Fast Transfer: 1-2 block confirmations
        // Ethereum: 20 seconds (2 blocks)
        assert_eq!(
            NamedChain::Mainnet
                .fast_transfer_confirmation_time_seconds()
                .unwrap(),
            20
        );
        // L2s and most chains: 8 seconds (1 block)
        assert_eq!(
            NamedChain::Arbitrum
                .fast_transfer_confirmation_time_seconds()
                .unwrap(),
            8
        );
        assert_eq!(
            NamedChain::Linea
                .fast_transfer_confirmation_time_seconds()
                .unwrap(),
            8
        );
        // High-performance chains: 5 seconds
        assert_eq!(
            NamedChain::Sonic
                .fast_transfer_confirmation_time_seconds()
                .unwrap(),
            5
        );
        assert_eq!(
            NamedChain::Sei
                .fast_transfer_confirmation_time_seconds()
                .unwrap(),
            5
        );
    }

    #[test]
    fn test_standard_transfer_confirmation_times() {
        // Standard Transfer: full finality required
        // Ethereum + L2s: 19 minutes (~65 ETH blocks)
        assert_eq!(
            NamedChain::Mainnet
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            19 * 60
        );
        assert_eq!(
            NamedChain::Arbitrum
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            19 * 60
        );
        assert_eq!(
            NamedChain::Base
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            19 * 60
        );
        // Avalanche: 20 seconds (native finality)
        assert_eq!(
            NamedChain::Avalanche
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            20
        );
        // Polygon: 8 minutes
        assert_eq!(
            NamedChain::Polygon
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            8 * 60
        );
        // Linea: 8 hours (zkEVM proof generation)
        assert_eq!(
            NamedChain::Linea
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            8 * 60 * 60
        );
        // High-performance chains: same as fast (already fast natively)
        assert_eq!(
            NamedChain::Sonic
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            5
        );
        assert_eq!(
            NamedChain::Sei
                .standard_transfer_confirmation_time_seconds()
                .unwrap(),
            5
        );
    }
}
