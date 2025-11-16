# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
