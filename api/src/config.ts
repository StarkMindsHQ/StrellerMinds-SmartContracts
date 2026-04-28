import dotenv from "dotenv";
dotenv.config();

function required(key: string): string {
  const val = process.env[key];
  if (!val) throw new Error(`Missing required env var: ${key}`);
  return val;
}

function integerEnv(key: string, defaultValue: number, minimum = 0): number {
  const raw = process.env[key];
  const parsed = raw ? parseInt(raw, 10) : defaultValue;
  if (Number.isNaN(parsed)) {
    return Math.max(defaultValue, minimum);
  }
  return Math.max(parsed, minimum);
}

const defaultRpcUrl =
  process.env.STELLAR_RPC_URL ?? "https://soroban-testnet.stellar.org";
const queryPoolSize = integerEnv("QUERY_POOL_SIZE", 4, 1);
const queryRpcUrls = (process.env.STELLAR_RPC_URLS ?? defaultRpcUrl)
  .split(",")
  .map((url) => url.trim())
  .filter(Boolean);

export const config = {
  port: integerEnv("PORT", 3000, 1),
  nodeEnv: process.env.NODE_ENV ?? "development",

  stellar: {
    rpcUrl: defaultRpcUrl,
    networkPassphrase:
      process.env.STELLAR_NETWORK_PASSPHRASE ??
      "Test SDF Network ; September 2015",
    contractId: process.env.CERTIFICATE_CONTRACT_ID ?? "",
  },

  jwt: {
    secret: process.env.JWT_SECRET ?? "dev-secret-change-in-production",
    expiresIn: process.env.JWT_EXPIRES_IN ?? "1h",
  },

  rateLimit: {
    windowMs: integerEnv("RATE_LIMIT_WINDOW_MS", 60000, 1),
    maxRequests: integerEnv("RATE_LIMIT_MAX_REQUESTS", 60, 1),
    verifyMax: integerEnv("RATE_LIMIT_VERIFY_MAX", 100, 1),
    // Per-user tier limits (requests per minute)
    tiers: {
      free:       { rpm: integerEnv("RATE_LIMIT_FREE_RPM", 30, 1), burst: integerEnv("RATE_LIMIT_FREE_BURST", 10, 1) },
      pro:        { rpm: integerEnv("RATE_LIMIT_PRO_RPM", 120, 1), burst: integerEnv("RATE_LIMIT_PRO_BURST", 30, 1) },
      enterprise: { rpm: integerEnv("RATE_LIMIT_ENT_RPM", 600, 1), burst: integerEnv("RATE_LIMIT_ENT_BURST", 100, 1) },
      internal:   { rpm: integerEnv("RATE_LIMIT_INTERNAL_RPM", 6000, 1), burst: integerEnv("RATE_LIMIT_INTERNAL_BURST", 500, 1) },
    },
  },

  cdn: {
    // Public CDN origin (e.g. CloudFront distribution URL)
    origin: process.env.CDN_ORIGIN ?? "",
    // Cache-Control max-age for static assets (seconds)
    maxAge: integerEnv("CDN_MAX_AGE", 31536000, 0),       // 1 year
    sMaxAge: integerEnv("CDN_S_MAX_AGE", 86400, 0),       // 1 day CDN TTL
    staleWhileRevalidate: integerEnv("CDN_SWR", 3600, 0), // 1 hour
    // Shared secret for cache invalidation webhook
    invalidationSecret: process.env.CDN_INVALIDATION_SECRET ?? "change-me",
    // CloudFront distribution ID (for AWS SDK invalidation calls)
    cloudfrontDistributionId: process.env.CLOUDFRONT_DISTRIBUTION_ID ?? "",
  },

  queryOptimization: {
    rpcUrls:
      queryRpcUrls.length >= queryPoolSize
        ? queryRpcUrls
        : Array.from({ length: queryPoolSize }, (_, index) => {
            return queryRpcUrls[index % queryRpcUrls.length] ?? defaultRpcUrl;
          }),
    poolSize: queryPoolSize,
    cacheMaxEntries: integerEnv("QUERY_CACHE_MAX_ENTRIES", 1000, 1),
    cacheDefaultTtlMs: integerEnv("QUERY_CACHE_DEFAULT_TTL_MS", 60000, 1),
    cacheAnalyticsTtlMs: integerEnv("QUERY_CACHE_ANALYTICS_TTL_MS", 15000, 1),
    cacheStudentCertsTtlMs: integerEnv("QUERY_CACHE_STUDENT_CERTS_TTL_MS", 30000, 1),
    cacheCertificateTtlMs: integerEnv("QUERY_CACHE_CERTIFICATE_TTL_MS", 60000, 1),
    cacheRevocationTtlMs: integerEnv("QUERY_CACHE_REVOCATION_TTL_MS", 300000, 1),
    slowThresholdMs: integerEnv("QUERY_SLOW_THRESHOLD_MS", 100, 1),
    targetAvgMs: integerEnv("QUERY_TARGET_AVG_MS", 100, 1),
    targetLoadReductionPercent: integerEnv(
      "QUERY_TARGET_LOAD_REDUCTION_PERCENT",
      40,
      0
    ),
  },

  cors: {
    origins: (process.env.CORS_ORIGINS ?? "http://localhost:3000").split(","),
  },

  slack: {
    // Default incoming webhook URL (required to enable Slack notifications)
    webhookUrl: process.env.SLACK_WEBHOOK_URL ?? "",
    // Optional per-channel webhook overrides
    alertsWebhookUrl: process.env.SLACK_ALERTS_WEBHOOK_URL ?? "",
    certificatesWebhookUrl: process.env.SLACK_CERTIFICATES_WEBHOOK_URL ?? "",
    // Default channel (e.g. "#notifications")
    defaultChannel: process.env.SLACK_DEFAULT_CHANNEL ?? "",
    // Channel routing overrides
    alertsChannel: process.env.SLACK_ALERTS_CHANNEL ?? "",
    certificatesChannel: process.env.SLACK_CERTIFICATES_CHANNEL ?? "",
    // Bot display name
    username: process.env.SLACK_USERNAME ?? "StrellerMinds",
    // Shared secret for the webhook management API
    signingSecret: process.env.SLACK_SIGNING_SECRET ?? "",
  },
} as const;
