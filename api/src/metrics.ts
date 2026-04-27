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

// ── Cache metrics ─────────────────────────────────────────────────────────────

export const cacheHits = new client.Counter({
  name: "cert_api_cache_hits_total",
  help: "In-memory cache hits",
  labelNames: ["cache"],
});

export const cacheMisses = new client.Counter({
  name: "cert_api_cache_misses_total",
  help: "In-memory cache misses",
  labelNames: ["cache"],
});

export const cacheSize = new client.Gauge({
  name: "cert_api_cache_size",
  help: "Current number of entries in each in-memory cache",
  labelNames: ["cache"],
});

export const registry = client.register;
