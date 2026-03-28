import { retry, isTransientNetworkError, retryMetrics } from '../utils/retry.js';

describe('retry utility', () => {
  beforeEach(() => {
    retryMetrics.attempts = 0;
    retryMetrics.successes = 0;
    retryMetrics.failures = 0;
    retryMetrics.retries = 0;
  });

  it('retries transient failures and succeeds', async () => {
    let calls = 0;
    const result = await retry(
      async () => {
        calls++;
        if (calls < 3) {
          throw new Error('temporary network timeout');
        }
        return 'ok';
      },
      {
        retries: 5,
        initialDelayMs: 10,
        maxDelayMs: 50,
        isRetryable: isTransientNetworkError,
      }
    );
    expect(result).toBe('ok');
    expect(calls).toBe(3);
    expect(retryMetrics.successes).toBe(1);
    expect(retryMetrics.failures).toBe(0);
  });

  it('fails fast on non-transient errors', async () => {
    let calls = 0;
    await expect(
      retry(
        async () => {
          calls++;
          throw new Error('permanent validation error');
        },
        {
          retries: 5,
          initialDelayMs: 10,
          maxDelayMs: 50,
          isRetryable: isTransientNetworkError,
        }
      )
    ).rejects.toThrow('permanent validation error');
    expect(calls).toBe(1);
    expect(retryMetrics.failures).toBe(1);
  });
});

