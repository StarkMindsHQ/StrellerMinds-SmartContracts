export type RetryOptions = {
  retries?: number;
  initialDelayMs?: number;
  maxDelayMs?: number;
  multiplier?: number;
  jitter?: boolean;
  onRetry?: (error: unknown, attempt: number, delayMs: number) => void;
  isRetryable?: (error: unknown) => boolean;
};

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const retryMetrics = {
  attempts: 0,
  successes: 0,
  failures: 0,
  retries: 0,
};

export async function retry<T>(
  task: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const {
    retries = 5,
    initialDelayMs = 200,
    maxDelayMs = 10_000,
    multiplier = 2,
    jitter = true,
    onRetry,
    isRetryable,
  } = options;

  let attempt = 0;
  let delayMs = Math.max(0, initialDelayMs);
  let lastError: unknown;

  while (attempt <= retries) {
    try {
      retryMetrics.attempts += 1;
      const result = await task();
      if (attempt > 0) retryMetrics.retries += attempt;
      retryMetrics.successes += 1;
      return result;
    } catch (error) {
      lastError = error;
      const retryable = isRetryable ? isRetryable(error) : true;
      if (!retryable || attempt === retries) {
        retryMetrics.failures += 1;
        throw error;
      }
      const computedDelay = jitter
        ? Math.min(maxDelayMs, Math.floor(delayMs * (1 + Math.random())))
        : Math.min(maxDelayMs, delayMs);
      if (onRetry) {
        try {
          onRetry(error, attempt + 1, computedDelay);
        } catch {
          // ignore
        }
      }
      await sleep(computedDelay);
      delayMs = Math.min(maxDelayMs, Math.max(1, Math.floor(delayMs * multiplier)));
      attempt++;
    }
  }
  // Should not reach here
  throw lastError ?? new Error('Retry failed with unknown error');
}

export function isTransientNetworkError(error: unknown): boolean {
  if (!error) return false;
  const message = String((error as any)?.message ?? error).toLowerCase();
  return (
    message.includes('timeout') ||
    message.includes('network') ||
    message.includes('temporary') ||
    message.includes('connection reset') ||
    message.includes('econnreset') ||
    message.includes('econnrefused') ||
    message.includes('503') ||
    message.includes('502') ||
    message.includes('429') ||
    message.includes('rate limit') ||
    message.includes('not_found')
  );
}

