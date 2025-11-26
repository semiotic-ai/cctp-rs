//! CCTP v2 Fast Transfer Example
//!
//! This example demonstrates how to perform a fast CCTP v2 transfer with <30 second settlement.
//! Fast transfers use finality threshold 1000 ("confirmed" level) and may incur fees (0-14 bps).
//!
//! Prerequisites:
//! - Arbitrum Sepolia ETH for gas
//! - Arbitrum Sepolia USDC from Circle faucet (https://faucet.circle.com/)
//! - Base Sepolia ETH for destination gas
//!
//! Environment variables (set these in .env file):
//! - TESTNET_PRIVATE_KEY: Your wallet private key (must start with 0x)
//! - TESTNET_API_KEY: Alchemy API key (used for all testnet RPCs)
//! - BASE_SEPOLIA_RPC_URL: (optional) Override Base Sepolia RPC
//! - ARBITRUM_SEPOLIA_RPC_URL: (optional) Override Arbitrum Sepolia RPC
//!
//! Run with: `cargo run --example v2_fast_transfer`

use alloy_chains::NamedChain;
use alloy_network::EthereumWallet;
use alloy_primitives::{address, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::sol;
use cctp_rs::{CctpError, CctpV2, CctpV2Bridge};
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

    println!("⚡ CCTP v2 Fast Transfer: Arbitrum Sepolia → Base Sepolia");
    println!("==========================================================\n");

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
        .unwrap_or_else(|_| format!("https://base-sepolia.g.alchemy.com/v2/{api_key}"));
    let arbitrum_sepolia_rpc = std::env::var("ARBITRUM_SEPOLIA_RPC_URL")
        .unwrap_or_else(|_| format!("https://arbitrum-sepolia.g.alchemy.com/v2/{api_key}"));

    println!("📍 Configuration:");
    println!("   Wallet: {wallet_address}");
    println!("   Source: Arbitrum Sepolia");
    println!("   Destination: Base Sepolia");
    println!("   Transfer Mode: ⚡ Fast (Confirmed)");
    println!("   Arbitrum Sepolia RPC: {arbitrum_sepolia_rpc}");
    println!("   Base Sepolia RPC: {base_sepolia_rpc}\n");

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
            CctpError::Provider(format!("Failed to get Arbitrum Sepolia ETH balance: {e}"))
        })?;

    let usdc_arb_contract = IERC20::new(usdc_arbitrum_sepolia, &arbitrum_sepolia_provider);
    let arb_usdc_balance = usdc_arb_contract
        .balanceOf(wallet_address)
        .call()
        .await
        .map_err(|e| {
            CctpError::ContractCall(format!("Failed to get Arbitrum Sepolia USDC balance: {e}"))
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
        .map_err(|e| CctpError::Provider(format!("Failed to get Base Sepolia ETH balance: {e}")))?;

    let usdc_base_contract = IERC20::new(usdc_base_sepolia, &base_sepolia_provider);
    let base_usdc_balance = usdc_base_contract
        .balanceOf(wallet_address)
        .call()
        .await
        .map_err(|e| {
            CctpError::ContractCall(format!("Failed to get Base Sepolia USDC balance: {e}"))
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
            println!("   {issue}\n");
        }
        println!("Please fund your wallet and try again.");
        return Ok(());
    }

    println!("✅ All balance requirements met!\n");

    // Fast transfer info
    println!("4️⃣  Fast Transfer Configuration:");
    println!("   ┌─────────────────┬─────────────┬──────────────┐");
    println!("   │ Feature         │ Fast        │ Standard     │");
    println!("   ├─────────────────┼─────────────┼──────────────┤");
    println!("   │ Settlement Time │ <30 seconds │ 10-15 minutes│");
    println!("   │ Finality Level  │ 1000        │ 2000         │");
    println!("   │ Fee             │ 0-14 bps    │ Free         │");
    println!("   │ Poll Interval   │ 5 seconds   │ 60 seconds   │");
    println!("   │ Security        │ Confirmed   │ Finalized    │");
    println!("   └─────────────────┴─────────────┴──────────────┘\n");

    // Safety exit - remove this line to proceed with the actual transfer
    println!("🛑 Dry run complete. To execute the actual fast transfer:");
    println!("   Set the environment variable: EXECUTE_TRANSFER=true");
    println!("   Then run: cargo run --example v2_fast_transfer\n");

    if std::env::var("EXECUTE_TRANSFER").unwrap_or_default() != "true" {
        return Ok(());
    }

    println!("🚀 EXECUTE_TRANSFER=true detected, proceeding with fast transfer...\n");

    // Create bridge with fast transfer enabled
    println!("5️⃣  Setting up CCTP v2 bridge with FAST TRANSFER...");

    // Set max_fee for fast transfer (optional fee cap in USDC atomic units)
    let max_fee = U256::from(1000); // 0.001 USDC max fee

    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::ArbitrumSepolia)
        .destination_chain(NamedChain::BaseSepolia)
        .source_provider(arbitrum_sepolia_provider)
        .destination_provider(base_sepolia_provider)
        .recipient(wallet_address)
        .fast_transfer(true) // Enable fast transfers!
        .max_fee(max_fee) // Optional: set maximum fee willing to pay
        .build();

    println!("   ✅ Fast transfer bridge created\n");

    // Display configuration
    println!("6️⃣  Bridge Configuration:");
    println!("   Transfer Type: ⚡ Fast");
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Fast Transfer Enabled: {}", bridge.is_fast_transfer());
    println!("   Max Fee: {max_fee} USDC atomic units (0.001 USDC)");
    println!("   Expected Settlement: <30 seconds\n");

    // Validate domain IDs
    println!("7️⃣  Domain ID Validation:");
    let source_domain = bridge.source_chain().cctp_v2_domain_id()?;
    let dest_domain = bridge.destination_domain_id()?;

    println!("   Source Domain (Arbitrum Sepolia): {source_domain}");
    println!("   Destination Domain (Base Sepolia): {dest_domain}");

    assert_eq!(
        source_domain.as_u32(),
        3,
        "Arbitrum Sepolia should have domain ID 3"
    );
    assert_eq!(
        dest_domain.as_u32(),
        6,
        "Base Sepolia should have domain ID 6"
    );
    println!("   ✅ Domain IDs correct\n");

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
    println!("   Token Address: {usdc_arbitrum_sepolia}");
    println!("   Amount: 1.0 USDC");
    println!("   From: {wallet_address}");
    println!("   To: {wallet_address} (same address on Base Sepolia)");
    println!("   Mode: ⚡ Fast Transfer\n");

    // Execute the transfer
    println!("\n🚀 Starting Fast Transfer...\n");

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
    println!("   TokenMessenger: {token_messenger}");

    if current_allowance < amount {
        println!("   ⚠️  Insufficient allowance, sending approval transaction...");

        let approval_tx = bridge
            .approve(usdc_arbitrum_sepolia, wallet_address, amount)
            .await?;
        println!("   ✅ Approval TX: {approval_tx}");
        println!(
            "   View on Arbitrum Sepolia Etherscan: https://sepolia.arbiscan.io/tx/{approval_tx}"
        );

        // Wait for approval to be mined
        println!("   Waiting for approval confirmation...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
        println!("   ✅ Sufficient allowance already granted");
    }

    println!("\n1️⃣1️⃣ Burn Phase (Fast Transfer):");
    println!("   Burning 1 USDC on Arbitrum Sepolia with fast finality...");

    let burn_tx = bridge
        .burn(amount, wallet_address, usdc_arbitrum_sepolia)
        .await?;
    println!("   ✅ Burn TX: {burn_tx}");
    println!("   View on Arbitrum Sepolia Etherscan: https://sepolia.arbiscan.io/tx/{burn_tx}");

    println!("\n1️⃣2️⃣ Attestation Phase (Fast Polling):");
    println!("   Polling Circle API for attestation and message...");
    println!("   ⚡ Fast transfer polling interval: 5 seconds");
    println!("   Expected wait: <30 seconds (vs 10-15 minutes standard)\n");

    // Poll for attestation with progress updates
    // V2 API uses transaction hash, not message hash
    // Fast transfers poll more frequently (5s vs 60s)
    let (message, attestation) = bridge
        .get_attestation(burn_tx, cctp_rs::PollingConfig::fast_transfer())
        .await?;
    println!("\n   ✅ Attestation and message received!");
    println!("   Message length: {} bytes", message.len());
    println!("   Attestation length: {} bytes", attestation.len());

    println!("\n1️⃣3️⃣ Mint Phase:");
    println!("   Minting 1 USDC on Base Sepolia...");

    let mint_tx = bridge.mint(message, attestation, wallet_address).await?;
    println!("   ✅ Mint TX: {mint_tx}");
    println!("   View on BaseScan: https://base-sepolia.blockscout.com/tx/{mint_tx}");

    println!("\n🎉 Fast Transfer Complete!");
    println!("   Your 1 USDC has been bridged from Arbitrum Sepolia to Base Sepolia.");
    println!("   ⚡ Settlement time: <30 seconds (fast transfer mode)");
    println!("\n   Summary:");
    println!("   - Burn TX: {burn_tx}");
    println!("   - Mint TX: {mint_tx}");
    println!("   - Transfer Mode: Fast (finality level 1000)");
    println!("\n✅ CCTP v2 Fast Transfer Validation: PASSED");

    Ok(())
}
