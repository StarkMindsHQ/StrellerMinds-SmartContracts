import client from "prom-client";

// Enable default Node.js metrics (CPU, memory, event loop lag, etc.)
client.collectDefaultMetrics({ prefix: "cert_api_" });

// ── Custom metrics ──────────────────────────────────────────────────────────

export const httpRequestDuration = new client.Histogram({
  name: "cert_api_http_request_duration_seconds",
  help: "Duration of HTTP requests in seconds",
  labelNames: ["method", "route", "status_code"],
  buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5],
});

export const httpRequestTotal = new client.Counter({
  name: "cert_api_http_requests_total",
  help: "Total number of HTTP requests",
  labelNames: ["method", "route", "status_code"],
});

export const verificationTotal = new client.Counter({
  name: "cert_api_verifications_total",
  help: "Total certificate verification attempts",
  labelNames: ["result"], // 'valid' | 'invalid' | 'not_found' | 'error'
});

export const contractCallDuration = new client.Histogram({
  name: "cert_api_contract_call_duration_seconds",
  help: "Duration of Soroban contract calls",
  labelNames: ["method", "success"],
  buckets: [0.1, 0.25, 0.5, 1, 2, 5, 10],
});

export const rateLimitHits = new client.Counter({
  name: "cert_api_rate_limit_hits_total",
  help: "Total number of rate-limited requests",
  labelNames: ["endpoint"],
});

// ── User rate limit metrics ──────────────────────────────────────────────────

export const userRateLimitHits = new client.Counter({
  name: "cert_api_user_rate_limit_hits_total",
  help: "Rate limit hits broken down by user tier",
  labelNames: ["tier", "endpoint"],
});

export const userRateLimitConsumed = new client.Histogram({
  name: "cert_api_user_rate_limit_consumed_ratio",
  help: "Ratio of rate limit consumed (0–1) per request",
  labelNames: ["tier"],
  buckets: [0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 1.0],
});

export const burstUsage = new client.Counter({
  name: "cert_api_burst_usage_total",
  help: "Number of requests that consumed burst allowance",
  labelNames: ["tier"],
});

// ── CDN metrics ──────────────────────────────────────────────────────────────

export const cdnCacheHits = new client.Counter({
  name: "cert_api_cdn_cache_hits_total",
  help: "CDN cache hits (X-Cache: Hit)",
  labelNames: ["route"],
});

export const cdnCacheMisses = new client.Counter({
  name: "cert_api_cdn_cache_misses_total",
  help: "CDN cache misses (X-Cache: Miss)",
  labelNames: ["route"],
});

export const cdnInvalidations = new client.Counter({
  name: "cert_api_cdn_invalidations_total",
  help: "Cache invalidation requests triggered",
  labelNames: ["pattern"],
});

export const assetServeTime = new client.Histogram({
  name: "cert_api_asset_serve_duration_seconds",
  help: "Time to serve static assets",
  labelNames: ["asset_type"],
  buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.2],
});

export const queryDuration = new client.Histogram({
  name: "cert_api_query_duration_seconds",
  help: "Duration of API query operations by source",
  labelNames: ["query", "source", "status"],
  buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 1, 2.5],
});

export const queryCacheEvents = new client.Counter({
  name: "cert_api_query_cache_events_total",
  help: "Query cache lifecycle events by query type",
  labelNames: ["query", "event"],
});

export const queryInFlightBackend = new client.Gauge({
  name: "cert_api_query_in_flight_backend",
  help: "Number of in-flight backend Soroban/indexer query operations",
});

export const queryPoolActiveSize = new client.Gauge({
  name: "cert_api_query_pool_active_size",
  help: "Configured active size of the query backend pool",
});

export const queryPoolAvailableSize = new client.Gauge({
  name: "cert_api_query_pool_available_size",
  help: "Approximate available capacity in the query backend pool",
});

export const queryCacheEntries = new client.Gauge({
  name: "cert_api_query_cache_entries",
  help: "Current number of cached query results",
});

export const queryEstimatedLoadReduction = new client.Gauge({
  name: "cert_api_query_estimated_load_reduction_percent",
  help: "Estimated backend load reduction percentage from caching and request coalescing",
});

export const queryAverageTimeMs = new client.Gauge({
  name: "cert_api_query_average_time_ms",
  help: "Average query time in milliseconds across observed requests",
});

export const queryCacheHitRatio = new client.Gauge({
  name: "cert_api_query_cache_hit_ratio",
  help: "Query cache hit ratio as a 0-1 value",
});

export const registry = client.register;
