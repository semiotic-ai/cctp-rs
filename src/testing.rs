//! Test utilities and fake implementations for testing CCTP v2.0
//!
//! This module provides fake/mock implementations of the CCTP traits that enable
//! comprehensive testing including adversarial scenarios without requiring actual
//! blockchain or API interactions.
//!
//! These fakes are designed to be used in integration tests to verify the behavior
//! of the `Cctp` bridge under various conditions like timeouts, rate limiting,
//! transaction failures, and attestation state progressions.

use alloy_network::{Ethereum, Network};
use alloy_primitives::{FixedBytes, TxHash};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::traits::{AttestationProvider, BlockchainProvider, Clock};
use crate::{AttestationResponse, AttestationStatus, CctpError, Result};

// ============================================================================
// Fake Blockchain Provider
// ============================================================================

/// A fake blockchain provider that returns pre-configured transaction receipts.
///
/// This allows testing scenarios like:
/// - Transaction not found
/// - Transaction found but no MessageSent event
/// - Malformed event data
/// - Delayed responses
#[derive(Clone, Debug, Default)]
pub struct FakeBlockchainProvider {
    receipts: Arc<Mutex<HashMap<TxHash, Option<<Ethereum as Network>::ReceiptResponse>>>>,
    failures: Arc<Mutex<Vec<TxHash>>>,
}

impl FakeBlockchainProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transaction receipt that will be returned for the given hash
    pub fn add_receipt(&self, tx_hash: TxHash, receipt: <Ethereum as Network>::ReceiptResponse) {
        self.receipts.lock().unwrap().insert(tx_hash, Some(receipt));
    }

    /// Configure a transaction hash to return None (not found)
    pub fn add_not_found(&self, tx_hash: TxHash) {
        self.receipts.lock().unwrap().insert(tx_hash, None);
    }

    /// Configure a transaction hash to return an error
    pub fn add_failure(&self, tx_hash: TxHash) {
        self.failures.lock().unwrap().push(tx_hash);
    }
}

#[async_trait]
impl BlockchainProvider<Ethereum> for FakeBlockchainProvider {
    async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<<Ethereum as Network>::ReceiptResponse>> {
        if self.failures.lock().unwrap().contains(&tx_hash) {
            return Err(CctpError::Provider("Simulated RPC error".to_string()));
        }

        Ok(self
            .receipts
            .lock()
            .unwrap()
            .get(&tx_hash)
            .cloned()
            .unwrap_or(None))
    }

    async fn get_block_number(&self) -> Result<u64> {
        Ok(12345)
    }
}

// ============================================================================
// Fake Attestation Provider
// ============================================================================

/// A fake attestation provider that simulates various API behaviors.
///
/// This allows testing scenarios like:
/// - Immediate success
/// - Pending → PendingConfirmations → Complete progression
/// - Rate limiting (429)
/// - Not found (404)
/// - Failed attestations
/// - Timeout scenarios
#[derive(Clone, Debug, Default)]
pub struct FakeAttestationProvider {
    responses: Arc<Mutex<HashMap<FixedBytes<32>, Vec<AttestationResponse>>>>,
    response_index: Arc<Mutex<HashMap<FixedBytes<32>, usize>>>,
}

impl FakeAttestationProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure a sequence of responses for a message hash.
    ///
    /// Each call to get_attestation will return the next response in the sequence.
    /// This allows testing state progressions like Pending → Complete.
    pub fn add_response_sequence(
        &self,
        message_hash: FixedBytes<32>,
        responses: Vec<AttestationResponse>,
    ) {
        self.responses
            .lock()
            .unwrap()
            .insert(message_hash, responses);
        self.response_index.lock().unwrap().insert(message_hash, 0);
    }

    /// Configure an immediate complete response with attestation data
    pub fn add_complete_response(&self, message_hash: FixedBytes<32>, attestation_hex: &str) {
        let response = AttestationResponse {
            status: AttestationStatus::Complete,
            attestation: Some(attestation_hex.to_string()),
        };
        self.add_response_sequence(message_hash, vec![response]);
    }

    /// Configure an immediate failed response
    pub fn add_failed_response(&self, message_hash: FixedBytes<32>) {
        let response = AttestationResponse {
            status: AttestationStatus::Failed,
            attestation: None,
        };
        self.add_response_sequence(message_hash, vec![response]);
    }

    /// Configure a pending response that will never complete (for timeout testing)
    pub fn add_always_pending(&self, message_hash: FixedBytes<32>) {
        let response = AttestationResponse {
            status: AttestationStatus::Pending,
            attestation: None,
        };
        self.add_response_sequence(message_hash, vec![response; 100]);
    }

    /// Configure a rate limit error (429) followed by success
    pub fn add_rate_limit_then_success(
        &self,
        message_hash: FixedBytes<32>,
        rate_limit_count: usize,
        attestation_hex: &str,
    ) {
        let mut responses = Vec::new();

        for _ in 0..rate_limit_count {
            responses.push(AttestationResponse {
                status: AttestationStatus::Pending,
                attestation: None,
            });
        }

        responses.push(AttestationResponse {
            status: AttestationStatus::Complete,
            attestation: Some(attestation_hex.to_string()),
        });

        self.add_response_sequence(message_hash, responses);
    }

    /// Get the current call count for a message hash
    pub fn get_call_count(&self, message_hash: FixedBytes<32>) -> usize {
        self.response_index
            .lock()
            .unwrap()
            .get(&message_hash)
            .copied()
            .unwrap_or(0)
    }
}

#[async_trait]
impl AttestationProvider for FakeAttestationProvider {
    async fn get_attestation(&self, message_hash: FixedBytes<32>) -> Result<AttestationResponse> {
        let responses = self.responses.lock().unwrap();
        let mut indices = self.response_index.lock().unwrap();

        if let Some(response_seq) = responses.get(&message_hash) {
            let index = indices.get(&message_hash).copied().unwrap_or(0);

            if index < response_seq.len() {
                let response = response_seq[index].clone();
                indices.insert(message_hash, index + 1);
                Ok(response)
            } else {
                Ok(response_seq.last().unwrap().clone())
            }
        } else {
            Err(CctpError::AttestationNotFound)
        }
    }
}

// ============================================================================
// Fake Clock
// ============================================================================

/// A fake clock that allows fast-forwarding time in tests.
///
/// This enables testing timeout behavior without actually waiting.
#[derive(Clone, Debug)]
pub struct FakeClock {
    current_time: Arc<Mutex<Instant>>,
    sleep_log: Arc<Mutex<Vec<Duration>>>,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self {
            current_time: Arc::new(Mutex::new(Instant::now())),
            sleep_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl FakeClock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast-forward the clock by the given duration
    pub fn advance(&self, duration: Duration) {
        let mut time = self.current_time.lock().unwrap();
        *time += duration;
    }

    /// Get the total time "slept" by this clock
    pub fn total_sleep_time(&self) -> Duration {
        self.sleep_log.lock().unwrap().iter().sum()
    }

    /// Get the number of times sleep was called
    pub fn sleep_count(&self) -> usize {
        self.sleep_log.lock().unwrap().len()
    }

    /// Clear the sleep log
    pub fn clear_sleep_log(&self) {
        self.sleep_log.lock().unwrap().clear();
    }
}

#[async_trait]
impl Clock for FakeClock {
    async fn sleep(&self, duration: Duration) {
        self.sleep_log.lock().unwrap().push(duration);
        self.advance(duration);
    }

    fn now(&self) -> Instant {
        *self.current_time.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_clock_tracks_sleep_calls() {
        let clock = FakeClock::new();

        clock.sleep(Duration::from_secs(60)).await;
        clock.sleep(Duration::from_secs(120)).await;

        assert_eq!(clock.sleep_count(), 2);
        assert_eq!(clock.total_sleep_time(), Duration::from_secs(180));
    }

    #[tokio::test]
    async fn test_fake_attestation_provider_sequence() {
        let provider = FakeAttestationProvider::new();
        let message_hash = FixedBytes::from([1u8; 32]);

        provider.add_response_sequence(
            message_hash,
            vec![
                AttestationResponse {
                    status: AttestationStatus::Pending,
                    attestation: None,
                },
                AttestationResponse {
                    status: AttestationStatus::Complete,
                    attestation: Some("0xdeadbeef".to_string()),
                },
            ],
        );

        let first = provider.get_attestation(message_hash).await.unwrap();
        assert!(matches!(first.status, AttestationStatus::Pending));

        let second = provider.get_attestation(message_hash).await.unwrap();
        assert!(matches!(second.status, AttestationStatus::Complete));
    }

    #[tokio::test]
    async fn test_fake_attestation_provider_not_found() {
        let provider = FakeAttestationProvider::new();
        let message_hash = FixedBytes::from([1u8; 32]);

        let result = provider.get_attestation(message_hash).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CctpError::AttestationNotFound
        ));
    }

    #[tokio::test]
    async fn test_fake_blockchain_provider_not_found() {
        let provider = FakeBlockchainProvider::new();
        let tx_hash = TxHash::from([1u8; 32]);

        provider.add_not_found(tx_hash);

        let result = provider.get_transaction_receipt(tx_hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_fake_blockchain_provider_failure() {
        let provider = FakeBlockchainProvider::new();
        let tx_hash = TxHash::from([1u8; 32]);

        provider.add_failure(tx_hash);

        let result = provider.get_transaction_receipt(tx_hash).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CctpError::Provider(_)));
    }
}
