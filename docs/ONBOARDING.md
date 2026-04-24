# Developer Onboarding Guide

Welcome to the StrellerMinds development team. This guide gets you from zero to productive in the codebase.

## Day 1 Checklist

- [ ] Clone and build the project
- [ ] Run the test suite
- [ ] Understand the contract architecture
- [ ] Make a small change and open a PR

## Project Overview

StrellerMinds is a suite of Soroban smart contracts on Stellar that powers an educational credentialing system.

### Contracts

| Contract | Purpose |
|----------|---------|
| `certificate` | Multi-sig certificate issuance and verification |
| `token` | Token mint, transfer, and balance |
| `assessment` | Assessment creation and grading |
| `progress` | Student progress tracking |
| `analytics` | Learning analytics and reporting |
| `gamification` | Achievements, guilds, leaderboards |
| `community` | Forum, events, mentorship |
| `marketplace` | Learning path marketplace |
| `shared` | RBAC, events, rate limiting utilities |

See [ARCHITECTURE.md](ARCHITECTURE.md) for system design.

## Environment Setup

```bash
# Clone
git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git
cd StrellerMinds-SmartContracts

# Setup (installs Rust, Soroban CLI, wasm target)
./scripts/setup.sh

# Build
make build

# Test
make test
```

Full details in [development.md](development.md).

## Key Workflows

### Making a Change

```bash
git checkout -b feature/your-feature
make unit-test          # fast feedback
make test               # full validation
make check-code         # format + lint
git commit -m "feat: description"
git push origin feature/your-feature
```

### Testing

```bash
make unit-test          # fast, no localnet
make e2e-test           # requires Docker
make e2e-test-quick     # smoke tests
```

### Debugging

```bash
make localnet-start
make build
soroban contract invoke --id <contract-id> --fn <function> --arg <arg>
make localnet-logs
```

### Deployment

```bash
export STELLAR_SECRET_KEY="your-secret-key"
make deploy-testnet      # testnet
make deploy-mainnet      # mainnet (requires authorization)
```

## Code Standards

- `snake_case` for variables/functions
- `PascalCase` for types
- `SCREAMING_SNAKE_CASE` for constants
- Format + lint: `make check-code`
- Pre-commit hooks: `pre-commit install`

Full guide in [contributing.md](contributing.md) and [CODE_STYLE.md](CODE_STYLE.md).

## Documentation Map

| Doc | Purpose |
|-----|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design |
| [development.md](development.md) | Dev environment and workflows |
| [contributing.md](contributing.md) | Contribution standards |
| [CODE_STYLE.md](CODE_STYLE.md) | Naming and formatting |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Deployment procedures |
| [API.md](API.md) | API reference |

## Common Issues

**wasm32 target not found**
```bash
rustup target add wasm32-unknown-unknown
```

**E2E tests failing**
```bash
docker system prune -f
make localnet-start
```

**Build errors after pulling**
```bash
make clean && make build
```

## Getting Help

- Check existing docs in `docs/`
- Search GitHub issues
- Ask in PR reviews