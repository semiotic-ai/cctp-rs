// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Batch call helpers for efficient RPC operations.
//!
//! This module provides utilities for batching multiple contract calls into
//! parallel RPC requests, reducing latency when fetching multiple values.
//!
//! # Benefits
//!
//! - Reduced latency: Multiple calls execute concurrently
//! - Better throughput: Multiple requests can be in-flight simultaneously
//! - Simpler code: Fetch related data in one logical operation
//!
//! # Example
//!
//! ```rust,ignore
//! use cctp_rs::batch_token_checks;
//!
//! // Fetch balance and allowance in parallel
//! let (allowance, balance) = batch_token_checks(
//!     &provider,
//!     usdc_address,
//!     owner_address,
//!     token_messenger_address,
//! ).await?;
//!
//! if allowance < amount && balance >= amount {
//!     // Need to approve before burning
//! }
//! ```
//!
//! # Implementation Note
//!
//! These helpers use `tokio::join!` for parallel execution rather than
//! on-chain Multicall3. This achieves similar latency benefits without
//! requiring the Multicall3 contract to be deployed on all chains.

use crate::contracts::erc20::Erc20Contract;
use crate::error::{CctpError, Result};
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use tracing::{debug, info};

/// Batch check token allowance and balance in parallel RPC calls.
///
/// This is more efficient than making sequential `allowance()` and `balanceOf()`
/// calls when you need both values, as the calls execute concurrently.
///
/// # Arguments
///
/// * `provider` - The Ethereum provider
/// * `token` - The ERC20 token contract address (e.g., USDC)
/// * `owner` - The address that owns the tokens
/// * `spender` - The address to check allowance for (e.g., TokenMessenger)
///
/// # Returns
///
/// A tuple of `(allowance, balance)` where both are `U256`.
///
/// # Example
///
/// ```rust,ignore
/// use cctp_rs::batch_token_checks;
///
/// let (allowance, balance) = batch_token_checks(
///     &provider,
///     usdc,
///     sender,
///     token_messenger,
/// ).await?;
///
/// if balance >= amount {
///     if allowance < amount {
///         // Need approval first
///         bridge.approve(usdc, sender, amount).await?;
///     }
///     // Can burn
///     bridge.burn(amount, sender, usdc).await?;
/// }
/// ```
pub async fn batch_token_checks<P>(
    provider: &P,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<(U256, U256)>
where
    P: Provider<Ethereum> + Clone,
{
    debug!(
        token = %token,
        owner = %owner,
        spender = %spender,
        event = "batch_token_checks_started"
    );

    let erc20 = Erc20Contract::new(token, provider.clone());

    // Execute both calls in parallel using tokio::join!
    let (allowance_result, balance_result) =
        tokio::join!(erc20.allowance(owner, spender), erc20.balance_of(owner));

    let allowance = allowance_result
        .map_err(|e| CctpError::ContractCall(format!("Failed to get allowance: {e}")))?;
    let balance = balance_result
        .map_err(|e| CctpError::ContractCall(format!("Failed to get balance: {e}")))?;

    info!(
        token = %token,
        owner = %owner,
        spender = %spender,
        allowance = %allowance,
        balance = %balance,
        event = "batch_token_checks_completed"
    );

    Ok((allowance, balance))
}

/// Token state containing balance and allowance information.
///
/// Returned by [`batch_token_state`] to provide a structured view
/// of an account's token state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenState {
    /// The token balance of the owner
    pub balance: U256,
    /// The allowance granted to the spender
    pub allowance: U256,
}

impl TokenState {
    /// Check if the owner can transfer the specified amount.
    ///
    /// Returns `true` if balance >= amount AND allowance >= amount.
    pub fn can_transfer(&self, amount: U256) -> bool {
        self.balance >= amount && self.allowance >= amount
    }

    /// Check if approval is needed for the specified amount.
    ///
    /// Returns `true` if allowance < amount.
    pub fn needs_approval(&self, amount: U256) -> bool {
        self.allowance < amount
    }

    /// Check if the owner has sufficient balance.
    pub fn has_sufficient_balance(&self, amount: U256) -> bool {
        self.balance >= amount
    }
}

/// Batch check token state (balance and allowance) returning a structured result.
///
/// This is a convenience wrapper around [`batch_token_checks`] that returns
/// a [`TokenState`] struct with helper methods.
///
/// # Example
///
/// ```rust,ignore
/// let state = batch_token_state(&provider, usdc, sender, token_messenger).await?;
///
/// if !state.has_sufficient_balance(amount) {
///     return Err("Insufficient USDC balance".into());
/// }
///
/// if state.needs_approval(amount) {
///     bridge.approve(usdc, sender, amount).await?;
/// }
///
/// // Now safe to burn
/// bridge.burn(amount, sender, usdc).await?;
/// ```
pub async fn batch_token_state<P>(
    provider: &P,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<TokenState>
where
    P: Provider<Ethereum> + Clone,
{
    let (allowance, balance) = batch_token_checks(provider, token, owner, spender).await?;
    Ok(TokenState { balance, allowance })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_state_can_transfer() {
        let state = TokenState {
            balance: U256::from(1000),
            allowance: U256::from(500),
        };

        assert!(state.can_transfer(U256::from(500)));
        assert!(state.can_transfer(U256::from(100)));
        assert!(!state.can_transfer(U256::from(501))); // exceeds allowance
        assert!(!state.can_transfer(U256::from(1001))); // exceeds balance
    }

    #[test]
    fn test_token_state_needs_approval() {
        let state = TokenState {
            balance: U256::from(1000),
            allowance: U256::from(500),
        };

        assert!(!state.needs_approval(U256::from(500)));
        assert!(!state.needs_approval(U256::from(100)));
        assert!(state.needs_approval(U256::from(501)));
        assert!(state.needs_approval(U256::from(1000)));
    }

    #[test]
    fn test_token_state_has_sufficient_balance() {
        let state = TokenState {
            balance: U256::from(1000),
            allowance: U256::from(500),
        };

        assert!(state.has_sufficient_balance(U256::from(1000)));
        assert!(state.has_sufficient_balance(U256::from(100)));
        assert!(!state.has_sufficient_balance(U256::from(1001)));
    }

    #[test]
    fn test_token_state_zero_allowance() {
        let state = TokenState {
            balance: U256::from(1000),
            allowance: U256::ZERO,
        };

        assert!(!state.can_transfer(U256::from(1)));
        assert!(state.needs_approval(U256::from(1)));
        assert!(state.has_sufficient_balance(U256::from(1000)));
    }
}
