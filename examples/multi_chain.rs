// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Example of multi-chain CCTP support
//!
//! This example demonstrates how to work with multiple chains and their configurations.
//!
//! Run with: `cargo run --example multi_chain`

use alloy_chains::NamedChain;
use cctp_rs::{CctpError, CctpV1};

fn main() -> Result<(), CctpError> {
    println!("🌐 Multi-Chain CCTP Configuration Example");
    println!("========================================\n");

    // Display all supported chains
    display_supported_chains()?;

    // Compare chain configurations
    compare_chain_configs()?;

    // Show routing possibilities
    display_routing_matrix()?;

    Ok(())
}

/// Display all chains that support CCTP
fn display_supported_chains() -> Result<(), CctpError> {
    println!("📋 Supported Chains:\n");

    let chains = vec![
        // Mainnets
        ("Ethereum", NamedChain::Mainnet),
        ("Arbitrum", NamedChain::Arbitrum),
        ("Base", NamedChain::Base),
        ("Optimism", NamedChain::Optimism),
        ("Avalanche", NamedChain::Avalanche),
        ("Polygon", NamedChain::Polygon),
        ("Unichain", NamedChain::Unichain),
        // Testnets
        ("Sepolia", NamedChain::Sepolia),
        ("Arbitrum Sepolia", NamedChain::ArbitrumSepolia),
        ("Base Sepolia", NamedChain::BaseSepolia),
        ("Optimism Sepolia", NamedChain::OptimismSepolia),
        ("Avalanche Fuji", NamedChain::AvalancheFuji),
        ("Polygon Amoy", NamedChain::PolygonAmoy),
    ];

    println!("Mainnets:");
    for (name, chain) in chains.iter().take(7) {
        if chain.is_supported() {
            let domain_id = chain.cctp_domain_id()?;
            println!("  ✅ {name} (Domain ID: {domain_id})");
        }
    }

    println!("\nTestnets:");
    for (name, chain) in chains.iter().skip(7) {
        if chain.is_supported() {
            let domain_id = chain.cctp_domain_id()?;
            println!("  ✅ {name} (Domain ID: {domain_id})");
        }
    }

    Ok(())
}

/// Compare confirmation times across chains
fn compare_chain_configs() -> Result<(), CctpError> {
    println!("\n⏱️  Chain Confirmation Times:\n");

    let mut configs: Vec<(String, u64)> = Vec::new();

    // Collect confirmation times
    let chains = vec![
        ("Ethereum", NamedChain::Mainnet),
        ("Arbitrum", NamedChain::Arbitrum),
        ("Avalanche", NamedChain::Avalanche),
        ("Polygon", NamedChain::Polygon),
    ];

    for (name, chain) in chains {
        let confirmation_time = chain.confirmation_average_time_seconds()?;
        configs.push((name.to_string(), confirmation_time));
    }

    // Sort by confirmation time
    configs.sort_by_key(|k| k.1);

    // Display sorted results
    for (name, time) in configs {
        let minutes = time / 60;
        let seconds = time % 60;

        let time_str = if minutes > 0 {
            format!("{minutes} min {seconds} sec")
        } else {
            format!("{seconds} sec")
        };

        let emoji = get_speed_emoji(time);
        println!("  {emoji} - {name}: {time_str}");
    }

    Ok(())
}

/// Display routing possibilities between chains
fn display_routing_matrix() -> Result<(), CctpError> {
    println!("\n🔀 Routing Matrix (Sample Routes):\n");

    let routes = vec![
        ("Ethereum", "Arbitrum", "Most popular L1 → L2 route"),
        ("Ethereum", "Base", "Coinbase's L2 solution"),
        ("Arbitrum", "Optimism", "L2 → L2 routing"),
        ("Polygon", "Avalanche", "Cross-chain DeFi"),
    ];

    for (source, dest, description) in routes {
        println!("  {source} → {dest}");
        println!("     └─ {description}");

        // Get domain IDs for the route
        let source_chain = match source {
            "Ethereum" => NamedChain::Mainnet,
            "Arbitrum" => NamedChain::Arbitrum,
            "Polygon" => NamedChain::Polygon,
            _ => continue,
        };

        let dest_chain = match dest {
            "Arbitrum" => NamedChain::Arbitrum,
            "Base" => NamedChain::Base,
            "Optimism" => NamedChain::Optimism,
            "Avalanche" => NamedChain::Avalanche,
            _ => continue,
        };

        let source_domain = source_chain.cctp_domain_id()?;
        let dest_domain = dest_chain.cctp_domain_id()?;

        println!("     └─ Domain IDs: {source_domain} → {dest_domain}\n");
    }

    Ok(())
}

/// Get emoji based on confirmation speed
fn get_speed_emoji(seconds: u64) -> &'static str {
    match seconds {
        0..=60 => "🚀",    // Less than 1 minute
        61..=300 => "⚡",  // 1-5 minutes
        301..=900 => "🔄", // 5-15 minutes
        _ => "⏳",         // More than 15 minutes
    }
}
