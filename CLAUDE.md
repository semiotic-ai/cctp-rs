<!--
SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.

SPDX-License-Identifier: Apache-2.0
-->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Rust SDK for Circle's Cross-Chain Transfer Protocol (CCTP), enabling type-safe USDC bridging across 26+ EVM chains. Supports both CCTP v1 (legacy) and v2 (fast transfers with <30s settlement).

## Commands

```bash
cargo build                    # Build
cargo test                     # Run all tests
cargo test test_name           # Run single test
cargo test -- --nocapture      # Tests with output
cargo clippy --all-targets --all-features -- -D warnings  # Lint (required before commits)
cargo fmt                      # Format
cargo doc --open               # Generate docs
pipx run reuse lint            # SPDX license compliance check
```

## Architecture

### Module Structure

```
src/
├── bridge/          # Bridge implementations
│   ├── cctp.rs      # V1 bridge (Cctp struct)
│   ├── v2.rs        # V2 bridge (CctpV2 struct) with fast transfers
│   ├── config.rs    # PollingConfig for attestation polling
│   └── bridge_trait.rs  # CctpBridge trait
├── chain/           # Chain configuration
│   ├── addresses.rs # All contract addresses (centralized)
│   ├── config.rs    # CctpV1 trait - 7 original chains
│   └── v2.rs        # CctpV2 trait - 26+ chains with fast transfer
├── contracts/       # Type-safe contract bindings
│   ├── token_messenger.rs      # V1 TokenMessenger
│   ├── message_transmitter.rs  # V1 MessageTransmitter
│   └── v2/                     # V2 contracts
├── protocol/        # Protocol types
│   ├── attestation.rs  # Circle Iris API response types
│   ├── domain_id.rs    # CCTP domain ID constants
│   ├── finality.rs     # V2 finality thresholds
│   └── message.rs      # V2 message parsing
├── error.rs         # CctpError enum
└── spans.rs         # OpenTelemetry instrumentation
```

### Key Concepts

**V1 vs V2**: V1 returns attestation only (you extract message from chain). V2 returns both message and attestation from Circle's API (on-chain message has zeroed nonce, API has canonical version).

**Builder Pattern**: Both `Cctp` and `CctpV2` use `bon::builder` for construction with required/optional fields.

**Chain Traits**: `CctpV1` and `CctpV2` traits on `NamedChain` provide chain-specific config (addresses, domain IDs, confirmation times).

### External Dependencies

- **Alloy**: Ethereum interaction (providers, primitives, contract bindings)
- **Circle Iris API**: Attestation service - mainnet uses `iris-api.circle.com`, testnet uses `iris-api-sandbox.circle.com`
- **Contract ABIs**: Located in `abis/` directory, bound via `sol!` macro

## Adding Chain Support

1. Add addresses to `src/chain/addresses.rs`
2. Add domain ID to `src/protocol/domain_id.rs`
3. Implement trait in `src/chain/config.rs` (v1) or `src/chain/v2.rs` (v2)
4. Add confirmation times for v2 fast/standard transfers
