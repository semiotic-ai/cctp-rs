//! CCTP v2 Fast Transfer Example
//!
//! This example demonstrates how to perform a fast CCTP v2 transfer with <30 second settlement.
//! Fast transfers use finality threshold 1000 ("confirmed" level) and may incur fees (0-14 bps).
//!
//! Run with: `cargo run --example v2_fast_transfer`

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use alloy_provider::ProviderBuilder;
use cctp_rs::{CctpError, CctpV2, CctpV2Bridge};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt::init();

    println!("⚡ CCTP v2 Fast Transfer Example");
    println!("=================================");
    println!("Bridging USDC with <30 second settlement\n");

    // Step 1: Set up providers
    println!("1️⃣ Setting up blockchain providers...");
    println!("   Note: Replace with your actual RPC endpoints\n");

    let eth_provider = ProviderBuilder::new().connect_http(
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
            .parse()
            .unwrap(),
    );

    let base_provider = ProviderBuilder::new().connect_http(
        "https://base-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
            .parse()
            .unwrap(),
    );

    // Step 2: Create the CCTP v2 bridge with fast transfer enabled
    println!("2️⃣ Creating CCTP v2 bridge with fast transfer...");

    let recipient = Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d")?;

    // Set max_fee for fast transfer (optional fee cap in USDC atomic units)
    let max_fee = U256::from(1000); // 0.001 USDC max fee

    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Base)
        .source_provider(eth_provider)
        .destination_provider(base_provider)
        .recipient(recipient)
        .fast_transfer(true) // Enable fast transfers!
        .max_fee(max_fee) // Optional: set maximum fee willing to pay
        .build();

    println!("   ✅ Fast transfer bridge created\n");

    // Step 3: Display fast transfer configuration
    println!("3️⃣ Fast Transfer Configuration:");
    println!("   Source Chain: {}", bridge.source_chain());
    println!("   Destination Chain: {}", bridge.destination_chain());
    println!("   Recipient: {}", bridge.recipient());
    println!("   Transfer Mode: ⚡ Fast (Confirmed)");
    println!("   Fast Transfer Enabled: {}", bridge.is_fast_transfer());
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Max Fee: {} USDC (0.001 USDC)", max_fee);
    println!("   Expected Settlement: <30 seconds (finality level 1000)\n");

    // Step 4: Fast transfer fee information
    println!("4️⃣ Fast Transfer Fees:");
    let fee_bps = bridge.source_chain().fast_transfer_fee_bps()?.unwrap_or(0);
    println!("   Fee rate: {} basis points (bps)", fee_bps);
    println!("   Fee range: 0-14 bps (most chains: 0 bps = free)");
    println!("   On 1 USDC transfer:");
    println!("      - 0 bps = $0.00 fee");
    println!("      - 10 bps = $0.001 fee (0.1%)");
    println!("      - 14 bps = $0.0014 fee (0.14%)\n");

    // Step 5: Key differences from standard transfers
    println!("5️⃣ Fast vs Standard Transfer:");
    println!("   ┌─────────────────┬─────────────┬──────────────┐");
    println!("   │ Feature         │ Fast        │ Standard     │");
    println!("   ├─────────────────┼─────────────┼──────────────┤");
    println!("   │ Settlement Time │ <30 seconds │ 10-15 minutes│");
    println!("   │ Finality Level  │ 1000        │ 2000         │");
    println!("   │ Fee             │ 0-14 bps    │ Free         │");
    println!("   │ Poll Interval   │ 5 seconds   │ 60 seconds   │");
    println!("   │ Security        │ Confirmed   │ Finalized    │");
    println!("   └─────────────────┴─────────────┴──────────────┘\n");

    // Step 6: Example transfer flow
    println!("6️⃣ Fast Transfer Flow:");
    println!("   The flow is the same as standard, but much faster:\n");

    let _amount = U256::from(1_000_000); // 1 USDC
    let _from_address = recipient;
    let _usdc_address = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;

    println!("   🔥 Step 1: Burn with fast transfer");
    println!("      → Uses depositForBurn() with fast finality (1000)");
    println!("      → Max fee: {} (optional cap)", max_fee);
    println!("      ```rust");
    println!("      let burn_tx = bridge.burn(");
    println!("          U256::from(1_000_000),");
    println!("          from_address,");
    println!("          usdc_address,");
    println!("      ).await?;");
    println!("      ```\n");

    println!("   ⚡ Step 2: Fast attestation polling");
    println!("      Polling interval: 5 seconds (vs 60s for standard)");
    println!("      Expected wait: <30 seconds");
    println!("      ```rust");
    println!("      // V2 uses tx hash directly (no need to extract message hash first)");
    println!("      let attestation = bridge.get_attestation(");
    println!("          burn_tx,");
    println!("          None,  // Defaults to fast polling (5s interval)");
    println!("          None,");
    println!("      ).await?;");
    println!("      ```\n");

    println!("   💰 Step 3: Mint on destination");
    println!("      Same as standard transfer");
    println!("      ```rust");
    println!("      let mint_tx = bridge.mint(message, attestation, from_address).await?;");
    println!("      ```\n");

    // Step 7: When to use fast transfers
    println!("7️⃣ When to Use Fast Transfers:");
    println!("   ✅ Use fast transfers for:");
    println!("      - Time-sensitive operations (DEX arbitrage, liquidations)");
    println!("      - User-facing applications (better UX)");
    println!("      - Trading and DeFi protocols");
    println!("      - Real-time settlements");
    println!("   ⚠️  Use standard transfers for:");
    println!("      - Large transfers requiring maximum security");
    println!("      - Batch operations where time isn't critical");
    println!("      - When avoiding all fees is important\n");

    // Step 8: Fee management
    println!("8️⃣ Fee Management:");
    println!("   Setting max_fee helps control costs:");
    println!("   ```rust");
    println!("   // Set maximum fee you're willing to pay");
    println!("   let max_fee = U256::from(5000);  // 0.005 USDC");
    println!("   ");
    println!("   let bridge = CctpV2::builder()");
    println!("       .fast_transfer(true)");
    println!("       .max_fee(max_fee)  // Optional: defaults to 0 if not set");
    println!("       .build();");
    println!("   ```");
    println!("   If actual fee > max_fee, transaction will revert\n");

    // Step 9: Supported chains
    println!("9️⃣ Fast Transfer Support:");
    println!("   All v2 chains support fast transfers:");
    println!("   ✅ Mainnet chains: Ethereum, Arbitrum, Base, Optimism, etc.");
    println!("   ✅ Testnet chains: Sepolia, Base Sepolia, Arbitrum Sepolia");
    println!("   ");
    println!("   Check programmatically:");
    println!("   ```rust");
    println!("   if bridge.supports_fast_transfer() {{");
    println!("       println!(\"Fast transfers available!\");");
    println!("   }}");
    println!("   ```\n");

    // Step 10: Complete example
    println!("🔟 Complete Fast Transfer:");
    println!("   ```rust");
    println!("   use cctp_rs::{{CctpV2Bridge, CctpBridge}};");
    println!("   ");
    println!("   // 1. Create fast transfer bridge");
    println!("   let bridge = CctpV2Bridge::builder()");
    println!("       .source_chain(NamedChain::Mainnet)");
    println!("       .destination_chain(NamedChain::Base)");
    println!("       .source_provider(eth_provider)");
    println!("       .destination_provider(base_provider)");
    println!("       .recipient(recipient)");
    println!("       .fast_transfer(true)");
    println!("       .max_fee(U256::from(1000))  // 0.001 USDC max");
    println!("       .build();");
    println!("   ");
    println!("   // 2. Execute fast transfer (all-in-one)");
    println!("   let (burn_tx, mint_tx) = bridge.transfer(");
    println!("       U256::from(1_000_000),  // 1 USDC");
    println!("       sender_address,");
    println!("       usdc_address,");
    println!("   ).await?;");
    println!("   ");
    println!("   println!(\"✅ Transfer complete in <30 seconds!\");");
    println!("   println!(\"   Burn TX: {{}}\", burn_tx);");
    println!("   println!(\"   Mint TX: {{}}\", mint_tx);");
    println!("   ```\n");

    // Step 11: Monitoring fast transfers
    println!("1️⃣1️⃣ Monitoring Fast Transfers:");
    println!("   Fast transfers poll more frequently:");
    println!("   - Attestation check every 5 seconds");
    println!("   - Typical completion: 15-25 seconds");
    println!("   - Maximum wait with defaults: 2.5 minutes (30 attempts × 5s)");
    println!("   ");
    println!("   Enable tracing to see polling progress:");
    println!("   ```rust");
    println!("   tracing_subscriber::fmt::init();");
    println!("   ```\n");

    // Step 12: Testing
    println!("1️⃣2️⃣ Testing Fast Transfers:");
    println!("   Test on Sepolia first:");
    println!("   ```rust");
    println!("   let test_bridge = CctpV2Bridge::builder()");
    println!("       .source_chain(NamedChain::Sepolia)");
    println!("       .destination_chain(NamedChain::BaseSepolia)");
    println!("       .fast_transfer(true)");
    println!("       .max_fee(U256::from(1000))");
    println!("       .build();");
    println!("   ```");
    println!("   Get testnet USDC: https://faucet.circle.com\n");

    println!("✅ Example complete!");
    println!("\n💡 Key Takeaways:");
    println!("   • Fast transfers settle in <30 seconds (vs 10-15 min standard)");
    println!("   • Fee range: 0-14 bps (most chains are free)");
    println!("   • Uses finality level 1000 (confirmed blocks)");
    println!("   • Perfect for time-sensitive DeFi operations");
    println!("   • All v2 chains support fast transfers");
    println!("   • Set max_fee to control costs");

    Ok(())
}
