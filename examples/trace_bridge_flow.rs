// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Example demonstrating OpenTelemetry span instrumentation for CCTP operations
//!
//! This example shows how spans are automatically created for bridge operations,
//! following production observability best practices.
//!
//! Run with tracing enabled:
//! ```bash
//! RUST_LOG=cctp_rs=debug cargo run --example trace_bridge_flow
//! ```
//!
//! For full trace output including all spans:
//! ```bash
//! RUST_LOG=trace cargo run --example trace_bridge_flow
//! ```

use alloy_chains::NamedChain;
use alloy_primitives::{FixedBytes, TxHash};
use alloy_provider::ProviderBuilder;
use cctp_rs::{Cctp, CctpError};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Initialize tracing subscriber with environment filter
    // This will output structured logs with span context
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false),
        )
        .init();

    tracing::info!("Starting CCTP bridge tracing example");

    // Set up bridge configuration
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

    tracing::info!(
        source_chain = %bridge.source_chain(),
        destination_chain = %bridge.destination_chain(),
        "Bridge initialized"
    );

    // Example 1: Demonstrate get_message_sent_event span
    println!("\n=== Example 1: Getting MessageSent event ===");
    println!("This operation creates a span: cctp_rs.get_message_sent_event");
    println!("With attributes: tx_hash, source_chain, destination_chain\n");

    let example_tx_hash: TxHash =
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .parse()
            .unwrap();

    // This will fail since it's not a real tx, but demonstrates the span creation
    match bridge.get_message_sent_event(example_tx_hash).await {
        Ok((message, hash)) => {
            tracing::info!(
                message_length = message.len(),
                message_hash = %hash,
                "Successfully extracted MessageSent event"
            );
        }
        Err(e) => {
            // Expected to fail for demo purposes
            tracing::warn!(error = %e, "Example transaction not found (expected)");
        }
    }

    // Example 2: Demonstrate attestation polling spans
    println!("\n=== Example 2: Attestation polling ===");
    println!("This operation creates multiple nested spans:");
    println!("  - cctp_rs.get_attestation_with_retry (parent)");
    println!("    ├─ cctp_rs.get_attestation (for each attempt)");
    println!("    └─ cctp_rs.process_attestation_response\n");

    let message_hash: FixedBytes<32> = FixedBytes::from([42u8; 32]);

    // Use minimal retries for demo purposes
    match bridge
        .get_attestation(
            message_hash,
            cctp_rs::PollingConfig::default()
                .with_max_attempts(2)
                .with_poll_interval_secs(5),
        )
        .await
    {
        Ok(attestation) => {
            tracing::info!(
                attestation_length = attestation.len(),
                "Successfully retrieved attestation"
            );
        }
        Err(e) => {
            // Expected to fail for demo purposes
            tracing::warn!(error = %e, "Attestation not found (expected for example data)");
        }
    }

    println!("\n=== Span Hierarchy Demonstration Complete ===");
    println!("\nKey Observability Features Demonstrated:");
    println!("✓ Static span names: cctp_rs.operation_name");
    println!("✓ Structured attributes: All dynamic data in fields");
    println!("✓ Async-safe: Spans propagate across .await boundaries");
    println!("✓ Parent-child relationships: Nested spans for complex operations");
    println!("✓ Low cardinality: Span names are static, data in attributes");

    println!("\n📊 In production, these spans would be:");
    println!("  • Exported to Tempo/Jaeger via OTLP");
    println!("  • Queryable via TraceQL");
    println!("  • Visualized in Grafana dashboards");

    println!("\n🔍 Example TraceQL queries:");
    println!("  # Find all attestation polling operations");
    println!("  {{ resource.service.name = \"cctp-rs\" && name = \"cctp_rs.get_attestation\" }}");
    println!("\n  # Find operations for specific chain pair");
    println!("  {{ span.source_chain = \"Mainnet\" && span.destination_chain = \"Arbitrum\" }}");

    Ok(())
}
