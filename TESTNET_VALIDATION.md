# Manual Testnet Validation Guide - v0.12.0

This guide walks through manual validation of CCTP v2 on Sepolia → Base Sepolia.

## Pre-requisites

### 1. Get Testnet Funds

- **Sepolia ETH**: https://sepoliafaucet.com/ or https://faucet.quicknode.com/ethereum/sepolia
- **Sepolia USDC**: https://faucet.circle.com/
- **Base Sepolia ETH**: https://bridge.base.org/deposit (bridge Sepolia ETH)

### 2. Set Up Environment Variables

```bash
# Private key for your test wallet
export PRIVATE_KEY="0x..."

# RPC endpoints (get free ones from Alchemy, Infura, or QuickNode)
export SEPOLIA_RPC_URL="https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY"
export BASE_SEPOLIA_RPC_URL="https://base-sepolia.g.alchemy.com/v2/YOUR_KEY"

# Your wallet address
export WALLET_ADDRESS="0x..."
```

### 3. Check Balances

```bash
# Verify you have funds
cast balance $WALLET_ADDRESS --rpc-url $SEPOLIA_RPC_URL
cast balance $WALLET_ADDRESS --rpc-url $BASE_SEPOLIA_RPC_URL

# Check USDC balance on Sepolia
# USDC on Sepolia: 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238
cast call 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238 "balanceOf(address)(uint256)" $WALLET_ADDRESS --rpc-url $SEPOLIA_RPC_URL
```

## Validation Test: Standard Transfer

### Expected Outcome
- Burn transaction succeeds on Sepolia
- Message extraction works correctly
- Attestation polling completes (10-15 min)
- Mint transaction succeeds on Base Sepolia
- USDC arrives at destination

### Step 1: Run the Integration Validation (No Network)

First, verify all configurations are correct:

```bash
cargo run --example v2_integration_validation
```

**Expected**: All 8 validation sections pass with green checkmarks.

### Step 2: Create Test Transfer Script

Create a file `test_transfer.sh`:

```bash
#!/bin/bash
set -e

echo "🧪 CCTP v2 Testnet Validation: Sepolia → Base Sepolia"
echo "======================================================"

# Configuration
AMOUNT="1000000"  # 1 USDC (6 decimals)
USDC_SEPOLIA="0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
TOKEN_MESSENGER_V2="0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA"  # v2 testnet unified address

echo ""
echo "1️⃣ Checking balances..."
echo "Sepolia USDC balance:"
cast call $USDC_SEPOLIA "balanceOf(address)(uint256)" $WALLET_ADDRESS --rpc-url $SEPOLIA_RPC_URL

echo ""
echo "2️⃣ Approving USDC spending..."
APPROVE_TX=$(cast send $USDC_SEPOLIA \
  "approve(address,uint256)" \
  $TOKEN_MESSENGER_V2 \
  $AMOUNT \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --json | jq -r '.transactionHash')

echo "Approval tx: $APPROVE_TX"
echo "Waiting for confirmation..."
cast receipt $APPROVE_TX --rpc-url $SEPOLIA_RPC_URL > /dev/null

echo ""
echo "3️⃣ Executing burn transaction..."
echo "This calls depositForBurn() with finality threshold 2000"
echo ""
echo "⚠️  MANUAL STEP REQUIRED:"
echo "You'll need to create a small Rust script or use cast to call:"
echo "depositForBurn(amount, destinationDomain, mintRecipient, burnToken)"
echo ""
echo "Parameters:"
echo "  amount: $AMOUNT (1 USDC)"
echo "  destinationDomain: 6 (Base)"
echo "  mintRecipient: $WALLET_ADDRESS"
echo "  burnToken: $USDC_SEPOLIA"
echo ""
echo "Alternative: Use the example code and uncomment the burn() call"
```

### Step 3: Minimal Rust Test Program

Create `examples/testnet_validation.rs`:

```rust
//! Manual testnet validation for v0.12.0
//! Run with: cargo run --example testnet_validation

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use cctp_rs::{CctpError, CctpV2Bridge};
use std::env;

#[tokio::main]
async fn main() -> Result<(), CctpError> {
    tracing_subscriber::fmt::init();

    println!("🧪 CCTP v2 Manual Testnet Validation");
    println!("====================================\n");

    // Get environment variables
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set");
    let sepolia_rpc = env::var("SEPOLIA_RPC_URL").expect("SEPOLIA_RPC_URL not set");
    let base_sepolia_rpc = env::var("BASE_SEPOLIA_RPC_URL").expect("BASE_SEPOLIA_RPC_URL not set");

    let signer: PrivateKeySigner = private_key.parse().expect("Invalid private key");
    let wallet_address = signer.address();

    println!("Wallet: {wallet_address}");
    println!("Source: Sepolia");
    println!("Destination: Base Sepolia\n");

    // Create providers
    let sepolia_provider = ProviderBuilder::new()
        .connect_http(sepolia_rpc.parse().unwrap());

    let base_sepolia_provider = ProviderBuilder::new()
        .connect_http(base_sepolia_rpc.parse().unwrap());

    // Create bridge
    let bridge = CctpV2Bridge::builder()
        .source_chain(NamedChain::Sepolia)
        .destination_chain(NamedChain::BaseSepolia)
        .source_provider(sepolia_provider)
        .destination_provider(base_sepolia_provider)
        .recipient(wallet_address)
        .build();

    println!("✅ Bridge Configuration:");
    println!("   Finality Threshold: {}", bridge.finality_threshold());
    println!("   Fast Transfer: {}", bridge.is_fast_transfer());
    println!("   Source Domain: {}", bridge.source_chain().cctp_v2_domain_id()?);
    println!("   Dest Domain: {}\n", bridge.destination_domain_id()?);

    // Validate contracts
    let tm = bridge.token_messenger_v2_contract()?;
    let mt = bridge.message_transmitter_v2_contract()?;

    println!("✅ Contract Addresses:");
    println!("   TokenMessenger: {tm}");
    println!("   MessageTransmitter: {mt}\n");

    println!("⚠️  Ready for manual testing!");
    println!("\nNext steps:");
    println!("1. Ensure you have Sepolia USDC (0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238)");
    println!("2. Approve spending: cast send <usdc> 'approve(address,uint256)' {tm} 1000000 ...");
    println!("3. Call burn(): bridge.burn(U256::from(1_000_000), wallet_address, usdc).await?");
    println!("4. Monitor attestation: bridge.get_attestation(&message_hash, None, None).await?");
    println!("5. Call mint() once attestation is ready");
    println!("\nExpected timeline: 10-15 minutes for attestation");

    Ok(())
}
```

### Step 4: Execute Validation

```bash
# 1. Run the validation setup
cargo run --example testnet_validation

# 2. You can either:
#    a) Modify the example to actually execute the transfer (uncomment the burn/mint calls)
#    b) Use cast to call the contracts directly
#    c) Use a wallet UI to interact with the contracts

# 3. Monitor the progress
# Track your burn transaction on Sepolia Etherscan
# Watch for the attestation from Circle's API
# Complete the mint on Base Sepolia
```

## Success Criteria

- ✅ Integration validation passes (all config correct)
- ✅ Burn transaction succeeds on Sepolia
- ✅ MessageSent event is extracted correctly
- ✅ Attestation polling completes without errors
- ✅ Attestation status transitions: pending → complete
- ✅ Mint transaction succeeds on Base Sepolia
- ✅ USDC balance increases on Base Sepolia
- ✅ Total time < 20 minutes

## Troubleshooting

### Issue: "Insufficient allowance"
**Solution**: Make sure you approved USDC spending first

### Issue: "Invalid domain"
**Solution**: Verify domain IDs match Circle's documentation (Sepolia=0, Base=6)

### Issue: "Attestation timeout"
**Solution**: Standard transfers take 10-15 min. Wait longer or check Circle API status

### Issue: "Invalid attestation signature"
**Solution**: Ensure you're using the correct attestation from Circle's API

## Alternative: Use Existing Transfer

If you want to skip the actual transfer, you can:

1. Find an existing Sepolia → Base Sepolia transfer
2. Use the burn tx hash to test message extraction
3. Verify attestation polling works
4. This validates the critical path without spending gas

Example tx to test with: [Find one on Sepolia Etherscan]

## Reporting Results

After validation, document:

```
✅ CCTP v2 Testnet Validation Results

Date: 2025-01-24
Version: v0.12.0
Route: Sepolia → Base Sepolia
Transfer Type: Standard (finality 2000)

Results:
- Burn TX: 0x...
- Message Hash: 0x...
- Attestation Time: X minutes
- Mint TX: 0x...
- Total Time: X minutes

Issues Found: None / [describe any issues]

Conclusion: READY FOR RELEASE / [needs fixes]
```
