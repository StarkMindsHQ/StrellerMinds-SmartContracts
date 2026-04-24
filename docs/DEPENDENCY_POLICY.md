# Dependency Policy

This document defines how dependencies are managed, updated, and audited across the StrellerMinds-SmartContracts workspace.

## Dependency Management Tools

| Tool | Purpose | Config |
|------|---------|--------|
| `cargo-audit` | CVE scanning against RustSec advisory database | `.cargo/audit.toml` |
| `cargo-deny` | License compliance, banned crates, duplicate versions | `.cargo/deny.toml` |
| `cargo-outdated` | Detect stale dependency versions | (no config needed) |
| Dependabot | Automated weekly PRs for updates | `.github/dependabot.yml` |

## Automated Updates

Dependabot opens pull requests every Monday for:
- **Cargo** dependencies (workspace root)
- **GitHub Actions** versions
- **npm** packages in `e2e/`

PRs are grouped by ecosystem (e.g., all `soroban-*` crates in one PR). A maximum of 5 open Dependabot PRs are allowed at once.

## Security Scanning

Two layers of security scanning run automatically:

1. **`security-audit.yml`** — runs `cargo audit` daily and on every push/PR that touches `Cargo.toml` or `Cargo.lock`.
2. **`dependency-management.yml`** — runs `cargo-deny` (license + policy) and `cargo-audit` on every dependency change.

### Handling Advisories

If an advisory cannot be immediately resolved (e.g., it exists only in a transitive Soroban SDK dependency):

1. Investigate whether the vulnerability is reachable in this codebase.
2. If not reachable, add the advisory ID to `.cargo/audit.toml` under `[advisories] ignore` with a comment explaining the rationale.
3. Open a tracking issue to revisit when the upstream dependency is patched.

## License Policy

Allowed licenses: `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `CC0-1.0`.

Denied licenses: `GPL-2.0`, `GPL-3.0`, `AGPL-3.0`, `LGPL-*`.

All license checks are enforced by `cargo-deny` in CI. Any new dependency with a non-approved license will fail the build.

## Version Compatibility Testing

The `dependency-management.yml` workflow builds and tests the workspace against both `stable` and `beta` Rust toolchains. This catches regressions introduced by dependency updates before they reach `main`.

## Adding a New Dependency

1. Prefer crates already used in the workspace (check `[workspace.dependencies]` in the root `Cargo.toml`).
2. Add shared dependencies to `[workspace.dependencies]` and reference them with `{ workspace = true }` in contract `Cargo.toml` files.
3. Verify the license is in the allow-list above.
4. Run `cargo deny check` locally before opening a PR.
5. Avoid pinning to exact versions unless required — use `"^x.y"` semver ranges.

## Updating Dependencies

```bash
# Check what's outdated
cargo outdated --workspace

# Update a specific crate
cargo update -p <crate-name>

# Update all (within semver constraints)
cargo update

# Verify nothing broke
cargo test --workspace --exclude e2e-tests
cargo deny check
```

## Workspace Dependency Versions

All shared crates are pinned in the workspace root `Cargo.toml`. Contract crates must not duplicate these with different versions.

| Crate | Version | Notes |
|-------|---------|-------|
| `soroban-sdk` | `22.0.0` | Core Soroban SDK — update with care, breaking changes are common |
| `stellar-strkey` | `0.0.7` | Stellar key encoding |
| `ed25519-dalek` | `2.0.0` | Signature verification |
| `rand` | `0.8.5` | RNG utilities |
| `proptest` | `1.0.0` | Property-based testing |
| `criterion` | `0.5` | Benchmarking |

## Soroban SDK Updates

`soroban-sdk` updates often include breaking API changes. Before bumping the version:

1. Review the [Soroban SDK changelog](https://github.com/stellar/rs-soroban-sdk/blob/main/CHANGELOG.md).
2. Update the version in `[workspace.dependencies]`.
3. Fix any compilation errors across all contracts.
4. Run the full test suite: `cargo test --workspace`.
5. Update this table above with the new version.
