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
  },

  cors: {
    origins: (process.env.CORS_ORIGINS ?? "http://localhost:3000").split(","),
  },
} as const;
