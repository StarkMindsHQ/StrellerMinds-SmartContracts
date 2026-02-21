# Cross-Chain Credentials Deployment Guide

## Prerequisites

- Rust toolchain with wasm32-unknown-unknown target
- Soroban CLI v21.5.0+
- Stellar CLI v21.5.0+
- Stellar testnet/mainnet account with XLM balance

## Quick Start

### 1. Build the Contract

```bash
cd /home/joash/Desktop/Drips/StrellerMinds-SmartContracts
cargo build --release --target wasm32-unknown-unknown -p cross-chain-credentials
```

The compiled WASM will be at:
```
target/wasm32-unknown-unknown/release/cross_chain_credentials.wasm
```

### 2. Run Tests

```bash
cargo test -p cross-chain-credentials
```

Expected output: `7 passed; 0 failed`

### 3. Deploy to Testnet

```bash
# Set your secret key
export STELLAR_SECRET_KEY='your_secret_key_here'

# Deploy using the deployment script
./scripts/deploy.sh \
  --network testnet \
  --contract cross-chain-credentials \
  --wasm target/wasm32-unknown-unknown/release/cross_chain_credentials.wasm
```

### 4. Initialize the Contract

```bash
# Get the deployed contract ID
CONTRACT_ID=$(cat target/cross-chain-credentials.testnet.id)

# Initialize with admin address
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <YOUR_ACCOUNT> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

## Usage Examples

### Issue a Credential

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <ADMIN_ACCOUNT> \
  --network testnet \
  -- issue_credential \
  --student <STUDENT_ADDRESS> \
  --achievement "Blockchain Fundamentals Certificate" \
  --metadata_hash "ipfs://QmHash..." \
  --chain_id Stellar
```

### Verify Cross-Chain

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- verify_cross_chain \
  --credential_id "CRED-..." \
  --target_chain Ethereum
```

### Generate Student Transcript

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- generate_transcript \
  --student <STUDENT_ADDRESS>
```

### Add Oracle

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account <ADMIN_ACCOUNT> \
  --network testnet \
  -- add_oracle \
  --oracle <ORACLE_ADDRESS>
```

## Integration with Frontend

### JavaScript/TypeScript Example

```typescript
import { Contract, SorobanRpc } from '@stellar/stellar-sdk';

const contractId = 'YOUR_CONTRACT_ID';
const rpcUrl = 'https://soroban-testnet.stellar.org';

const server = new SorobanRpc.Server(rpcUrl);
const contract = new Contract(contractId);

// Issue credential
async function issueCredential(student, achievement, metadataHash) {
  const tx = await contract.call(
    'issue_credential',
    student,
    achievement,
    metadataHash,
    { type: 'symbol', value: 'Stellar' }
  );
  
  const result = await server.sendTransaction(tx);
  return result;
}

// Get student credentials
async function getStudentCredentials(studentAddress) {
  const result = await contract.call(
    'get_student_credentials',
    studentAddress
  );
  return result;
}

// Generate transcript
async function generateTranscript(studentAddress) {
  const result = await contract.call(
    'generate_transcript',
    studentAddress
  );
  return result;
}
```

### Python Example

```python
from stellar_sdk import Keypair, Network, SorobanServer, TransactionBuilder
from stellar_sdk.soroban_rpc import GetTransactionStatus

# Setup
server = SorobanServer("https://soroban-testnet.stellar.org")
source = Keypair.from_secret("YOUR_SECRET_KEY")
contract_id = "YOUR_CONTRACT_ID"

# Issue credential
def issue_credential(student_address, achievement, metadata_hash):
    tx = (
        TransactionBuilder(
            source_account=source.public_key,
            network_passphrase=Network.TESTNET_NETWORK_PASSPHRASE,
            base_fee=100
        )
        .append_invoke_contract_function_op(
            contract_id=contract_id,
            function_name="issue_credential",
            parameters=[
                student_address,
                achievement,
                metadata_hash,
                "Stellar"
            ]
        )
        .build()
    )
    
    tx.sign(source)
    response = server.send_transaction(tx)
    return response

# Get credentials
def get_student_credentials(student_address):
    response = server.invoke_contract_function(
        contract_id=contract_id,
        function_name="get_student_credentials",
        parameters=[student_address]
    )
    return response
```

## Monitoring & Maintenance

### Check Contract Status

```bash
stellar contract info \
  --id $CONTRACT_ID \
  --network testnet
```

### View Contract Events

```bash
stellar events \
  --start-ledger <START> \
  --contract-ids $CONTRACT_ID \
  --network testnet
```

### Extend Contract TTL

```bash
stellar contract extend \
  --id $CONTRACT_ID \
  --ledgers-to-extend 100000 \
  --network testnet
```

## Security Considerations

1. **Admin Key Management**: Store admin private keys securely (HSM, KMS)
2. **Oracle Selection**: Only authorize trusted oracle operators
3. **Metadata Storage**: Store sensitive credential data off-chain (IPFS, encrypted storage)
4. **Rate Limiting**: Implement rate limiting at application layer
5. **Monitoring**: Set up alerts for suspicious activities

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build --release --target wasm32-unknown-unknown -p cross-chain-credentials
```

### Deployment Failures

```bash
# Check account balance
stellar account --account <YOUR_ACCOUNT> --network testnet

# Verify WASM file
ls -lh target/wasm32-unknown-unknown/release/cross_chain_credentials.wasm
```

### Transaction Failures

```bash
# Simulate transaction first
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  --simulate-only \
  -- <function_name> <args>
```

## Cost Estimation

Approximate costs on Stellar testnet/mainnet:

- Contract deployment: ~0.5 XLM
- Credential issuance: ~0.001 XLM per credential
- Cross-chain verification: ~0.002 XLM per verification
- Transcript generation: ~0.001 XLM per transcript

## Support & Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/cli)
- [Project Repository](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts)
- [Issue Tracker](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues)

## Next Steps

1. Deploy to testnet and test all functions
2. Integrate with frontend application
3. Set up oracle network
4. Configure monitoring and alerts
5. Prepare for mainnet deployment
6. Conduct security audit
7. Document API endpoints
8. Create user guides
