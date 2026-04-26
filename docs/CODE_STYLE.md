# Rust Code Style Guide

This repository uses a strict style baseline to keep all contracts readable and predictable.

## Naming Conventions

- Variables and function names: `snake_case`
- Type names (`struct`, `enum`, `trait`, `type` aliases): `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules and file names: `snake_case`

## Formatting Rules

- Formatting is standardized with `rustfmt` for the entire workspace.
- The canonical formatter settings are stored in `rustfmt.toml`.
- Run formatting locally before commit:

```bash
cargo fmt --all
```

## Linting and Style Enforcement

- Use clippy for style and correctness checks:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style
```

- `nonstandard-style` ensures naming lints (for example: non-snake-case and non-camel-case type names) are treated as errors.

## Contract Code Review Checklist

Before opening a PR that touches `contracts/`:

- Confirm `cargo fmt --all` produces no diffs.
- Confirm naming conventions in new and modified code follow this guide.
- Run clippy style checks with `-D warnings -D nonstandard-style`.
- Ensure public interfaces remain clear and consistently named across contracts.

## Automation

- CI validates formatting and style checks on each PR.
- Pre-commit hooks run local formatting and style checks before commit.
