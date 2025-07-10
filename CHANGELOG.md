# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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