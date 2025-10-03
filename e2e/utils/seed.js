// Seed data script for Soroban localnet
// This script can be run after localnet is up to fund accounts or deploy contracts
// Example usage: node seed.js

const { Server, Keypair } = require('stellar-sdk');
require('dotenv').config({ path: '.env.local' });

const server = new Server(process.env.RPC_URL);
const seedAccount = process.env.SEED_ACCOUNT;

async function fundAccount(publicKey) {
  // Implement funding logic here
  // ...
}

async function main() {
  // Example: Fund a test account
  // await fundAccount('G...');
  // Deploy contracts if needed
}

main().catch(console.error);
