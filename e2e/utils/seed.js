// Seed data script for Soroban localnet
// This script can be run after localnet is up to fund accounts or deploy contracts
// Example usage: node seed.js


const { Server, Keypair, TransactionBuilder, Networks, Operation, Asset } = require('stellar-sdk');
const fs = require('fs');
require('dotenv').config({ path: '.env.local' });

const server = new Server(process.env.RPC_URL);
const seedAccount = process.env.SEED_ACCOUNT;
const testAccounts = [process.env.TEST_ACCOUNT_1, process.env.TEST_ACCOUNT_2].filter(Boolean);
const contractWasmPath = process.env.CONTRACT_WASM_PATH;

async function fundAccount(publicKey) {
  // Example: Send 100 XLM from seed to publicKey
  // This is a placeholder; actual implementation may differ for Soroban localnet
  console.log(`Funding account: ${publicKey}`);
  // ...funding logic...
}

async function deployContract() {
  if (!contractWasmPath || !fs.existsSync(contractWasmPath)) {
    console.log('No contract WASM found or path not set. Skipping contract deployment.');
    return;
  }
  console.log(`Deploying contract from WASM: ${contractWasmPath}`);
  // ...contract deployment logic...
}

async function main() {
  for (const acct of testAccounts) {
    await fundAccount(acct);
  }
  await deployContract();
}

main().catch(console.error);
