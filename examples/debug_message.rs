// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Debug script to compare message extraction vs Circle API
//!
//! Run with: `cargo run --example debug_message`

use alloy_primitives::{b256, hex, B256};
use alloy_provider::{Provider, ProviderBuilder};
use dotenvy::dotenv;

const BURN_TX_HASH: B256 =
    b256!("f2ca30dd25939d665a0c2f69692777f2b3577f645e1c833ce54959ca0905ecc6");

// Circle API message (without 0x prefix)
const API_MESSAGE: &str = "000000010000000300000006eb20f0033c7fbbc8a633e215369eff6c48f0a36037134713ed435a206c044b8f0000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000000000000000000000000000000000000000000000000007d0000007d00000000100000000000000000000000075faf114eafb1bdbe2f0316df893fd58ce46aa4d0000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a019200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a0192000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("🔍 Debug: Comparing Message Extraction\n");

    let api_key = std::env::var("TESTNET_API_KEY")?;
    let arb_rpc = std::env::var("ARBITRUM_SEPOLIA_RPC_URL")?;
    let full_url = format!("{arb_rpc}{api_key}");

    let provider = ProviderBuilder::new().connect_http(full_url.parse()?);

    // Get transaction receipt
    println!("1️⃣  Fetching transaction receipt...");
    let receipt = provider
        .get_transaction_receipt(BURN_TX_HASH)
        .await?
        .expect("Receipt not found");

    println!("   Found {} logs\n", receipt.inner.logs().len());

    // Find MessageSent log
    let message_sent_topic = alloy_primitives::keccak256(b"MessageSent(bytes)");
    println!("2️⃣  MessageSent topic: {}\n", message_sent_topic);

    for (i, log) in receipt.inner.logs().iter().enumerate() {
        println!("   Log {}: topic0 = {:?}", i, log.topics().first());

        if log
            .topics()
            .first()
            .is_some_and(|t| t.as_slice() == message_sent_topic)
        {
            println!("\n3️⃣  Found MessageSent log!");
            println!("   Raw data length: {} bytes", log.data().data.len());
            println!("   Raw data (hex): 0x{}\n", hex::encode(&log.data().data));

            // The log data is ABI-encoded: offset (32 bytes) + length (32 bytes) + message
            let raw_data = &log.data().data;

            // First 32 bytes: offset to the bytes data (should be 0x20 = 32)
            let offset = &raw_data[0..32];
            println!("   Offset (first 32 bytes): 0x{}", hex::encode(offset));

            // Next 32 bytes: length of the bytes data
            let length_bytes = &raw_data[32..64];
            let length = u64::from_be_bytes(length_bytes[24..32].try_into()?);
            println!("   Length field: {} bytes", length);

            // Remaining bytes: the actual message
            let message = &raw_data[64..64 + length as usize];
            println!("   Extracted message length: {} bytes", message.len());
            println!("   Extracted message: 0x{}\n", hex::encode(message));

            // Compare with Circle API
            let api_message_bytes = hex::decode(API_MESSAGE)?;
            println!("4️⃣  Comparison with Circle API:");
            println!("   API message length: {} bytes", api_message_bytes.len());

            if message == api_message_bytes.as_slice() {
                println!("   ✅ MATCH! Messages are identical");
            } else {
                println!("   ❌ MISMATCH!");

                // Find where they differ
                let min_len = message.len().min(api_message_bytes.len());
                for i in 0..min_len {
                    if message[i] != api_message_bytes[i] {
                        println!(
                            "   First difference at byte {}: extracted=0x{:02x}, api=0x{:02x}",
                            i, message[i], api_message_bytes[i]
                        );
                        break;
                    }
                }

                if message.len() != api_message_bytes.len() {
                    println!(
                        "   Length difference: extracted={}, api={}",
                        message.len(),
                        api_message_bytes.len()
                    );
                }
            }

            // Compute hashes
            let extracted_hash = alloy_primitives::keccak256(message);
            let api_hash = alloy_primitives::keccak256(&api_message_bytes);

            println!("\n5️⃣  Hash comparison:");
            println!("   Extracted message hash: {}", extracted_hash);
            println!("   API message hash:       {}", api_hash);
            println!("   Expected from logs:     0x6c3f18b0822232dfa4b41429a62bfa5241d28db1edce05ea265896cba4075ed9");
        }
    }

    Ok(())
}
