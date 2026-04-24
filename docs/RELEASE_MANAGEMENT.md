# Release Management Guide

## Overview

This document describes the complete release process for StrellerMinds Smart Contracts, including automated release workflows, testing requirements, and monitoring procedures.

## Table of Contents

1. [Release Process Overview](#release-process-overview)
2. [Pre-Release Checklist](#pre-release-checklist)
3. [Creating a Release](#creating-a-release)
4. [Release Testing](#release-testing)
5. [Post-Release Activities](#post-release-activities)
6. [Release Monitoring](#release-monitoring)
7. [Release Review](#release-review)
8. [Troubleshooting](#troubleshooting)

## Release Process Overview

### Automated Release Workflow

The project uses GitHub Actions for automated releases. When a version tag is pushed:

1. **Automated Build** - All contracts are built and optimized
2. **Security Scanning** - SBOM generation and vulnerability scanning
3. **Artifact Signing** - WASM files are cryptographically signed
4. **GitHub Release** - Release is created with changelog and artifacts
5. **Distribution** - Artifacts are made available for download

### Release Types

- **Major Release** (vX.0.0) - Breaking changes
- **Minor Release** (v1.X.0) - New features, backward compatible
- **Patch Release** (v1.2.X) - Bug fixes, backward compatible
- **Pre-Release** (v1.2.3-rc.1) - Release candidates and betas

## Pre-Release Checklist

Before creating a release, ensure:

### Code Quality
- [ ] All tests pass (unit, integration, E2E)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] Security audit passes (`cargo audit`)

### Documentation
- [ ] CHANGELOG.md is updated
- [ ] README.md reflects current version
- [ ] API documentation is current
- [ ] Migration guide prepared (if breaking changes)

### Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Performance benchmarks acceptable

### Security
- [ ] No known vulnerabilities
- [ ] Reentrancy guards in place where needed
- [ ] Access controls verified
- [ ] Gas optimization reviewed

## Creating a Release

### Step 1: Update Version Numbers

Update version in relevant files:
```bash
# Update Cargo.toml files if needed
# Update documentation references
```

### Step 2: Update CHANGELOG

Ensure `CHANGELOG.md` has proper entries:
```markdown
## [Unreleased]

### Features
- New feature description

### Bug Fixes
- Fix description

### Breaking Changes
- Change description
```

### Step 3: Run Pre-Release Validation

Execute the validation script:
```bash
./scripts/pre-release-validation.sh v1.2.3
```

This will check:
- Version format
- Git state
- Test results
- Build success
- Documentation completeness
- Security status

### Step 4: Create and Push Tag

If validation passes:
```bash
# Create annotated tag
git tag -a v1.2.3 -m "Release version 1.2.3"

# Push tag to trigger release workflow
git push origin v1.2.3
```

### Step 5: Monitor Automated Release

Watch the GitHub Actions workflow:
1. Navigate to repository Actions tab
2. Select "Release" workflow
3. Monitor progress
4. Verify all steps complete successfully

### Step 6: Verify Release

After workflow completes:
1. Check GitHub Releases page
2. Verify all artifacts present
3. Download and test artifacts
4. Confirm checksums match

## Release Testing

### Automated Testing

The release workflow includes:

1. **Build Tests** - Verifies all contracts compile
2. **Unit Tests** - Runs all unit tests
3. **Integration Tests** - Tests contract interactions
4. **E2E Tests** - Full end-to-end scenarios
5. **Security Scans** - Vulnerability detection
6. **Performance Checks** - WASM size validation

### Manual Testing

After automated release:

```bash
# Download release artifacts
wget https://github.com/your-org/repo/releases/download/v1.2.3/artifacts.zip

# Extract and verify
unzip artifacts.zip
sha256sum -c SHA256SUMS.txt

# Deploy to test network
# Run integration tests
```

### Smoke Tests

Quick validation tests:
```bash
./scripts/release-test.sh --quick
```

### Full Test Suite

Comprehensive testing:
```bash
./scripts/release-test.sh all
```

## Post-Release Activities

### Immediate (Within 1 hour)

1. **Verify Deployment** - Check release is live
2. **Monitor Logs** - Watch for errors
3. **Social Announcement** - Notify community

### Short-term (Within 24 hours)

1. **User Feedback** - Monitor issues/discussions
2. **Adoption Metrics** - Track download counts
3. **Documentation** - Update getting started guides

### Long-term (Within 1 week)

1. **Performance Analysis** - Review gas metrics
2. **Security Monitoring** - Watch for vulnerabilities
3. **Community Support** - Help with migration

## Release Monitoring

### Key Metrics

Track these metrics post-release:

#### Adoption Metrics
- Download count
- Clone count
- Package installs
- Dependency updates

#### Quality Metrics
- Issue count (bugs)
- PR count (fixes)
- Community feedback
- Error reports

#### Performance Metrics
- Gas usage patterns
- Transaction success rate
- Contract interaction costs
- Network performance

### Monitoring Tools

#### GitHub Insights
```bash
# Repository analytics
# Traffic graphs
# Clone counts
# Referrer data
```

#### On-Chain Analytics
```bash
# Contract deployments
# Transaction volumes
# Gas consumption
# User adoption
```

#### Community Channels
- GitHub Issues
- Discord/Slack
- Twitter mentions
- Forum discussions

### Alert Thresholds

Set up alerts for:
- Critical bugs reported (> 0)
- High error rate (> 1%)
- Performance degradation (> 10%)
- Security vulnerabilities (> 0)

## Release Review

### Release Retrospective

Conduct retrospective within 1 week:

#### Agenda
1. What went well?
2. What could be improved?
3. Action items for next release

#### Participants
- Development team
- QA team
- DevOps team
- Stakeholders (optional)

### Review Template

```markdown
## Release Review: v1.2.3

### Summary
- Release date: YYYY-MM-DD
- Release manager: @username
- Status: Success/Issues

### Metrics
- Issues reported: X
- Downloads: Y
- Adoption rate: Z%

### What Went Well
- Item 1
- Item 2

### Areas for Improvement
- Item 1
- Item 2

### Action Items
- [ ] Action 1
- [ ] Action 2
```

### Continuous Improvement

Update processes based on learnings:
- Refine checklists
- Improve automation
- Enhance testing
- Better documentation

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Clean and rebuild
cargo clean
./scripts/build.sh

# Check toolchain versions
rustc --version
soroban --version
```

#### Test Failures
```bash
# Run specific test suite
./scripts/release-test.sh unit

# View detailed logs
cat logs/release_test_*.log
```

#### Release Workflow Failure
1. Check GitHub Actions logs
2. Identify failing step
3. Fix issue locally
4. Delete failed tag
5. Recreate and push

#### Artifact Issues
```bash
# Verify checksums
sha256sum -c SHA256SUMS.txt

# Rebuild artifacts
./scripts/build.sh

# Compare sizes
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

### Rollback Procedure

If critical issues found:

1. **Assess Impact** - Determine severity
2. **Notify Users** - Issue public notice
3. **Create Hotfix** - Develop and test fix
4. **Patch Release** - Release new version
5. **Deprecate** - Mark problematic release

### Emergency Contacts

- Release Manager: @username
- Security Team: security@domain.com
- DevOps: devops@domain.com

## Best Practices

### Before Release
- Start testing early
- Automate repetitive tasks
- Document everything
- Communicate with stakeholders

### During Release
- Follow checklist
- Monitor continuously
- Be ready to rollback
- Keep team informed

### After Release
- Gather feedback
- Analyze metrics
- Conduct retrospective
- Improve processes

## Automation Scripts

### Available Scripts

1. **pre-release-validation.sh** - Pre-release checks
2. **release-test.sh** - Comprehensive testing
3. **build.sh** - Build automation
4. **deploy.sh** - Deployment automation

### Usage Examples

```bash
# Full pre-release validation
./scripts/pre-release-validation.sh v1.2.3

# Run all release tests
./scripts/release-test.sh all

# Quick smoke tests
./scripts/release-test.sh --quick

# Build all contracts
./scripts/build.sh
```

## Version Numbering

Follow Semantic Versioning (SemVer):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- Increment MAJOR for breaking changes
- Increment MINOR for new features (backward compatible)
- Increment PATCH for bug fixes (backward compatible)

Examples:
- 1.0.0 → 2.0.0 (breaking change)
- 1.0.0 → 1.1.0 (new feature)
- 1.1.0 → 1.1.1 (bug fix)

## Security Considerations

### Signing
- All artifacts cryptographically signed
- Use Cosign for signature
- Verify signatures before deployment

### SBOM
- Software Bill of Materials generated
- Includes all dependencies
- Enables vulnerability tracking

### Audit Trail
- Git history preserved
- Changelog maintained
- Release notes documented

## Compliance

### Regulatory Requirements
- Maintain audit logs
- Document changes
- Track vulnerabilities
- Ensure data privacy

### Industry Standards
- Follow OWASP guidelines
- Adhere to smart contract best practices
- Implement defense in depth

## Resources

### Documentation
- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Rust Book](https://doc.rust-lang.org/book/)

### Tools
- cargo-audit - Security auditing
- wasm-opt - WASM optimization
- Cosign - Artifact signing
- Syft - SBOM generation

### Support
- GitHub Issues
- Community Forum
- Discord Channel
- Email Support

---

**Last Updated:** 2026-03-27  
**Version:** 1.0.0  
**Maintained By:** DevOps Team
