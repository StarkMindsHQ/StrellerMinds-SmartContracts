import { describe, test, expect, beforeAll } from '@jest/globals';
import { SorobanClient } from '../utils/soroban-client.js';
import { loadDeployments } from '../utils/test-helpers.js';

describe('E2E Test Setup', () => {
  let client: SorobanClient;
  let deployments: Record<string, string>;

  beforeAll(async () => {
    client = new SorobanClient();
    deployments = loadDeployments();
  });

  test('should connect to localnet', async () => {
    const server = client.getServer();
    expect(server).toBeDefined();

    // Verify network is accessible
    const health = await server.getHealth();
    expect(health.status).toBe('healthy');
  });

  test('should have deployed contracts', () => {
    expect(deployments).toBeDefined();
    expect(Object.keys(deployments).length).toBeGreaterThan(0);

    console.log('Deployed contracts:', deployments);
  });

  test('should be able to create and fund test accounts', async () => {
    const testAccount = await client.getFundedAccount();
    expect(testAccount).toBeDefined();
    expect(testAccount.publicKey()).toBeDefined();

    // Verify account is funded
    const account = await client.getServer().getAccount(testAccount.publicKey());
    expect(account).toBeDefined();
  });
});
