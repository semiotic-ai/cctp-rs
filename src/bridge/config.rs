/// Circle Iris API environment URLs
///
/// See <https://developers.circle.com/stablecoins/cctp-apis>
///
pub const IRIS_API: &str = "https://iris-api.circle.com";
pub const IRIS_API_SANDBOX: &str = "https://iris-api-sandbox.circle.com";

/// CCTP v1 attestation API path
pub const ATTESTATION_PATH_V1: &str = "/v1/attestations/";

/// CCTP v2 messages API path
///
/// V2 uses a different endpoint format than v1:
/// - V1: `/v1/attestations/{messageHash}`
/// - V2: `/v2/messages/{sourceDomain}?transactionHash={txHash}`
pub const MESSAGES_PATH_V2: &str = "/v2/messages/";

/// Configuration for attestation polling behavior.
///
/// Controls how the bridge polls Circle's Iris API for attestation availability.
/// Use the builder methods to customize, or use preset configurations for common scenarios.
///
/// # Examples
///
/// ```rust
/// use cctp_rs::PollingConfig;
///
/// // Use defaults (30 attempts, 60 second intervals)
/// let config = PollingConfig::default();
///
/// // Customize polling behavior
/// let config = PollingConfig::default()
///     .with_max_attempts(20)
///     .with_poll_interval_secs(30);
///
/// // Use preset for fast transfers (30 attempts, 5 second intervals)
/// let config = PollingConfig::fast_transfer();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollingConfig {
    /// Maximum number of polling attempts before giving up.
    pub max_attempts: u32,
    /// Seconds to wait between polling attempts.
    pub poll_interval_secs: u64,
}

impl Default for PollingConfig {
    /// Creates a default polling configuration suitable for standard CCTP v1 transfers.
    ///
    /// - `max_attempts`: 30
    /// - `poll_interval_secs`: 60 (1 minute)
    ///
    /// This results in a maximum wait time of ~30 minutes, which accommodates
    /// the typical 13-19 minute attestation time for v1 transfers.
    fn default() -> Self {
        Self {
            max_attempts: 30,
            poll_interval_secs: 60,
        }
    }
}

impl PollingConfig {
    /// Creates a polling configuration optimized for CCTP v2 fast transfers.
    ///
    /// - `max_attempts`: 30
    /// - `poll_interval_secs`: 5
    ///
    /// Fast transfers typically complete in under 30 seconds, so this configuration
    /// polls more frequently with shorter intervals.
    pub fn fast_transfer() -> Self {
        Self {
            max_attempts: 30,
            poll_interval_secs: 5,
        }
    }

    /// Sets the maximum number of polling attempts.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Maximum number of times to poll the attestation API
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::PollingConfig;
    ///
    /// let config = PollingConfig::default().with_max_attempts(60);
    /// assert_eq!(config.max_attempts, 60);
    /// ```
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Sets the interval between polling attempts in seconds.
    ///
    /// # Arguments
    ///
    /// * `secs` - Seconds to wait between each polling attempt
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::PollingConfig;
    ///
    /// let config = PollingConfig::default().with_poll_interval_secs(30);
    /// assert_eq!(config.poll_interval_secs, 30);
    /// ```
    pub fn with_poll_interval_secs(mut self, secs: u64) -> Self {
        self.poll_interval_secs = secs;
        self
    }

    /// Returns the total maximum wait time in seconds.
    ///
    /// This is calculated as `max_attempts * poll_interval_secs`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cctp_rs::PollingConfig;
    ///
    /// let config = PollingConfig::default();
    /// assert_eq!(config.total_timeout_secs(), 30 * 60); // 30 minutes
    /// ```
    pub fn total_timeout_secs(&self) -> u64 {
        self.max_attempts as u64 * self.poll_interval_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PollingConfig::default();
        assert_eq!(config.max_attempts, 30);
        assert_eq!(config.poll_interval_secs, 60);
        assert_eq!(config.total_timeout_secs(), 1800); // 30 minutes
    }

    #[test]
    fn test_fast_transfer_config() {
        let config = PollingConfig::fast_transfer();
        assert_eq!(config.max_attempts, 30);
        assert_eq!(config.poll_interval_secs, 5);
        assert_eq!(config.total_timeout_secs(), 150); // 2.5 minutes
    }

    #[test]
    fn test_builder_methods() {
        let config = PollingConfig::default()
            .with_max_attempts(20)
            .with_poll_interval_secs(30);
        assert_eq!(config.max_attempts, 20);
        assert_eq!(config.poll_interval_secs, 30);
        assert_eq!(config.total_timeout_secs(), 600); // 10 minutes
    }

    #[test]
    fn test_config_is_copy() {
        let config = PollingConfig::default();
        let copied = config;
        assert_eq!(config, copied);
    }
}
