import {
  queryDuration,
  queryCacheEvents,
  queryInFlightBackend,
  queryPoolActiveSize,
  queryPoolAvailableSize,
  queryCacheEntries,
  queryEstimatedLoadReduction,
  queryAverageTimeMs,
  queryCacheHitRatio,
} from "../metrics";
import { logger } from "../logger";

const MAX_SAMPLE_COUNT = 200;
const MAX_EFFECTIVE_TTL_MS = 300_000;
const SHORT_NULL_CACHE_TTL_MS = 5_000;

type CacheEvent = "hit" | "miss" | "stale" | "eviction";
type QuerySource = "cache" | "backend";
type QueryStatus = "success" | "error";
type BackendFetchReason = "request" | "prewarm";

export interface QueryOptimizerOptions {
  defaultTtlMs: number;
  maxEntries: number;
  slowThresholdMs: number;
  targetAvgMs: number;
  targetLoadReductionPercent: number;
  poolSize: number;
}

export interface ExecuteQueryOptions<T> {
  queryName: string;
  cacheKey: string;
  cacheTtlMs?: number;
  cacheNull?: boolean;
  nullCacheTtlMs?: number;
  singleton?: boolean;
  canPrewarm?: boolean;
  shouldCacheValue?: (value: T) => boolean;
}

interface CacheEntry<T> {
  queryName: string;
  key: string;
  value: T;
  createdAt: number;
  lastAccessedAt: number;
  expiresAt: number;
  ttlMs: number;
  baseTtlMs: number;
}

interface QueryStatBucket {
  totalRequests: number;
  totalDurationMs: number;
  totalCacheHits: number;
  totalCacheMisses: number;
  totalCacheStale: number;
  totalEvictions: number;
  totalErrors: number;
  totalBackendCalls: number;
  totalBackendRequestCalls: number;
  totalBackgroundRefreshes: number;
  slowQueryCount: number;
  lastDurationMs: number;
  lastBackendDurationMs: number;
  lastObservedAt: string | null;
  configuredBaseTtlMs: number;
  requestSamplesMs: number[];
  backendSamplesMs: number[];
}

export interface IndexOptimizationSpec {
  query: string;
  backend: "soroban_contract" | "offchain_indexer";
  accessPattern: string;
  recommendedLookupKey: string;
  recommendedIndexSpec: string;
  cacheCoverage: "full" | "partial" | "none";
  directLookupCovered: boolean;
  notes: string;
}

export interface QueryOptimizationReport {
  timestamp: string;
  targets: {
    avgQueryTimeMs: number;
    loadReductionPercent: number;
  };
  summary: {
    averageQueryTimeMs: number;
    meetsAverageQueryTarget: boolean;
    estimatedLoadReductionPercent: number;
    meetsLoadReductionTarget: boolean;
    cacheHitRatio: number;
    cacheEntries: number;
    totalRequests: number;
    totalBackendCalls: number;
    totalBackendRequestCalls: number;
    totalBackgroundRefreshes: number;
    slowQueries: number;
  };
  cache: {
    maxEntries: number;
    activeEntries: number;
    hitRatio: number;
    totals: {
      hits: number;
      misses: number;
      stale: number;
      evictions: number;
    };
  };
  pool: {
    size: number;
    available: number;
    inFlightBackendQueries: number;
    configuredRpcUrls: string[];
    roundRobinCursor: number;
  };
  slowQueries: Array<{
    query: string;
    slowCount: number;
    lastDurationMs: number;
    averageDurationMs: number;
    p95DurationMs: number;
  }>;
  topQueries: Array<{
    query: string;
    requests: number;
    averageDurationMs: number;
    p95DurationMs: number;
    p99DurationMs: number;
    cacheHitRatio: number;
    estimatedLoadReductionPercent: number;
    backendCalls: number;
    backendRequestCalls: number;
    backgroundRefreshes: number;
    lastDurationMs: number;
    lastBackendDurationMs: number;
    effectiveTtlMs: number;
  }>;
  optimization: {
    adaptiveCachingEnabled: boolean;
    knownSingletonPrewarmQueries: string[];
    recommendations: string[];
    indexOptimizationSpecs: IndexOptimizationSpec[];
  };
}

export class QueryOptimizer {
  private readonly cache = new Map<string, CacheEntry<unknown>>();
  private readonly inFlight = new Map<string, Promise<unknown>>();
  private readonly prewarmInFlight = new Map<string, Promise<unknown>>();
  private readonly stats = new Map<string, QueryStatBucket>();
  private readonly options: QueryOptimizerOptions;
  private readonly indexSpecs: IndexOptimizationSpec[];
  private activeBackendFetches = 0;

  constructor(options: QueryOptimizerOptions) {
    this.options = options;
    this.indexSpecs = [
      {
        query: "get_certificate",
        backend: "soroban_contract",
        accessPattern: "certificate lookup by certificate ID",
        recommendedLookupKey: "certificate_id",
        recommendedIndexSpec: "Primary direct key lookup on certificate_id; off-chain unique index on certificate_id",
        cacheCoverage: "full",
        directLookupCovered: true,
        notes: "Soroban contract storage is already keyed by certificate identifier; cache fronts repeat reads.",
      },
      {
        query: "get_revocation_record",
        backend: "soroban_contract",
        accessPattern: "revocation lookup by certificate ID",
        recommendedLookupKey: "certificate_id",
        recommendedIndexSpec: "Primary direct key lookup on certificate_id; off-chain secondary index on revocation.certificate_id",
        cacheCoverage: "full",
        directLookupCovered: true,
        notes: "Revocation reads are point lookups and benefit from long TTL caching because records are append-only after revocation.",
      },
      {
        query: "get_student_certificates",
        backend: "offchain_indexer",
        accessPattern: "list certificate IDs by student address",
        recommendedLookupKey: "student_address",
        recommendedIndexSpec: "Off-chain composite index on (student_address, issued_at DESC) with certificate_id as covering field",
        cacheCoverage: "partial",
        directLookupCovered: true,
        notes: "Contract read is address-keyed; off-chain indexers should maintain a student-address fan-out index for pagination and history queries.",
      },
      {
        query: "get_analytics",
        backend: "offchain_indexer",
        accessPattern: "singleton aggregate analytics snapshot",
        recommendedLookupKey: "analytics_singleton",
        recommendedIndexSpec: "Materialized singleton aggregate keyed as analytics/current or a single-row aggregate snapshot",
        cacheCoverage: "full",
        directLookupCovered: true,
        notes: "This is a singleton query and the best candidate for adaptive TTL growth plus background prewarming.",
      },
    ];

    queryPoolActiveSize.set(this.options.poolSize);
    this.updateRuntimeGauges();
  }

  async execute<T>(
    options: ExecuteQueryOptions<T>,
    fetcher: () => Promise<T>
  ): Promise<T> {
    const startedAt = Date.now();
    const stat = this.getStatBucket(options.queryName);
    stat.configuredBaseTtlMs = options.cacheTtlMs ?? this.options.defaultTtlMs;
    const cachedEntry = this.cache.get(options.cacheKey) as CacheEntry<T> | undefined;

    if (cachedEntry) {
      const now = Date.now();
      if (cachedEntry.expiresAt > now) {
        this.touchCacheEntry(options.cacheKey, cachedEntry);
        this.recordCacheEvent(options.queryName, "hit");
        this.observeRequest(options.queryName, Date.now() - startedAt, "cache", "success");
        this.maybeSchedulePrewarm(options, fetcher, cachedEntry);
        return cachedEntry.value;
      }

      this.cache.delete(options.cacheKey);
      this.recordCacheEvent(options.queryName, "stale");
      this.recordCacheEvent(options.queryName, "miss");
      this.updateRuntimeGauges();
    } else {
      this.recordCacheEvent(options.queryName, "miss");
    }

    const inFlightRequest = this.inFlight.get(options.cacheKey) as Promise<T> | undefined;
    if (inFlightRequest) {
      try {
        const value = await inFlightRequest;
        this.observeRequest(options.queryName, Date.now() - startedAt, "backend", "success");
        return value;
      } catch (error) {
        this.observeRequest(options.queryName, Date.now() - startedAt, "backend", "error");
        throw error;
      }
    }

    const backendPromise = this.fetchAndCache(options, fetcher, "request");
    this.inFlight.set(options.cacheKey, backendPromise as Promise<unknown>);

    try {
      const value = await backendPromise;
      this.observeRequest(options.queryName, Date.now() - startedAt, "backend", "success");
      return value;
    } catch (error) {
      this.observeRequest(options.queryName, Date.now() - startedAt, "backend", "error");
      throw error;
    } finally {
      this.inFlight.delete(options.cacheKey);
      this.updateRuntimeGauges();
    }
  }

  invalidate(filters?: { queryName?: string; keyPrefix?: string }): { invalidated: number } {
    if (!filters?.queryName && !filters?.keyPrefix) {
      const invalidated = this.cache.size;
      this.cache.clear();
      this.updateRuntimeGauges();
      return { invalidated };
    }

    let invalidated = 0;
    for (const [key, entry] of this.cache.entries()) {
      const matchesQuery = !filters.queryName || entry.queryName.startsWith(filters.queryName);
      const matchesKeyPrefix = !filters.keyPrefix || key.startsWith(filters.keyPrefix);

      if (matchesQuery && matchesKeyPrefix) {
        this.cache.delete(key);
        invalidated += 1;
      }
    }

    this.updateRuntimeGauges();
    return { invalidated };
  }

  getReport(poolState?: {
    configuredRpcUrls: string[];
    roundRobinCursor: number;
  }): QueryOptimizationReport {
    const perQuery = Array.from(this.stats.entries()).map(([query, stat]) => {
      const averageDurationMs = stat.totalRequests > 0
        ? stat.totalDurationMs / stat.totalRequests
        : 0;
      const cacheHitRatioValue = stat.totalRequests > 0
        ? stat.totalCacheHits / stat.totalRequests
        : 0;
      const estimatedLoadReductionPercent = stat.totalRequests > 0
        ? ((stat.totalRequests - stat.totalBackendRequestCalls) / stat.totalRequests) * 100
        : 0;
      const effectiveTtlMs = this.getEffectiveTtlMs(query, stat.configuredBaseTtlMs);

      return {
        query,
        requests: stat.totalRequests,
        averageDurationMs,
        p95DurationMs: percentile(stat.requestSamplesMs, 95),
        p99DurationMs: percentile(stat.requestSamplesMs, 99),
        cacheHitRatio: cacheHitRatioValue,
        estimatedLoadReductionPercent,
        backendCalls: stat.totalBackendCalls,
        backendRequestCalls: stat.totalBackendRequestCalls,
        backgroundRefreshes: stat.totalBackgroundRefreshes,
        slowCount: stat.slowQueryCount,
        lastDurationMs: stat.lastDurationMs,
        lastBackendDurationMs: stat.lastBackendDurationMs,
        effectiveTtlMs,
      };
    });

    const totals = Array.from(this.stats.values()).reduce(
      (acc, stat) => {
        acc.requests += stat.totalRequests;
        acc.durationMs += stat.totalDurationMs;
        acc.hits += stat.totalCacheHits;
        acc.misses += stat.totalCacheMisses;
        acc.stale += stat.totalCacheStale;
        acc.evictions += stat.totalEvictions;
        acc.errors += stat.totalErrors;
        acc.backendCalls += stat.totalBackendCalls;
        acc.backendRequestCalls += stat.totalBackendRequestCalls;
        acc.backgroundRefreshes += stat.totalBackgroundRefreshes;
        acc.slowQueries += stat.slowQueryCount;
        return acc;
      },
      {
        requests: 0,
        durationMs: 0,
        hits: 0,
        misses: 0,
        stale: 0,
        evictions: 0,
        errors: 0,
        backendCalls: 0,
        backendRequestCalls: 0,
        backgroundRefreshes: 0,
        slowQueries: 0,
      }
    );

    const averageQueryTimeMs = totals.requests > 0 ? totals.durationMs / totals.requests : 0;
    const cacheHitRatioValue = totals.requests > 0 ? totals.hits / totals.requests : 0;
    const estimatedLoadReductionPercent = totals.requests > 0
      ? ((totals.requests - totals.backendRequestCalls) / totals.requests) * 100
      : 0;
    const recommendations = this.buildRecommendations({
      averageQueryTimeMs,
      cacheHitRatio: cacheHitRatioValue,
      estimatedLoadReductionPercent,
      totals,
      perQuery,
    });

    return {
      timestamp: new Date().toISOString(),
      targets: {
        avgQueryTimeMs: this.options.targetAvgMs,
        loadReductionPercent: this.options.targetLoadReductionPercent,
      },
      summary: {
        averageQueryTimeMs,
        meetsAverageQueryTarget: averageQueryTimeMs <= this.options.targetAvgMs,
        estimatedLoadReductionPercent,
        meetsLoadReductionTarget:
          estimatedLoadReductionPercent >= this.options.targetLoadReductionPercent,
        cacheHitRatio: cacheHitRatioValue,
        cacheEntries: this.cache.size,
        totalRequests: totals.requests,
        totalBackendCalls: totals.backendCalls,
        totalBackendRequestCalls: totals.backendRequestCalls,
        totalBackgroundRefreshes: totals.backgroundRefreshes,
        slowQueries: totals.slowQueries,
      },
      cache: {
        maxEntries: this.options.maxEntries,
        activeEntries: this.cache.size,
        hitRatio: cacheHitRatioValue,
        totals: {
          hits: totals.hits,
          misses: totals.misses,
          stale: totals.stale,
          evictions: totals.evictions,
        },
      },
      pool: {
        size: this.options.poolSize,
        available: Math.max(this.options.poolSize - this.activeBackendFetches, 0),
        inFlightBackendQueries: this.activeBackendFetches,
        configuredRpcUrls: poolState?.configuredRpcUrls ?? [],
        roundRobinCursor: poolState?.roundRobinCursor ?? 0,
      },
      slowQueries: perQuery
        .filter((entry) => entry.slowCount > 0)
        .sort((a, b) => b.slowCount - a.slowCount)
        .slice(0, 5)
        .map((entry) => ({
          query: entry.query,
          slowCount: entry.slowCount,
          lastDurationMs: entry.lastDurationMs,
          averageDurationMs: entry.averageDurationMs,
          p95DurationMs: entry.p95DurationMs,
        })),
      topQueries: perQuery
        .sort((a, b) => b.requests - a.requests || b.averageDurationMs - a.averageDurationMs)
        .slice(0, 8),
      optimization: {
        adaptiveCachingEnabled: true,
        knownSingletonPrewarmQueries: ["get_analytics"],
        recommendations,
        indexOptimizationSpecs: this.indexSpecs,
      },
    };
  }

  private async fetchAndCache<T>(
    options: ExecuteQueryOptions<T>,
    fetcher: () => Promise<T>,
    reason: BackendFetchReason
  ): Promise<T> {
    const backendStartedAt = Date.now();
    this.activeBackendFetches += 1;
    queryInFlightBackend.inc();
    this.updateRuntimeGauges();

    try {
      const value = await fetcher();
      const backendDurationMs = Date.now() - backendStartedAt;
      this.recordBackendCall(options.queryName, backendDurationMs, reason);
      this.writeCacheEntry(options, value);
      return value;
    } catch (error) {
      const backendDurationMs = Date.now() - backendStartedAt;
      this.recordBackendCall(options.queryName, backendDurationMs, reason);
      throw error;
    } finally {
      this.activeBackendFetches = Math.max(this.activeBackendFetches - 1, 0);
      queryInFlightBackend.dec();
      this.updateRuntimeGauges();
    }
  }

  private writeCacheEntry<T>(options: ExecuteQueryOptions<T>, value: T): void {
    const shouldCache = options.shouldCacheValue
      ? options.shouldCacheValue(value)
      : value !== undefined;

    if (!shouldCache) {
      return;
    }

    if (value === null && !options.cacheNull) {
      return;
    }

    const baseTtlMs = options.cacheTtlMs ?? this.options.defaultTtlMs;
    const ttlMs = value === null
      ? Math.min(options.nullCacheTtlMs ?? SHORT_NULL_CACHE_TTL_MS, baseTtlMs)
      : this.getEffectiveTtlMs(options.queryName, baseTtlMs);
    const now = Date.now();

    this.cache.delete(options.cacheKey);
    this.cache.set(options.cacheKey, {
      queryName: options.queryName,
      key: options.cacheKey,
      value,
      createdAt: now,
      lastAccessedAt: now,
      expiresAt: now + ttlMs,
      ttlMs,
      baseTtlMs,
    });

    this.evictIfNeeded();
    this.updateRuntimeGauges();
  }

  private evictIfNeeded(): void {
    while (this.cache.size > this.options.maxEntries) {
      const oldest = this.cache.entries().next().value as [string, CacheEntry<unknown>] | undefined;
      if (!oldest) {
        return;
      }
      this.cache.delete(oldest[0]);
      this.recordCacheEvent(oldest[1].queryName, "eviction");
    }
  }

  private touchCacheEntry<T>(key: string, entry: CacheEntry<T>): void {
    entry.lastAccessedAt = Date.now();
    this.cache.delete(key);
    this.cache.set(key, entry as CacheEntry<unknown>);
    this.updateRuntimeGauges();
  }

  private maybeSchedulePrewarm<T>(
    options: ExecuteQueryOptions<T>,
    fetcher: () => Promise<T>,
    entry: CacheEntry<T>
  ): void {
    if (!options.singleton || !options.canPrewarm || !this.isHotStable(options.queryName)) {
      return;
    }

    const timeRemainingMs = entry.expiresAt - Date.now();
    const prewarmThresholdMs = Math.max(Math.floor(entry.ttlMs * 0.15), 1_000);
    if (timeRemainingMs > prewarmThresholdMs || this.prewarmInFlight.has(options.cacheKey)) {
      return;
    }

    const prewarmPromise = this.fetchAndCache(options, fetcher, "prewarm")
      .catch((error) => {
        logger.warn("Query prewarm failed", {
          query: options.queryName,
          key: options.cacheKey,
          error,
        });
      })
      .finally(() => {
        this.prewarmInFlight.delete(options.cacheKey);
      });

    this.prewarmInFlight.set(options.cacheKey, prewarmPromise as Promise<unknown>);
  }

  private recordCacheEvent(queryName: string, event: CacheEvent): void {
    const stat = this.getStatBucket(queryName);

    if (event === "hit") stat.totalCacheHits += 1;
    if (event === "miss") stat.totalCacheMisses += 1;
    if (event === "stale") stat.totalCacheStale += 1;
    if (event === "eviction") stat.totalEvictions += 1;

    queryCacheEvents.inc({ query: queryName, event });
    this.updateRuntimeGauges();
  }

  private recordBackendCall(
    queryName: string,
    backendDurationMs: number,
    reason: BackendFetchReason
  ): void {
    const stat = this.getStatBucket(queryName);
    stat.totalBackendCalls += 1;
    stat.lastBackendDurationMs = backendDurationMs;
    stat.lastObservedAt = new Date().toISOString();
    pushSample(stat.backendSamplesMs, backendDurationMs);

    if (reason === "request") {
      stat.totalBackendRequestCalls += 1;
    }
    if (reason === "prewarm") {
      stat.totalBackgroundRefreshes += 1;
    }

    this.updateRuntimeGauges();
  }

  private observeRequest(
    queryName: string,
    durationMs: number,
    source: QuerySource,
    status: QueryStatus
  ): void {
    const stat = this.getStatBucket(queryName);
    stat.totalRequests += 1;
    stat.totalDurationMs += durationMs;
    stat.lastDurationMs = durationMs;
    stat.lastObservedAt = new Date().toISOString();
    if (status === "error") {
      stat.totalErrors += 1;
    }
    if (durationMs > this.options.slowThresholdMs) {
      stat.slowQueryCount += 1;
    }

    pushSample(stat.requestSamplesMs, durationMs);
    queryDuration.observe(
      { query: queryName, source, status },
      durationMs / 1_000
    );
    this.updateRuntimeGauges();
  }

  private getStatBucket(queryName: string): QueryStatBucket {
    const existing = this.stats.get(queryName);
    if (existing) {
      return existing;
    }

    const created: QueryStatBucket = {
      totalRequests: 0,
      totalDurationMs: 0,
      totalCacheHits: 0,
      totalCacheMisses: 0,
      totalCacheStale: 0,
      totalEvictions: 0,
      totalErrors: 0,
      totalBackendCalls: 0,
      totalBackendRequestCalls: 0,
      totalBackgroundRefreshes: 0,
      slowQueryCount: 0,
      lastDurationMs: 0,
      lastBackendDurationMs: 0,
      lastObservedAt: null,
      configuredBaseTtlMs: this.options.defaultTtlMs,
      requestSamplesMs: [],
      backendSamplesMs: [],
    };

    this.stats.set(queryName, created);
    return created;
  }

  private updateRuntimeGauges(): void {
    const totals = Array.from(this.stats.values()).reduce(
      (acc, stat) => {
        acc.requests += stat.totalRequests;
        acc.durationMs += stat.totalDurationMs;
        acc.cacheHits += stat.totalCacheHits;
        acc.backendRequestCalls += stat.totalBackendRequestCalls;
        return acc;
      },
      { requests: 0, durationMs: 0, cacheHits: 0, backendRequestCalls: 0 }
    );

    const averageQueryTimeMs = totals.requests > 0 ? totals.durationMs / totals.requests : 0;
    const cacheHitRatioValue = totals.requests > 0 ? totals.cacheHits / totals.requests : 0;
    const loadReductionPercent = totals.requests > 0
      ? ((totals.requests - totals.backendRequestCalls) / totals.requests) * 100
      : 0;

    queryCacheEntries.set(this.cache.size);
    queryAverageTimeMs.set(averageQueryTimeMs);
    queryCacheHitRatio.set(cacheHitRatioValue);
    queryEstimatedLoadReduction.set(loadReductionPercent);
    queryPoolActiveSize.set(this.options.poolSize);
    queryPoolAvailableSize.set(Math.max(this.options.poolSize - this.activeBackendFetches, 0));
  }

  private getEffectiveTtlMs(queryName: string, baseTtlMs: number): number {
    const stat = this.stats.get(queryName);
    if (!stat || !this.isHotStable(queryName)) {
      return baseTtlMs;
    }

    if (queryName === "get_analytics") {
      return Math.min(baseTtlMs * 2, MAX_EFFECTIVE_TTL_MS);
    }

    return Math.min(Math.round(baseTtlMs * 1.5), MAX_EFFECTIVE_TTL_MS);
  }

  private isHotStable(queryName: string): boolean {
    const stat = this.stats.get(queryName);
    if (!stat || stat.totalRequests < 6 || stat.totalBackendRequestCalls < 2) {
      return false;
    }

    const averageDurationMs = stat.totalDurationMs / stat.totalRequests;
    const hitRatio = stat.totalCacheHits / stat.totalRequests;
    const spread = stddev(stat.requestSamplesMs, averageDurationMs);
    const slowRatio = stat.slowQueryCount / stat.totalRequests;

    return hitRatio >= 0.5 && spread <= Math.max(averageDurationMs * 0.2, 15) && slowRatio <= 0.1;
  }

  private buildRecommendations(input: {
    averageQueryTimeMs: number;
    cacheHitRatio: number;
    estimatedLoadReductionPercent: number;
    totals: {
      requests: number;
      backendCalls: number;
      backendRequestCalls: number;
      backgroundRefreshes: number;
      slowQueries: number;
      hits: number;
      misses: number;
      stale: number;
      evictions: number;
      errors: number;
      durationMs: number;
    };
    perQuery: Array<{
      query: string;
      requests: number;
      averageDurationMs: number;
      p95DurationMs: number;
      p99DurationMs: number;
      cacheHitRatio: number;
      estimatedLoadReductionPercent: number;
      backendCalls: number;
      backendRequestCalls: number;
      backgroundRefreshes: number;
      slowCount: number;
      lastDurationMs: number;
      lastBackendDurationMs: number;
      effectiveTtlMs: number;
    }>;
  }): string[] {
    const recommendations: string[] = [];

    if (input.totals.requests < 10) {
      recommendations.push("Warm up read traffic before treating cache-hit and load-reduction ratios as steady-state signals.");
    }
    if (input.averageQueryTimeMs > this.options.targetAvgMs) {
      recommendations.push("Average API query latency is above target; inspect backend RPC health, cache miss patterns, and slow-query hotspots.");
    }
    if (input.estimatedLoadReductionPercent < this.options.targetLoadReductionPercent) {
      recommendations.push("Estimated backend load reduction is below target; increase cache warm-up coverage for hot certificate and student lookup paths.");
    }
    if (input.totals.evictions > Math.max(5, Math.floor(this.options.maxEntries * 0.1))) {
      recommendations.push("Cache eviction volume is elevated; consider increasing QUERY_CACHE_MAX_ENTRIES or narrowing invalidation scope.");
    }

    for (const query of input.perQuery) {
      if (query.p95DurationMs > this.options.slowThresholdMs) {
        recommendations.push(`Query ${query.query} has p95 latency above slow-threshold; prioritize RPC/indexer path analysis for that access pattern.`);
      }
      if (query.cacheHitRatio < 0.4 && query.requests >= 5) {
        recommendations.push(`Query ${query.query} has a low hit ratio; verify TTL sizing and key reuse for repeated reads.`);
      }
    }

    if (recommendations.length === 0) {
      recommendations.push("Query optimization targets are currently being met; continue monitoring adaptive TTL behaviour and pool headroom.");
    }

    return recommendations;
  }
}

function pushSample(samples: number[], value: number): void {
  samples.push(value);
  if (samples.length > MAX_SAMPLE_COUNT) {
    samples.shift();
  }
}

function percentile(samples: number[], percentileRank: number): number {
  if (samples.length === 0) {
    return 0;
  }

  const sorted = [...samples].sort((a, b) => a - b);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((percentileRank / 100) * sorted.length) - 1)
  );
  return sorted[index];
}

function stddev(samples: number[], average: number): number {
  if (samples.length <= 1) {
    return 0;
  }

  const variance = samples.reduce((sum, sample) => {
    const diff = sample - average;
    return sum + diff * diff;
  }, 0) / samples.length;

  return Math.sqrt(variance);
}
