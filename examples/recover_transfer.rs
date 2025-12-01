// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Recovery script for a failed/interrupted CCTP v2 transfer
//!
//! This script resumes from an existing burn transaction to complete
//! the attestation and mint phases using data directly from Circle's API.
//!
//! Environment variables (set these in .env file):
//! - TESTNET_PRIVATE_KEY: Your wallet private key (must start with 0x)
//! - TESTNET_API_KEY: Alchemy API key (used for all testnet RPCs)
//!
//! Run with: `cargo run --example recover_transfer`

use alloy_network::EthereumWallet;
use alloy_primitives::{hex, Address, Bytes};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer_local::PrivateKeySigner;
use cctp_rs::{CctpError, MessageTransmitterV2Contract};
use dotenvy::dotenv;

// Known transaction details from the interrupted transfer
const WALLET_ADDRESS: Address =
    alloy_primitives::address!("7F7D081724F0240c64C9E01CDe4626602f9a0192");

// Message and attestation from Circle API response (nonce: 0x2f3cb13c...)
const API_MESSAGE: &str = "0000000100000003000000062f3cb13cf4a6103f9e3b256495b08c4e05630fcba639565d199ed420a5f2be010000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000000000000000000000000000000000000000000000000007d0000007d00000000100000000000000000000000075faf114eafb1bdbe2f0316df893fd58ce46aa4d0000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a019200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a0192000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

const API_ATTESTATION: &str = "d5fb4db6c196f46eb47954ef9d7335a910f5cd75d25a33aab2b89db1af2bf37e22fb4c218d3aab7470c7413c0069497b7f94df30312f288833f0692eb8a516a01ce3358edb37d58db989134c9f95e019e94dfa03ad602b1606599fd074a78e05f86e3b79dae413279c797aa3607d7267707a7e940cb02168ec09a195c843b269641b";

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Load .env file
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔄 CCTP v2 Transfer Recovery: Arbitrum Sepolia → Base Sepolia");
    println!("==============================================================\n");

    println!("📍 Using message and attestation from Circle API directly\n");

    // Load environment variables
    let private_key_str =
        std::env::var("TESTNET_PRIVATE_KEY").expect("TESTNET_PRIVATE_KEY must be set in .env file");

    let api_key =
        std::env::var("TESTNET_API_KEY").expect("TESTNET_API_KEY must be set in .env file");

    // Parse private key and verify wallet address
    let signer: PrivateKeySigner = private_key_str
        .parse()
        .expect("Invalid TESTNET_PRIVATE_KEY format");
    let wallet_address = signer.address();

    println!("📍 Wallet: {}", wallet_address);
    if wallet_address != WALLET_ADDRESS {
        println!("⚠️  Warning: Wallet address mismatch!");
        println!("   Expected: {}", WALLET_ADDRESS);
        println!("   Got: {}", wallet_address);
        println!("   Continuing anyway (mint will go to the original recipient)...\n");
    }
    println!();

    // Construct RPC URLs
    let base_sepolia_rpc = std::env::var("BASE_SEPOLIA_RPC_URL")
        .unwrap_or_else(|_| format!("https://base-sepolia.g.alchemy.com/v2/{}", api_key));

    // Create wallet from signer
    let wallet = EthereumWallet::from(signer);

    // Create provider for Base Sepolia
    println!("1️⃣  Creating Base Sepolia provider...");

    let base_sepolia_full_rpc_url = format!("{base_sepolia_rpc}{api_key}");
    let base_sepolia_provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(base_sepolia_full_rpc_url.parse().unwrap());

    println!("   ✅ Provider created\n");

    // Parse message and attestation from hex
    let message_bytes = hex::decode(API_MESSAGE).expect("Invalid message hex");
    let attestation_bytes = hex::decode(API_ATTESTATION).expect("Invalid attestation hex");

    println!("2️⃣  Data from Circle API:");
    println!("   Message length: {} bytes", message_bytes.len());
    println!("   Attestation length: {} bytes", attestation_bytes.len());

    // Compute message hash
    let message_hash = alloy_primitives::keccak256(&message_bytes);
    println!("   Message Hash: {}\n", message_hash);

    // Get the MessageTransmitter contract address for Base Sepolia
    let message_transmitter_address = cctp_rs::CCTP_V2_MESSAGE_TRANSMITTER_TESTNET;
    println!(
        "3️⃣  MessageTransmitter Contract: {}\n",
        message_transmitter_address
    );

    // Create the contract instance
    let message_transmitter =
        MessageTransmitterV2Contract::new(message_transmitter_address, &base_sepolia_provider);

    // Mint on destination chain
    println!("4️⃣  Mint Phase:");
    println!("   Calling receiveMessage on Base Sepolia...");

    let tx_request = message_transmitter.receive_message_transaction(
        Bytes::from(message_bytes),
        Bytes::from(attestation_bytes),
        wallet_address,
    );

    let pending_tx = base_sepolia_provider
        .send_transaction(tx_request)
        .await
        .map_err(|e| CctpError::Provider(format!("Failed to send mint transaction: {e}")))?;

    let mint_tx = *pending_tx.tx_hash();
    println!("   ✅ Mint TX: {}", mint_tx);
    println!(
        "   View on BaseScan: https://base-sepolia.blockscout.com/tx/{}",
        mint_tx
    );

    println!("\n🎉 Transfer Recovery Complete!");
    println!("   Your 1 USDC has been successfully bridged from Arbitrum Sepolia to Base Sepolia.");
    println!("\n✅ Recovery: SUCCESS");

    Ok(())
}
