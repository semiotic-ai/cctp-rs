//! CCTP v2 Standard Transfer Example
//!
//! This example demonstrates how to perform a standard CCTP v2 transfer between chains.
//! Standard transfers use finality threshold 2000 ("finalized" level) for maximum security
//! and typically complete in 10-15 minutes.
//!
//! Run with: `cargo run --example v2_standard_transfer`

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use alloy_provider::ProviderBuilder;
use cctp_rs::{CctpError, CctpV2, CctpV2Bridge};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt::init();

    println!("🌉 CCTP v2 Standard Transfer Example");
    println!("=====================================");
    println!("Bridging USDC from Ethereum Mainnet to Linea\n");

    // Step 1: Set up providers
    println!("1️⃣ Setting up blockchain providers...");
    println!("   Note: Replace with your actual RPC endpoints\n");

    // In production, use your actual RPC endpoints with API keys
    let eth_provider = ProviderBuilder::new().connect_http(
        "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
            .parse()
            .unwrap(),
    );

    let linea_provider = ProviderBuilder::new().connect_http(
        "https://linea-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
            .parse()
            .unwrap(),
    );

    // Step 2: Create the CCTP v2 bridge
    println!("2️⃣ Creating CCTP v2 bridge...");

    // Example recipient address (replace with your actual address)
    let recipient = Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f8fA0d")?;

    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Linea)
        .source_provider(eth_provider)
        .destination_provider(linea_provider)
        .recipient(recipient)
        // No fast_transfer flag = standard transfer
        .build();

    println!("   ✅ Bridge created successfully\n");

    // Step 3: Display bridge configuration
    println!("3️⃣ Bridge Configuration:");
    println!("   Source Chain: {}", bridge.source_chain());
    println!("   Destination Chain: {}", bridge.destination_chain());
    println!("   Recipient: {}", bridge.recipient());
    println!("   Transfer Mode: Standard (Finalized)");
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Expected Settlement: 10-15 minutes (finality level 2000)\n");

    // Step 4: Get v2 contract addresses
    println!("4️⃣ V2 Contract Addresses:");

    let token_messenger = bridge.token_messenger_v2_contract()?;
    let message_transmitter = bridge.message_transmitter_v2_contract()?;

    println!("   Token Messenger: {token_messenger}");
    println!("   Message Transmitter: {message_transmitter}");
    println!("   Note: V2 uses unified addresses across all chains\n");

    // Step 5: Show domain ID information
    println!("5️⃣ Domain IDs:");
    let source_domain = bridge.source_chain().cctp_v2_domain_id()?;
    let dest_domain = bridge.destination_domain_id()?;
    println!(
        "   Source ({}) Domain ID: {}",
        bridge.source_chain(),
        source_domain
    );
    println!(
        "   Destination ({}) Domain ID: {}\n",
        bridge.destination_chain(),
        dest_domain
    );

    // Step 6: Example transfer flow
    println!("6️⃣ Standard Transfer Flow:");
    println!("   This is the high-level flow for a standard v2 transfer:\n");

    // Example amounts for demonstration
    let amount = U256::from(1_000_000); // 1 USDC (6 decimals)
    let _from_address = recipient; // In reality, this is the sender's address
    let _usdc_address = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?; // Mainnet USDC

    println!("   📝 Step 1: Approve USDC");
    println!("      Amount: {} USDC", amount);
    println!("      Spender: {token_messenger} (Token Messenger)");
    println!("      → Call: USDC.approve(tokenMessenger, amount)\n");

    println!("   🔥 Step 2: Burn USDC on source chain");
    println!("      Method: bridge.burn()");
    println!("      → Uses depositForBurn() (standard, no fees)");
    println!("      → Finality threshold: 2000 (finalized level)");
    println!("      Code example:");
    println!("      ```rust");
    println!("      let burn_tx = bridge.burn(");
    println!("          U256::from(1_000_000),  // 1 USDC");
    println!("          from_address,");
    println!("          usdc_address,");
    println!("      ).await?;");
    println!("      ```\n");

    println!("   ⏳ Step 3: Wait for attestation");
    println!("      Polling interval: 60 seconds (standard)");
    println!("      Expected wait: 10-15 minutes");
    println!(
        "      API endpoint: {}",
        bridge.create_url([0u8; 32].into())?.as_str()
    );
    println!("      Code example:");
    println!("      ```rust");
    println!("      // V2 uses tx hash directly (no need to extract message hash first)");
    println!("      let attestation = bridge.get_attestation(");
    println!("          burn_tx,");
    println!("          None,  // Use default max attempts (30)");
    println!("          None,  // Use default poll interval (60s)");
    println!("      ).await?;");
    println!("      ```\n");

    println!("   💰 Step 4: Mint USDC on destination chain");
    println!("      Method: bridge.mint()");
    println!("      → Uses receiveMessage() to complete transfer");
    println!("      Code example:");
    println!("      ```rust");
    println!("      let mint_tx = bridge.mint(");
    println!("          message,");
    println!("          attestation,");
    println!("          from_address,");
    println!("      ).await?;");
    println!("      ```\n");

    // Step 7: Full transfer example
    println!("7️⃣ Complete Transfer Example:");
    println!("   Full end-to-end transfer in one call:\n");
    println!("   ```rust");
    println!("   // This handles burn → attestation → mint automatically");
    println!("   let (burn_tx, mint_tx) = bridge.transfer(");
    println!("       U256::from(1_000_000),  // 1 USDC");
    println!("       from_address,");
    println!("       usdc_address,");
    println!("   ).await?;");
    println!("   ```\n");

    // Step 8: Key differences from v1
    println!("8️⃣ V2 Standard Transfer vs V1:");
    println!("   ✅ Unified contract addresses across chains");
    println!("   ✅ Explicit finality threshold (2000 = finalized)");
    println!("   ✅ Support for fast transfers (with fast_transfer flag)");
    println!("   ✅ Support for programmable hooks (with hook_data)");
    println!("   ✅ Same security level as v1 (finalized blocks)");
    println!("   ✅ No fees for standard transfers\n");

    // Step 9: Best practices
    println!("9️⃣ Best Practices:");
    println!("   📌 Use standard transfers for:");
    println!("      - Large transfers requiring maximum security");
    println!("      - When time is not critical (10-15 min is acceptable)");
    println!("      - When you want to avoid any fees");
    println!("   📌 Always verify:");
    println!("      - USDC balance before burning");
    println!("      - Gas availability on both chains");
    println!("      - Recipient address is correct");
    println!("   📌 Error handling:");
    println!("      - Check burn transaction confirmation");
    println!("      - Handle attestation timeout gracefully");
    println!("      - Verify mint transaction success\n");

    // Step 10: Testing recommendations
    println!("🔟 Testing on Sepolia:");
    println!("   For testing, use Sepolia testnet:");
    println!("   ```rust");
    println!("   let bridge = CctpV2Bridge::builder()");
    println!("       .source_chain(NamedChain::Sepolia)");
    println!("       .destination_chain(NamedChain::BaseSepolia)");
    println!("       .source_provider(sepolia_provider)");
    println!("       .destination_provider(base_sepolia_provider)");
    println!("       .recipient(recipient)");
    println!("       .build();");
    println!("   ```");
    println!("   Note: Get testnet USDC from Circle's faucet\n");

    println!("✅ Example complete!");
    println!("\n⚠️  Important Notes:");
    println!("   - This example shows configuration only (no actual transactions)");
    println!("   - Replace RPC endpoints with your actual providers");
    println!("   - Test on Sepolia before using mainnet");
    println!("   - Ensure you have sufficient USDC and gas on both chains");
    println!("   - Standard transfers take 10-15 minutes (use fast transfers for <30s)");

    Ok(())
}
