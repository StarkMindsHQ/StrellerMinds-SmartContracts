# Getting Started with StrellerMinds API

This guide helps you set up your environment and make your first API calls to StrellerMinds smart contracts.

## Prerequisites

- Node.js 18+ or Rust 1.70+
- Stellar account with testnet funding
- Basic understanding of blockchain smart contracts

## Environment Setup

### Option 1: JavaScript/TypeScript

```bash
# Install Stellar SDK
npm install @stellar/stellar-sdk

# Install Soroban CLI
cargo install soroban-cli
```

### Option 2: Rust

```bash
# Add to your Cargo.toml
[dependencies]
soroban-sdk = "22.0.0"
```

## Connect to Testnet

StrellerMinds contracts run on Stellar testnet. Configure your SDK:

```typescript
import { Networks, SorobanRpc } from '@stellar/stellar-sdk';

const rpcUrl = 'https://soroban-testnet.stellar.org:443';
const networkPassphrase = Networks.TESTNET;
```

## Contract Addresses

| Contract | Testnet Address |
|----------|-----------------|
| Token | `CDLZFC3SYJBDYDAJSKJWZCF7HGH4GZ3C6DBJJTYWA7WZ4WMZBFHCNPOB` |
| Certificate | `CBZCHZNEB4WN6MSCASSE4KC3HQBK27WYEGH3HRVS6Z3LOJGS43LF64XY` |
| Progress | `CDB3HJWVCCZCDVIGEILEFGMOBJLHLG3NRYPBZ3Z2GVOIPNRBQ3WCNZXII` |
| Assessment | `CDHHZ5Z6FVGCJLCMTL5BSCHEWF5MGHGGYWLLWLDILWMMF6LYLVLXQ6K5H` |
| Analytics | `CCZ7V2WN4L4O3FOIPN5FGSLIOONJ62K2PJWS4K6BHPGAFZEBK4J6HRLOO` |
| Gamification | `CDJ7RNPJEA2WVLTV5MSZM7LHQGDYQRPZ4QTO6ZBHMEYLBYYBQM4XW6EHA` |
| Community | `CAJNF7U5OEF3XCNBZEMWDGSHEIYNLBWSY76E7G26XBCMNWJBBLGGJVY7K` |
| Marketplace | `CDN7HQEV3JLGL7NHLGBHNUBCKPOEJHPGGVFNPEYPAT7W5WSZZXTKWQ6XU` |

## Your First API Call

### JavaScript: Record Progress

```typescript
import { Contract, Keypair, Networks } from '@stellar/stellar-sdk';

const progressContract = new Contract('CDB3HJWVCCZCDVIGEILEFGMOBJLHLG3NRYPBZ3Z2GVOIPNRBQ3WCNZXII');

// Student keypair
const student = Keypair.fromSecret('SDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA');

// Record progress (requires student auth)
await progressContract.call(
  'record_progress',
  student.publicKey(),  // student address
  'course_101',         // course ID
  75                     // progress percentage
);

console.log('Progress recorded!');
```

### Rust: Record Progress

```rust
use soroban_sdk::{Env, Symbol};

pub fn record_progress(env: &Env, student: &Address, course_id: Symbol, progress: u32) {
    let client = progress::ProgressClient::new(env, &progress_contract_id);
    client.record_progress(student, &course_id, progress);
}
```

## Common Patterns

### Authorization Flow

Most write operations require authorization:

```typescript
// 1. Create the transaction
const tx = progressContract.call(
  'record_progress',
  student.publicKey(),
  'course_101',
  75
);

// 2. Sign with student's keypair
tx.sign(student);

// 3. Submit
await tx.submit();
```

### Query Data (No Auth Required)

Read operations don't require authorization:

```typescript
// Get progress (no auth needed)
const progress = await progressContract.call(
  'get_progress',
  student.publicKey(),
  'course_101'
);

console.log(`Progress: ${progress}%`);
```

### Error Handling

```typescript
try {
  await progressContract.call('record_progress', student, 'course_101', 75);
} catch (e) {
  if (e.message.includes('RateLimitExceeded')) {
    console.log('Daily limit reached. Try again tomorrow.');
  } else if (e.message.includes('InvalidProgress')) {
    console.log('Progress must be 0-100.');
  } else {
    throw e;
  }
}
```

## Rate Limits

| Operation | Limit | Reset |
|-----------|-------|-------|
| Record progress | 100/day | 24 hours |
| Mint tokens | 50/day | 24 hours |
| Transfer tokens | 100/day | 24 hours |
| Start assessment | 3/day | 24 hours |
| Submit answers | 5/day | 24 hours |

## Next Steps

- [Authentication Guide](authentication.md) - Learn about Soroban auth
- [Code Examples - Rust](code-examples/rust.md) - Rust SDK examples
- [Code Examples - TypeScript](code-examples/typescript.md) - JS/TS SDK examples