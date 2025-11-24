//! Manual Testnet Validation for CCTP v2
//!
//! This example performs an actual USDC transfer from Sepolia to Base Sepolia.
//!
//! Prerequisites:
//! - Sepolia ETH for gas
//! - Sepolia USDC from Circle faucet (https://faucet.circle.com/)
//! - Base Sepolia ETH for destination gas
//!
//! Environment variables (set these in .env file):
//! - TESTNET_PRIVATE_KEY: Your wallet private key (must start with 0x)
//! - TESTNET_API_KEY: Alchemy API key (used for all testnet RPCs)
//! - BASE_SEPOLIA_RPC_URL: (optional) Override Base Sepolia RPC
//! - ARBITRUM_SEPOLIA_RPC_URL: (optional) Override Arbitrum Sepolia RPC
//!
//! Run with: `cargo run --example testnet_validation`

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use cctp_rs::{CctpError, CctpV2, CctpV2Bridge};
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Load .env file
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 CCTP v2 Testnet Transfer: Sepolia → Base Sepolia");
    println!("====================================================\n");

    // Load environment variables
    let private_key_str =
        std::env::var("TESTNET_PRIVATE_KEY").expect("TESTNET_PRIVATE_KEY must be set in .env file");

    let api_key =
        std::env::var("TESTNET_API_KEY").expect("TESTNET_API_KEY must be set in .env file");

    // Parse private key and get wallet address
    let signer: PrivateKeySigner = private_key_str
        .parse()
        .expect("Invalid TESTNET_PRIVATE_KEY format");
    let wallet_address = signer.address();

    // Construct RPC URLs
    let sepolia_rpc = format!("https://eth-sepolia.g.alchemy.com/v2/{}", api_key);
    let base_sepolia_rpc = std::env::var("BASE_SEPOLIA_RPC_URL")
        .unwrap_or_else(|_| format!("https://base-sepolia.g.alchemy.com/v2/{}", api_key));

    println!("📍 Configuration:");
    println!("   Wallet: {}", wallet_address);
    println!("   Source: Sepolia");
    println!("   Destination: Base Sepolia");
    println!("   Sepolia RPC: {}", sepolia_rpc);
    println!("   Base Sepolia RPC: {}\n", base_sepolia_rpc);

    // Create providers
    println!("1️⃣  Creating blockchain providers...");
    let sepolia_provider = ProviderBuilder::new().connect_http(sepolia_rpc.parse().unwrap());

    let base_sepolia_provider =
        ProviderBuilder::new().connect_http(base_sepolia_rpc.parse().unwrap());

    println!("   ✅ Providers created\n");

    // Create bridge
    println!("2️⃣  Setting up CCTP v2 bridge...");
    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::Sepolia)
        .destination_chain(NamedChain::BaseSepolia)
        .source_provider(sepolia_provider)
        .destination_provider(base_sepolia_provider)
        .recipient(wallet_address)
        .build();

    println!("   ✅ Bridge created\n");

    // Display configuration
    println!("3️⃣  Bridge Configuration:");
    println!("   Transfer Type: Standard");
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Fast Transfer: {}", bridge.is_fast_transfer());
    println!("   Expected Settlement: 10-15 minutes\n");

    // Validate domain IDs
    println!("4️⃣  Domain ID Validation:");
    let source_domain = bridge.source_chain().cctp_v2_domain_id()?;
    let dest_domain = bridge.destination_domain_id()?;

    println!("   Source Domain (Sepolia): {}", source_domain);
    println!("   Destination Domain (Base): {}", dest_domain);

    assert_eq!(source_domain.as_u32(), 0, "Sepolia should have domain ID 0");
    assert_eq!(dest_domain.as_u32(), 6, "Base should have domain ID 6");
    println!("   ✅ Domain IDs correct\n");

    // Validate contract addresses
    println!("5️⃣  Contract Addresses:");
    let token_messenger = bridge.token_messenger_v2_contract()?;
    let message_transmitter = bridge.message_transmitter_v2_contract()?;

    println!("   TokenMessenger: {}", token_messenger);
    println!("   MessageTransmitter: {}", message_transmitter);

    let expected_tm: Address = "0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"
        .parse()
        .unwrap();
    let expected_mt: Address = "0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275"
        .parse()
        .unwrap();

    assert_eq!(
        token_messenger, expected_tm,
        "TokenMessenger address mismatch"
    );
    assert_eq!(
        message_transmitter, expected_mt,
        "MessageTransmitter address mismatch"
    );
    println!("   ✅ Addresses correct\n");

    // Validate API endpoint
    println!("6️⃣  API Endpoint:");
    let api_url = bridge.api_url();
    println!("   {}", api_url.as_str());
    assert!(
        api_url.as_str().contains("sandbox"),
        "Should use sandbox API for testnet"
    );
    println!("   ✅ Using sandbox API\n");

    // Transfer configuration
    let usdc_sepolia: Address = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
        .parse()
        .unwrap();
    let amount = U256::from(1_000_000); // 1 USDC (6 decimals)

    println!("7️⃣  Transfer Details:");
    println!("   Token: USDC (Sepolia)");
    println!("   Token Address: {}", usdc_sepolia);
    println!("   Amount: 1.0 USDC");
    println!("   From: {}", wallet_address);
    println!("   To: {} (same address on Base Sepolia)\n", wallet_address);

    // Prompt user to continue
    println!("⚠️  Ready to execute transfer!");
    println!("   This will:");
    println!("   1. Approve USDC spending (if not already approved)");
    println!("   2. Burn 1 USDC on Sepolia (~$0.50 gas)");
    println!("   3. Wait for attestation from Circle (10-15 min)");
    println!("   4. Mint 1 USDC on Base Sepolia (~$0.50 gas)");
    println!("\n   Press Ctrl+C to cancel, or Enter to continue...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Execute the transfer
    println!("\n🚀 Starting Transfer...\n");

    println!("8️⃣  Burn Phase:");
    println!("   Burning 1 USDC on Sepolia...");
    println!("   (This will prompt for approval if needed)");

    let burn_tx = bridge.burn(amount, wallet_address, usdc_sepolia).await?;
    println!("   ✅ Burn TX: {}", burn_tx);
    println!(
        "   View on Etherscan: https://sepolia.etherscan.io/tx/{}",
        burn_tx
    );

    println!("\n9️⃣  Attestation Phase:");
    println!("   Extracting message from burn transaction...");

    let (message, message_hash) = bridge.get_message_sent_event(burn_tx).await?;
    println!("   ✅ Message Hash: {}", message_hash);

    println!("\n   Polling Circle API for attestation...");
    println!("   This typically takes 10-15 minutes for Sepolia finality.");
    println!("   Progress will be shown every 60 seconds.\n");

    // Poll for attestation with progress updates
    // Default: max 20 attempts, 60 second intervals = up to 20 minutes
    let attestation = bridge
        .get_attestation_with_retry(message_hash, None, None)
        .await?;
    println!("\n   ✅ Attestation received!");

    println!("\n🔟 Mint Phase:");
    println!("   Minting 1 USDC on Base Sepolia...");

    let mint_tx = bridge.mint(message, attestation, wallet_address).await?;
    println!("   ✅ Mint TX: {}", mint_tx);
    println!(
        "   View on BaseScan: https://sepolia.basescan.org/tx/{}",
        mint_tx
    );

    println!("\n🎉 Transfer Complete!");
    println!("   Your 1 USDC has been successfully bridged from Sepolia to Base Sepolia.");
    println!("\n   Summary:");
    println!("   - Burn TX: {}", burn_tx);
    println!("   - Message Hash: {}", message_hash);
    println!("   - Mint TX: {}", mint_tx);
    println!("\n✅ v0.12.0 Testnet Validation: PASSED");

    Ok(())
}
