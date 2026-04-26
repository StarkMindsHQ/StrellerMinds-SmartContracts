# StrellerMinds Developer Tools Documentation

## Overview

This document describes the comprehensive developer experience tools implemented for the StrellerMinds Smart Contracts project. These tools are designed to streamline development, debugging, testing, and deployment processes.

## Table of Contents

1. [Developer CLI](#developer-cli)
2. [Debugging Tools](#debugging-tools)
3. [Testing Utilities](#testing-utilities)
4. [CI/CD Pipeline](#cicd-pipeline)
5. [Getting Started](#getting-started)
6. [Best Practices](#best-practices)

## Developer CLI

### Overview

The StrellerMinds CLI (`streller-cli`) is an interactive command-line interface that provides easy access to all development tools and workflows.

### Installation

```bash
cd utils/streller-cli
cargo build --release
```

### Usage

Run the CLI with:

```bash
cargo run --bin streller-cli
# or if built:
./target/release/streller-cli
```

### Main Menu Options

| Option | Description | Command |
|--------|-------------|---------|
| 🔍 System: Check Prerequisites | Verifies development environment setup | `make check` |
| 🌐 Network: Start Localnet | Starts Soroban local development network | `make localnet-start` |
| 🛑 Network: Stop Localnet | Stops Soroban local network | `make localnet-stop` |
| 📊 Network: Check Status | Shows local network status | `make localnet-status` |
| 🏗️ Build: Compile All Contracts | Builds all smart contracts | `make build` |
| 🚀 Deploy: Launch to Testnet | Deploys contracts to testnet | `make deploy-testnet` |
| 🧪 Test: Run All Tests | Executes complete test suite | `make test` |
| 🐛 Debug: Debug Tools | Access debugging utilities | [Debug Menu](#debugging-tools) |
| 🧪 Test Utils: Testing Utilities | Access testing tools | [Testing Menu](#testing-utilities) |
| 🧹 Clean: Remove Build Artifacts | Cleans build artifacts | `make clean` |

## Debugging Tools

### Access

From the CLI main menu, select "🐛 Debug: Debug Tools" to access the debugging menu.

### Debug Menu Options

#### 🔍 Contract Analysis
- Analyzes smart contract structure and dependencies
- Checks for Cargo.toml files and source code organization
- Reports contract statistics and metadata

**Features:**
- Contract discovery and enumeration
- Dependency analysis
- Source file counting
- Cargo.toml validation

#### 📊 Network Diagnostics
- Checks Soroban localnet status
- Validates Docker container health
- Tests network connectivity
- Shows Stellar network configuration

**Features:**
- Localnet status monitoring
- Docker container inspection
- Connectivity testing
- Network configuration display

#### 📝 Log Analysis
- Scans for available log files
- Analyzes log content for errors and warnings
- Provides log statistics and recent entries

**Features:**
- Log file discovery
- Error/warning pattern matching
- Log statistics (lines, errors, warnings)
- Recent log entry display

#### 🔧 Gas Profiling
- Executes gas profiling scripts
- Provides estimated gas costs for operations
- Analyzes contract deployment costs

**Features:**
- Gas cost estimation
- Contract deployment analysis
- Performance metrics

#### 📈 Performance Metrics
- Collects system and build metrics
- Analyzes build times and artifact sizes
- Provides performance insights

**Features:**
- System information gathering
- Build performance analysis
- Artifact size tracking
- Contract metrics

#### 🗂️ File System Check
- Validates project structure
- Checks file permissions
- Verifies important directories and files

**Features:**
- Directory structure validation
- File permission checking
- Important file verification

#### 🔗 Dependency Analysis
- Analyzes workspace dependencies
- Checks contract-specific dependencies
- Provides dependency insights

**Features:**
- Workspace dependency analysis
- Contract dependency mapping
- Version tracking

## Testing Utilities

### Access

From the CLI main menu, select "🧪 Test Utils: Testing Utilities" to access the testing menu.

### Testing Menu Options

#### 🧪 Run All Tests
- Executes complete test suite
- Runs unit tests and integration tests
- Provides comprehensive test coverage

**Features:**
- Prerequisite checking
- Unit test execution
- Integration test execution
- Result aggregation

#### 🔬 Unit Tests Only
- Runs unit tests exclusively
- Offers various unit test options

**Options:**
- Run all unit tests
- Run specific contract tests
- Run with verbose output
- Run with release optimizations

#### 🌐 Integration Tests
- Executes integration and E2E tests
- Provides network connectivity testing
- Validates contract deployment

**Options:**
- Full E2E test suite
- Quick smoke tests
- Network connectivity tests
- Contract deployment tests

#### 📊 Test Coverage
- Generates test coverage reports
- Uses cargo-tarpaulin for coverage analysis
- Provides HTML coverage reports

**Features:**
- Coverage tool installation
- HTML report generation
- Coverage statistics

#### 🏃 Performance Tests
- Analyzes build performance
- Tests contract execution performance
- Provides memory usage analysis
- Analyzes gas consumption

**Options:**
- Build performance testing
- Contract execution performance
- Memory usage analysis
- Gas consumption analysis

#### 🔍 Test Analysis
- Analyzes test results and patterns
- Counts test modules and patterns
- Provides testing insights

**Features:**
- Test module counting
- Pattern analysis
- Test statistics

#### 📝 Test Report
- Generates comprehensive test reports
- Provides test summaries and recommendations
- Creates markdown documentation

**Features:**
- Automated report generation
- Performance metrics
- Recommendations for improvement

#### ⚙️ Test Configuration
- Views current test configuration
- Configures test environment
- Sets up test data and networks

**Options:**
- View current configuration
- Configure test environment
- Set up test data
- Configure test networks

## CI/CD Pipeline

### Current Pipeline Analysis

The project implements a comprehensive CI/CD pipeline using GitHub Actions:

#### Workflow Files
- `ci.yml` - Main continuous integration pipeline
- `release.yml` - Release automation
- `security-audit.yml` - Security vulnerability scanning

#### CI Pipeline Stages

1. **Format Check**
   - Validates code formatting using rustfmt
   - Ensures consistent code style

2. **Clippy Lint**
   - Runs comprehensive linting
   - Enforces code quality standards
   - Catches potential bugs and performance issues

3. **Build Check**
   - Builds all contracts in release mode
   - Validates compilation success
   - Checks binary sizes

4. **Test Suite**
   - Runs unit tests across workspace
   - Excludes E2E tests for CI efficiency
   - Provides verbose test output

#### Pipeline Features

- **Caching**: Optimizes build times with cargo registry caching
- **Parallel Execution**: Runs jobs in parallel for faster feedback
- **Comprehensive Validation**: Format, lint, build, and test verification
- **Error Reporting**: Detailed failure reporting with job status tracking

### Pipeline Commands

The Makefile provides pipeline-compatible commands:

- `make ci-test` - CI workflow simulation
- `make dev-test` - Development workflow
- `make check-code` - Code formatting and linting

## Getting Started

### Prerequisites

Ensure you have the following installed:

1. **Rust/Cargo** - Latest stable version
2. **Soroban CLI** - Stellar smart contract development
3. **Docker** - For localnet and container operations
4. **Make** - For build automation

### Quick Start

1. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd StrellerMinds-SmartContracts
   ```

2. **Check Prerequisites**
   ```bash
   make check
   # or use CLI:
   cargo run --bin streller-cli
   # Select "System: Check Prerequisites"
   ```

3. **Start Development Environment**
   ```bash
   make localnet-start
   make build
   ```

4. **Run Tests**
   ```bash
   make test
   # or use CLI testing utilities
   ```

5. **Deploy Contracts**
   ```bash
   make deploy-testnet
   ```

### Development Workflow

1. **Daily Development**
   ```bash
   make clean
   make build
   make unit-test
   ```

2. **Full Testing**
   ```bash
   make e2e-test
   ```

3. **Before Deployment**
   ```bash
   make check-code
   make test
   make build
   ```

## Best Practices

### Development Best Practices

1. **Use the CLI Tools**
   - Leverage the interactive CLI for common tasks
   - Use debugging tools for troubleshooting
   - Utilize testing utilities for comprehensive testing

2. **Regular Testing**
   - Run unit tests frequently during development
   - Use integration tests before deployment
   - Monitor test coverage regularly

3. **Debugging Workflow**
   - Use contract analysis for code review
   - Monitor network diagnostics for connectivity issues
   - Analyze logs for error tracking

4. **Performance Monitoring**
   - Track build performance metrics
   - Monitor gas consumption
   - Analyze memory usage patterns

### CI/CD Best Practices

1. **Pipeline Optimization**
   - Use caching for faster builds
   - Run tests in parallel when possible
   - Monitor pipeline performance

2. **Code Quality**
   - Ensure all formatting checks pass
   - Address clippy warnings promptly
   - Maintain high test coverage

3. **Security**
   - Regular security audits
   - Keep dependencies updated
   - Review security scan results

### Troubleshooting

#### Common Issues

1. **Build Failures**
   - Check prerequisites with `make check`
   - Verify Rust toolchain version
   - Clean build artifacts with `make clean`

2. **Network Issues**
   - Use network diagnostics tools
   - Verify Docker is running
   - Check localnet status

3. **Test Failures**
   - Analyze test reports
   - Check test configuration
   - Verify test environment setup

#### Getting Help

1. **Use Debug Tools**
   - Contract analysis for code issues
   - Log analysis for error tracking
   - Network diagnostics for connectivity

2. **Documentation**
   - Refer to this documentation
   - Check inline code documentation
   - Review test cases for examples

## Tool Effectiveness Review

### Metrics for Success

1. **Developer Productivity**
   - Reduced setup time
   - Faster development cycles
   - Simplified debugging process

2. **Code Quality**
   - Improved test coverage
   - Reduced bug count
   - Better code consistency

3. **Pipeline Efficiency**
   - Faster CI/CD execution
   - Better error reporting
   - Improved deployment reliability

### Review Process

1. **Usage Analytics**
   - Track CLI usage patterns
   - Monitor tool adoption
   - Collect developer feedback

2. **Performance Metrics**
   - Measure build time improvements
   - Track test execution speed
   - Monitor resource usage

3. **Quality Metrics**
   - Code coverage tracking
   - Bug detection rates
   - Security vulnerability detection

### Continuous Improvement

1. **Regular Reviews**
   - Monthly tool effectiveness reviews
   - Quarterly developer surveys
   - Annual comprehensive assessment

2. **Feedback Integration**
   - Developer feedback collection
   - Tool usage pattern analysis
   - Performance metric tracking

3. **Tool Evolution**
   - Adding new debugging capabilities
   - Enhancing testing utilities
   - Improving CLI user experience

## Conclusion

The StrellerMinds Developer Tools provide a comprehensive suite of utilities designed to enhance the developer experience, streamline workflows, and improve code quality. By leveraging these tools effectively, developers can focus on building high-quality smart contracts while maintaining robust testing and deployment practices.

For questions, issues, or suggestions for improvement, please refer to the project's issue tracker or contact the development team.
