//! CCTP v2 finality threshold types
//!
//! Circle's CCTP v2 introduces finality thresholds to enable Fast Transfers.
//! Messages can specify a minimum finality requirement, determining how quickly
//! attestations are issued.
//!
//! Reference: <https://developers.circle.com/cctp/technical-guide>

use std::fmt;

/// Finality threshold for CCTP v2 messages
///
/// Determines the level of finality required before Circle's attestation service
/// will sign a message. Lower thresholds enable faster transfers but may have
/// slightly higher fees.
///
/// # Examples
///
/// ```rust
/// use cctp_rs::FinalityThreshold;
///
/// let fast = FinalityThreshold::Fast;
/// assert_eq!(fast.as_u32(), 1000);
/// assert_eq!(fast.name(), "Fast Transfer");
///
/// let standard = FinalityThreshold::Standard;
/// assert_eq!(standard.as_u32(), 2000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FinalityThreshold {
    /// Fast Transfer - Attestation at confirmed block level (threshold: 1000)
    ///
    /// - Settlement time: Under 30 seconds
    /// - Fee: 0-14 basis points (chain-dependent)
    /// - Use case: Time-sensitive operations, arbitrage, real-time DeFi
    Fast = 1000,

    /// Standard Transfer - Attestation at finalized block level (threshold: 2000)
    ///
    /// - Settlement time: 13-19 minutes (same as v1)
    /// - Fee: 0 basis points
    /// - Use case: Non-urgent transfers, maximum security
    Standard = 2000,
}

impl FinalityThreshold {
    /// Returns the numeric threshold value
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::FinalityThreshold;
    ///
    /// assert_eq!(FinalityThreshold::Fast.as_u32(), 1000);
    /// assert_eq!(FinalityThreshold::Standard.as_u32(), 2000);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Attempts to create a FinalityThreshold from a u32 value
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::FinalityThreshold;
    ///
    /// assert_eq!(
    ///     FinalityThreshold::from_u32(1000),
    ///     Some(FinalityThreshold::Fast)
    /// );
    /// assert_eq!(
    ///     FinalityThreshold::from_u32(2000),
    ///     Some(FinalityThreshold::Standard)
    /// );
    /// assert_eq!(FinalityThreshold::from_u32(1500), None);
    /// ```
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            1000 => Some(Self::Fast),
            2000 => Some(Self::Standard),
            _ => None,
        }
    }

    /// Returns a descriptive name for this threshold
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::FinalityThreshold;
    ///
    /// assert_eq!(FinalityThreshold::Fast.name(), "Fast Transfer");
    /// assert_eq!(FinalityThreshold::Standard.name(), "Standard Transfer");
    /// ```
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fast => "Fast Transfer",
            Self::Standard => "Standard Transfer",
        }
    }

    /// Returns true if this is a Fast Transfer threshold
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::FinalityThreshold;
    ///
    /// assert!(FinalityThreshold::Fast.is_fast());
    /// assert!(!FinalityThreshold::Standard.is_fast());
    /// ```
    #[inline]
    pub const fn is_fast(self) -> bool {
        matches!(self, Self::Fast)
    }

    /// Returns true if this is a Standard Transfer threshold
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::FinalityThreshold;
    ///
    /// assert!(FinalityThreshold::Standard.is_standard());
    /// assert!(!FinalityThreshold::Fast.is_standard());
    /// ```
    #[inline]
    pub const fn is_standard(self) -> bool {
        matches!(self, Self::Standard)
    }
}

impl Default for FinalityThreshold {
    /// Returns Standard as the default threshold
    ///
    /// Standard transfers have no fees and are the safest option.
    fn default() -> Self {
        Self::Standard
    }
}

impl From<FinalityThreshold> for u32 {
    #[inline]
    fn from(threshold: FinalityThreshold) -> Self {
        threshold.as_u32()
    }
}

impl TryFrom<u32> for FinalityThreshold {
    type Error = InvalidFinalityThreshold;

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_u32(value).ok_or(InvalidFinalityThreshold(value))
    }
}

impl fmt::Display for FinalityThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.as_u32())
    }
}

/// Error returned when attempting to convert an invalid u32 to a FinalityThreshold
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFinalityThreshold(pub u32);

impl fmt::Display for InvalidFinalityThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid finality threshold: {} (expected 1000 or 2000)",
            self.0
        )
    }
}

impl std::error::Error for InvalidFinalityThreshold {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_values() {
        assert_eq!(FinalityThreshold::Fast.as_u32(), 1000);
        assert_eq!(FinalityThreshold::Standard.as_u32(), 2000);
    }

    #[test]
    fn test_from_u32_valid() {
        assert_eq!(
            FinalityThreshold::from_u32(1000),
            Some(FinalityThreshold::Fast)
        );
        assert_eq!(
            FinalityThreshold::from_u32(2000),
            Some(FinalityThreshold::Standard)
        );
    }

    #[test]
    fn test_from_u32_invalid() {
        assert_eq!(FinalityThreshold::from_u32(0), None);
        assert_eq!(FinalityThreshold::from_u32(500), None);
        assert_eq!(FinalityThreshold::from_u32(1500), None);
        assert_eq!(FinalityThreshold::from_u32(3000), None);
    }

    #[test]
    fn test_try_from_valid() {
        assert_eq!(
            FinalityThreshold::try_from(1000).unwrap(),
            FinalityThreshold::Fast
        );
        assert_eq!(
            FinalityThreshold::try_from(2000).unwrap(),
            FinalityThreshold::Standard
        );
    }

    #[test]
    fn test_try_from_invalid() {
        assert!(FinalityThreshold::try_from(1500).is_err());
        let err = FinalityThreshold::try_from(1500).unwrap_err();
        assert_eq!(err, InvalidFinalityThreshold(1500));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", FinalityThreshold::Fast),
            "Fast Transfer (1000)"
        );
        assert_eq!(
            format!("{}", FinalityThreshold::Standard),
            "Standard Transfer (2000)"
        );
    }

    #[test]
    fn test_name() {
        assert_eq!(FinalityThreshold::Fast.name(), "Fast Transfer");
        assert_eq!(FinalityThreshold::Standard.name(), "Standard Transfer");
    }

    #[test]
    fn test_is_fast() {
        assert!(FinalityThreshold::Fast.is_fast());
        assert!(!FinalityThreshold::Standard.is_fast());
    }

    #[test]
    fn test_is_standard() {
        assert!(FinalityThreshold::Standard.is_standard());
        assert!(!FinalityThreshold::Fast.is_standard());
    }

    #[test]
    fn test_default() {
        assert_eq!(FinalityThreshold::default(), FinalityThreshold::Standard);
    }

    #[test]
    fn test_conversion_roundtrip() {
        for threshold in [FinalityThreshold::Fast, FinalityThreshold::Standard] {
            let value: u32 = threshold.into();
            let parsed = FinalityThreshold::try_from(value).unwrap();
            assert_eq!(threshold, parsed);
        }
    }
}
