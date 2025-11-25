# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.0] - 2025-11-25

### Changed

- **BREAKING**: Replaced `confirmation_average_time_seconds()` in `CctpV2` trait with two explicit methods:
  - `fast_transfer_confirmation_time_seconds()` - Returns attestation times with Fast Transfer enabled (5-20 seconds)
  - `standard_transfer_confirmation_time_seconds()` - Returns attestation times for Standard Transfer (5 seconds to 8 hours)

### Fixed

- Fixed v2 `CctpV2` trait returning block production times instead of Circle attestation times
  - Previously returned 1-12 seconds (block times)
  - Now returns actual attestation confirmation times based on Circle's documentation
  - Fast Transfer: Ethereum 20s, most chains 8s, high-perf chains 5s
  - Standard Transfer: Ethereum/L2s 19min, Avalanche 20s, Polygon 8min, Linea 8hrs

### Migration Guide

```rust
// Before (0.14.0)
let time = chain.confirmation_average_time_seconds()?; // Returned block times (wrong!)

// After (0.15.0)
// Choose based on your transfer mode:
let time = chain.fast_transfer_confirmation_time_seconds()?;     // Fast Transfer
let time = chain.standard_transfer_confirmation_time_seconds()?; // Standard (default)
```

## [0.14.0] - 2025-11-25

### Added

- **ERC20 Contract Wrapper**: New `Erc20Contract` for token approval and allowance operations
  - `approve()` for granting token spending permission before CCTP burns
  - `allowance()` for checking current approval amounts
  - Essential for the complete CCTP transfer workflow
  - Exposed publicly for direct use

- **Public Chain Addresses Module**: Exposed `chain::addresses` module for accessing contract addresses directly

- **V2-specific span function**: Added `get_v2_attestation_with_retry` for proper v2 observability with `TxHash` instead of message hash

### Changed

- **BREAKING**: Simplified attestation API with type-safe methods per version
  - V1 `Cctp`: `get_attestation(message_hash, ...)` - takes message hash directly
  - V2 `CctpV2`: `get_attestation(tx_hash, ...)` - takes transaction hash directly
  - Removed `AttestationQuery` enum in favor of compile-time type safety
  - Method signatures now clearly indicate what parameters are needed

- **BREAKING**: V2 `get_attestation()` now returns `(Vec<u8>, AttestationBytes)` instead of just `AttestationBytes`
  - Always returns both the canonical message and attestation from Circle's API
  - Eliminates foot-gun where users might use wrong message for minting
  - The `MessageSent` event log contains zeros in the nonce field; Circle fills this in before signing

- **BREAKING**: Removed `get_attestation_with_message()` from V2 - use `get_attestation()` instead

### Removed

- **BREAKING**: Removed `BridgeParams` from public API (was unused)

### Fixed

- Fixed v2 API path: renamed constant to `MESSAGES_PATH_V2` reflecting the actual endpoint format
- V2 now correctly uses `/v2/messages/{domain}?transactionHash={tx}` format
- Fixed critical v2 attestation bug: MessageSent event contains template message with zero nonce; now always uses canonical message from Circle's API

### Migration Guide

```rust
// V1 - Before (0.13.0)
bridge.get_attestation(AttestationQuery::by_message_hash(hash), max_attempts, poll_interval).await?;

// V1 - After (0.14.0)
let attestation = bridge.get_attestation(hash, None, None).await?;

// V2 - Before (0.13.0)
let attestation = bridge.get_attestation(tx_hash, None, None).await?;
let (message, _hash) = bridge.get_message_sent_event(tx_hash).await?; // BUG: wrong message!

// V2 - After (0.14.0)
let (message, attestation) = bridge.get_attestation(tx_hash, None, None).await?;
// Now use `message` (canonical from Circle API) for minting - this is the correct message!
```

## [0.13.0] - 2025-01-24

### Added

- **v1 MessageTransmitterContract**: Exposed `MessageTransmitterContract` in the public API
  - `receive_message_transaction()` for receiving cross-chain messages with attestation
  - `is_nonce_used()` for checking anti-replay protection
  - Matches v2 API symmetry for consistent developer experience

### Changed

- Removed unnecessary `#[allow(dead_code)]` attributes from public contract wrappers
  - Applies to `TokenMessengerContract`, `MessageTransmitterContract` (v1)
  - Applies to `TokenMessengerV2Contract`, `MessageTransmitterV2Contract` (v2)

## [0.12.0] - 2025-01-24

### Added

- **CCTP v2 Support**: Complete implementation of Circle's CCTP v2 protocol
  - Support for 26+ chains (10 mainnet, 6 testnet v2 chains + v1 chains)
  - New v2-only chains: Linea, Sonic, Sei
  - `CctpV2Bridge` and `CctpV2` traits with unified contract addresses
  - Comprehensive test suite with 149 unit tests (100% pass rate)

- **Fast Transfers**: Sub-30 second settlement with optimized finality thresholds
  - Standard transfers: 2000 confirmations (finalized)
  - Fast transfers: 1000 confirmations (confirmed)
  - Optional fee support for fast finality
  - Smart contract method selection based on configuration

- **Programmable Hooks**: Advanced integration capabilities
  - `depositForBurnWithHook()` support for custom destination logic
  - Hook data validation and configuration
  - Priority handling when combined with fast transfers

- **Enhanced Testing Infrastructure**
  - 149 comprehensive unit tests covering all business logic
  - `v2_integration_validation` example for CI/CD validation (no network required)
  - Comprehensive examples: `v2_standard_transfer`, `v2_fast_transfer`
  - Testing guidelines documentation for maintaining test quality

- **Contract Wrappers**: Direct access to type-safe contract interfaces
  - Exported `TokenMessengerContract` (v1) for direct contract interaction
  - Exported `TokenMessengerV2Contract` and `MessageTransmitterV2Contract` (v2)
  - Enables advanced use cases beyond the bridge abstraction
  - All wrappers include OpenTelemetry instrumentation built-in

### Changed

- **BREAKING**: New trait system for v1/v2 polymorphism
  - `CctpBridge` trait enables version-independent code
  - `CctpV1` and `CctpV2` traits for version-specific configuration
  - Clean separation between v1 (legacy) and v2 (current) implementations

- v2-specific modules added: `bridge/v2.rs`, `chain/v2.rs`, `contracts/v2/`

### Documentation

- Added comprehensive testing strategy section to README
- Documented integration test limitations and pragmatic approach
- Added v2-specific usage examples
- Created testing-guidelines.md for maintaining test quality
- Documented fast transfer behavior and fee structures

## [0.11.0] - 2025-01-24

### Changed

- **BREAKING**: Replaced primitive domain ID types with strongly-typed `DomainId` enum
  - Domain IDs are now type-safe with compile-time validation
  - Added `DomainId::from_u32()`, `DomainId::as_u32()`, `DomainId::name()`, and `Display` trait support
  - Improved API ergonomics with meaningful type names instead of raw integers
- Reorganized crate into conceptual modules for better code organization
  - Split code into logical modules: `bridge`, `chain`, `contracts`, `protocol`, and `spans`
  - Improved maintainability and discoverability of functionality
- Updated all Cargo dependencies to latest versions

### Added

- Enhanced OpenTelemetry instrumentation with comprehensive error tracking
  - Added structured error recording with full context preservation
  - Improved observability for debugging production issues
  - Better span hierarchy for cross-chain transfer tracing

## [0.10.1] - 2025-01-21

### Fixed

- **CRITICAL**: Fixed deserialization failure when Circle's Iris API returns `"attestation": "PENDING"` as a string instead of `null`
  - Added custom deserializer that gracefully handles the "PENDING" string by treating it as `None`
  - This fixes production crashes on Arbitrum and other chains where attestation polling would fail with: `JSON error: invalid value: string "PENDING", expected a valid hex string`
  - Enhanced error logging to include raw response body, message hash, and attempt number for better debugging
  - Added comprehensive test coverage for all attestation response formats (valid hex, "PENDING" string, null, empty string, missing field)
  - No breaking changes to public API

### Documentation

- Documented Circle API quirk where attestation field may be "PENDING" instead of null
- Added inline comments explaining the custom deserializer workaround

## [0.10.0] - 2024-11-16

### Changed

- **BREAKING**: Minimized public API surface for improved semver stability
  - Reduced from 30+ exports to 6 core types: `AttestationBytes`, `AttestationResponse`, `AttestationStatus`, `BridgeParams`, `Cctp`, `CctpV1`, `CctpError`, `Result`
  - Removed public exports of domain ID constants, contract address constants, and internal configuration
  - Internal modules (`domain_id`, `message_transmitter`, `token_messenger`) remain accessible via trait methods
- **BREAKING**: Made `BridgeParams` fields private (public getter methods remain available)
- **BREAKING**: Made `TokenMessengerContract.instance` field private
- Refactored `bridge.rs` (589 lines) into clean submodules (`bridge/cctp.rs`, `bridge/params.rs`, `bridge/config.rs`)

### Added

- Exposed `spans` module publicly for advanced OpenTelemetry instrumentation
- Added comprehensive documentation to `spans` module with usage examples

### Fixed

- Fixed all documentation warnings by referencing types instead of private modules

## [0.9.0] - 2025-01-XX

### Changed

- Updated Cargo dependencies to latest versions
- Converted snapshot tests to inline snapshots for better maintainability

### Fixed

- Replaced Anvil provider with HTTP provider in tests for improved stability

## [0.8.1] - 2025-01-XX

### Fixed

- Fixed attestation URL construction

## [0.8.0] - 2025-01-XX

### Changed

- Updated Cargo dependencies to latest versions

### Fixed

- Fixed typo in Iris API URL creation

## [0.7.0] - 2025-01-XX

### Added

- Implemented OpenTelemetry logging with structured spans for observability

### Changed

- Updated Cargo dependencies to latest versions

## [0.6.1] - 2025-01-XX

### Changed

- Refactored attestation to use `Bytes` type for improved type safety

## [0.6.0] - 2025-01-XX

### Changed

- **BREAKING**: Refactored to use `Url` type instead of `String` for API endpoints
- Improved type safety for URL handling

## [0.5.1] - 2025-01-XX

### Changed

- Updated Cargo dependencies

## [0.5.0] - 2025-01-XX

### Added

- Initial support for improved error handling
- Enhanced API ergonomics

## [0.4.0] - 2025-10-14

### Changed

- **BREAKING**: Replaced all `const &str` address constants with typed `const Address` using `address!()` macro
- **BREAKING**: Removed `InvalidAddress` error variant from `CctpError` enum (addresses now validated at compile time)
- Improved structured logging with static messages and event fields for better observability
- Eliminated runtime address parsing overhead with compile-time validation

### Fixed

- Removed potential runtime failures from address string parsing

## [0.3.0]

### Added

- Comprehensive CI/CD pipeline with GitHub Actions
- Security audit workflows with cargo-audit and cargo-deny
- Automated dependency updates with Dependabot
- Code coverage reporting with codecov
- Documentation generation and deployment
- Issue and PR templates for better contribution workflow
- Contribution guidelines and security policy

### Changed

- Updated examples to use modern Alloy provider API
- Improved error handling with detailed error types
- Enhanced documentation with better examples

### Fixed

- All clippy warnings resolved with strict linting
- Deprecated method usage in examples
- Format string improvements for better performance

## [0.2.2] - 2024-01-XX

### Added

- Support for Unichain network
- Comprehensive test suite with 69 tests
- Custom error types replacing anyhow
- Builder pattern for CCTP bridge configuration
- Attestation polling with configurable retry logic
- Multi-chain examples and documentation

### Changed

- Improved type safety with custom error handling
- Better API design with builder patterns
- Enhanced documentation with usage examples

### Fixed

- Removed all unsafe unwrap() and panic! calls
- Fixed chain support validation logic
- Improved error propagation throughout the library

### Security

- Eliminated potential panics from unsafe operations
- Added input validation for addresses and parameters
- Implemented proper error handling for network operations

## [0.2.1] - 2024-01-XX

### Fixed

- Repository field in Cargo.toml manifest

## [0.2.0] - 2024-01-XX

### Added

- Initial CCTP bridge implementation
- Support for major EVM chains (Ethereum, Arbitrum, Base, Optimism, Avalanche, Polygon)
- Circle Iris API integration for attestations
- Contract bindings for TokenMessenger and MessageTransmitter
- Basic examples and documentation

### Changed

- Updated to Alloy 1.0 framework
- Improved chain configuration system

## [0.1.0]

### Added

- Initial project structure
- Basic CCTP domain ID support
- Chain configuration foundations
