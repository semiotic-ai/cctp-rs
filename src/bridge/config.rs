use alloy_chains::NamedChain;
use std::time::Duration;

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

/// Default confirmation requirements and timeouts for different chains
#[allow(dead_code)]
pub const DEFAULT_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes default
#[allow(dead_code)]
pub const CHAIN_CONFIRMATION_CONFIG: &[(NamedChain, u64, Duration)] = &[
    // (Chain, Required Confirmations, Timeout)
    (NamedChain::Mainnet, 2, Duration::from_secs(300)), // 5 mins for Ethereum
    (NamedChain::Arbitrum, 1, Duration::from_secs(120)), // 2 mins for Arbitrum
    (NamedChain::Optimism, 1, Duration::from_secs(120)), // 2 mins for Optimism
    (NamedChain::Polygon, 15, Duration::from_secs(180)), // More confirmations for Polygon
    (NamedChain::Avalanche, 3, Duration::from_secs(120)), // 2 mins for Avalanche
    (NamedChain::BinanceSmartChain, 2, Duration::from_secs(120)), // 2 mins for BNB Chain
    (NamedChain::Base, 1, Duration::from_secs(120)),    // 2 mins for Base
    (NamedChain::Unichain, 1, Duration::from_secs(120)), // 2 mins for Unichain
];

/// Gets the chain-specific confirmation configuration
#[allow(dead_code)]
pub fn get_chain_confirmation_config(chain: &NamedChain) -> (u64, Duration) {
    CHAIN_CONFIRMATION_CONFIG
        .iter()
        .find(|(ch, _, _)| ch == chain)
        .map(|(_, confirmations, timeout)| (*confirmations, *timeout))
        .unwrap_or((1, DEFAULT_CONFIRMATION_TIMEOUT))
}
