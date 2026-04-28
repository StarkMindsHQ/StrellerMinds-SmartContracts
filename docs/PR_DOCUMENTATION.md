# Pull Request Documentation

> **Version:** 2.0 | **Last Updated:** April 2026 | **Maintainer:** @LaGodxy

## Overview

This document provides templates and guidelines for creating and reviewing pull requests in the StrellerMinds-SmartContracts project.

---

## Table of Contents

1. [PR Title Format](#pr-title-format)
2. [PR Description Template](#pr-description-template)
3. [PR Types and Scopes](#pr-types-and-scopes)
4. [Complete PR Example](#complete-pr-example)
5. [Review Checklist](#review-checklist)
6. [CI Pipeline](#ci-pipeline)
7. [Automation Scripts](#automation-scripts)
8. [Quick Reference](#quick-reference)

Use conventional commits format:

```
<type>(<scope>): <description>
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature or functionality |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `style` | Code style changes (formatting, no logic change) |
| `refactor` | Code refactoring |
| `test` | Adding or updating tests |
| `chore` | Maintenance tasks |
| `perf` | Performance improvements |
| `security` | Security-related changes |

### Examples

```
feat(analytics): add new engagement metrics tracking
fix(token): correct balance calculation overflow
docs(onboarding): enhance developer onboarding guide
refactor(shared): simplify RBAC permission checks
test(progress): add property-based tests for validation
```

---

## PR Description Template

### Template

```markdown
## Description
<!-- Brief description of what this PR does -->

## Type of Change
- [ ] New feature
- [ ] Bug fix
- [ ] Documentation update
- [ ] Refactoring
- [ ] Tests

## Related Issue
<!-- Link to the issue: Fixes #123 -->

## Changes Made
<!-- List of specific changes -->

1. 
2. 
3. 

## Testing
<!-- How was this tested? -->

- [ ] Unit tests pass
- [ ] E2E tests pass
- [ ] Manual testing done

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated (if applicable)
- [ ] Tests added/updated
- [ ] All CI checks pass
```

---

## PR Types and Scopes

### Type Definitions

| Type | Use When | Example |
|------|----------|---------|
| `feat` | Adding new functionality | New contract, new function, new feature |
| `fix` | Bug fixes | Runtime errors, logic errors, edge cases |
| `docs` | Documentation only | README, guides, comments |
| `style` | Formatting only | rustfmt changes, no logic change |
| `refactor` | Restructuring code | Moving functions, simplifying logic |
| `test` | Test-related | Unit tests, E2E tests, test fixtures |
| `chore` | Maintenance | Dependencies, config, build scripts |
| `perf` | Performance | Gas optimization, execution speed |
| `security` | Security fixes | Vulnerability patches, access control |

### Scope Definitions

| Scope | Description | Affected Files |
|-------|-------------|----------------|
| `analytics` | Analytics contract changes | `contracts/analytics/` |
| `token` | Token contract changes | `contracts/token/` |
| `shared` | Shared utilities/RBAC | `contracts/shared/` |
| `mobile-optimizer` | Mobile optimizer | `contracts/mobile-optimizer/` |
| `progress` | Progress tracking | `contracts/progress/` |
| `proxy` | Upgradeable contracts | `contracts/proxy/` |
| `search` | Search functionality | `contracts/search/` |
| `student-progress` | Student progress tracker | `contracts/student-progress-tracker/` |
| `onboarding` | Developer onboarding docs | `docs/ONBOARDING.md` |
| `deployment` | Deployment scripts | `scripts/deploy*.sh` |
| `e2e` | E2E test changes | `e2e-tests/` |
| `ci` | GitHub Actions | `.github/workflows/` |

### Examples

```
# Feature
feat(analytics): add engagement metrics dashboard
feat(token): implement staking reward distribution
feat(shared): add new RBAC role for oracles

# Fix
fix(token): correct overflow in balance calculation
fix(analytics): handle empty student records
fix(shared): validate address before storage access

# Documentation
docs(onboarding): add verification checklist
docs(README): update quick start guide
docs(ARCHITECTURE): document cross-chain flow

# Refactor
refactor(shared): simplify permission checks
refactor(token): optimize storage layout
refactor(progress): consolidate duplicate logic

# Test
test(analytics): add property-based tests
test(token): increase edge case coverage
test(e2e): add integration test for deployment

# Performance
perf(token): batch storage operations
perf(analytics): cache frequent queries
perf(shared): optimize role lookups

# Security
security(shared): patch reentrancy vulnerability
security(token): add signature verification
security(progress): validate admin permissions
```

---

## Complete PR Example

### Full Template

```markdown
## Summary
<!-- One-line summary of the PR -->

## Description
<!-- Detailed description of what this PR does and why -->

## Type of Change
- [ ] New feature (feat)
- [ ] Bug fix (fix)
- [ ] Documentation update (docs)
- [ ] Refactoring (refactor)
- [ ] Tests (test)
- [ ] Performance (perf)
- [ ] Security (security)
- [ ] Maintenance (chore)

## Related Issue
<!-- Link to the issue: Fixes #123 or Closes #456 -->

## Breaking Changes
<!-- List any breaking changes or migration needed -->

## Changes Made
<!-- List of specific changes -->

1. Added new function `calculate_rewards()` in token contract
2. Updated RBAC to support oracle role
3. Added unit tests for reward calculation
4. Updated documentation

## Testing
<!-- How was this tested? -->

| Test Type | Command | Result |
|-----------|---------|--------|
| Unit Tests | `cargo test -p token` | ✅ Pass |
| Format | `cargo fmt --all` | ✅ Pass |
| Clippy | `cargo clippy -p token -- -D warnings` | ✅ Pass |
| Property | `cargo test -p token --lib property_tests` | ✅ Pass |

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated (if applicable)
- [ ] Tests added/updated
- [ ] All CI checks pass
- [ ] Commit messages follow conventions
- [ ] No security vulnerabilities introduced
```

### Real Example: Documentation Enhancement

```markdown
## Summary
Enhanced developer onboarding documentation with verification checklist and best practices

## Description
This PR significantly expands the developer onboarding guide to include:
- Complete verification checklist for assignment completion
- Step-by-step process with timing estimates
- Best practices section with code examples
- Quick reference card for daily development

## Type of Change
- [x] Documentation update (docs)

## Related Issue
N/A - Standalone documentation improvement

## Breaking Changes
None

## Changes Made

1. **Added Verification Checklist (New Section 12)**
   - Phase 1: Environment Setup (7 steps)
   - Phase 2: Build & Test (4 steps)
   - Phase 3: Development Workflow (5 steps)
   - Phase 4: E2E Testing (3 steps)

2. **Added Step-by-Step Process**
   - Step 1: Clone and Setup (10 minutes)
   - Step 2: Build Contracts (5 minutes)
   - Step 3: Run Tests (10 minutes)
   - Step 4: Code Quality Checks (5 minutes)
   - Step 5: Explore Contract Structure (15 minutes)
   - Step 6: Make First Contribution (20 minutes)
   - Step 7: Review Architecture (15 minutes)

3. **Added Best Practices Section**
   - Security-first development examples
   - Reentrancy protection patterns
   - Error handling conventions
   - Gas optimization techniques
   - Testing best practices
   - Documentation standards

4. **Updated README.md**
   - Added Developer Onboarding section
   - Included onboarding checklist table
   - Linked to comprehensive guide

## Testing

| Test Type | Command | Result |
|-----------|---------|--------|
| Format | `cargo fmt --all` | ✅ Pass |
| Links | Manual verification | ✅ All valid |
| Code Examples | Syntax check | ✅ Valid |

## Checklist
- [x] Code follows project style guidelines
- [x] Documentation updated
- [x] No tests required for documentation
- [x] All CI checks pass
- [x] Commit messages follow conventions
```

---

## CI Pipeline

### Title

```
docs(onboarding): enhance developer onboarding guide with comprehensive structure and details
```

### Description

```markdown
## Description
Enhanced the developer onboarding documentation with comprehensive structure, code examples, and troubleshooting guides for new team members.

## Type of Change
- [ ] Documentation update

## Related Issue
N/A - Standalone documentation improvement

## Changes Made

1. **Expanded Project Overview**
   - Added technology stack table with versions
   - Included component descriptions
   - Documented what the project builds

2. **Comprehensive Environment Setup**
   - Automated setup instructions (`./scripts/setup.sh`)
   - Manual setup alternative
   - Makefile command reference

3. **Project Structure Documentation**
   - Full directory breakdown
   - Contract descriptions table
   - Key scripts reference

4. **Development Workflow**
   - Step-by-step change process
   - Code review checklist
   - Pre-commit hooks setup

5. **Testing Guidelines**
   - Unit tests
   - Property-based tests
   - E2E tests with prerequisites
   - Manual localnet management

6. **Deployment Procedures**
   - Building contracts
   - Network options (local/testnet/mainnet)
   - Post-deployment verification
   - TTL extension

7. **Code Standards**
   - Naming conventions table
   - Formatting and linting commands
   - Commit message format
   - Documentation requirements

8. **Troubleshooting**
   - Common issues and solutions
   - wasm32 target not found
   - E2E tests failing
   - Build errors
   - Port conflicts

9. **Getting Help**
   - Documentation map
   - External resources
   - Quick reference commands

## Testing
- [x] Documentation builds correctly
- [x] Links verified
- [x] Code examples syntactically correct

## Checklist
- [x] Code follows project style guidelines
- [x] Documentation updated
- [x] No tests required for documentation
- [x] All CI checks pass
```

---

## Review Checklist

### For Authors

Before opening a PR:

- [ ] PR links to related issue
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings
- [ ] Documentation updated (if needed)
- [ ] Commit messages follow conventions
- [ ] Branch is up to date with main

### For Reviewers

When reviewing a PR:

- [ ] Code is correct and secure
- [ ] Tests are comprehensive
- [ ] Documentation is clear
- [ ] No unintended breaking changes
- [ ] Follows project conventions
- [ ] Security implications considered

---

## CI Pipeline

### Automated Checks

The project runs the following CI checks on every PR:

| Check | Command | Timeout | Required |
|-------|---------|---------|----------|
| **Format** | `cargo fmt --all` | 1 min | ✅ Yes |
| **Lint** | `cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style` | 5 min | ✅ Yes |
| **Unit Tests** | `cargo test --workspace --exclude e2e-tests` | 10 min | ✅ Yes |
| **Build** | `cargo build --release --target wasm32-unknown-unknown` | 15 min | ✅ Yes |
| **E2E Tests** | `./scripts/run-e2e-tests.sh` | 20 min | ⚠️ Optional |
| **Coverage** | `cargo llvm-cov --workspace --exclude e2e-tests` | 15 min | ✅ ≥80% |

### Workflow Files

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push to main, PRs | Format, lint, build, test |
| `e2e.yml` | Manual, schedule | E2E test suite |
| `release.yml` | Tags | Release automation |

### Passing CI Before Merge

```bash
# Run all checks locally before pushing
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style
cargo test --workspace --exclude e2e-tests
cargo build --release --target wasm32-unknown-unknown
```

---

## Automation Scripts

### Pre-PR Validation Script

Run this script before opening a PR:

```bash
#!/bin/bash
# pre-pr-check.sh - Run all validation checks before creating PR

set -e

echo "Running pre-PR validation..."

# 1. Format check
echo "📝 Checking format..."
cargo fmt --all

# 2. Lint check
echo "🔍 Running clippy..."
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style

# 3. Build
echo "🏗️ Building contracts..."
cargo build --release --target wasm32-unknown-unknown

# 4. Tests
echo "🧪 Running tests..."
cargo test --workspace --exclude e2e-tests

echo "✅ All checks passed! Ready to create PR."
```

### Using the Makefile

```bash
# Full validation before PR
make check          # Prerequisites
make build          # Build contracts
make unit-test      # Run tests
make fmt           # Format code
make lint          # Run linter

# Or run all at once
make dev-test      # clean + build + test
```

---

## Quick Reference

### Common PR Commands

```bash
# Create new branch
git checkout -b feature/your-feature

# Stage changes
git add .
git add -p  # Interactive staging

# Commit with conventional format
git commit -m "feat(scope): description"

# Push to remote
git push origin feature/your-feature

# Update branch with main
git fetch origin
git rebase origin/main
```

### CI Checks

| Check | Command |
|-------|---------|
| Format | `cargo fmt --all` |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style` |
| Test | `cargo test --workspace --exclude e2e-tests` |
| E2E | `./scripts/run-e2e-tests.sh` |

---

## Resources

| Document | Purpose |
|----------|---------|
| [Contributing Guidelines](contributing.md) | Full contribution process |
| [Code Style Guide](CODE_STYLE.md) | Naming and formatting rules |
| [Development Guide](development.md) | Dev environment and workflows |
| [Deployment Guide](DEPLOYMENT.md) | Deployment procedures |
| [Onboarding Guide](ONBOARDING.md) | New team member setup |
| [Architecture Overview](ARCHITECTURE.md) | System design |

### External Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar SDK](https://developers.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)

---

## Quick Commands Reference

```bash
# === CREATE PR ===

# 1. Sync with main
git fetch origin
git checkout main
git pull origin main

# 2. Create feature branch
git checkout -b type/scope-description

# 3. Make changes and commit
git add .
git commit -m "type(scope): description"

# 4. Run pre-PR checks
make dev-test

# 5. Push and create PR
git push -u origin type/scope-description
# Then open URL from output to create PR

# === UPDATE PR ===

# Rebase on main
git fetch origin
git rebase origin/main
git push --force-with-lease

# Amend last commit
git commit --amend
git push --force-with-lease

# === REVIEW PR ===

# Checkout PR branch locally
git fetch origin
git checkout origin/PR-number

# View diff
git diff main..HEAD

# Test locally
cargo test -p <affected-package>
```

---

*Last Updated: April 2026*