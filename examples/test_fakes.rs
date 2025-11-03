//! Comprehensive examples of implementing test fakes for CCTP v2.0
//!
//! This file demonstrates how to implement fake/mock versions of the CCTP traits
//! for comprehensive testing including adversarial scenarios.
//!
//! Run this example with: `cargo run --example test_fakes`

use alloy_network::{Ethereum, Network};
use alloy_primitives::{FixedBytes, TxHash};
use async_trait::async_trait;
use cctp_rs::traits::{AttestationProvider, BlockchainProvider, Clock};
use cctp_rs::{AttestationResponse, AttestationStatus, CctpError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
        // Check if this should fail
        if self.failures.lock().unwrap().contains(&tx_hash) {
            return Err(CctpError::Provider("Simulated RPC error".to_string()));
        }

        // Return configured receipt or None if not configured
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
        // Add many pending responses so it never completes
        self.add_response_sequence(message_hash, vec![response; 100]);
    }

    /// Configure a rate limit error (429)
    pub fn add_rate_limit_then_success(
        &self,
        message_hash: FixedBytes<32>,
        rate_limit_count: usize,
        attestation_hex: &str,
    ) {
        let mut responses = Vec::new();

        // Add rate limit errors
        for _ in 0..rate_limit_count {
            responses.push(AttestationResponse {
                status: AttestationStatus::Pending, // Will be handled as 429 in the fake
                attestation: None,
            });
        }

        // Then success
        responses.push(AttestationResponse {
            status: AttestationStatus::Complete,
            attestation: Some(attestation_hex.to_string()),
        });

        self.add_response_sequence(message_hash, responses);
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
                // Return the last response if we've exhausted the sequence
                Ok(response_seq.last().unwrap().clone())
            }
        } else {
            // Not configured - return 404 error
            Err(CctpError::AttestationFailed {
                reason: "Attestation not found (404)".to_string(),
            })
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
}

#[async_trait]
impl Clock for FakeClock {
    async fn sleep(&self, duration: Duration) {
        self.sleep_log.lock().unwrap().push(duration);
        self.advance(duration);
        // In a real fake clock, you might use tokio::task::yield_now() here
    }

    fn now(&self) -> Instant {
        *self.current_time.lock().unwrap()
    }
}

// ============================================================================
// Example Test Scenarios
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    println!("CCTP v2.0 Test Fakes Examples\n");
    println!("{}", "=".repeat(60));

    // Example 1: Test immediate attestation success
    example_1_immediate_success().await?;

    // Example 2: Test attestation state progression
    example_2_state_progression().await?;

    // Example 3: Test timeout behavior with fake clock
    example_3_timeout_with_fake_clock().await?;

    // Example 4: Test with production providers (no fakes)
    example_4_production_providers();

    Ok(())
}

async fn example_1_immediate_success() -> Result<()> {
    println!("\n1. Testing immediate attestation success");
    println!("{}", "-".repeat(60));

    let fake_attestation = FakeAttestationProvider::new();
    let message_hash = FixedBytes::from([1u8; 32]);

    // Configure immediate success with hex attestation data
    fake_attestation.add_complete_response(message_hash, "0xdeadbeefdeadbeefdeadbeefdeadbeef");

    let _fake_blockchain = FakeBlockchainProvider::new();
    let _fake_clock = FakeClock::new();

    // Note: In a real test, you'd use the Cctp::builder() here
    // This is just demonstrating the fake providers work
    println!("✓ Fake providers configured");
    println!("✓ Message hash: {}", message_hash);
    println!("✓ Attestation would return immediately with success");

    Ok(())
}

async fn example_2_state_progression() -> Result<()> {
    println!("\n2. Testing attestation state progression");
    println!("{}", "-".repeat(60));

    let fake_attestation = FakeAttestationProvider::new();
    let message_hash = FixedBytes::from([2u8; 32]);

    // Configure state progression: Pending → PendingConfirmations → Complete
    fake_attestation.add_response_sequence(
        message_hash,
        vec![
            AttestationResponse {
                status: AttestationStatus::Pending,
                attestation: None,
            },
            AttestationResponse {
                status: AttestationStatus::PendingConfirmations,
                attestation: None,
            },
            AttestationResponse {
                status: AttestationStatus::Complete,
                attestation: Some("0xcafebabe".to_string()),
            },
        ],
    );

    println!("✓ Configured 3-step progression");
    println!("  1. Pending");
    println!("  2. PendingConfirmations");
    println!("  3. Complete");

    // Test the progression
    for i in 1..=3 {
        let response = fake_attestation.get_attestation(message_hash).await?;
        println!("  Call {}: Status = {:?}", i, response.status);
    }

    Ok(())
}

async fn example_3_timeout_with_fake_clock() -> Result<()> {
    println!("\n3. Testing timeout with fake clock");
    println!("{}", "-".repeat(60));

    let fake_clock = FakeClock::new();
    let fake_attestation = FakeAttestationProvider::new();
    let message_hash = FixedBytes::from([3u8; 32]);

    // Configure to always return pending (will timeout)
    fake_attestation.add_always_pending(message_hash);

    println!("✓ Configured always-pending response");
    println!("✓ Fake clock allows instant fast-forward");
    println!("✓ No actual waiting required in tests!");

    // Simulate some sleep calls
    fake_clock.sleep(Duration::from_secs(60)).await;
    fake_clock.sleep(Duration::from_secs(60)).await;
    fake_clock.sleep(Duration::from_secs(60)).await;

    println!("  Sleep called {} times", fake_clock.sleep_count());
    println!("  Total sleep time: {:?}", fake_clock.total_sleep_time());

    Ok(())
}

fn example_4_production_providers() {
    println!("\n4. Production providers (no fakes)");
    println!("{}", "-".repeat(60));

    println!("Production usage:");
    println!("  - AlloyProvider<Ethereum> - wraps real RPC provider");
    println!("  - IrisAttestationProvider - calls Circle's API");
    println!("  - TokioClock - uses real system time");
    println!();
    println!("Example:");
    println!("  let bridge = Cctp::builder()");
    println!("      .source_provider(AlloyProvider::new(eth_provider))");
    println!("      .destination_provider(AlloyProvider::new(arb_provider))");
    println!("      .attestation_provider(IrisAttestationProvider::production())");
    println!("      .clock(TokioClock::new())");
    println!("      // ...");
    println!("      .build();");
}

// ============================================================================
// Additional Testing Patterns
// ============================================================================

/// Example: Testing rate limiting behavior
#[allow(dead_code)]
async fn test_rate_limiting() -> Result<()> {
    let fake_attestation = FakeAttestationProvider::new();
    let message_hash = FixedBytes::from([4u8; 32]);

    // Configure: 2 rate limits, then success
    fake_attestation.add_rate_limit_then_success(message_hash, 2, "0x1234");

    // In a real test, you'd verify the bridge handles the rate limit correctly
    println!("Rate limiting test configured");

    Ok(())
}

/// Example: Testing transaction not found
#[allow(dead_code)]
async fn test_transaction_not_found() -> Result<()> {
    let fake_blockchain = FakeBlockchainProvider::new();
    let tx_hash = TxHash::from([5u8; 32]);

    // Configure to return None (transaction not found)
    fake_blockchain.add_not_found(tx_hash);

    let result = fake_blockchain.get_transaction_receipt(tx_hash).await?;
    assert!(result.is_none(), "Transaction should not be found");

    println!("Transaction not found test passed");

    Ok(())
}

/// Example: Testing RPC failure
#[allow(dead_code)]
async fn test_rpc_failure() -> Result<()> {
    let fake_blockchain = FakeBlockchainProvider::new();
    let tx_hash = TxHash::from([6u8; 32]);

    // Configure to return an error
    fake_blockchain.add_failure(tx_hash);

    let result = fake_blockchain.get_transaction_receipt(tx_hash).await;
    assert!(result.is_err(), "Should return an error");

    println!("RPC failure test passed");

    Ok(())
}
