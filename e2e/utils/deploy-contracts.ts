#!/usr/bin/env ts-node

import { SorobanClient } from './soroban-client.js';
import { config, getWasmPath } from './config.js';
import { Keypair } from '@stellar/stellar-sdk';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface DeploymentResult {
  contractName: string;
  contractId: string;
  wasmPath: string;
}

async function deployAllContracts(): Promise<DeploymentResult[]> {
  console.log('Starting contract deployment to localnet...\n');

  const client = new SorobanClient();
  const results: DeploymentResult[] = [];

  // Get or create admin account
  const admin = await client.getFundedAccount(config.adminSecret);
  console.log(`Using admin account: ${admin.publicKey()}\n`);

  // List of contracts to deploy
  const contracts = ['certificate', 'token', 'progress', 'analytics'];

  for (const contractName of contracts) {
    try {
      console.log(`Deploying ${contractName} contract...`);
      const wasmPath = getWasmPath(contractName);

      if (!fs.existsSync(wasmPath)) {
        console.warn(`⚠️  WASM file not found: ${wasmPath}`);
        console.warn(`   Skipping ${contractName}\n`);
        continue;
      }

      const contractId = await client.deployContract(wasmPath, admin);

      results.push({
        contractName,
        contractId,
        wasmPath,
      });

      console.log(`✅ ${contractName} deployed: ${contractId}\n`);
    } catch (error) {
      console.error(`❌ Failed to deploy ${contractName}:`, error);
      console.error('');
    }
  }

  // Save deployment results to file
  const deploymentsPath = path.join(__dirname, '../config/deployments.json');
  const deploymentsData = {
    network: 'localnet',
    timestamp: new Date().toISOString(),
    adminPublicKey: admin.publicKey(),
    contracts: results.reduce((acc, result) => {
      acc[result.contractName] = result.contractId;
      return acc;
    }, {} as Record<string, string>),
  };

  fs.writeFileSync(deploymentsPath, JSON.stringify(deploymentsData, null, 2));
  console.log(`\n📝 Deployment details saved to: ${deploymentsPath}`);

  return results;
}

// Run deployment if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  deployAllContracts()
    .then((results) => {
      console.log('\n✅ Deployment completed successfully!');
      console.log(`   Deployed ${results.length} contract(s)`);
      process.exit(0);
    })
    .catch((error) => {
      console.error('\n❌ Deployment failed:', error);
      process.exit(1);
    });
}

export { deployAllContracts };
