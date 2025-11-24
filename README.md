# cctp-rs

[![Crates.io](https://img.shields.io/crates/v/cctp-rs.svg)](https://crates.io/crates/cctp-rs)
[![Documentation](https://docs.rs/cctp-rs/badge.svg)](https://docs.rs/cctp-rs)
[![License](https://img.shields.io/crates/l/cctp-rs.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/semiotic-ai/cctp-rs/ci.yml?branch=main)](https://github.com/semiotic-ai/cctp-rs/actions)

A production-ready Rust implementation of Circle's Cross-Chain Transfer Protocol (CCTP), enabling seamless USDC transfers across blockchain networks.

## Features

- 🚀 **Type-safe** contract interactions using Alloy
- 🔄 **Multi-chain support** for 26+ mainnet and testnet networks
- 📦 **Builder pattern** for intuitive API usage
- ⚡ **CCTP v2 support** with fast transfers (<30s settlement)
- 🎯 **Programmable hooks** for advanced use cases
- 🔍 **Comprehensive observability** with OpenTelemetry integration

## Supported Chains

### CCTP v2 (Current)

#### Mainnet
- Ethereum, Arbitrum, Base, Optimism, Avalanche, Polygon, Unichain
- Linea, Sonic, Sei (v2-only chains)

#### Testnet
- Sepolia, Arbitrum Sepolia, Base Sepolia, Optimism Sepolia
- Avalanche Fuji, Polygon Amoy

### CCTP v1 (Legacy)
Also supported for backwards compatibility

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
cctp-rs = "0.11.0"
```

### Basic Example

```rust
use cctp_rs::{Cctp, CctpError};
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, ProviderBuilder};

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    // Create providers for source and destination chains
    let eth_provider = ProviderBuilder::new()
        .on_http("https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY".parse()?);
    
    let arb_provider = ProviderBuilder::new()
        .on_http("https://arb-mainnet.g.alchemy.com/v2/YOUR_API_KEY".parse()?);

    // Set up the CCTP bridge
    let bridge = Cctp::builder()
        .source_chain(NamedChain::Mainnet)
        .destination_chain(NamedChain::Arbitrum)
        .source_provider(eth_provider)
        .destination_provider(arb_provider)
        .recipient("0xYourRecipientAddress".parse()?)
        .build();

    // Get contract addresses
    let token_messenger = bridge.token_messenger_contract()?;
    let destination_domain = bridge.destination_domain_id()?;
    
    println!("Token Messenger: {}", token_messenger);
    println!("Destination Domain: {}", destination_domain);
    
    Ok(())
}
```

### Bridging USDC

```rust
use cctp_rs::{Cctp, BridgeParams, CctpError};
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};

async fn bridge_usdc(bridge: &Cctp<impl Provider>) -> Result<(), CctpError> {
    // Step 1: Prepare bridge parameters
    let params = BridgeParams::builder()
        .from_address("0xYourAddress".parse()?)
        .recipient("0xRecipientAddress".parse()?)
        .token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?) // USDC on Ethereum
        .amount(U256::from(1_000_000)) // 1 USDC (6 decimals)
        .build();

    // Step 2: Approve USDC spending (implement this based on your provider)
    // approve_usdc_spending(&params).await?;

    // Step 3: Burn USDC on source chain (get tx hash)
    let burn_tx_hash = "0x...".parse()?; // From your burn transaction

    // Step 4: Get message and attestation
    let (message, message_hash) = bridge.get_message_sent_event(burn_tx_hash).await?;
    
    // Step 5: Wait for attestation
    let attestation = bridge.get_attestation(&message_hash, None, None).await?;
    
    println!("Bridge successful! Attestation: {:?}", attestation);
    
    // Step 6: Mint on destination chain using the attestation
    // mint_on_destination(&message, &attestation).await?;
    
    Ok(())
}
```

## Architecture

The library is organized into several key modules:

- **`bridge`** - Core CCTP bridge implementation
- **`chain`** - Chain-specific configurations and support
- **`attestation`** - Attestation response types from Circle's Iris API
- **`error`** - Comprehensive error types for proper error handling
- **`contracts`** - Type-safe bindings for TokenMessenger and MessageTransmitter

## Error Handling

cctp-rs provides detailed error types for different failure scenarios:

```rust
use cctp_rs::{CctpError, Cctp};

match bridge.get_attestation(&message_hash, None, None).await {
    Ok(attestation) => println!("Success: {:?}", attestation),
    Err(CctpError::AttestationTimeout) => println!("Timeout waiting for attestation"),
    Err(CctpError::ChainNotSupported { chain }) => println!("Chain {} not supported", chain),
    Err(e) => println!("Other error: {}", e),
}
```

## Advanced Usage

### Custom Polling Configuration

```rust
// Wait up to 10 minutes with 30-second intervals
let attestation = bridge.get_attestation(
    &message_hash,
    Some(20),        // max attempts
    Some(30),        // poll interval in seconds
).await?;
```

### Chain Configuration

```rust
use cctp_rs::CctpV1;
use alloy_chains::NamedChain;

// Get chain-specific information
let chain = NamedChain::Arbitrum;
let confirmation_time = chain.confirmation_average_time_seconds()?;
let domain_id = chain.cctp_domain_id()?;
let token_messenger = chain.token_messenger_address()?;

println!("Arbitrum confirmation time: {} seconds", confirmation_time);
println!("Domain ID: {}", domain_id);
println!("Token Messenger: {}", token_messenger);
```

## Examples

Check out the [`examples/`](examples/) directory for complete working examples:

### CCTP v2 Examples
- [`v2_integration_validation.rs`](examples/v2_integration_validation.rs) - Comprehensive v2 validation (no network required)
- [`v2_standard_transfer.rs`](examples/v2_standard_transfer.rs) - Standard transfer with finality
- [`v2_fast_transfer.rs`](examples/v2_fast_transfer.rs) - Fast transfer (<30s settlement)

### CCTP v1 Examples (Legacy)
- [`basic_bridge.rs`](examples/basic_bridge.rs) - Simple USDC bridge example
- [`attestation_monitoring.rs`](examples/attestation_monitoring.rs) - Monitor attestation status
- [`multi_chain.rs`](examples/multi_chain.rs) - Bridge across multiple chains

Run examples with:

```bash
# Recommended: Run v2 integration validation
cargo run --example v2_integration_validation

# Or run specific examples
cargo run --example v2_fast_transfer
cargo run --example basic_bridge
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Testing

### Unit Tests

Run the full test suite with:

```bash
cargo test --all-features
```

All 149 unit tests validate:
- Contract method selection logic
- Domain ID resolution and mapping
- Configuration validation
- URL construction for Circle's Iris API
- Error handling and edge cases
- Cross-chain compatibility
- Fast transfer support
- Hooks integration

### Integration Validation

We provide comprehensive runnable examples that validate the complete v2 API without requiring network access:

```bash
# Validate all v2 configurations (no network required)
cargo run --example v2_integration_validation

# Educational examples showing complete flows
cargo run --example v2_standard_transfer
cargo run --example v2_fast_transfer
```

The `v2_integration_validation` example validates:
- Chain support matrix (26+ chains)
- Domain ID mappings against Circle's official values
- Contract address consistency (unified v2 addresses)
- Bridge configuration variations (standard, fast, hooks)
- API endpoint construction (mainnet vs testnet)
- Fast transfer support and fee structures
- Error handling for unsupported chains
- Cross-chain compatibility

### Live Testnet Testing

For pre-release validation on testnet:

1. Get testnet tokens from [Circle's faucet](https://faucet.circle.com)
2. Update examples with your addresses and RPC endpoints
3. Set environment variables for private keys
4. Execute and monitor the full flow

**Note**: Integration tests requiring Circle's Iris API and live blockchains are not run in CI due to:
- Cost (gas fees on every test run)
- Time (10-15 minutes per transfer for attestation)
- Flakiness (network dependencies and rate limits)
- Complexity (requires funded wallets with private keys)

Instead, we validate via extensive unit tests and runnable examples. This approach ensures reliability while maintaining fast CI/CD pipelines.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Circle](https://www.circle.com/) for creating CCTP
- [Alloy](https://github.com/alloy-rs) for the excellent Ethereum libraries
- The Rust community for amazing tools and support

## Resources

- [CCTP Documentation](https://developers.circle.com/stablecoins/cctp-getting-started)
- [API Reference](https://docs.rs/cctp-rs)
- [GitHub Repository](https://github.com/semiotic-ai/cctp-rs)
