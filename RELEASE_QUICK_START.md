# Quick Release Guide

## Overview

This guide provides quick reference for the release automation system.

## Prerequisites

Ensure you have:
- Rust toolchain installed
- Soroban CLI installed
- Git configured
- GitHub access

### Install Required Tools

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli

# Install git-cliff (for changelog generation)
cargo install git-cliff

# Install cargo-audit (for security checks)
cargo install cargo-audit
```

## Quick Start

### 1. Run Pre-Release Validation

```bash
./scripts/pre-release-validation.sh v1.2.3
```

This checks:
- ✅ Version format
- ✅ Git state
- ✅ Tests passing
- ✅ Build success
- ✅ Documentation
- ✅ Security

### 2. Create Release

```bash
# Automated (recommended)
./scripts/release.sh v1.2.3

# With auto-confirm
./scripts/release.sh -y v1.2.3

# Dry run first
./scripts/release.sh --dry-run v1.2.3
```

### 3. Monitor

Watch GitHub Actions: https://github.com/your-org/repo/actions

The workflow will:
- Build all contracts
- Optimize WASM files
- Generate SBOM
- Sign artifacts
- Create GitHub Release

## Scripts Reference

| Script | Purpose | Command |
|--------|---------|---------|
| `pre-release-validation.sh` | Pre-release checks | `./scripts/pre-release-validation.sh v1.2.3` |
| `release-test.sh` | Comprehensive testing | `./scripts/release-test.sh all` |
| `release.sh` | Full automation | `./scripts/release.sh v1.2.3` |
| `release-monitor.sh` | Metrics collection | `./scripts/release-monitor.sh` |
| `post-release-review.sh` | Retrospective | `./scripts/post-release-review.sh v1.2.3` |
| `build.sh` | Build contracts | `./scripts/build.sh` |

## Test Options

```bash
# All tests
./scripts/release-test.sh all

# Quick smoke tests
./scripts/release-test.sh --quick

# Specific suite
./scripts/release-test.sh build
./scripts/release-test.sh unit
./scripts/release-test.sh security

# Skip suites
./scripts/release-test.sh --skip-e2e
./scripts/release-test.sh --skip-security
```

## Release Checklist

### Before Release
- [ ] Run pre-release validation
- [ ] Review test results
- [ ] Update CHANGELOG.md
- [ ] Verify documentation is current
- [ ] Check for breaking changes

### During Release
- [ ] Use automated script
- [ ] Monitor GitHub Actions
- [ ] Verify artifact upload
- [ ] Check release page

### After Release (Immediate)
- [ ] Download artifacts
- [ ] Verify checksums
- [ ] Test deployment
- [ ] Announce release

### After Release (7 days)
- [ ] Run post-release review
- [ ] Analyze metrics
- [ ] Gather team feedback
- [ ] Document improvements

## Version Numbering

Follow SemVer: **MAJOR.MINOR.PATCH**

- **MAJOR** (v2.0.0) - Breaking changes
- **MINOR** (v1.3.0) - New features (backward compatible)
- **PATCH** (v1.2.4) - Bug fixes (backward compatible)

Pre-releases:
- Release Candidate: v1.2.3-rc.1
- Beta: v1.2.3-beta.1
- Alpha: v1.2.3-alpha.1

## Troubleshooting

### Build Fails

```bash
# Clean and rebuild
cargo clean
./scripts/build.sh

# Check toolchain
rustc --version
soroban --version
```

### Tests Fail

```bash
# Run specific test suite
./scripts/release-test.sh unit

# View logs
cat logs/release_test_*.log
```

### Tag Already Exists

```bash
# Delete local tag
git tag -d v1.2.3

# Delete remote tag
git push origin :refs/tags/v1.2.3

# Recreate
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

### Workflow Fails

1. Check GitHub Actions logs
2. Fix identified issues
3. Delete failed tag
4. Recreate and push

## Monitoring

### Collect Metrics

```bash
# Manual collection
./scripts/release-monitor.sh

# Automated (weekly via GitHub Actions)
# See: .github/workflows/release-metrics.yml
```

### View Reports

```bash
# Latest metrics report
ls -lt metrics/ | head -1

# Latest review
ls -lt reviews/ | head -1

# Build logs
ls -lt logs/ | head -1
```

## Release Timeline

### Typical Schedule

- **T-7 days**: Finalize features
- **T-3 days**: Code freeze
- **T-2 days**: Testing begins
- **T-1 day**: Pre-release validation
- **T-0**: Release day
- **T+1 day**: Monitor adoption
- **T+7 days**: Post-release review

### Emergency Hotfix

For critical bugs:

1. Fix issue
2. Run quick tests: `./scripts/release-test.sh --quick`
3. Create patch: `./scripts/release.sh -y v1.2.4`
4. Monitor deployment
5. Document in retrospective

## Best Practices

### Do's
- ✅ Always run pre-release validation
- ✅ Test thoroughly before release
- ✅ Use annotated tags
- ✅ Document breaking changes
- ✅ Communicate with stakeholders
- ✅ Conduct retrospectives

### Don'ts
- ❌ Skip tests to save time
- ❌ Release on Fridays
- ❌ Skip documentation updates
- ❌ Ignore failing checks
- ❌ Forget post-release review

## Support

### Resources
- Full documentation: `docs/RELEASE_MANAGEMENT.md`
- Process guide: `docs/release-process.md`
- GitHub Actions: `.github/workflows/release.yml`

### Contacts
- Release Manager: @username
- DevOps Team: #devops-channel
- Security Issues: security@domain.com

## Automation Features

### GitHub Actions Workflows

1. **CI Pipeline** (`.github/workflows/ci.yml`)
   - Format checks
   - Linting
   - Build verification
   - Unit tests

2. **Release Pipeline** (`.github/workflows/release.yml`)
   - Build artifacts
   - Security scanning
   - Artifact signing
   - Release creation

3. **Test Pipeline** (`.github/workflows/release-test.yml`)
   - Pre-release testing
   - Code quality
   - Security audit

4. **Metrics Pipeline** (`.github/workflows/release-metrics.yml`)
   - Weekly metrics collection
   - Trend analysis
   - Reporting

## Metrics Tracked

### Quality Metrics
- Test pass rate
- Code coverage
- Bug count
- Hotfix frequency

### Adoption Metrics
- Download count
- Clone count
- Dependency updates
- User growth

### Performance Metrics
- WASM file sizes
- Gas usage patterns
- Build times
- Deployment success rate

---

**Quick Help:** `./scripts/release.sh --help`  
**Full Docs:** `docs/RELEASE_MANAGEMENT.md`
