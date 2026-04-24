# PR: Add Multi-Language SDKs

## Description
This PR addresses Issue #156 by implementing SDKs for StrellerMinds Smart Contracts in multiple programming languages. This enables developers to interact with our contracts using their preferred language.

## Changes
- **TypeScript SDK**: Added in `sdks/typescript`. Includes strict typing, `AnalyticsClient`, and basic setup.
- **Python SDK**: Added in `sdks/python`. Standard package structure with `AnalyticsClient`.
- **Go SDK**: Added in `sdks/go`. proper module structure with helper functions.
- **Rust SDK Extensions**: Added in `sdks/rust`. Provides contract types and client helpers for Rust integrations.
- **Documentation**: Added `sdks/README.md` with installation and usage instructions for all SDKs.
- **Git Configuration**: Updated `.gitignore` to exclude build artifacts for all new SDKs.

## Technical Details
- **Consistent API**: All SDKs expose a similar `AnalyticsClient` interface with methods like `recordSession` and `getSession`.
- **Type Safety**:
  - TypeScript: Full interface definitions.
  - Go: Struct-based requests/responses.
  - Rust: Native Soroban types.
  - Python: Type hints included.
- **Error Handling**: Implemented basic error propagation patterns in all languages.

## Verification
- [x] TypeScript project builds (`tsc`)
- [x] Go module compiles
- [x] Python package is structurally correct
- [x] Rust crate compiles
- [x] Code formatting applied

## Tasks Done
- [x] Create TypeScript/JavaScript SDK
- [x] Add Python SDK
- [x] Implement Go SDK
- [x] Add Rust SDK extensions
- [x] Create SDK documentation
