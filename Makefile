# StrellerMinds Smart Contracts - Makefile
# 
# This Makefile provides convenient commands for development and testing

.PHONY: help build test unit-test e2e-test localnet-start localnet-stop localnet-status clean deploy-testnet deploy-mainnet

# Colors for output
GREEN=\033[0;32m
YELLOW=\033[1;33m
BLUE=\033[0;34m
NC=\033[0m # No Color

# Default target
help:
	@echo "$(BLUE)StrellerMinds Smart Contracts$(NC)"
	@echo "=============================="
	@echo ""
	@echo "Available commands:"
	@echo ""
	@echo "  $(GREEN)build$(NC)               - Build all smart contracts"
	@echo "  $(GREEN)test$(NC)                - Run all tests (unit + E2E)"
	@echo "  $(GREEN)unit-test$(NC)           - Run unit tests only"
	@echo "  $(GREEN)e2e-test$(NC)            - Run E2E tests (starts localnet)"
	@echo "  $(GREEN)e2e-test-quick$(NC)      - Run quick E2E smoke tests"
	@echo "  $(GREEN)e2e-test-keep$(NC)       - Run E2E tests, keep localnet running"
	@echo ""
	@echo "  $(YELLOW)localnet-start$(NC)      - Start Soroban localnet"
	@echo "  $(YELLOW)localnet-stop$(NC)       - Stop Soroban localnet" 
	@echo "  $(YELLOW)localnet-status$(NC)     - Show localnet status"
	@echo "  $(YELLOW)localnet-logs$(NC)       - Show localnet logs"
	@echo ""
	@echo "  $(GREEN)clean$(NC)               - Clean build artifacts"
	@echo "  $(GREEN)clean-full$(NC)          - Clean all artifacts including target/"
	@echo ""
	@echo "  $(GREEN)deploy-testnet$(NC)      - Deploy contracts to testnet"
	@echo "  $(GREEN)deploy-mainnet$(NC)      - Deploy contracts to mainnet"
	@echo ""
	@echo "Examples:"
	@echo "  make e2e-test              # Full E2E test cycle"
	@echo "  make e2e-test-quick        # Quick connectivity tests"
	@echo "  make localnet-start && make unit-test"
	@echo ""

# Build contracts
build:
	@echo "$(GREEN)[BUILD]$(NC) Building smart contracts..."
	./scripts/build.sh

# Run all tests
test: unit-test e2e-test

# Run unit tests only
unit-test:
	@echo "$(GREEN)[TEST]$(NC) Running unit tests..."
	cargo test --workspace --exclude e2e-tests

# Run E2E tests with full lifecycle
e2e-test:
	@echo "$(GREEN)[E2E]$(NC) Running E2E tests (full lifecycle)..."
	./scripts/run_e2e_tests.sh

# Run quick E2E smoke tests
e2e-test-quick:
	@echo "$(GREEN)[E2E]$(NC) Running quick E2E tests..."
	./scripts/run_e2e_tests.sh --quick

# Run E2E tests and keep localnet running
e2e-test-keep:
	@echo "$(GREEN)[E2E]$(NC) Running E2E tests (keep localnet running)..."
	./scripts/run_e2e_tests.sh --keep-running

# Run E2E tests only (assumes localnet is running)
e2e-test-only:
	@echo "$(GREEN)[E2E]$(NC) Running E2E tests only (localnet must be running)..."
	./scripts/run_e2e_tests.sh --tests-only

# Start Soroban localnet
localnet-start:
	@echo "$(YELLOW)[LOCALNET]$(NC) Starting Soroban localnet..."
	./scripts/start_localnet.sh start

# Stop Soroban localnet  
localnet-stop:
	@echo "$(YELLOW)[LOCALNET]$(NC) Stopping Soroban localnet..."
	./scripts/start_localnet.sh stop

# Show localnet status
localnet-status:
	@echo "$(YELLOW)[LOCALNET]$(NC) Localnet status:"
	./scripts/start_localnet.sh status

# Show localnet logs
localnet-logs:
	@echo "$(YELLOW)[LOCALNET]$(NC) Localnet logs:"
	./scripts/start_localnet.sh logs

# Restart localnet
localnet-restart:
	@echo "$(YELLOW)[LOCALNET]$(NC) Restarting Soroban localnet..."
	./scripts/start_localnet.sh restart

# Clean build artifacts
clean:
	@echo "$(GREEN)[CLEAN]$(NC) Cleaning build artifacts..."
	cargo clean

# Clean all artifacts including target directory
clean-full: clean
	@echo "$(GREEN)[CLEAN]$(NC) Removing target directory..."
	rm -rf target/

# Deploy to testnet
deploy-testnet: build
	@echo "$(GREEN)[DEPLOY]$(NC) Deploying to testnet..."
	./scripts/deploy_testnet.sh

# Deploy to mainnet
deploy-mainnet: build
	@echo "$(GREEN)[DEPLOY]$(NC) Deploying to mainnet..."
	./scripts/deploy_mainnet.sh

# Development workflow: clean build and test
dev-test: clean build e2e-test

# CI workflow: build, test, but don't keep localnet running
ci-test: build unit-test e2e-test-quick

# Check prerequisites for development
check:
	@echo "$(BLUE)[CHECK]$(NC) Checking development prerequisites..."
	@command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo not found"; exit 1; }
	@command -v soroban >/dev/null 2>&1 || { echo "❌ Soroban CLI not found"; exit 1; }
	@command -v docker >/dev/null 2>&1 || { echo "❌ Docker not found"; exit 1; }
	@docker info >/dev/null 2>&1 || { echo "❌ Docker not running"; exit 1; }
	@echo "✅ All prerequisites satisfied"

# Show project info
info:
	@echo "$(BLUE)StrellerMinds Smart Contracts$(NC)"
	@echo "=============================="
	@echo ""
	@echo "Project structure:"
	@echo "  contracts/        - Smart contract source code"
	@echo "  scripts/          - Build and deployment scripts"  
	@echo "  docs/             - Documentation"
	@echo "  e2e-tests/        - End-to-end test suite"
	@echo ""
	@echo "Key files:"
	@echo "  Cargo.toml        - Workspace configuration"
	@echo "  Makefile          - This file with convenient commands"
	@echo ""
	@rustc --version 2>/dev/null || echo "Rust: Not installed"
	@soroban version 2>/dev/null || echo "Soroban CLI: Not installed"
	@echo ""

# Format code
fmt:
	@echo "$(GREEN)[FORMAT]$(NC) Formatting code..."
	cargo fmt --all

# Run linter
lint:
	@echo "$(GREEN)[LINT]$(NC) Running clippy..."
	cargo clippy --all-targets --all-features

# Run fmt and lint together
check-code: fmt lint
	@echo "$(GREEN)[CHECK]$(NC) Code formatting and linting complete"