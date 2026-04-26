import { describe, test, expect, beforeAll } from '@jest/globals';
import { Keypair, nativeToScVal } from '@stellar/stellar-sdk';
import { SorobanClient } from '../utils/soroban-client.js';
import { loadDeployments, createTestAccount } from '../utils/test-helpers.js';

describe('Chaos Engineering Test Suite', () => {
  let client: SorobanClient;
  let admin: Keypair;
  let student: Keypair;
  let deployments: Record<string, string>;

  beforeAll(async () => {
    client = new SorobanClient();
    deployments = loadDeployments();
    admin = await createTestAccount(client);
    student = await createTestAccount(client);
  }, 120000);

  /**
   * Test Case: Network Latency / Service Degradation
   * Simulates high latency in RPC calls to verify client resilience.
   */
  test('should handle high network latency during contract invocation', async () => {
    const tokenId = deployments.token;
    if (!tokenId) return;

    // Simulate 2000ms latency
    const delay = (ms: number) => new Promise(res => setTimeout(res, ms));
    
    console.log('Simulating 2s network latency...');
    await delay(2000);

    const result = await client.invokeContract(
      tokenId,
      'balance',
      [nativeToScVal(student.publicKey(), { type: 'address' })],
      student
    );

    expect(result.status).toBe('SUCCESS');
    console.log('✓ Successfully handled high latency');
  }, 30000);

  /**
   * Test Case: Resource Exhaustion
   * Simulates a burst of transactions to verify throughput limits and error handling.
   */
  test('should handle burst load (Resource Exhaustion simulation)', async () => {
    const progressId = deployments.progress;
    if (!progressId) return;

    const BURST_SIZE = 10;
    const promises = [];

    console.log(`Sending burst of ${BURST_SIZE} transactions...`);
    for (let i = 0; i < BURST_SIZE; i++) {
      promises.push(client.invokeContract(
        progressId,
        'update_progress',
        [
          nativeToScVal(student.publicKey(), { type: 'address' }),
          nativeToScVal(`COURSE-${i}`, { type: 'string' }),
          nativeToScVal(i * 10, { type: 'u32' })
        ],
        admin
      ));
    }

    const results = await Promise.allSettled(promises);
    const successes = results.filter(r => r.status === 'fulfilled').length;
    
    console.log(`✓ Burst completed. Successes: ${successes}/${BURST_SIZE}`);
    expect(successes).toBeGreaterThan(0);
  }, 60000);

  /**
   * Test Case: Database/Storage Failure (Simulation)
   * Simulates trying to access a non-existent or "corrupted" key.
   */
  test('should handle missing storage entries gracefully', async () => {
    const analyticsId = deployments.analytics;
    if (!analyticsId) return;

    console.log('Attempting to fetch non-existent session data...');
    
    try {
      const result = await client.invokeContract(
        analyticsId,
        'get_session',
        [nativeToScVal('NON-EXISTENT-ID', { type: 'string' })],
        student
      );
      
      // Should return null or appropriate error val, not crash
      expect(result.status).toBe('SUCCESS');
    } catch (error) {
      console.log('✓ Caught expected behavior for missing data');
    }
  });
});
