import dotenv from 'dotenv';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load environment variables
dotenv.config({ path: path.join(__dirname, '../.env') });

export interface TestConfig {
  networkUrl: string;
  networkPassphrase: string;
  testAccountSecret?: string;
  testAccountPublic?: string;
  adminSecret?: string;
  adminPublic?: string;
  wasmPaths: {
    certificate?: string;
    token?: string;
    progress?: string;
    analytics?: string;
  };
}

export const config: TestConfig = {
  networkUrl: process.env.SOROBAN_NETWORK_URL || 'http://localhost:8000/soroban/rpc',
  networkPassphrase: process.env.SOROBAN_NETWORK_PASSPHRASE || 'Standalone Network ; February 2017',
  testAccountSecret: process.env.TEST_ACCOUNT_SECRET,
  testAccountPublic: process.env.TEST_ACCOUNT_PUBLIC,
  adminSecret: process.env.ADMIN_SECRET,
  adminPublic: process.env.ADMIN_PUBLIC,
  wasmPaths: {
    certificate: process.env.CERTIFICATE_WASM,
    token: process.env.TOKEN_WASM,
    progress: process.env.PROGRESS_WASM,
    analytics: process.env.ANALYTICS_WASM,
  },
};

export function getWasmPath(contractName: string): string {
  const relativePath = config.wasmPaths[contractName as keyof typeof config.wasmPaths];
  if (!relativePath) {
    throw new Error(`WASM path not configured for contract: ${contractName}`);
  }
  return path.join(__dirname, '../../', relativePath);
}
