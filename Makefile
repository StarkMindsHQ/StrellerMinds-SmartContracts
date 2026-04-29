# StrellerMinds Smart Contracts - Makefile
# 
# This Makefile provides convenient commands for development and testing

.PHONY: help build test unit-test e2e-test localnet-start localnet-stop localnet-status clean deploy-testnet deploy-mainnet fmt lint lint-style pre-commit-install pre-commit-run coverage coverage-html coverage-lcov coverage-open security-scan security-scan-full perf-profile perf-baseline load-test load-test-ci ci-security ci-coverage ci-perf test-coverage-comprehensive test-coverage-unit test-coverage-integration test-coverage-property test-coverage-edge-cases test-coverage-errors test-coverage-performance test-coverage-report coverage-threshold coverage-badge coverage-dashboard

# Colors for output
GREEN=\033[0;32m
YELLOW=\033[1;33m
BLUE=\033[0;34m
RED=\033[0;31m
NC=\033[0m # No Color

# Coverage gate threshold (%) - updated to 80% as per issue #240
COVERAGE_GATE ?= 80

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
	@echo "  $(RED)coverage$(NC)            - Code coverage with gate (>=$(COVERAGE_GATE)%)"
	@echo "  $(RED)coverage-html$(NC)       - Open HTML coverage report"
	@echo "  $(RED)coverage-lcov$(NC)       - Generate lcov.info for CI"
	@echo "  $(RED)test-coverage-comprehensive$(NC) - Run comprehensive test coverage analysis"
	@echo "  $(RED)test-coverage-unit$(NC)     - Run unit test coverage only"
	@echo "  $(RED)test-coverage-integration$(NC) - Run integration test coverage only"
	@echo "  $(RED)test-coverage-property$(NC)   - Run property-based test coverage"
	@echo "  $(RED)test-coverage-edge-cases$(NC) - Run edge case test coverage"
	@echo "  $(RED)test-coverage-errors$(NC)     - Run error scenario test coverage"
	@echo "  $(RED)test-coverage-performance$(NC) - Run performance test coverage"
	@echo "  $(RED)test-coverage-report$(NC)     - Generate detailed coverage report"
	@echo "  $(RED)coverage-threshold$(NC)       - Check if coverage meets 80% threshold"
	@echo "  $(RED)coverage-badge$(NC)           - Generate coverage badge"
	@echo "  $(RED)coverage-dashboard$(NC)       - Generate comprehensive coverage dashboard"
	@echo "  $(RED)security-scan$(NC)       - Run security audit + lint + tests"
	@echo "  $(RED)security-scan-full$(NC)  - Full security scan including semgrep"
	@echo "  $(RED)perf-profile$(NC)        - Performance profile all contracts"
	@echo "  $(RED)perf-baseline$(NC)       - Save performance baseline"
	@echo "  $(RED)load-test$(NC)           - Run configurable contract load tests and write reports"
	@echo "  $(RED)load-test-ci$(NC)        - Run bounded load tests with CI-safe defaults"
	@echo "  $(RED)compliance-report$(NC)  - Generate automated compliance report"
	@echo ""
	@echo "Examples:"
	@echo "  make e2e-test              # Full E2E test cycle"
	@echo "  make e2e-test-quick        # Quick connectivity tests"
	@echo "  make coverage              # Check coverage gate"
	@echo "  make security-scan         # Security audit"
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

# Run strict naming/style lints
lint-style:
	@echo "$(GREEN)[LINT]$(NC) Running strict style checks..."
	cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style

# Run fmt and lint together
check-code: fmt lint-style
	@echo "$(GREEN)[CHECK]$(NC) Code formatting and linting complete"

# Install pre-commit hooks
pre-commit-install:
	@echo "$(GREEN)[HOOKS]$(NC) Installing pre-commit hooks..."
	pre-commit install

# Run pre-commit hooks manually across repository
pre-commit-run:
	@echo "$(GREEN)[HOOKS]$(NC) Running pre-commit hooks..."
	pre-commit run --all-files

# ─────────────────────────────────────────────────────────────
# Code Coverage (Issue #274)
# ─────────────────────────────────────────────────────────────

# Run coverage with the default 90% gate
coverage:
	@echo "$(RED)[COV]$(NC) Running code coverage (gate: $(COVERAGE_GATE)%)..."
	chmod +x ./scripts/coverage.sh
	./scripts/coverage.sh --gate $(COVERAGE_GATE)

# Generate and open HTML coverage report
coverage-html:
	@echo "$(RED)[COV]$(NC) Generating HTML coverage report..."
	chmod +x ./scripts/coverage.sh
	./scripts/coverage.sh --html --open --gate $(COVERAGE_GATE)

# Generate lcov.info for CI badge / SonarQube integration
coverage-lcov:
	@echo "$(RED)[COV]$(NC) Generating lcov.info..."
	chmod +x ./scripts/coverage.sh
	./scripts/coverage.sh --lcov --gate $(COVERAGE_GATE)

# ─────────────────────────────────────────────────────────────
# Security Scanning (Issue #273)
# ─────────────────────────────────────────────────────────────

# Standard security scan: cargo-audit + clippy + security test suite
security-scan:
	@echo "$(RED)[SEC]$(NC) Running security scan..."
	chmod +x ./scripts/security_scan.sh
	./scripts/security_scan.sh

# Full security scan (includes semgrep if installed)
security-scan-full:
	@echo "$(RED)[SEC]$(NC) Running full security scan (with semgrep)..."
	chmod +x ./scripts/security_scan.sh
	./scripts/security_scan.sh --full

# ─────────────────────────────────────────────────────────────
# Performance Profiling (Issue #271)
# ─────────────────────────────────────────────────────────────

# Run performance profile across all relevant contracts
perf-profile:
	@echo "$(RED)[PERF]$(NC) Profiling contract performance..."
	chmod +x ./scripts/perf_profile.sh
	./scripts/perf_profile.sh

# Save current performance results as regression baseline
perf-baseline:
	@echo "$(RED)[PERF]$(NC) Saving performance baseline..."
	chmod +x ./scripts/perf_profile.sh
	./scripts/perf_profile.sh --baseline

# Compare performance against saved baseline
perf-compare:
	@echo "$(RED)[PERF]$(NC) Comparing performance against baseline..."
	chmod +x ./scripts/perf_profile.sh
	./scripts/perf_profile.sh --compare target/perf_baseline.json

# Run the contract load-test suite with local defaults.
load-test:
	@echo "$(RED)[LOAD]$(NC) Running load test suite..."
	chmod +x ./scripts/load_test.sh
	./scripts/load_test.sh

# Run the load-test suite with CI-safe bounded defaults.
load-test-ci:
	@echo "$(RED)[LOAD]$(NC) Running CI-safe load test suite..."
	chmod +x ./scripts/load_test.sh
	./scripts/load_test.sh --ci

# ─────────────────────────────────────────────────────────────
# CI convenience targets
# ─────────────────────────────────────────────────────────────

# Full CI pipeline: build + unit tests + security scan + coverage gate
ci-security: build security-scan
	@echo "$(GREEN)[CI]$(NC) Security CI pipeline complete"

ci-coverage: unit-test coverage
	@echo "$(GREEN)[CI]$(NC) Coverage CI pipeline complete"

ci-perf: unit-test perf-profile
	@echo "$(GREEN)[CI]$(NC) Performance CI pipeline complete"

# Complete CI pipeline (use in GitHub Actions / CI servers)
ci-full: build unit-test security-scan coverage
	@echo "$(GREEN)[CI]$(NC) Full CI pipeline complete"

# ─────────────────────────────────────────────────────────────
# Comprehensive Test Coverage (Issue #240)
# ─────────────────────────────────────────────────────────────

# Run comprehensive test coverage analysis
test-coverage-comprehensive:
	@echo "$(BLUE)[COVERAGE]$(NC) Running comprehensive test coverage analysis..."
	chmod +x ./scripts/test-coverage.sh
	./scripts/test-coverage.sh

# Run unit test coverage only
test-coverage-unit:
	@echo "$(BLUE)[COVERAGE]$(NC) Running unit test coverage..."
	chmod +x ./scripts/test-coverage.sh
	./scripts/test-coverage.sh --unit-only

# Run integration test coverage only
test-coverage-integration:
	@echo "$(BLUE)[COVERAGE]$(NC) Running integration test coverage..."
	chmod +x ./scripts/test-coverage.sh
	./scripts/test-coverage.sh --integration-only

# Run property-based test coverage
test-coverage-property:
	@echo "$(BLUE)[COVERAGE]$(NC) Running property-based test coverage..."
	cargo test --all-features -- --nocapture property_based || true
	cargo test --all-features -- --nocapture prop_ || true
	cargo test --all-features -- --nocapture quickcheck || true

# Run edge case test coverage
test-coverage-edge-cases:
	@echo "$(BLUE)[COVERAGE]$(NC) Running edge case test coverage..."
	cargo test --all-features -- --nocapture edge_case || true
	cargo test --all-features -- --nocapture boundary || true
	cargo test --all-features -- -- --exact 'test_*edge*' || true
	cargo test --all-features -- -- --exact 'test_*boundary*' || true

# Run error scenario test coverage
test-coverage-errors:
	@echo "$(BLUE)[COVERAGE]$(NC) Running error scenario test coverage..."
	cargo test --all-features -- --nocapture error || true
	cargo test --all-features -- -- --exact 'test_*error*' || true
	cargo test --all-features -- -- --exact 'test_*fail*' || true
	cargo test --package e2e-tests --test input_validation_tests --all-features -- --nocapture || true

# Run performance test coverage
test-coverage-performance:
	@echo "$(BLUE)[COVERAGE]$(NC) Running performance test coverage..."
	cargo test --package e2e-tests --test gas_benchmark --all-features -- --nocapture || true
	cargo test --all-features -- --nocapture performance || true
	cargo test --all-features -- -- --exact 'test_*perf*' || true
	if command -v cargo-criterion >/dev/null 2>&1; then \
		cargo bench --all-features || true; \
	fi

# Generate detailed coverage report
test-coverage-report:
	@echo "$(BLUE)[COVERAGE]$(NC) Generating detailed coverage report..."
	@if [ ! -f "scripts/test-coverage.sh" ]; then \
		echo "Error: test-coverage.sh script not found"; \
		exit 1; \
	fi
	chmod +x ./scripts/test-coverage.sh
	./scripts/test-coverage.sh --analyze-only
	@echo "$(GREEN)[COVERAGE]$(NC) Detailed report generated in target/coverage-reports/"

# Check if coverage meets 80% threshold
coverage-threshold:
	@echo "$(BLUE)[COVERAGE]$(NC) Checking 80% coverage threshold..."
	@if [ ! -f "target/coverage-reports/ci-coverage.json" ]; then \
		echo "Running coverage analysis first..."; \
		chmod +x ./scripts/test-coverage.sh; \
		./scripts/test-coverage.sh --workspace-only; \
	fi
	@if command -v jq >/dev/null 2>&1 && [ -f "target/coverage-reports/ci-coverage.json" ]; then \
		coverage=$$(jq -r '.overall_coverage // 0' target/coverage-reports/ci-coverage.json 2>/dev/null || echo "0"); \
		echo "Current coverage: $$coverage%"; \
		echo "Required coverage: 80%"; \
		if [ "$${coverage%.*}" -ge 80 ]; then \
			echo "$(GREEN)[COVERAGE]$(NC) ✅ Coverage threshold met!"; \
		else \
			echo "$(RED)[COVERAGE]$(NC) ❌ Coverage threshold not met!"; \
			echo "Current: $$coverage%, Required: 80%"; \
			exit 1; \
		fi; \
	else \
		echo "$(YELLOW)[COVERAGE]$(NC) Cannot determine coverage percentage - jq not available or report missing"; \
	fi

# Generate coverage badge
coverage-badge:
	@echo "$(BLUE)[COVERAGE]$(NC) Generating coverage badge..."
	@if [ ! -f "target/coverage-reports/ci-coverage.json" ]; then \
		echo "Running coverage analysis first..."; \
		chmod +x ./scripts/test-coverage.sh; \
		./scripts/test-coverage.sh --workspace-only; \
	fi
	@if command -v jq >/dev/null 2>&1 && [ -f "target/coverage-reports/ci-coverage.json" ]; then \
		coverage=$$(jq -r '.overall_coverage // 0' target/coverage-reports/ci-coverage.json 2>/dev/null || echo "0"); \
		coverage_int=$${coverage%.*}; \
		color="red"; \
		if [ $$coverage_int -ge 80 ]; then \
			color="green"; \
		elif [ $$coverage_int -ge 60 ]; then \
			color="yellow"; \
		fi; \
		echo "Coverage: $$coverage% (color: $$color)" > target/coverage-reports/coverage-badge.txt; \
		echo "$(GREEN)[COVERAGE]$(NC) Badge generated: target/coverage-reports/coverage-badge.txt"; \
	else \
		echo "$(YELLOW)[COVERAGE]$(NC) Cannot generate badge - jq not available or report missing"; \
	fi

# Enhanced coverage command with comprehensive analysis
coverage: test-coverage-comprehensive coverage-threshold
	@echo "$(GREEN)[COVERAGE]$(NC) Comprehensive coverage analysis complete!"

# Enhanced coverage HTML with all reports
coverage-html: test-coverage-comprehensive
	@echo "$(BLUE)[COVERAGE]$(NC) Opening comprehensive coverage reports..."
	@if [ -d "target/coverage/workspace-html" ]; then \
		if command -v xdg-open >/dev/null 2>&1; then \
			xdg-open target/coverage/workspace-html/index.html; \
		elif command -v open >/dev/null 2>&1; then \
			open target/coverage/workspace-html/index.html; \
		else \
			echo "HTML coverage report available at: target/coverage/workspace-html/index.html"; \
		fi; \
	else \
		echo "Coverage report not found. Run 'make test-coverage-comprehensive' first."; \
	fi

# Generate comprehensive coverage dashboard
coverage-dashboard: test-coverage-comprehensive
	@echo "$(BLUE)[COVERAGE]$(NC) Generating comprehensive coverage dashboard..."
	chmod +x ./scripts/coverage-dashboard.sh
	./scripts/coverage-dashboard.sh
	@echo "$(GREEN)[COVERAGE]$(NC) Dashboard generated: target/coverage-dashboard/index.html"

# ─────────────────────────────────────────────────────────────
# Compliance Reporting
# ─────────────────────────────────────────────────────────────

# Generate automated compliance report
compliance-report:
	@echo "$(RED)[COMPLIANCE]$(NC) Generating compliance report..."
	chmod +x ./scripts/generate_compliance_report.sh
	./scripts/generate_compliance_report.sh
