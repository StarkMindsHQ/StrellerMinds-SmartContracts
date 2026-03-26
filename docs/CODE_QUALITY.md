# Code Quality and Linting

This document outlines the comprehensive code quality and linting setup for the StrellerMinds Smart Contracts project.

## Overview

We maintain high code quality standards through automated linting, security checks, and continuous integration. All code must pass these checks before being merged.

## Linting Tools

### 1. Rustfmt
- **Purpose**: Code formatting consistency
- **Configuration**: Standard Rustfmt with custom settings in `rustfmt.toml`
- **Usage**: `cargo fmt --all`

### 2. Clippy
- **Purpose**: Catching common mistakes and improving code quality
- **Configuration**: Strict pedantic and nursery lints enabled
- **Usage**: `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### 3. Cargo Audit
- **Purpose**: Security vulnerability scanning
- **Usage**: `cargo audit`

### 4. Cargo Deny
- **Purpose**: License compliance and dependency security
- **Configuration**: `deny.toml`
- **Usage**: `cargo deny check`

### 5. Cargo Machete
- **Purpose**: Detect unused dependencies
- **Usage**: `cargo machete`

## Configuration Files

### Workspace Configuration
- **`Cargo.toml`**: Workspace-level lint configuration
- **`clippy.toml`**: Comprehensive Clippy lint rules
- **`deny.toml`**: License and security policies
- **`rustfmt.toml`**: Code formatting rules

### Pre-commit Hooks
- **`.pre-commit-config.yaml`**: Automated checks before commits
- **Tools**: Rustfmt, Clippy, security scans, documentation checks

## CI/CD Integration

### GitHub Actions Workflows
1. **`ci.yml`**: Basic CI with format, clippy, and build checks
2. **`lint.yml`**: Comprehensive quality assurance pipeline
3. **`security-audit.yml`**: Security vulnerability scanning

### Quality Gates
- All checks must pass before merge
- Zero tolerance for security vulnerabilities
- Strict linting with `-D warnings` flag

## Running Checks Locally

### Quick Checks
```bash
# Format and basic linting
./scripts/lint.sh --fast

# Auto-fix formatting issues
./scripts/lint.sh --fix
```

### Comprehensive Checks
```bash
# Run all quality checks
./scripts/lint.sh

# Individual checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo audit
cargo deny check
cargo test --workspace
```

## Linting Rules

### Enabled Clippy Lints
- **Pedantic**: Strict code quality guidelines
- **Nursery**: Experimental lints for future-proofing
- **Cargo**: Cargo-specific best practices

### Key Rules
- No unused imports or variables
- Consistent error handling patterns
- Proper documentation for public APIs
- Security best practices enforcement
- License compliance verification

## Code Quality Standards

### Documentation Requirements
- All public functions must have documentation
- Examples for complex APIs
- Clear error messages and handling

### Security Standards
- No hardcoded secrets or keys
- Proper input validation
- Secure dependency management
- Regular vulnerability scanning

### Performance Standards
- Efficient gas usage for smart contracts
- Minimal binary sizes
- No unnecessary dependencies
- Optimized data structures

## Troubleshooting

### Common Issues
1. **Formatting errors**: Run `cargo fmt --all` to fix
2. **Clippy warnings**: Address each warning individually
3. **Security vulnerabilities**: Update affected dependencies
4. **License issues**: Review and update dependency licenses

### Getting Help
- Check the [Rust Documentation](https://doc.rust-lang.org/)
- Review [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- Consult the project's issue tracker

## Contributing

All contributors must:
1. Install pre-commit hooks: `pre-commit install`
2. Pass all local checks before pushing
3. Address any CI/CD failures promptly
4. Follow the established coding standards

## Continuous Improvement

We regularly:
- Review and update linting rules
- Add new security checks
- Improve documentation coverage
- Optimize performance benchmarks

---

For more details, see the [Contributing Guidelines](CONTRIBUTING.md) and [Security Policy](SECURITY.md).
