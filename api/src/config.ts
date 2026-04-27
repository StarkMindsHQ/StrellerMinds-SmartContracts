import dotenv from "dotenv";
dotenv.config();

function required(key: string): string {
  const val = process.env[key];
  if (!val) throw new Error(`Missing required env var: ${key}`);
  return val;
}

export const config = {
  port: parseInt(process.env.PORT ?? "3000", 10),
  nodeEnv: process.env.NODE_ENV ?? "development",

  stellar: {
    rpcUrl:
      process.env.STELLAR_RPC_URL ?? "https://soroban-testnet.stellar.org",
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
    windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS ?? "60000", 10),
    maxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS ?? "60", 10),
    verifyMax: parseInt(process.env.RATE_LIMIT_VERIFY_MAX ?? "100", 10),
    // Per-user tier limits (requests per minute)
    tiers: {
      free:       { rpm: parseInt(process.env.RATE_LIMIT_FREE_RPM       ?? "30",   10), burst: parseInt(process.env.RATE_LIMIT_FREE_BURST       ?? "10",  10) },
      pro:        { rpm: parseInt(process.env.RATE_LIMIT_PRO_RPM        ?? "120",  10), burst: parseInt(process.env.RATE_LIMIT_PRO_BURST        ?? "30",  10) },
      enterprise: { rpm: parseInt(process.env.RATE_LIMIT_ENT_RPM        ?? "600",  10), burst: parseInt(process.env.RATE_LIMIT_ENT_BURST        ?? "100", 10) },
      internal:   { rpm: parseInt(process.env.RATE_LIMIT_INTERNAL_RPM   ?? "6000", 10), burst: parseInt(process.env.RATE_LIMIT_INTERNAL_BURST   ?? "500", 10) },
    },
  },

  cdn: {
    // Public CDN origin (e.g. CloudFront distribution URL)
    origin: process.env.CDN_ORIGIN ?? "",
    // Cache-Control max-age for static assets (seconds)
    maxAge: parseInt(process.env.CDN_MAX_AGE ?? "31536000", 10),       // 1 year
    sMaxAge: parseInt(process.env.CDN_S_MAX_AGE ?? "86400", 10),       // 1 day CDN TTL
    staleWhileRevalidate: parseInt(process.env.CDN_SWR ?? "3600", 10), // 1 hour
    // Shared secret for cache invalidation webhook
    invalidationSecret: process.env.CDN_INVALIDATION_SECRET ?? "change-me",
    // CloudFront distribution ID (for AWS SDK invalidation calls)
    cloudfrontDistributionId: process.env.CLOUDFRONT_DISTRIBUTION_ID ?? "",
  },

  cors: {
    origins: (process.env.CORS_ORIGINS ?? "http://localhost:3000").split(","),
  },

  redis: {
    url: process.env.REDIS_URL ?? "redis://localhost:6379",
    // TTLs in seconds
    ttl: {
      certificate: parseInt(process.env.REDIS_TTL_CERTIFICATE ?? "300", 10),   // 5 min
      studentCerts: parseInt(process.env.REDIS_TTL_STUDENT_CERTS ?? "120", 10), // 2 min
      analytics:    parseInt(process.env.REDIS_TTL_ANALYTICS ?? "60", 10),      // 1 min
      revocation:   parseInt(process.env.REDIS_TTL_REVOCATION ?? "300", 10),    // 5 min
    },
  },
} as const;
