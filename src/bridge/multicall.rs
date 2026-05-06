// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Batch call helpers for efficient RPC operations.
//!
//! This module provides utilities for batching multiple contract calls into
//! parallel RPC requests, reducing latency when fetching multiple values.
//!
//! # Example
//!
//! ```rust,ignore
//! use cctp_rs::batch_token_state;
//!
//! let state = batch_token_state(
//!     &provider,
//!     usdc_address,
//!     owner_address,
//!     token_messenger_address,
//! ).await?;
//!
//! if state.needs_approval(amount) && state.has_sufficient_balance(amount) {
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
use crate::error::Result;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;

/// Batch check token allowance and balance in parallel RPC calls.
///
/// Returns a tuple of `(allowance, balance)`. Prefer [`batch_token_state`] for
/// new code — it returns a [`TokenState`] with named fields and predicate
/// helpers (`can_transfer`, `needs_approval`, `has_sufficient_balance`).
#[deprecated(since = "3.3.0", note = "use `batch_token_state` instead")]
pub async fn batch_token_checks<P>(
    provider: &P,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<(U256, U256)>
where
    P: Provider<Ethereum> + Clone,
{
    let state = batch_token_state(provider, token, owner, spender).await?;
    Ok((state.allowance, state.balance))
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
    ///
    /// Returns `true` if balance >= amount.
    pub fn has_sufficient_balance(&self, amount: U256) -> bool {
        self.balance >= amount
    }
}

/// Fetch token balance and allowance in parallel and return them as a [`TokenState`].
///
/// Use this when you want predicate helpers (`can_transfer`, `needs_approval`,
/// `has_sufficient_balance`) over the raw values.
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
    let erc20 = Erc20Contract::new(token, provider.clone());

    let (allowance, balance) =
        tokio::join!(erc20.allowance(owner, spender), erc20.balance_of(owner));

    Ok(TokenState {
        balance: balance?,
        allowance: allowance?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_state_predicates() {
        let state = TokenState {
            balance: U256::from(1000),
            allowance: U256::from(500),
        };

        assert!(state.can_transfer(U256::from(500)));
        assert!(state.can_transfer(U256::from(100)));
        assert!(!state.can_transfer(U256::from(501)));
        assert!(!state.can_transfer(U256::from(1001)));

        assert!(!state.needs_approval(U256::from(500)));
        assert!(state.needs_approval(U256::from(501)));

        assert!(state.has_sufficient_balance(U256::from(1000)));
        assert!(!state.has_sufficient_balance(U256::from(1001)));

        let no_allowance = TokenState {
            balance: U256::from(1000),
            allowance: U256::ZERO,
        };
        assert!(!no_allowance.can_transfer(U256::from(1)));
        assert!(no_allowance.needs_approval(U256::from(1)));
        assert!(no_allowance.has_sufficient_balance(U256::from(1000)));
    }
}
