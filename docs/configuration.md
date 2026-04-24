# Configuration Management

This repository now uses shared configuration profiles to reduce hardcoded runtime defaults in contract initialization paths.

## Goals

- replace scattered numeric defaults with named configuration builders
- support environment-specific defaults for `development`, `staging`, and `production`
- validate contract configs before they are persisted

## Shared Config Module

The central config module lives in `contracts/shared/src/config.rs`.

It exposes:

- `DeploymentEnv`
- `ContractConfig::documentation(profile)`
- `ContractConfig::mobile(profile)`
- `ContractConfig::security(profile)`

Each builder returns validated defaults for the selected environment profile.

## Current Integrations

- `documentation` now builds `DocumentationConfig` from shared profile defaults
- `mobile-optimizer` now builds and validates `MobileOptimizerConfig` from shared profile defaults
- `mobile-optimizer` session expiry now uses configured timeout values instead of hardcoded TTLs
- `security-monitor` now exposes `SecurityConfig::for_env(...)` and validates thresholds before use

## Environment Profiles

- `Development`: looser limits for rapid iteration and debugging
- `Staging`: representative pre-production defaults
- `Production`: conservative operational defaults intended for live deployments

## Validation Rules

- document size must be greater than zero
- mobile session timeout, queue limits, and device limits must be positive
- security thresholds and time windows must be positive

## Extension Pattern

When a contract needs more configuration:

1. add a profile-specific defaults struct in `shared::config`
2. expose a `for_env(...)` constructor on the contract config type
3. add a lightweight `validate()` method before persisting config
4. replace raw constants in initialization or runtime code with the validated config values
