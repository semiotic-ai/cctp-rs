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
use alloy_network::EthereumWallet;
use alloy_primitives::{address, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::sol;
use cctp_rs::{
    CctpError, CctpV2, CctpV2Bridge, CCTP_V2_MESSAGE_TRANSMITTER_TESTNET,
    CCTP_V2_TOKEN_MESSENGER_TESTNET,
};
use dotenvy::dotenv;

// Minimal ERC20 interface for balance checking
sol! {
    #[sol(rpc)]
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
    }
}

/// Format ETH balance (18 decimals) for display
fn format_eth_balance(balance: U256) -> String {
    let eth = balance.to::<u128>() as f64 / 1e18;
    format!("{:.6}", eth)
}

/// Format USDC balance (6 decimals) for display
fn format_usdc_balance(balance: U256) -> String {
    let usdc = balance.to::<u128>() as f64 / 1e6;
    format!("{:.6}", usdc)
}

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Load .env file
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 CCTP v2 Testnet Transfer: Arbitrum Sepolia → Base Sepolia");
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
    let base_sepolia_rpc = std::env::var("BASE_SEPOLIA_RPC_URL")
        .unwrap_or_else(|_| format!("https://base-sepolia.g.alchemy.com/v2/{}", api_key));
    let arbitrum_sepolia_rpc = std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
        .unwrap_or_else(|_| format!("https://arbitrum-sepolia.g.alchemy.com/v2/{}", api_key));

    println!("📍 Configuration:");
    println!("   Wallet: {}", wallet_address);
    println!("   Source: Sepolia");
    println!("   Destination: Base Sepolia");
    println!("   Arbitrum Sepolia RPC: {}", arbitrum_sepolia_rpc);
    println!("   Base Sepolia RPC: {}\n", base_sepolia_rpc);

    // Create wallet from signer
    let wallet = EthereumWallet::from(signer);

    // Create providers with wallet for signing transactions
    println!("1️⃣  Creating blockchain providers...");

    let arb_sepolia_full_rpc_url = format!("{arbitrum_sepolia_rpc}{api_key}");
    let arbitrum_sepolia_provider = ProviderBuilder::new()
        .wallet(wallet.clone())
        .connect_http(arb_sepolia_full_rpc_url.parse().unwrap());

    let base_sepolia_full_rpc_url = format!("{base_sepolia_rpc}{api_key}");
    let base_sepolia_provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(base_sepolia_full_rpc_url.parse().unwrap());

    println!("   ✅ Providers created (with wallet signer)\n");

    // USDC contract addresses
    let usdc_arbitrum_sepolia = address!("75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");
    let usdc_base_sepolia = address!("036CbD53842c5426634e7929541eC2318f3dCF7e");

    // Check balances on Arbitrum Sepolia
    println!("2️⃣  Checking Arbitrum Sepolia Balances...");

    let arb_eth_balance = arbitrum_sepolia_provider
        .get_balance(wallet_address)
        .await
        .map_err(|e| {
            CctpError::Provider(format!("Failed to get Arbitrum Sepolia ETH balance: {}", e))
        })?;

    let usdc_arb_contract = IERC20::new(usdc_arbitrum_sepolia, &arbitrum_sepolia_provider);
    let arb_usdc_balance = usdc_arb_contract
        .balanceOf(wallet_address)
        .call()
        .await
        .map_err(|e| {
            CctpError::ContractCall(format!(
                "Failed to get Arbitrum Sepolia USDC balance: {}",
                e
            ))
        })?;

    println!(
        "   ETH Balance: {} ETH",
        format_eth_balance(arb_eth_balance)
    );
    println!(
        "   USDC Balance: {} USDC",
        format_usdc_balance(arb_usdc_balance)
    );
    println!("   ✅ Arbitrum Sepolia balances retrieved\n");

    // Check balances on Base Sepolia
    println!("3️⃣  Checking Base Sepolia Balances...");

    let base_eth_balance = base_sepolia_provider
        .get_balance(wallet_address)
        .await
        .map_err(|e| {
            CctpError::Provider(format!("Failed to get Base Sepolia ETH balance: {}", e))
        })?;

    let usdc_base_contract = IERC20::new(usdc_base_sepolia, &base_sepolia_provider);
    let base_usdc_balance = usdc_base_contract
        .balanceOf(wallet_address)
        .call()
        .await
        .map_err(|e| {
            CctpError::ContractCall(format!("Failed to get Base Sepolia USDC balance: {}", e))
        })?;

    println!(
        "   ETH Balance: {} ETH",
        format_eth_balance(base_eth_balance)
    );
    println!(
        "   USDC Balance: {} USDC",
        format_usdc_balance(base_usdc_balance)
    );
    println!("   ✅ Base Sepolia balances retrieved\n");

    // Summary
    println!("📊 Balance Summary:");
    println!("┌─────────────────────┬──────────────────┬──────────────────┐");
    println!("│ Chain               │ ETH Balance      │ USDC Balance     │");
    println!("├─────────────────────┼──────────────────┼──────────────────┤");
    println!(
        "│ Arbitrum Sepolia    │ {:>16} │ {:>16} │",
        format_eth_balance(arb_eth_balance),
        format_usdc_balance(arb_usdc_balance)
    );
    println!(
        "│ Base Sepolia        │ {:>16} │ {:>16} │",
        format_eth_balance(base_eth_balance),
        format_usdc_balance(base_usdc_balance)
    );
    println!("└─────────────────────┴──────────────────┴──────────────────┘\n");

    // Check if we have sufficient balances
    let min_eth = U256::from(1_000_000_000_000_000u64); // 0.001 ETH minimum
    let min_usdc = U256::from(1_000_000u64); // 1 USDC minimum (6 decimals)

    let mut issues: Vec<String> = Vec::new();

    if arb_eth_balance < min_eth {
        issues.push(format!(
            "❌ Insufficient ETH on Arbitrum Sepolia: {} (need >= 0.001 ETH)\n   \
             → Get testnet ETH: https://faucet.quicknode.com/arbitrum/sepolia",
            format_eth_balance(arb_eth_balance)
        ));
    }

    if base_eth_balance < min_eth {
        issues.push(format!(
            "❌ Insufficient ETH on Base Sepolia: {} (need >= 0.001 ETH)\n   \
             → Get testnet ETH: https://faucet.quicknode.com/base/sepolia",
            format_eth_balance(base_eth_balance)
        ));
    }

    if arb_usdc_balance < min_usdc {
        issues.push(format!(
            "❌ Insufficient USDC on Arbitrum Sepolia: {} (need >= 1 USDC)\n   \
             → Get testnet USDC: https://faucet.circle.com/",
            format_usdc_balance(arb_usdc_balance)
        ));
    }

    if !issues.is_empty() {
        println!("⚠️  Cannot proceed - insufficient balances:\n");
        for issue in &issues {
            println!("   {}\n", issue);
        }
        println!("Please fund your wallet and try again.");
        return Ok(());
    }

    println!("✅ All balance requirements met!\n");

    // Safety exit - remove this line to proceed with the actual transfer
    println!("🛑 Dry run complete. To execute the actual transfer:");
    println!("   Set the environment variable: EXECUTE_TRANSFER=true");
    println!("   Then run: cargo run --example testnet_validation\n");

    if std::env::var("EXECUTE_TRANSFER").unwrap_or_default() != "true" {
        return Ok(());
    }

    println!("🚀 EXECUTE_TRANSFER=true detected, proceeding with transfer...\n");

    // Create bridge
    println!("4️⃣  Setting up CCTP v2 bridge...");
    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::ArbitrumSepolia)
        .destination_chain(NamedChain::BaseSepolia)
        .source_provider(arbitrum_sepolia_provider)
        .destination_provider(base_sepolia_provider)
        .recipient(wallet_address)
        .build();

    println!("   ✅ Bridge created\n");

    // Display configuration
    println!("5️⃣  Bridge Configuration:");
    println!("   Transfer Type: Standard");
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Fast Transfer: {}", bridge.is_fast_transfer());
    println!("   Expected Settlement: 10-15 minutes\n");

    // Validate domain IDs
    println!("6️⃣  Domain ID Validation:");
    let source_domain = bridge.source_chain().cctp_v2_domain_id()?;
    let dest_domain = bridge.destination_domain_id()?;

    println!("   Source Domain (Arbitrum Sepolia): {}", source_domain);
    println!("   Destination Domain (Base): {}", dest_domain);

    assert_eq!(
        source_domain.as_u32(),
        3,
        "Arbitrum Sepolia should have domain ID 3"
    );
    assert_eq!(dest_domain.as_u32(), 6, "Base should have domain ID 6");
    println!("   ✅ Domain IDs correct\n");

    // Validate contract addresses
    println!("7️⃣  Contract Addresses:");
    let token_messenger = bridge.token_messenger_v2_contract()?;
    let message_transmitter = bridge.message_transmitter_v2_contract()?;

    println!("   TokenMessenger: {}", token_messenger);
    println!("   MessageTransmitter: {}", message_transmitter);

    let expected_tm = CCTP_V2_TOKEN_MESSENGER_TESTNET;
    let expected_mt = CCTP_V2_MESSAGE_TRANSMITTER_TESTNET;

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
    println!("8️⃣  API Endpoint:");
    let api_url = bridge.api_url();
    println!("   {}", api_url.as_str());
    assert!(
        api_url.as_str().contains("sandbox"),
        "Should use sandbox API for testnet"
    );
    println!("   ✅ Using sandbox API\n");

    // Transfer configuration
    let amount = U256::from(1_000_000); // 1 USDC (6 decimals)

    println!("9️⃣  Transfer Details:");
    println!("   Token: USDC (Arbitrum Sepolia)");
    println!("   Token Address: {}", usdc_arbitrum_sepolia);
    println!("   Amount: 1.0 USDC");
    println!("   From: {}", wallet_address);
    println!("   To: {} (same address on Base Sepolia)\n", wallet_address);

    // Execute the transfer
    println!("\n🚀 Starting Transfer...\n");

    // Check and handle ERC20 approval
    println!("🔟 Approval Phase:");
    println!("   Checking TokenMessenger allowance...");

    let token_messenger = bridge.token_messenger_v2_contract()?;
    let current_allowance = bridge
        .get_allowance(usdc_arbitrum_sepolia, wallet_address)
        .await?;

    println!(
        "   Current allowance: {} USDC",
        format_usdc_balance(current_allowance)
    );
    println!("   TokenMessenger: {}", token_messenger);

    if current_allowance < amount {
        println!("   ⚠️  Insufficient allowance, sending approval transaction...");

        let approval_tx = bridge
            .approve(usdc_arbitrum_sepolia, wallet_address, amount)
            .await?;
        println!("   ✅ Approval TX: {}", approval_tx);
        println!(
            "   View on Arbitrum Sepolia Etherscan: https://sepolia.arbiscan.io/tx/{}",
            approval_tx
        );

        // Wait for approval to be mined
        println!("   Waiting for approval confirmation...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
        println!("   ✅ Sufficient allowance already granted");
    }

    println!("\n1️⃣1️⃣ Burn Phase:");
    println!("   Burning 1 USDC on Arbitrum Sepolia...");

    let burn_tx = bridge
        .burn(amount, wallet_address, usdc_arbitrum_sepolia)
        .await?;
    println!("   ✅ Burn TX: {}", burn_tx);
    println!(
        "   View on Arbitrum Sepolia Etherscan: https://sepolia.arbiscan.io/tx/{}",
        burn_tx
    );

    println!("\n1️⃣2️⃣ Attestation Phase:");
    println!("   Polling Circle API for attestation and message...");
    println!("   This typically takes 10-15 minutes for Arbitrum Sepolia finality.");
    println!("   Progress will be shown every 60 seconds.\n");

    // Poll for attestation with progress updates
    // V2 API uses transaction hash, not message hash
    // IMPORTANT: get_attestation returns both the canonical message and attestation from Circle's API
    // The MessageSent event log contains zeros in the nonce field - Circle fills this in
    let (message, attestation) = bridge
        .get_attestation(burn_tx, cctp_rs::PollingConfig::fast_transfer())
        .await?;
    println!("\n   ✅ Attestation and message received!");
    println!("   Message length: {} bytes", message.len());
    println!("   Attestation length: {} bytes", attestation.len());

    println!("\n1️⃣3️⃣ Mint Phase:");
    println!("   Minting 1 USDC on Base Sepolia...");

    let mint_tx = bridge.mint(message, attestation, wallet_address).await?;
    println!("   ✅ Mint TX: {}", mint_tx);
    println!(
        "   View on BaseScan: https://base-sepolia.blockscout.com/tx/{}",
        mint_tx
    );

    println!("\n🎉 Transfer Complete!");
    println!("   Your 1 USDC has been successfully bridged from Arbitrum Sepolia to Base Sepolia.");
    println!("\n   Summary:");
    println!("   - Burn TX: {}", burn_tx);
    println!("   - Mint TX: {}", mint_tx);
    println!("\n✅ v0.15.0 Testnet Validation: PASSED");

    Ok(())
}
