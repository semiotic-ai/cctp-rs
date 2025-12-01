// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! ERC20 contract bindings for approval and allowance operations
//!
//! This module provides utilities for checking and setting ERC20 token allowances,
//! which are required before calling CCTP burn operations.

use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use tracing::{debug, info};

use Erc20::Erc20Instance;

/// ERC20 contract wrapper for approval operations
///
/// Provides methods to check allowances and approve spenders, which is required
/// before initiating CCTP transfers.
///
/// # Example
///
/// ```rust,no_run
/// use cctp_rs::Erc20Contract;
/// use alloy_primitives::{address, U256};
/// use alloy_provider::ProviderBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new().connect("http://localhost:8545").await?;
/// let usdc = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
///
/// let erc20 = Erc20Contract::new(usdc, provider);
///
/// // Check current allowance
/// let owner = address!("1234567890123456789012345678901234567890");
/// let spender = address!("0987654321098765432109876543210987654321");
/// let allowance = erc20.allowance(owner, spender).await?;
///
/// // Approve if needed
/// if allowance < U256::from(1_000_000u64) {
///     let tx = erc20.approve_transaction(owner, spender, U256::from(1_000_000u64));
///     // Send transaction...
/// }
/// # Ok(())
/// # }
/// ```
pub struct Erc20Contract<P: Provider<Ethereum>> {
    instance: Erc20Instance<P>,
}

impl<P: Provider<Ethereum>> Erc20Contract<P> {
    /// Create a new ERC20 contract wrapper
    pub fn new(address: Address, provider: P) -> Self {
        debug!(
            contract_address = %address,
            event = "erc20_contract_initialized"
        );
        Self {
            instance: Erc20Instance::new(address, provider),
        }
    }

    /// Get the current allowance for a spender
    ///
    /// Returns the amount of tokens that `spender` is allowed to spend on behalf of `owner`.
    ///
    /// # Arguments
    ///
    /// * `owner` - The address that owns the tokens
    /// * `spender` - The address that is allowed to spend the tokens
    ///
    /// # Returns
    ///
    /// The current allowance as a `U256`
    pub async fn allowance(
        &self,
        owner: Address,
        spender: Address,
    ) -> Result<U256, alloy_contract::Error> {
        debug!(
            owner = %owner,
            spender = %spender,
            contract_address = %self.instance.address(),
            event = "checking_allowance"
        );

        let result = self.instance.allowance(owner, spender).call().await?;

        info!(
            owner = %owner,
            spender = %spender,
            allowance = %result,
            contract_address = %self.instance.address(),
            event = "allowance_retrieved"
        );

        Ok(result)
    }

    /// Create a transaction request to approve a spender
    ///
    /// This creates but does not send the approval transaction. The caller is
    /// responsible for signing and sending the transaction.
    ///
    /// # Arguments
    ///
    /// * `from` - The address that owns the tokens and will sign the transaction
    /// * `spender` - The address to approve for spending
    /// * `amount` - The amount to approve
    ///
    /// # Returns
    ///
    /// A `TransactionRequest` ready to be signed and sent
    pub fn approve_transaction(
        &self,
        from: Address,
        spender: Address,
        amount: U256,
    ) -> TransactionRequest {
        info!(
            from = %from,
            spender = %spender,
            amount = %amount,
            contract_address = %self.instance.address(),
            event = "approve_transaction_created"
        );

        self.instance
            .approve(spender, amount)
            .from(from)
            .into_transaction_request()
    }

    /// Get the token balance of an address
    ///
    /// # Arguments
    ///
    /// * `account` - The address to check the balance of
    ///
    /// # Returns
    ///
    /// The token balance as a `U256`
    pub async fn balance_of(&self, account: Address) -> Result<U256, alloy_contract::Error> {
        debug!(
            account = %account,
            contract_address = %self.instance.address(),
            event = "checking_balance"
        );

        let result = self.instance.balanceOf(account).call().await?;

        info!(
            account = %account,
            balance = %result,
            contract_address = %self.instance.address(),
            event = "balance_retrieved"
        );

        Ok(result)
    }

    /// Returns the contract address
    pub fn address(&self) -> Address {
        *self.instance.address()
    }
}

// Minimal ERC20 interface for approval operations
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract Erc20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
    }
);
