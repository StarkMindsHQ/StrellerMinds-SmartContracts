# End-to-End (E2E) Test Harness

This directory contains the end-to-end test suite for StellarMinds smart contracts running on Soroban localnet.

## Overview

The E2E test harness validates integrated behavior across all contracts in a local Soroban network environment. It provides automated testing for contract deployment, initialization, and common workflows.

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Docker and Docker Compose
- Rust and Soroban CLI (for building contracts)

### Single Command Test Execution

Run the complete E2E test suite with a single command:

```bash
# From project root
./scripts/run-e2e-tests.sh
```

This script will:
1. Build all smart contracts
2. Start the Soroban localnet
3. Install test dependencies
4. Deploy contracts to localnet
5. Run all E2E tests
6. Clean up localnet (unless `--skip-cleanup` is specified)

### Manual Step-by-Step Execution

#### 1. Build Contracts

```bash
# From project root
./scripts/build.sh
```

#### 2. Start Localnet

```bash
./scripts/localnet-up.sh
```

#### 3. Install Dependencies

```bash
cd e2e
npm install
```

#### 4. Deploy Contracts

```bash
npm run deploy:local
```

#### 5. Run Tests

```bash
npm test
```

#### 6. Stop Localnet

```bash
# From project root
./scripts/localnet-down.sh
```

## Test Structure

### Directory Layout

```
e2e/
├── config/           # Configuration files
│   └── deployments.json  # Deployed contract IDs (generated)
├── tests/            # Test files
│   ├── setup.test.ts      # Network and setup verification
│   ├── certificate.test.ts # Certificate contract tests
│   └── integration.test.ts # Multi-contract integration tests
├── utils/            # Utility modules
│   ├── soroban-client.ts   # Soroban RPC client wrapper
│   ├── config.ts           # Configuration management
│   ├── deploy-contracts.ts # Contract deployment script
│   └── test-helpers.ts     # Test helper functions
├── package.json      # Node.js dependencies and scripts
├── tsconfig.json     # TypeScript configuration
├── jest.config.js    # Jest test configuration
└── .env.example      # Environment variables template
```

### Test Files

- **setup.test.ts**: Verifies localnet connection and basic setup
- **certificate.test.ts**: Tests certificate contract flows (mint, verify, revoke)
- **integration.test.ts**: Tests cross-contract interactions and workflows

## Configuration

### Environment Variables

Create a `.env` file in the `e2e/` directory based on `.env.example`:

```bash
cp .env.example .env
```

Key configuration options:
- `SOROBAN_NETWORK_URL`: Localnet RPC endpoint (default: http://localhost:8000/soroban/rpc)
- `SOROBAN_NETWORK_PASSPHRASE`: Network passphrase (default: Standalone Network ; February 2017)
- Contract WASM paths for deployment

### Docker Compose

The localnet is configured in `docker-compose.yml` at the project root. It uses the official Stellar quickstart image with Soroban RPC enabled.

## Available Scripts

From the `e2e/` directory:

| Script | Description |
|--------|-------------|
| `npm test` | Run all tests |
| `npm run test:ci` | Run tests in CI mode with JUnit output |
| `npm run deploy:local` | Deploy contracts to localnet |
| `npm run localnet:up` | Start localnet |
| `npm run localnet:down` | Stop and clean localnet |
| `npm run localnet:logs` | View localnet logs |

## Test Output

### Console Output

Tests display detailed progress and results in the console with colored output.

### JUnit XML Report

When running in CI mode (`npm run test:ci`), a JUnit-compatible XML report is generated:

```
e2e/test-results/junit.xml
```

This format is compatible with most CI/CD systems for test reporting and visualization.

## Writing New Tests

### Basic Test Structure

```typescript
import { describe, test, expect, beforeAll } from '@jest/globals';
import { SorobanClient } from '../utils/soroban-client.js';
import { loadDeployments, createTestAccount } from '../utils/test-helpers.js';

describe('My Contract Tests', () => {
  let client: SorobanClient;
  let contractId: string;

  beforeAll(async () => {
    client = new SorobanClient();
    const deployments = loadDeployments();
    contractId = deployments.my_contract;
  });

  test('should perform contract operation', async () => {
    // Test implementation
  });
});
```

### Helper Functions

- `loadDeployments()`: Load deployed contract IDs
- `createTestAccount()`: Create and fund a test account
- `toScVal()`: Convert values to Soroban ScVal format
- `waitFor()`: Wait for a condition with timeout

## CI Integration

The E2E tests are integrated into the CI pipeline via `.github/workflows/e2e.yml`. The workflow:

1. Checks out the repository
2. Sets up Docker and Node.js
3. Builds contracts
4. Runs E2E tests
5. Publishes JUnit test results
6. Archives test artifacts

The tests run automatically on pull requests to the `main` branch.

## Troubleshooting

### Localnet won't start

```bash
# Check if Docker is running
docker info

# View localnet logs
docker-compose logs -f

# Restart localnet
./scripts/localnet-down.sh
./scripts/localnet-up.sh
```

### Contract deployment fails

```bash
# Verify contracts are built
ls -la target/wasm32-unknown-unknown/release/*.optimized.wasm

# Rebuild contracts
./scripts/build.sh
```

### Tests fail with "deployments.json not found"

```bash
# Deploy contracts first
cd e2e
npm run deploy:local
```

### Port conflicts

If port 8000 is already in use, modify `docker-compose.yml` to use a different port.

## Common Test Scenarios

### 1. Minting a Certificate

```typescript
test('should mint certificate', async () => {
  const result = await client.invokeContract(
    certificateId,
    'mint',
    [recipientAddress, mintParams],
    issuer
  );
  expect(result.status).toBe('SUCCESS');
});
```

### 2. Verifying Contract State

```typescript
test('should verify contract state', async () => {
  const data = await client.getContractData(contractId);
  expect(data).toBeDefined();
});
```

### 3. Multi-Contract Workflow

```typescript
test('should complete multi-contract flow', async () => {
  // Update progress
  await client.invokeContract(progressId, 'update', [...], user);

  // Mint certificate based on progress
  await client.invokeContract(certificateId, 'mint', [...], issuer);

  // Verify analytics recorded
  const analytics = await client.invokeContract(analyticsId, 'get', [...], user);
  expect(analytics).toBeDefined();
});
```

## Contributing

When adding new tests:

1. Follow the existing test structure
2. Use descriptive test names
3. Add appropriate timeout values for contract calls
4. Include console logging for debugging
5. Handle expected errors gracefully
6. Update this README if adding new test categories

## License

MIT
