// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Example of monitoring attestation status
//!
//! This example shows how to monitor the attestation process and handle different states.
//!
//! Run with: `cargo run --example attestation_monitoring`

use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::{hex, Bytes, FixedBytes, TxHash};
use alloy_provider::{Provider, ProviderBuilder};
use cctp_rs::{AttestationResponse, AttestationStatus, Cctp, CctpError};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    println!("📊 CCTP Attestation Monitoring Example");
    println!("=====================================\n");

    // Set up a basic bridge configuration
    let eth_provider = ProviderBuilder::new()
        .connect_http("https://eth-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    let arb_provider = ProviderBuilder::new()
        .connect_http("https://arb-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(eth_provider)
        .destination_provider(arb_provider)
        .recipient(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d"
                .parse()
                .unwrap(),
        )
        .build();

    // Example: Monitoring attestation for a transaction
    // In a real scenario, this would be a real transaction hash
    let example_tx_hash: TxHash =
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .parse()
            .unwrap();

    println!("🔍 Monitoring attestation for transaction: {example_tx_hash}");

    // Simulate getting a message hash (in reality, this comes from the transaction receipt)
    let message_hash: FixedBytes<32> = FixedBytes::from([42u8; 32]);

    // Show how to poll for attestation with custom parameters
    println!("\n⏳ Polling for attestation...");
    println!("   Max attempts: 10");
    println!("   Poll interval: 30 seconds");

    // Demonstrate manual attestation checking
    let api_url = bridge.create_url(message_hash)?;
    println!("\n🌐 Attestation API URL: {}", api_url.as_str());

    // Simulate attestation status monitoring
    simulate_attestation_monitoring(&bridge).await;

    // Show how to use custom polling parameters
    println!("\n💡 Custom polling example:");
    println!("```rust");
    println!("let attestation = bridge.get_attestation(");
    println!("    &message_hash,");
    println!("    Some(10),  // max attempts");
    println!("    Some(30),  // poll interval in seconds");
    println!(").await?;");
    println!("```");

    Ok(())
}

/// Simulates monitoring attestation status changes
async fn simulate_attestation_monitoring(_bridge: &Cctp<impl Provider<Ethereum> + Clone>) {
    println!("\n📈 Simulating attestation status progression:");

    let statuses = [
        (
            AttestationStatus::Pending,
            "Transaction submitted, waiting for confirmations",
        ),
        (
            AttestationStatus::PendingConfirmations,
            "Transaction confirmed, waiting for attestation",
        ),
        (
            AttestationStatus::Complete,
            "Attestation ready! Can now mint on destination",
        ),
    ];

    for (i, (status, description)) in statuses.iter().enumerate() {
        sleep(Duration::from_secs(1)).await;

        println!("\n   Step {}: {:?}", i + 1, status);
        println!("   └─ {description}");

        match status {
            AttestationStatus::Pending => {
                println!("      ⏳ Waiting for block confirmations...");
            }
            AttestationStatus::PendingConfirmations => {
                println!("      🔄 Circle is processing the attestation...");
            }
            AttestationStatus::Complete => {
                println!("      ✅ Ready to complete bridge on destination chain!");

                // Show example attestation response
                let example_response = AttestationResponse {
                    status: AttestationStatus::Complete,
                    attestation: Some(Bytes::from_static(b"\xab\xcd\xef")),
                };

                println!("\n   📄 Example attestation response:");
                println!("      Status: {:?}", example_response.status);
                println!(
                    "      Attestation: 0x{}",
                    hex::encode(example_response.attestation.as_ref().unwrap())
                );
            }
            AttestationStatus::Failed => {
                println!("      ❌ Attestation failed - check transaction");
            }
        }
    }

    println!("\n🎯 Monitoring complete!");
}

// Note: In a production environment, you would implement proper error handling
// and retry logic for failed attestations.
