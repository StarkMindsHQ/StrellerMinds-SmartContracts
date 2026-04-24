import { Keypair, xdr, nativeToScVal } from '@stellar/stellar-sdk';
import { SorobanClient } from './soroban-client.js';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Load deployed contract IDs from deployments.json
 */
export function loadDeployments(): Record<string, string> {
  const deploymentsPath = path.join(__dirname, '../config/deployments.json');

  if (!fs.existsSync(deploymentsPath)) {
    throw new Error(
      'Deployments file not found. Please run deployment first: npm run deploy:local'
    );
  }

  const data = JSON.parse(fs.readFileSync(deploymentsPath, 'utf-8'));
  return data.contracts;
}

/**
 * Create a funded test account
 */
export async function createTestAccount(
  client: SorobanClient
): Promise<Keypair> {
  return await client.getFundedAccount();
}

/**
 * Wait for a condition to be true
 */
export async function waitFor(
  condition: () => Promise<boolean>,
  timeoutMs: number = 30000,
  intervalMs: number = 1000
): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    if (await condition()) {
      return;
    }
    await sleep(intervalMs);
  }

  throw new Error('Timeout waiting for condition');
}

/**
 * Sleep for a specified number of milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Convert a value to ScVal for contract invocation
 */
export function toScVal(value: any): xdr.ScVal {
  return nativeToScVal(value);
}

/**
 * Generate a random string
 */
export function randomString(length: number = 10): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Generate a random number within a range
 */
export function randomNumber(min: number = 0, max: number = 1000): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}
