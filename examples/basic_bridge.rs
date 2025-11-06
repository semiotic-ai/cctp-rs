//! Basic example of bridging USDC using cctp-rs
//!
//! This example demonstrates the basic flow of bridging USDC from Ethereum to Arbitrum.
//!
//! Run with: `cargo run --example basic_bridge`

use alloy_chains::NamedChain;
use alloy_primitives::Address;
use alloy_provider::ProviderBuilder;
use cctp_rs::{Cctp, CctpError, CctpV1};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    println!("🌉 CCTP Bridge Example - Ethereum to Arbitrum");
    println!("============================================\n");

    // Step 1: Set up providers
    println!("1️⃣ Setting up providers...");

    // In a real application, use your actual RPC endpoints
    let eth_provider = ProviderBuilder::new()
        .connect_http("https://eth-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    let arb_provider = ProviderBuilder::new()
        .connect_http("https://arb-mainnet.g.alchemy.com/v2/demo".parse().unwrap());

    // Step 2: Create the CCTP bridge
    println!("2️⃣ Creating CCTP bridge...");

    // Example recipient address (replace with actual address)
    let recipient = Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d")?;

    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(eth_provider)
        .destination_provider(arb_provider)
        .recipient(recipient)
        .build();

    // Step 3: Display bridge configuration
    println!("3️⃣ Bridge Configuration:");
    println!("   Source Chain: {}", bridge.source_chain());
    println!("   Destination Chain: {}", bridge.destination_chain());
    println!("   Recipient: {}", bridge.recipient());

    // Get important contract addresses
    let token_messenger = bridge.token_messenger_contract()?;
    let message_transmitter = bridge.message_transmitter_contract()?;
    let destination_domain = bridge.destination_domain_id()?;

    println!("\n📋 Contract Information:");
    println!("   Token Messenger (source): {token_messenger}");
    println!("   Message Transmitter (dest): {message_transmitter}");
    println!("   Destination Domain ID: {destination_domain}");

    // Step 4: Display chain information
    println!("\n🔗 Chain Information:");

    let source_chain = bridge.source_chain();
    let dest_chain = bridge.destination_chain();

    println!(
        "   {} confirmation time: {} seconds",
        source_chain,
        source_chain.confirmation_average_time_seconds()?
    );
    println!(
        "   {} confirmation time: {} seconds",
        dest_chain,
        dest_chain.confirmation_average_time_seconds()?
    );

    // Step 5: Show how to bridge USDC (without actually executing)
    println!("\n💸 To bridge USDC:");
    println!("   1. Approve USDC spending to Token Messenger contract");
    println!("   2. Call depositForBurn() on Token Messenger");
    println!("   3. Wait for transaction confirmation");
    println!("   4. Get attestation from Circle's API");
    println!("   5. Call receiveMessage() on destination chain");

    // Example of how to get attestation URL
    let example_message_hash = [0u8; 32].into();
    let attestation_url = bridge.iris_api_url(&example_message_hash);
    println!(
        "\n🔍 Attestation API endpoint: {}",
        attestation_url.as_str()
    );

    println!("\n✅ Bridge setup complete!");
    println!("\n⚠️  Note: This example shows the configuration only.");
    println!("To perform an actual bridge, you need to:");
    println!("- Have USDC tokens on the source chain");
    println!("- Execute the approve and burn transactions");
    println!("- Wait for attestation and complete the mint");

    Ok(())
}
