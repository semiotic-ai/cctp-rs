// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Provider utilities for CCTP operations.
//!
//! This module provides helpers for gas estimation and provider configuration
//! to improve reliability of cross-chain transfers.

use crate::error::{CctpError, Result};
use alloy_network::Ethereum;
use alloy_primitives::U256;
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use std::time::Duration;

/// Default gas buffer percentage (20%)
pub const DEFAULT_GAS_BUFFER_PERCENT: u64 = 20;

/// Default request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default number of retry attempts
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;

/// Estimate gas for a transaction with an optional safety buffer.
///
/// This helper calls the provider's `estimate_gas` method and adds a configurable
/// percentage buffer to prevent out-of-gas failures on complex transfers like
/// CCTP burns and mints.
///
/// # Arguments
///
/// * `provider` - The Ethereum provider to use for estimation
/// * `tx` - The transaction request to estimate gas for
/// * `buffer_percent` - Optional percentage buffer to add (defaults to 20%)
///
/// # Returns
///
/// The estimated gas limit with the buffer applied.
///
/// # Example
///
/// ```rust,ignore
/// use cctp_rs::provider::estimate_gas_with_buffer;
///
/// let gas_limit = estimate_gas_with_buffer(&provider, &tx, Some(20)).await?;
/// let tx = tx.with_gas_limit(gas_limit);
/// ```
pub async fn estimate_gas_with_buffer<P: Provider<Ethereum>>(
    provider: &P,
    tx: &TransactionRequest,
    buffer_percent: Option<u64>,
) -> Result<u64> {
    let buffer = buffer_percent.unwrap_or(DEFAULT_GAS_BUFFER_PERCENT);

    let estimate = provider
        .estimate_gas(tx.clone())
        .await
        .map_err(|e| CctpError::Provider(format!("Gas estimation failed: {e}")))?;

    // Apply buffer: estimate * (100 + buffer) / 100
    let with_buffer = estimate.saturating_mul(100 + buffer) / 100;

    Ok(with_buffer)
}

/// Configuration for creating production-ready providers.
///
/// This struct encapsulates recommended settings for CCTP operations,
/// including retry behavior and timeouts.
///
/// # Example
///
/// ```rust
/// use cctp_rs::ProviderConfig;
/// use std::time::Duration;
///
/// // Use defaults
/// let config = ProviderConfig::default();
///
/// // Or customize
/// let config = ProviderConfig::builder()
///     .retry_attempts(5)
///     .timeout(Duration::from_secs(60))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Number of retry attempts for failed requests
    pub retry_attempts: u32,
    /// Request timeout duration
    pub timeout: Duration,
    /// Optional rate limit (requests per second)
    pub rate_limit_rps: Option<u32>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            retry_attempts: DEFAULT_RETRY_ATTEMPTS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            rate_limit_rps: None,
        }
    }
}

impl ProviderConfig {
    /// Creates a new builder for ProviderConfig
    pub fn builder() -> ProviderConfigBuilder {
        ProviderConfigBuilder::default()
    }

    /// Creates a configuration optimized for fast transfers
    ///
    /// Uses shorter timeouts and more aggressive retry settings
    /// suitable for time-sensitive fast transfer operations.
    pub fn fast_transfer() -> Self {
        Self {
            retry_attempts: 5,
            timeout: Duration::from_secs(15),
            rate_limit_rps: None,
        }
    }

    /// Creates a configuration for high-reliability operations
    ///
    /// Uses longer timeouts and more retry attempts for
    /// operations where reliability is more important than speed.
    pub fn high_reliability() -> Self {
        Self {
            retry_attempts: 10,
            timeout: Duration::from_secs(60),
            rate_limit_rps: None,
        }
    }

    /// Creates a configuration for rate-limited public endpoints
    ///
    /// Includes rate limiting to avoid hitting provider limits
    /// on public RPC endpoints.
    pub fn rate_limited(rps: u32) -> Self {
        Self {
            retry_attempts: DEFAULT_RETRY_ATTEMPTS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            rate_limit_rps: Some(rps),
        }
    }
}

/// Builder for [`ProviderConfig`]
#[derive(Debug, Clone, Default)]
pub struct ProviderConfigBuilder {
    retry_attempts: Option<u32>,
    timeout: Option<Duration>,
    rate_limit_rps: Option<u32>,
}

impl ProviderConfigBuilder {
    /// Sets the number of retry attempts
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = Some(attempts);
        self
    }

    /// Sets the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the rate limit in requests per second
    pub fn rate_limit_rps(mut self, rps: u32) -> Self {
        self.rate_limit_rps = Some(rps);
        self
    }

    /// Builds the ProviderConfig
    pub fn build(self) -> ProviderConfig {
        ProviderConfig {
            retry_attempts: self.retry_attempts.unwrap_or(DEFAULT_RETRY_ATTEMPTS),
            timeout: self
                .timeout
                .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
            rate_limit_rps: self.rate_limit_rps,
        }
    }
}

/// Helper to calculate gas price with a tip buffer for EIP-1559 transactions.
///
/// This adds a configurable percentage buffer to the max priority fee
/// to help ensure transactions are included in blocks during congestion.
///
/// # Arguments
///
/// * `base_fee` - The current base fee from the latest block
/// * `max_priority_fee` - The desired priority fee (tip)
/// * `buffer_percent` - Percentage buffer to add to the priority fee
///
/// # Returns
///
/// A tuple of (max_fee_per_gas, max_priority_fee_per_gas) with buffer applied
///
/// # Example
///
/// ```rust
/// use cctp_rs::calculate_gas_price_with_buffer;
/// use alloy_primitives::U256;
///
/// let base_fee = U256::from(30_000_000_000u64); // 30 gwei
/// let priority_fee = U256::from(2_000_000_000u64); // 2 gwei
///
/// let (max_fee, max_priority) = calculate_gas_price_with_buffer(
///     base_fee,
///     priority_fee,
///     20, // 20% buffer
/// );
/// ```
pub fn calculate_gas_price_with_buffer(
    base_fee: U256,
    max_priority_fee: U256,
    buffer_percent: u64,
) -> (U256, U256) {
    // Apply buffer to priority fee
    let buffered_priority = max_priority_fee * U256::from(100 + buffer_percent) / U256::from(100);

    // Max fee = 2 * base_fee + buffered_priority (standard EIP-1559 formula with buffer)
    let max_fee = base_fee * U256::from(2) + buffered_priority;

    (max_fee, buffered_priority)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.retry_attempts, 3);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.rate_limit_rps.is_none());
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::builder()
            .retry_attempts(5)
            .timeout(Duration::from_secs(60))
            .rate_limit_rps(10)
            .build();

        assert_eq!(config.retry_attempts, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.rate_limit_rps, Some(10));
    }

    #[test]
    fn test_provider_config_fast_transfer() {
        let config = ProviderConfig::fast_transfer();
        assert_eq!(config.retry_attempts, 5);
        assert_eq!(config.timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_provider_config_high_reliability() {
        let config = ProviderConfig::high_reliability();
        assert_eq!(config.retry_attempts, 10);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_provider_config_rate_limited() {
        let config = ProviderConfig::rate_limited(5);
        assert_eq!(config.rate_limit_rps, Some(5));
    }

    #[test]
    fn test_gas_price_with_buffer() {
        let base_fee = U256::from(30_000_000_000u64); // 30 gwei
        let priority_fee = U256::from(2_000_000_000u64); // 2 gwei

        let (max_fee, max_priority) = calculate_gas_price_with_buffer(base_fee, priority_fee, 20);

        // Priority should be 2.4 gwei (2 + 20%)
        assert_eq!(max_priority, U256::from(2_400_000_000u64));

        // Max fee should be 2 * 30 + 2.4 = 62.4 gwei
        assert_eq!(max_fee, U256::from(62_400_000_000u64));
    }

    #[test]
    fn test_gas_price_with_zero_buffer() {
        let base_fee = U256::from(30_000_000_000u64);
        let priority_fee = U256::from(2_000_000_000u64);

        let (max_fee, max_priority) = calculate_gas_price_with_buffer(base_fee, priority_fee, 0);

        assert_eq!(max_priority, priority_fee);
        assert_eq!(max_fee, base_fee * U256::from(2) + priority_fee);
    }
}
