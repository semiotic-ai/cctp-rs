# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-11-03

### Added

- **Trait-based architecture** enabling comprehensive testing through dependency injection
- New `traits` module with `BlockchainProvider`, `AttestationProvider`, and `Clock` traits
- New `providers` module with production implementations:
  - `AlloyProvider` - Wraps Alloy RPC providers for blockchain operations
  - `IrisAttestationProvider` - Production/sandbox Circle API client
  - `TokioClock` - Real tokio::time-based clock implementation
- New `receipt_adapter` module with `ReceiptAdapter` trait for network-specific receipt handling
- `EthereumReceiptAdapter` implementation for Ethereum-compatible networks
- Comprehensive example in `examples/test_fakes.rs` demonstrating fake implementations for testing
- `AttestationResponse` now derives `Clone` to support test scenarios

### Changed

- **BREAKING**: `Cctp` struct now has 7 type parameters (up from 2) for full dependency injection:
  - `SN` - Source network type
  - `DN` - Destination network type
  - `SP` - Source blockchain provider
  - `DP` - Destination blockchain provider
  - `A` - Attestation provider
  - `C` - Clock implementation
  - `RA` - Receipt adapter
- **BREAKING**: Builder API now requires explicit provider injection:
  - Must wrap RPC providers with `AlloyProvider::new(provider)`
  - Must provide `attestation_provider(IrisAttestationProvider::production())`
  - Must provide `clock(TokioClock::new())`
  - Must provide `receipt_adapter(EthereumReceiptAdapter)`
- **BREAKING**: Updated to Alloy 1.0+ API conventions
- All examples updated to use new trait-based API
- Library documentation updated with migration guide

### Benefits

- Full testability with ability to inject fake implementations for adversarial testing
- Time control in tests without actual waiting
- Network flexibility through receipt adapter abstraction
- Type-safe external dependencies
- Maintained backward compatibility for chain configurations and contract addresses

### Migration

See the [Migration Guide](README.md#migration-guide-from-version-1x-to-200) in README.md for detailed upgrade instructions.

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