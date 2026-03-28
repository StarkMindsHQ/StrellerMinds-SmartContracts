export type RetryOptions = {
    retries?: number;
    initialDelayMs?: number;
    maxDelayMs?: number;
    multiplier?: number;
    jitter?: boolean;
    onRetry?: (error: unknown, attempt: number, delayMs: number) => void;
    isRetryable?: (error: unknown) => boolean;
    signal?: AbortSignal;
};

function sleep(delayMs: number, signal?: AbortSignal): Promise<void> {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(resolve, delayMs);
        if (signal) {
            const onAbort = () => {
                clearTimeout(timer);
                reject(new Error('Retry aborted'));
            };
            if (signal.aborted) {
                onAbort();
                return;
            }
            signal.addEventListener('abort', onAbort, { once: true });
        }
    });
}

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
        signal,
    } = options;

    let attempt = 0;
    let delayMs = Math.max(0, initialDelayMs);
    let lastError: unknown;

    while (attempt <= retries) {
        try {
            return await task();
        } catch (error) {
            lastError = error;
            const retryable = isRetryable ? isRetryable(error) : true;
            if (!retryable || attempt === retries) {
                throw error;
            }
            const computedDelay = jitter
                ? Math.min(maxDelayMs, Math.floor(delayMs * (1 + Math.random())))
                : Math.min(maxDelayMs, delayMs);
            if (onRetry) {
                try {
                    onRetry(error, attempt + 1, computedDelay);
                } catch {
                    // ignore onRetry errors
                }
            }
            await sleep(computedDelay, signal);
            delayMs = Math.min(maxDelayMs, Math.max(1, Math.floor(delayMs * multiplier)));
            attempt += 1;
        }
    }
    // Should never reach here
    throw lastError ?? new Error('Retry failed with unknown error');
}

export function isTransientNetworkError(error: unknown): boolean {
    if (!error) return false;
    const message = String((error as any).message ?? error).toLowerCase();
    // Heuristics for transient conditions
    return (
        message.includes('timeout') ||
        message.includes('timed out') ||
        message.includes('network') ||
        message.includes('temporary') ||
        message.includes('connection reset') ||
        message.includes('econnreset') ||
        message.includes('econnrefused') ||
        message.includes('503') ||
        message.includes('502') ||
        message.includes('429') ||
        message.includes('rate limit') ||
        message.includes('not_found') // polling window race
    );
}

