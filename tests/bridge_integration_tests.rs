//! Integration tests for CCTP bridge using fake implementations
//!
//! These tests demonstrate the core value proposition of v2.0: comprehensive
//! testability through trait-based design and fake implementations.

use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Address, FixedBytes};
use cctp_rs::testing::{FakeAttestationProvider, FakeBlockchainProvider, FakeClock};
use cctp_rs::{AttestationResponse, AttestationStatus, Cctp, CctpError, UniversalReceiptAdapter};
use std::time::Duration;

/// Helper function to create a test bridge with fake providers
fn create_test_bridge(
    attestation_provider: FakeAttestationProvider,
    clock: FakeClock,
) -> Cctp<
    Ethereum,
    Ethereum,
    FakeBlockchainProvider,
    FakeBlockchainProvider,
    FakeAttestationProvider,
    FakeClock,
    UniversalReceiptAdapter,
> {
    Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(FakeBlockchainProvider::new())
        .destination_provider(FakeBlockchainProvider::new())
        .attestation_provider(attestation_provider)
        .clock(clock)
        .receipt_adapter(UniversalReceiptAdapter)
        .recipient(Address::ZERO)
        .build()
}

#[tokio::test]
async fn test_attestation_timeout_with_fake_clock() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([1u8; 32]);

    fake_attestation.add_always_pending(message_hash);

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let max_attempts = 5;
    let poll_interval = 60;

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(max_attempts), Some(poll_interval))
        .await;

    assert!(result.is_err(), "Expected timeout error");
    assert!(
        matches!(result.unwrap_err(), CctpError::AttestationTimeout),
        "Expected AttestationTimeout error"
    );

    assert_eq!(
        fake_clock.sleep_count(),
        max_attempts as usize,
        "Should have slept max_attempts times"
    );

    let expected_sleep = Duration::from_secs(poll_interval * max_attempts as u64);
    assert_eq!(
        fake_clock.total_sleep_time(),
        expected_sleep,
        "Total sleep time should match poll_interval * max_attempts"
    );

    assert_eq!(
        fake_attestation.get_call_count(message_hash),
        max_attempts as usize,
        "Should have called attestation provider max_attempts times"
    );
}

#[tokio::test]
async fn test_rate_limiting_backoff() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([2u8; 32]);

    fake_attestation.add_response_sequence(
        message_hash,
        vec![
            AttestationResponse {
                status: AttestationStatus::Pending,
                attestation: None,
            },
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

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(10), Some(30))
        .await;

    assert!(result.is_ok(), "Should eventually succeed");

    let attestation_bytes = result.unwrap();
    assert!(
        !attestation_bytes.is_empty(),
        "Should return attestation bytes"
    );

    assert_eq!(
        fake_attestation.get_call_count(message_hash),
        3,
        "Should have made 3 calls: 2 pending + 1 complete"
    );

    assert_eq!(
        fake_clock.sleep_count(),
        2,
        "Should have slept twice (once after each pending response)"
    );
}

#[tokio::test]
async fn test_transaction_not_found() {
    let fake_blockchain = FakeBlockchainProvider::new();
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let tx_hash = FixedBytes::from([3u8; 32]);

    fake_blockchain.add_not_found(tx_hash);

    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(fake_blockchain)
        .destination_provider(FakeBlockchainProvider::new())
        .attestation_provider(fake_attestation)
        .clock(fake_clock)
        .receipt_adapter(UniversalReceiptAdapter)
        .recipient(Address::ZERO)
        .build();

    let result = bridge.get_message_sent_event(tx_hash).await;

    assert!(
        result.is_err(),
        "Should return error for missing transaction"
    );
    assert!(
        matches!(result.unwrap_err(), CctpError::TransactionFailed { .. }),
        "Should return TransactionFailed error"
    );
}

#[tokio::test]
async fn test_attestation_state_progression() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([4u8; 32]);

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

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(10), Some(5))
        .await;

    assert!(result.is_ok(), "Should eventually complete");

    let attestation_bytes = result.unwrap();
    assert_eq!(
        hex::encode(&attestation_bytes),
        "cafebabe",
        "Should return correct attestation bytes"
    );

    assert_eq!(
        fake_attestation.get_call_count(message_hash),
        3,
        "Should progress through 3 states: Pending → PendingConfirmations → Complete"
    );

    assert_eq!(
        fake_clock.sleep_count(),
        2,
        "Should sleep twice (after Pending and PendingConfirmations)"
    );

    assert_eq!(
        fake_clock.total_sleep_time(),
        Duration::from_secs(10),
        "Should have slept for 2 * 5 seconds"
    );
}

#[tokio::test]
async fn test_attestation_failed_status() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([5u8; 32]);

    fake_attestation.add_failed_response(message_hash);

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(10), Some(5))
        .await;

    assert!(
        result.is_err(),
        "Should return error for failed attestation"
    );
    assert!(
        matches!(result.unwrap_err(), CctpError::AttestationFailed { .. }),
        "Should return AttestationFailed error"
    );

    assert_eq!(
        fake_attestation.get_call_count(message_hash),
        1,
        "Should only call once before failing"
    );

    assert_eq!(
        fake_clock.sleep_count(),
        0,
        "Should not sleep if failed immediately"
    );
}

#[tokio::test]
async fn test_attestation_immediate_success() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([6u8; 32]);

    fake_attestation.add_complete_response(message_hash, "0x1234567890abcdef");

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(10), Some(5))
        .await;

    assert!(result.is_ok(), "Should succeed immediately");

    let attestation_bytes = result.unwrap();
    assert_eq!(
        hex::encode(&attestation_bytes),
        "1234567890abcdef",
        "Should return correct attestation bytes"
    );

    assert_eq!(
        fake_attestation.get_call_count(message_hash),
        1,
        "Should only call once for immediate success"
    );

    assert_eq!(
        fake_clock.sleep_count(),
        0,
        "Should not sleep if successful immediately"
    );
}

#[tokio::test]
async fn test_attestation_not_found_then_timeout() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let message_hash = FixedBytes::from([7u8; 32]);

    // Don't configure any response - will return AttestationNotFound on every call

    let bridge = create_test_bridge(fake_attestation.clone(), fake_clock.clone());

    let result = bridge
        .get_attestation_with_retry(message_hash, Some(5), Some(10))
        .await;

    assert!(
        result.is_err(),
        "Should timeout when attestation never exists"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, CctpError::AttestationTimeout),
        "Should return timeout error after max attempts with 404s, got: {:?}",
        err
    );

    assert_eq!(
        fake_clock.sleep_count(),
        5,
        "Should sleep after each 404 response"
    );

    assert_eq!(
        fake_clock.total_sleep_time(),
        Duration::from_secs(50),
        "Should have slept for 5 * 10 seconds"
    );
}

#[tokio::test]
async fn test_provider_rpc_failure() {
    let fake_blockchain = FakeBlockchainProvider::new();
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();
    let tx_hash = FixedBytes::from([8u8; 32]);

    fake_blockchain.add_failure(tx_hash);

    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(fake_blockchain)
        .destination_provider(FakeBlockchainProvider::new())
        .attestation_provider(fake_attestation)
        .clock(fake_clock)
        .receipt_adapter(UniversalReceiptAdapter)
        .recipient(Address::ZERO)
        .build();

    let result = bridge.get_message_sent_event(tx_hash).await;

    assert!(result.is_err(), "Should return error for RPC failure");
    assert!(
        matches!(result.unwrap_err(), CctpError::Provider(_)),
        "Should return Provider error"
    );
}

#[tokio::test]
async fn test_hex_prefix_handling() {
    let fake_attestation = FakeAttestationProvider::new();
    let fake_clock = FakeClock::new();

    // Test with 0x prefix
    let message_hash_with_prefix = FixedBytes::from([9u8; 32]);
    fake_attestation.add_complete_response(message_hash_with_prefix, "0xabcdef");

    // Test without 0x prefix
    let message_hash_without_prefix = FixedBytes::from([10u8; 32]);
    fake_attestation.add_complete_response(message_hash_without_prefix, "123456");

    let bridge = create_test_bridge(fake_attestation, fake_clock);

    let result_with_prefix = bridge
        .get_attestation_with_retry(message_hash_with_prefix, Some(5), Some(5))
        .await
        .unwrap();

    let result_without_prefix = bridge
        .get_attestation_with_retry(message_hash_without_prefix, Some(5), Some(5))
        .await
        .unwrap();

    assert_eq!(hex::encode(&result_with_prefix), "abcdef");
    assert_eq!(hex::encode(&result_without_prefix), "123456");
}
