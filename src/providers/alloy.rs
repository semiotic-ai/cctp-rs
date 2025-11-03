//! Alloy-based blockchain provider implementation.

use alloy_network::Network;
use alloy_primitives::TxHash;
use alloy_provider::Provider;
use async_trait::async_trait;
use tracing::{debug, instrument, trace};

use crate::error::{CctpError, Result};
use crate::traits::BlockchainProvider;

/// Production blockchain provider wrapping Alloy's [`Provider`] trait.
///
/// This struct adapts Alloy's provider interface to our [`BlockchainProvider`]
/// trait, enabling uniform access to blockchain operations across different
/// networks (Ethereum, Optimism, etc.).
///
/// # Type Parameters
///
/// - `N`: The network type (e.g., `Ethereum`, `Optimism`)
/// - `P`: The underlying Alloy provider implementation
///
/// # Examples
///
/// ```rust,no_run
/// use cctp_rs::providers::AlloyProvider;
/// use alloy_provider::ProviderBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = ProviderBuilder::new()
///     .on_builtin("https://eth.llamarpc.com")
///     .await?;
///
/// let blockchain_provider = AlloyProvider::new(provider);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AlloyProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    provider: P,
    _network: std::marker::PhantomData<N>,
}

impl<N, P> AlloyProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    /// Creates a new [`AlloyProvider`] wrapping the given Alloy provider.
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            _network: std::marker::PhantomData,
        }
    }

    /// Returns a reference to the underlying Alloy provider.
    pub fn inner(&self) -> &P {
        &self.provider
    }
}

#[async_trait]
impl<N, P> BlockchainProvider<N> for AlloyProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone + Send + Sync,
{
    #[instrument(skip(self), fields(tx_hash = %tx_hash))]
    async fn get_transaction_receipt(&self, tx_hash: TxHash) -> Result<Option<N::ReceiptResponse>> {
        trace!("Fetching transaction receipt");
        let result = self
            .provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| CctpError::Provider(e.to_string()))?;

        if result.is_some() {
            debug!("Transaction receipt found");
        } else {
            debug!("Transaction receipt not found");
        }

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn get_block_number(&self) -> Result<u64> {
        trace!("Fetching current block number");
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| CctpError::Provider(e.to_string()))?;

        debug!(
            block_number = block_number,
            "Current block number retrieved"
        );
        Ok(block_number)
    }
}
