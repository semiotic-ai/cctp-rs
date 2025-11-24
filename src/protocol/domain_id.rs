//! CCTP domain ID types for identifying blockchain networks
//!
//! Circle's Cross-Chain Transfer Protocol uses domain IDs as unique identifiers
//! for each supported blockchain network. This module provides a strongly-typed
//! enum to prevent invalid domain IDs at compile time.
//!
//! Reference: <https://developers.circle.com/stablecoins/evm-smart-contracts>

use std::fmt;

/// CCTP domain identifier for blockchain networks
///
/// Each blockchain network supported by Circle's CCTP has a unique domain ID.
/// This enum provides type-safe representation of these identifiers.
///
/// # CCTP Version Support
///
/// - Domains 0-10: Supported in CCTP v1 and v2
/// - Domains 11+: Only supported in CCTP v2
///
/// # Example
///
/// ```rust
/// use cctp_rs::DomainId;
///
/// let ethereum_domain = DomainId::Ethereum;
/// let domain_value: u32 = ethereum_domain.into();
/// assert_eq!(domain_value, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum DomainId {
    /// Ethereum mainnet and Sepolia testnet (Domain ID: 0)
    Ethereum = 0,
    /// Avalanche C-Chain (Domain ID: 1)
    Avalanche = 1,
    /// Optimism (Domain ID: 2)
    Optimism = 2,
    /// Arbitrum One and Arbitrum Sepolia (Domain ID: 3)
    Arbitrum = 3,
    /// Solana (Domain ID: 5) - Non-EVM chain, v2 only
    Solana = 5,
    /// Base and Base Sepolia (Domain ID: 6)
    Base = 6,
    /// Polygon PoS (Domain ID: 7)
    Polygon = 7,
    /// Unichain (Domain ID: 10)
    Unichain = 10,
    /// Linea (Domain ID: 11) - v2 only
    Linea = 11,
    /// Codex (Domain ID: 12) - v2 only
    Codex = 12,
    /// Sonic (Domain ID: 13) - v2 only
    Sonic = 13,
    /// World Chain (Domain ID: 14) - v2 only
    WorldChain = 14,
    /// Monad (Domain ID: 15) - v2 only
    Monad = 15,
    /// Sei (Domain ID: 16) - v2 only
    Sei = 16,
    /// BNB Smart Chain (Domain ID: 17) - v2 only
    BnbSmartChain = 17,
    /// XDC Network (Domain ID: 18) - v2 only
    Xdc = 18,
    /// HyperEVM (Domain ID: 19) - v2 only
    HyperEvm = 19,
    /// Ink (Domain ID: 21) - v2 only
    Ink = 21,
    /// Plume (Domain ID: 22) - v2 only
    Plume = 22,
    /// Starknet Testnet (Domain ID: 25) - Non-EVM chain, v2 only
    StarknetTestnet = 25,
    /// Arc Testnet (Domain ID: 26) - v2 only
    ArcTestnet = 26,
}

impl DomainId {
    /// Returns the numeric domain ID value
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::DomainId;
    ///
    /// assert_eq!(DomainId::Ethereum.as_u32(), 0);
    /// assert_eq!(DomainId::Arbitrum.as_u32(), 3);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Attempts to create a DomainId from a u32 value
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::DomainId;
    ///
    /// assert_eq!(DomainId::from_u32(0), Some(DomainId::Ethereum));
    /// assert_eq!(DomainId::from_u32(3), Some(DomainId::Arbitrum));
    /// assert_eq!(DomainId::from_u32(11), Some(DomainId::Linea));
    /// assert_eq!(DomainId::from_u32(999), None);
    /// ```
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Ethereum),
            1 => Some(Self::Avalanche),
            2 => Some(Self::Optimism),
            3 => Some(Self::Arbitrum),
            5 => Some(Self::Solana),
            6 => Some(Self::Base),
            7 => Some(Self::Polygon),
            10 => Some(Self::Unichain),
            11 => Some(Self::Linea),
            12 => Some(Self::Codex),
            13 => Some(Self::Sonic),
            14 => Some(Self::WorldChain),
            15 => Some(Self::Monad),
            16 => Some(Self::Sei),
            17 => Some(Self::BnbSmartChain),
            18 => Some(Self::Xdc),
            19 => Some(Self::HyperEvm),
            21 => Some(Self::Ink),
            22 => Some(Self::Plume),
            25 => Some(Self::StarknetTestnet),
            26 => Some(Self::ArcTestnet),
            _ => None,
        }
    }

    /// Returns the chain name as a string
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::DomainId;
    ///
    /// assert_eq!(DomainId::Ethereum.name(), "Ethereum");
    /// assert_eq!(DomainId::Arbitrum.name(), "Arbitrum");
    /// assert_eq!(DomainId::Linea.name(), "Linea");
    /// ```
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Avalanche => "Avalanche",
            Self::Optimism => "Optimism",
            Self::Arbitrum => "Arbitrum",
            Self::Solana => "Solana",
            Self::Base => "Base",
            Self::Polygon => "Polygon",
            Self::Unichain => "Unichain",
            Self::Linea => "Linea",
            Self::Codex => "Codex",
            Self::Sonic => "Sonic",
            Self::WorldChain => "World Chain",
            Self::Monad => "Monad",
            Self::Sei => "Sei",
            Self::BnbSmartChain => "BNB Smart Chain",
            Self::Xdc => "XDC",
            Self::HyperEvm => "HyperEVM",
            Self::Ink => "Ink",
            Self::Plume => "Plume",
            Self::StarknetTestnet => "Starknet Testnet",
            Self::ArcTestnet => "Arc Testnet",
        }
    }
}

impl From<DomainId> for u32 {
    #[inline]
    fn from(domain: DomainId) -> Self {
        domain.as_u32()
    }
}

impl TryFrom<u32> for DomainId {
    type Error = InvalidDomainId;

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(InvalidDomainId(value))
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.as_u32())
    }
}

/// Error returned when attempting to convert an invalid u32 to a DomainId
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDomainId(pub u32);

impl fmt::Display for InvalidDomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid CCTP domain ID: {}", self.0)
    }
}

impl std::error::Error for InvalidDomainId {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_id_values() {
        // v1 and v2 chains
        assert_eq!(DomainId::Ethereum.as_u32(), 0);
        assert_eq!(DomainId::Avalanche.as_u32(), 1);
        assert_eq!(DomainId::Optimism.as_u32(), 2);
        assert_eq!(DomainId::Arbitrum.as_u32(), 3);
        assert_eq!(DomainId::Base.as_u32(), 6);
        assert_eq!(DomainId::Polygon.as_u32(), 7);
        assert_eq!(DomainId::Unichain.as_u32(), 10);

        // v2 only chains
        assert_eq!(DomainId::Solana.as_u32(), 5);
        assert_eq!(DomainId::Linea.as_u32(), 11);
        assert_eq!(DomainId::Codex.as_u32(), 12);
        assert_eq!(DomainId::Sonic.as_u32(), 13);
        assert_eq!(DomainId::WorldChain.as_u32(), 14);
        assert_eq!(DomainId::Monad.as_u32(), 15);
        assert_eq!(DomainId::Sei.as_u32(), 16);
        assert_eq!(DomainId::BnbSmartChain.as_u32(), 17);
        assert_eq!(DomainId::Xdc.as_u32(), 18);
        assert_eq!(DomainId::HyperEvm.as_u32(), 19);
        assert_eq!(DomainId::Ink.as_u32(), 21);
        assert_eq!(DomainId::Plume.as_u32(), 22);
        assert_eq!(DomainId::StarknetTestnet.as_u32(), 25);
        assert_eq!(DomainId::ArcTestnet.as_u32(), 26);
    }

    #[test]
    fn test_from_u32_valid() {
        // v1 and v2 chains
        assert_eq!(DomainId::from_u32(0), Some(DomainId::Ethereum));
        assert_eq!(DomainId::from_u32(1), Some(DomainId::Avalanche));
        assert_eq!(DomainId::from_u32(2), Some(DomainId::Optimism));
        assert_eq!(DomainId::from_u32(3), Some(DomainId::Arbitrum));
        assert_eq!(DomainId::from_u32(6), Some(DomainId::Base));
        assert_eq!(DomainId::from_u32(7), Some(DomainId::Polygon));
        assert_eq!(DomainId::from_u32(10), Some(DomainId::Unichain));

        // v2 only chains - priority chains
        assert_eq!(DomainId::from_u32(11), Some(DomainId::Linea));
        assert_eq!(DomainId::from_u32(13), Some(DomainId::Sonic));
        assert_eq!(DomainId::from_u32(16), Some(DomainId::Sei));
        assert_eq!(DomainId::from_u32(17), Some(DomainId::BnbSmartChain));

        // v2 only chains - other
        assert_eq!(DomainId::from_u32(5), Some(DomainId::Solana));
        assert_eq!(DomainId::from_u32(12), Some(DomainId::Codex));
        assert_eq!(DomainId::from_u32(14), Some(DomainId::WorldChain));
        assert_eq!(DomainId::from_u32(15), Some(DomainId::Monad));
        assert_eq!(DomainId::from_u32(18), Some(DomainId::Xdc));
        assert_eq!(DomainId::from_u32(19), Some(DomainId::HyperEvm));
        assert_eq!(DomainId::from_u32(21), Some(DomainId::Ink));
        assert_eq!(DomainId::from_u32(22), Some(DomainId::Plume));
        assert_eq!(DomainId::from_u32(25), Some(DomainId::StarknetTestnet));
        assert_eq!(DomainId::from_u32(26), Some(DomainId::ArcTestnet));
    }

    #[test]
    fn test_from_u32_invalid() {
        // Test gaps in domain ID space
        assert_eq!(DomainId::from_u32(4), None); // Gap
        assert_eq!(DomainId::from_u32(8), None); // Gap
        assert_eq!(DomainId::from_u32(9), None); // Gap
        assert_eq!(DomainId::from_u32(20), None); // Gap
        assert_eq!(DomainId::from_u32(23), None); // Gap
        assert_eq!(DomainId::from_u32(24), None); // Gap
        assert_eq!(DomainId::from_u32(27), None); // Beyond current
        assert_eq!(DomainId::from_u32(999), None); // Way beyond
    }

    #[test]
    fn test_try_from_valid() {
        assert_eq!(DomainId::try_from(0).unwrap(), DomainId::Ethereum);
        assert_eq!(DomainId::try_from(3).unwrap(), DomainId::Arbitrum);
    }

    #[test]
    fn test_try_from_invalid() {
        assert!(DomainId::try_from(999).is_err());
        let err = DomainId::try_from(999).unwrap_err();
        assert_eq!(err, InvalidDomainId(999));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DomainId::Ethereum), "Ethereum (0)");
        assert_eq!(format!("{}", DomainId::Arbitrum), "Arbitrum (3)");
        assert_eq!(format!("{}", DomainId::Base), "Base (6)");
    }

    #[test]
    fn test_name() {
        assert_eq!(DomainId::Ethereum.name(), "Ethereum");
        assert_eq!(DomainId::Arbitrum.name(), "Arbitrum");
        assert_eq!(DomainId::Avalanche.name(), "Avalanche");
    }

    #[test]
    fn test_conversion_roundtrip() {
        for domain in [
            // v1 and v2 chains
            DomainId::Ethereum,
            DomainId::Avalanche,
            DomainId::Optimism,
            DomainId::Arbitrum,
            DomainId::Base,
            DomainId::Polygon,
            DomainId::Unichain,
            // v2 only chains
            DomainId::Solana,
            DomainId::Linea,
            DomainId::Codex,
            DomainId::Sonic,
            DomainId::WorldChain,
            DomainId::Monad,
            DomainId::Sei,
            DomainId::BnbSmartChain,
            DomainId::Xdc,
            DomainId::HyperEvm,
            DomainId::Ink,
            DomainId::Plume,
            DomainId::StarknetTestnet,
            DomainId::ArcTestnet,
        ] {
            let value: u32 = domain.into();
            let parsed = DomainId::try_from(value).unwrap();
            assert_eq!(domain, parsed);
        }
    }
}
