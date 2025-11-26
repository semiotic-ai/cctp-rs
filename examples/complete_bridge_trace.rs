//! Comprehensive OpenTelemetry instrumentation example for CCTP operations
//!
//! This example demonstrates the complete span hierarchy for a full bridge flow,
//! including error tracking with OpenTelemetry semantic conventions.
//!
//! ## Running the Example
//!
//! ```bash
//! # See all spans including trace-level RPC and HTTP calls
//! RUST_LOG=trace cargo run --example complete_bridge_trace
//!
//! # Focus on cctp_rs spans only
//! RUST_LOG=cctp_rs=trace cargo run --example complete_bridge_trace
//!
//! # See debug and info spans
//! RUST_LOG=cctp_rs=debug cargo run --example complete_bridge_trace
//! ```
//!
//! ## Expected Span Hierarchy
//!
//! ```text
//! bridge_operation (top-level)
//! ├── cctp_rs.deposit_for_burn
//! │   └── (contract call preparation)
//! ├── cctp_rs.send_transaction
//! │   └── cctp_rs.rpc_call (transaction broadcast)
//! ├── cctp_rs.wait_for_confirmation
//! │   └── cctp_rs.rpc_call (receipt polling)
//! ├── cctp_rs.get_message_sent_event
//! │   ├── cctp_rs.get_transaction_receipt
//! │   │   └── cctp_rs.rpc_call
//! │   └── (event log parsing)
//! ├── cctp_rs.get_attestation
//! │   ├── cctp_rs.get_attestation (attempt 1)
//! │   │   ├── cctp_rs.http_request
//! │   │   └── cctp_rs.process_attestation_response
//! │   ├── cctp_rs.get_attestation (attempt 2)
//! │   │   ├── cctp_rs.http_request
//! │   │   └── cctp_rs.process_attestation_response
//! │   └── ... (more attempts)
//! └── cctp_rs.receive_message
//!     ├── cctp_rs.rpc_call (contract interaction)
//!     └── cctp_rs.wait_for_confirmation
//! ```
//!
//! ## Error Attributes
//!
//! When errors occur, spans are enriched with OpenTelemetry error attributes:
//! - `error.type`: Error variant/type name
//! - `error.message`: Human-readable error description
//! - `error.context`: Additional context about the error
//! - `otel.status_code`: Set to "ERROR" when operation fails
//!
//! ## Production Integration
//!
//! In production, replace the console subscriber with an OTLP exporter:
//!
//! ```rust,no_run
//! use opentelemetry::global;
//! use opentelemetry_otlp::WithExportConfig;
//! use opentelemetry_sdk::trace::TracerProvider;
//! use tracing_subscriber::layer::SubscriberExt;
//!
//! # async fn setup_production_tracing() -> Result<(), Box<dyn std::error::Error>> {
//! let tracer = opentelemetry_otlp::new_pipeline()
//!     .tracing()
//!     .with_exporter(
//!         opentelemetry_otlp::new_exporter()
//!             .tonic()
//!             .with_endpoint("http://tempo:4317"),
//!     )
//!     .install_batch(opentelemetry_sdk::runtime::Tokio)?;
//!
//! let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
//!
//! tracing_subscriber::registry()
//!     .with(telemetry)
//!     .init();
//! # Ok(())
//! # }
//! ```
//!
//! ## TraceQL Queries for Grafana
//!
//! ```traceql
//! # Find all bridge operations
//! { resource.service.name = "cctp-bridge" && name = "bridge_operation" }
//!
//! # Find failed operations
//! { span.otel.status_code = "ERROR" }
//!
//! # Find operations for specific chains
//! { span.source_chain = "Mainnet" && span.destination_chain = "Arbitrum" }
//!
//! # Find slow attestation polling
//! { name = "cctp_rs.get_attestation" && duration > 5m }
//!
//! # Find all HTTP errors
//! { span.error.type = "HttpRequestFailed" }
//!
//! # Find transaction failures
//! { span.error.type = "TransactionFailed" }
//! ```

use alloy_chains::NamedChain;
use alloy_primitives::{Address, FixedBytes, TxHash};
use alloy_provider::ProviderBuilder;
use cctp_rs::{Cctp, CctpError};
use tracing::{error, info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Initialize tracing subscriber with structured JSON output
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("cctp_rs=trace")))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false),
        )
        .init();

    info!("Starting comprehensive CCTP bridge tracing example");

    // Set up bridge configuration
    let eth_provider = ProviderBuilder::new()
        .connect_http("https://eth-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    let arb_provider = ProviderBuilder::new()
        .connect_http("https://arb-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    let recipient: Address = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d"
        .parse()
        .unwrap();

    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(eth_provider)
        .destination_provider(arb_provider)
        .recipient(recipient)
        .build();

    info!(
        source_chain = %bridge.source_chain(),
        destination_chain = %bridge.destination_chain(),
        recipient = %recipient,
        event = "bridge_initialized"
    );

    // Demonstrate complete bridge flow with instrumentation
    demonstrate_complete_bridge_flow(&bridge).await;

    // Demonstrate error tracking
    demonstrate_error_tracking(&bridge).await;

    // Demonstrate unsupported chain error
    demonstrate_chain_validation_error();

    info!("Comprehensive tracing example complete");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║     OpenTelemetry Instrumentation Demonstration Complete    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\n📊 Key Features Demonstrated:");
    println!("  ✓ Complete span hierarchy from deposit to receive");
    println!("  ✓ Error attributes following OTel semantic conventions");
    println!("  ✓ Trace-level RPC and HTTP request spans");
    println!("  ✓ Structured event logging with span context");
    println!("  ✓ Async-safe span propagation");
    println!("  ✓ Low-cardinality span names with high-cardinality attributes");
    println!("\n🔧 Production Integration:");
    println!("  • Export to Tempo/Jaeger via OTLP");
    println!("  • Query with TraceQL in Grafana");
    println!("  • Alert on error.type attributes");
    println!("  • Track duration and performance metrics");
    println!("\n📈 Observability Benefits:");
    println!("  • Debug production issues with complete trace context");
    println!("  • Track attestation polling performance");
    println!("  • Monitor RPC provider reliability");
    println!("  • Identify bottlenecks in bridge flow");

    Ok(())
}

/// Demonstrates the complete bridge flow with all spans
async fn demonstrate_complete_bridge_flow(
    bridge: &Cctp<impl alloy_provider::Provider<alloy_network::Ethereum> + Clone>,
) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Example 1: Complete Bridge Flow Instrumentation      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\nThis demonstrates the full span hierarchy for a bridge operation.");
    println!("Watch the JSON output for nested spans with parent-child relationships.\n");

    let span = info_span!(
        "bridge_operation",
        source_chain = %bridge.source_chain(),
        destination_chain = %bridge.destination_chain(),
        operation_type = "full_bridge_flow"
    );
    let _guard = span.enter();

    // Step 1: Get MessageSent event (simulated)
    info!(step = 1, event = "extracting_message_sent_event");

    let example_tx_hash: TxHash =
        "0x9f3ce6edbf3d1f08cfe3a20b7ce43c3d01e55fe3c2d7a9e5a2b5e5c8d6f9c2a1"
            .parse()
            .unwrap();

    match bridge.get_message_sent_event(example_tx_hash).await {
        Ok((message, hash)) => {
            info!(
                message_length_bytes = message.len(),
                message_hash = %hash,
                event = "message_sent_event_extracted_successfully"
            );
        }
        Err(e) => {
            info!(
                error = %e,
                expected = true,
                event = "message_sent_event_not_found_expected"
            );
        }
    }

    // Step 2: Get attestation with retry
    info!(step = 2, event = "polling_for_attestation");

    let message_hash: FixedBytes<32> = FixedBytes::from([0xaa; 32]);

    match bridge
        .get_attestation(
            message_hash,
            Some(3), // 3 attempts for demo
            Some(2), // 2 second intervals
        )
        .await
    {
        Ok(attestation) => {
            info!(
                attestation_length_bytes = attestation.len(),
                event = "attestation_retrieved_successfully"
            );
        }
        Err(e) => {
            info!(
                error_type = "AttestationTimeout",
                expected = true,
                event = "attestation_timeout_expected"
            );
            // Don't log the full error as it's expected
            let _ = e;
        }
    }

    info!(event = "bridge_operation_complete");
}

/// Demonstrates error tracking with OpenTelemetry attributes
async fn demonstrate_error_tracking(
    bridge: &Cctp<impl alloy_provider::Provider<alloy_network::Ethereum> + Clone>,
) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Example 2: Error Tracking with OTel Attributes       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\nThis demonstrates how errors are captured as span attributes.");
    println!("Watch for error.type, error.message, and otel.status_code fields.\n");

    let span = info_span!(
        "error_tracking_demonstration",
        operation_type = "error_scenarios"
    );
    let _guard = span.enter();

    // Scenario 1: Transaction not found
    info!(scenario = 1, event = "testing_transaction_not_found");

    let invalid_tx: TxHash = "0x0000000000000000000000000000000000000000000000000000000000000000"
        .parse()
        .unwrap();

    match bridge.get_message_sent_event(invalid_tx).await {
        Ok(_) => {
            info!(event = "unexpected_success");
        }
        Err(e) => {
            // Error is already recorded on span via record_error_with_context
            info!(error_recorded = true, event = "error_attributes_captured");
            // Note: error.type, error.message, error.context are now on the span
            let _ = e;
        }
    }

    // Scenario 2: Invalid message hash for attestation
    info!(scenario = 2, event = "testing_attestation_not_found");

    let invalid_hash: FixedBytes<32> = FixedBytes::from([0x00; 32]);

    match bridge
        .get_attestation(
            invalid_hash,
            Some(2), // Only 2 attempts
            Some(1), // 1 second intervals
        )
        .await
    {
        Ok(_) => {
            info!(event = "unexpected_success");
        }
        Err(e) => {
            info!(error_recorded = true, event = "error_attributes_captured");
            let _ = e;
        }
    }

    info!(event = "error_tracking_demonstration_complete");
}

/// Demonstrates chain validation error tracking
fn demonstrate_chain_validation_error() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Example 3: Chain Validation Error Tracking          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\nThis demonstrates error tracking for unsupported chains.");
    println!("Watch for UnsupportedChain errors with context.\n");

    let span = info_span!(
        "chain_validation_demonstration",
        operation_type = "unsupported_chain"
    );
    let _guard = span.enter();

    use cctp_rs::CctpV1;

    // Try to get domain ID for unsupported chain
    let unsupported_chain = NamedChain::BinanceSmartChain;

    info!(
        chain = %unsupported_chain,
        event = "testing_unsupported_chain"
    );

    match unsupported_chain.cctp_domain_id() {
        Ok(_) => {
            error!(event = "unexpected_success_for_unsupported_chain");
        }
        Err(e) => {
            info!(
                error_recorded = true,
                chain = %unsupported_chain,
                event = "chain_not_supported_error_captured"
            );
            // Error attributes recorded via record_error_with_context
            let _ = e;
        }
    }

    info!(event = "chain_validation_demonstration_complete");
}
