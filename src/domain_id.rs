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
    /// Base and Base Sepolia (Domain ID: 6)
    Base = 6,
    /// Polygon PoS (Domain ID: 7)
    Polygon = 7,
    /// Unichain (Domain ID: 10)
    Unichain = 10,
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
    /// assert_eq!(DomainId::from_u32(999), None);
    /// ```
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Ethereum),
            1 => Some(Self::Avalanche),
            2 => Some(Self::Optimism),
            3 => Some(Self::Arbitrum),
            6 => Some(Self::Base),
            7 => Some(Self::Polygon),
            10 => Some(Self::Unichain),
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
    /// ```
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Avalanche => "Avalanche",
            Self::Optimism => "Optimism",
            Self::Arbitrum => "Arbitrum",
            Self::Base => "Base",
            Self::Polygon => "Polygon",
            Self::Unichain => "Unichain",
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
        assert_eq!(DomainId::Ethereum.as_u32(), 0);
        assert_eq!(DomainId::Avalanche.as_u32(), 1);
        assert_eq!(DomainId::Optimism.as_u32(), 2);
        assert_eq!(DomainId::Arbitrum.as_u32(), 3);
        assert_eq!(DomainId::Base.as_u32(), 6);
        assert_eq!(DomainId::Polygon.as_u32(), 7);
        assert_eq!(DomainId::Unichain.as_u32(), 10);
    }

    #[test]
    fn test_from_u32_valid() {
        assert_eq!(DomainId::from_u32(0), Some(DomainId::Ethereum));
        assert_eq!(DomainId::from_u32(1), Some(DomainId::Avalanche));
        assert_eq!(DomainId::from_u32(2), Some(DomainId::Optimism));
        assert_eq!(DomainId::from_u32(3), Some(DomainId::Arbitrum));
        assert_eq!(DomainId::from_u32(6), Some(DomainId::Base));
        assert_eq!(DomainId::from_u32(7), Some(DomainId::Polygon));
        assert_eq!(DomainId::from_u32(10), Some(DomainId::Unichain));
    }

    #[test]
    fn test_from_u32_invalid() {
        assert_eq!(DomainId::from_u32(4), None);
        assert_eq!(DomainId::from_u32(5), None);
        assert_eq!(DomainId::from_u32(8), None);
        assert_eq!(DomainId::from_u32(9), None);
        assert_eq!(DomainId::from_u32(11), None);
        assert_eq!(DomainId::from_u32(999), None);
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
            DomainId::Ethereum,
            DomainId::Avalanche,
            DomainId::Optimism,
            DomainId::Arbitrum,
            DomainId::Base,
            DomainId::Polygon,
            DomainId::Unichain,
        ] {
            let value: u32 = domain.into();
            let parsed = DomainId::try_from(value).unwrap();
            assert_eq!(domain, parsed);
        }
    }
}
