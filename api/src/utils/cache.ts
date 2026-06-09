/**
 * In-memory TTL cache with in-flight request coalescing.
 *
 * Coalescing: concurrent calls for the same key share one Promise, so only
 * one upstream RPC fires even under heavy parallelism.
 */

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

export interface CacheStats {
  hits: number;
  misses: number;
  size: number;
}

export class TtlCache<T = unknown> {
  private store = new Map<string, CacheEntry<T>>();
  private inflight = new Map<string, Promise<T>>();

  hits = 0;
  misses = 0;

  constructor(private defaultTtlMs: number) {}

  get(key: string): T | undefined {
    const entry = this.store.get(key);
    if (!entry) return undefined;
    if (Date.now() > entry.expiresAt) {
      this.store.delete(key);
      return undefined;
    }
    return entry.value;
  }

  set(key: string, value: T, ttlMs = this.defaultTtlMs): void {
    this.store.set(key, { value, expiresAt: Date.now() + ttlMs });
  }

  delete(key: string): void {
    this.store.delete(key);
    this.inflight.delete(key);
  }

  deletePrefix(prefix: string): void {
    for (const key of this.store.keys()) {
      if (key.startsWith(prefix)) this.store.delete(key);
    }
  }

  stats(): CacheStats {
    return { hits: this.hits, misses: this.misses, size: this.store.size };
  }

  /**
   * Fetch with coalescing: if a request for `key` is already in-flight, return
   * its Promise. Otherwise check the cache, then call `fn` and cache the result.
   */
  async getOrFetch(
    key: string,
    fn: () => Promise<T>,
    ttlMs = this.defaultTtlMs
  ): Promise<T> {
    const cached = this.get(key);
    if (cached !== undefined) {
      this.hits++;
      return cached;
    }

    const existing = this.inflight.get(key);
    if (existing) return existing;

    this.misses++;
    const promise = fn().then(
      (value) => {
        this.set(key, value, ttlMs);
        this.inflight.delete(key);
        return value;
      },
      (err) => {
        this.inflight.delete(key);
        throw err;
      }
    );

    this.inflight.set(key, promise);
    return promise;
  }
}
