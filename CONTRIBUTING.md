# Contributing to cctp-rs

Thank you for your interest in contributing to cctp-rs! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Security](#security)

## Code of Conduct

This project adheres to the highest standards of professional conduct. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature/fix
4. Make your changes
5. Test your changes thoroughly
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75.0 or later
- Git

### Setup

```bash
# Clone your fork
git clone https://github.com/your-username/cctp-rs.git
cd cctp-rs

# Install dependencies
cargo build

# Run tests to ensure everything works
cargo test
```

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feature/add-new-chain-support`
- `fix/attestation-timeout-bug`
- `docs/improve-examples`

### Commit Messages

Follow conventional commit format:

- `feat: add support for Polygon chain`
- `fix: resolve attestation polling timeout`
- `docs: improve README examples`
- `test: add integration tests for Base chain`

## Testing

We maintain comprehensive test coverage. Please ensure:

1. **Unit Tests**: Add tests for all new functionality

```bash
cargo test
```

2. **Integration Tests**: Test end-to-end flows when applicable

3. **Documentation Tests**: Ensure all code examples in docs work

```bash
cargo test --doc
```

4. **Examples**: Verify examples compile and run

```bash
cargo build --examples
```

### Test Guidelines

- Write clear, descriptive test names
- Test both success and error cases
- Use `rstest` for parameterized tests
- Mock external dependencies when appropriate

## Code Style

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy with strict settings
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

- Document all public APIs with clear examples
- Use `///` for documentation comments
- Include usage examples in module-level docs
- Keep documentation up-to-date with code changes

### Rust Guidelines

- Follow Rust API guidelines
- Use `Result<T, CctpError>` for error handling
- Prefer explicit error types over `anyhow`
- Use the `?` operator for error propagation
- Follow naming conventions (snake_case for functions, PascalCase for types)

## Submitting Changes

### Pull Request Process

1. **Update Documentation**: Ensure README, docs, and examples reflect your changes
2. **Add Tests**: Include comprehensive tests for new functionality
3. **Run CI Locally**: Ensure all checks pass
4. **Fill PR Template**: Provide thorough description of changes
5. **Link Issues**: Reference any related issues

### PR Requirements

- [ ] All tests pass
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Examples work correctly
- [ ] Breaking changes are clearly documented

### Review Process

- PRs require review before merging
- Address reviewer feedback promptly
- Keep PRs focused and reasonably sized
- Squash commits before merging if requested

## Chain Support

When adding support for new chains:

1. **Research**: Verify CCTP deployment addresses
2. **Domain IDs**: Confirm correct domain IDs from Circle documentation
3. **Testing**: Test against both mainnet and testnet deployments
4. **Documentation**: Update chain support documentation
5. **Examples**: Add chain to multi-chain examples

### Required Information for New Chains

- Chain name and ID
- CCTP domain ID
- TokenMessenger contract address
- MessageTransmitter contract address
- Average confirmation time
- RPC endpoints for testing

## Security

### Security Guidelines

- Never commit private keys, mnemonics, or API keys
- Validate all user inputs
- Use secure random number generation
- Follow principle of least privilege
- Report security vulnerabilities privately

### Responsible Disclosure

For security vulnerabilities:

1. **Do not** create public issues
2. Email security concerns privately
3. Allow time for fixes before disclosure
4. Follow coordinated disclosure timeline

## CCTP Protocol Knowledge

Contributors should understand:

- [Circle's CCTP documentation](https://developers.circle.com/stablecoins/cctp-protocol)
- Message attestation flow
- Burn and mint mechanism
- Domain ID system
- Contract interaction patterns

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and community support
- **Documentation**: Check existing docs and examples first

## Recognition

Contributors will be recognized in:

- Git commit history
- Release notes for significant contributions
- README contributor section (for major contributions)

Thank you for contributing to cctp-rs! 🚀
