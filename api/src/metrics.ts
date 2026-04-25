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

export const registry = client.register;
