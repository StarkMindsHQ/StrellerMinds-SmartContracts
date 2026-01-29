import { SorobanClient } from './soroban-client.js';
import { config } from './config.js';
import * as fs from 'fs';

async function main() {
  const client = new SorobanClient();

  // Simple CLI usage: NODE_OPTIONS=--experimental-fetch node gas_profiler.js <contractWasm> <method> [args...]
  const args = process.argv.slice(2);
  if (args.length < 2) {
    console.error('Usage: node gas_profiler.js <wasmPath> <method> [args...]');
    process.exit(1);
  }

  const [wasmPath, method, ...methodArgs] = args;

  const deployer = await client.getFundedAccount(process.env.DEPLOYER_SECRET);

  console.log('Deploying contract (for profiling)...');
  const contractId = await client.deployContract(wasmPath, deployer);

  console.log(`Invoking ${method} on ${contractId}...`);

  // Convert simple string args to ScVal wrapped values; for complex types adapt tests.
  const scArgs: any[] = methodArgs.map(a => {
    // try parse as number
    const n = Number(a);
    if (!Number.isNaN(n)) return a; // leave numeric strings; Soroban client will convert as needed
    return a;
  });

  try {
    const response = await client.invokeContract(contractId, method, scArgs, deployer);
    // Print raw response for inspection
    console.log('Transaction response:');
    console.log(JSON.stringify(response, null, 2));

    // Try to extract resources/footprint information if present
    const meta = response?.metadata;
    if (meta) {
      console.log('\nExtracted metadata:');
      console.log(JSON.stringify(meta, null, 2));
    } else if (response?.result) {
      console.log('\nResult object:');
      console.log(JSON.stringify(response.result, null, 2));
    } else {
      console.log('No metadata/result found in tx response; check localnet or RPC output format.');
    }

    // Save a snapshot for offline analysis
    const snapshotDir = (config as any).snapshotDir || process.env.SNAPSHOT_DIR || '.';
  const outPath = `${snapshotDir}/gas_profile_${Date.now()}.json`;
    fs.writeFileSync(outPath, JSON.stringify(response, null, 2));
    console.log(`Saved profiling snapshot to ${outPath}`);
  } catch (e) {
    console.error('Invocation failed:', e);
    process.exit(2);
  }
}

main().catch(e => { console.error(e); process.exit(1); });
