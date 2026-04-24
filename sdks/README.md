# StrellerMinds SDKs

This directory contains client SDKs for interacting with StrellerMinds Smart Contracts in various languages.

## Available SDKs

### TypeScript / JavaScript
Located in `typescript/`.
- **Installation**: `npm install @strellerminds/sdk`
- **Usage**:
  ```typescript
  import { AnalyticsClient } from '@strellerminds/sdk';
  const client = new AnalyticsClient('contract-id', 'rpc-url', 'secret');
  ```

### Python
Located in `python/`.
- **Installation**: `pip install strellerminds-sdk`
- **Usage**:
  ```python
  from strellerminds_sdk import AnalyticsClient
  client = AnalyticsClient('contract-id', 'rpc-url', 'secret')
  ```

### Go
Located in `go/`.
- **Installation**: `go get github.com/strellerminds/sdk-go`
- **Usage**:
  ```go
  import "github.com/strellerminds/sdk-go"
  client := strellerminds.NewClient("contract-id", "rpc-url", "secret")
  ```

### Rust
Located in `rust/`.
- **Usage**: Add to `Cargo.toml`.
  ```toml
  [dependencies]
  strellerminds-sdk = { path = "sdks/rust" }
  ```
