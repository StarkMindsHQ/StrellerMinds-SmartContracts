/**
 * User-specific rate limiter with:
 *  - Per-user (JWT sub) limits keyed by tier
 *  - Token-bucket burst handling via rate-limiter-flexible
 *  - Graduated limits: free < pro < enterprise < internal
 *  - Usage analytics via Prometheus
 *  - Clear limit communication via response headers
 */
import { Request, Response, NextFunction } from "express";
import { RateLimiterMemory, RateLimiterRes } from "rate-limiter-flexible";
import { config } from "../config";
import { sendError } from "../utils/response";
import {
  userRateLimitHits,
  userRateLimitConsumed,
  burstUsage,
} from "../metrics";
import { logger } from "../logger";
import type { RateLimitTier } from "../types";

// ── In-memory usage store (swap for Redis in production) ─────────────────────

interface UsageRecord {
  requests: number;
  windowStart: number;
  burstConsumed: number;
}

const usageStore = new Map<string, UsageRecord>();

function getUsage(userId: string): UsageRecord {
  const now = Date.now();
  const existing = usageStore.get(userId);
  // Reset window every minute
  if (!existing || now - existing.windowStart > 60_000) {
    const fresh: UsageRecord = { requests: 0, windowStart: now, burstConsumed: 0 };
    usageStore.set(userId, fresh);
    return fresh;
  }
  return existing;
}

// Prune stale entries every 5 minutes to prevent unbounded growth
setInterval(() => {
  const cutoff = Date.now() - 120_000;
  for (const [key, rec] of usageStore.entries()) {
    if (rec.windowStart < cutoff) usageStore.delete(key);
  }
}, 300_000).unref();

// ── Token-bucket limiters per tier ────────────────────────────────────────────
// Each limiter uses a sliding window of 60 seconds.
// Points = requests per minute. Burst is handled by allowing extra points
// that refill faster (every 1s instead of 60s).

function makeLimiter(rpm: number) {
  return new RateLimiterMemory({
    points: rpm,
    duration: 60, // per 60 seconds
    blockDuration: 0, // don't hard-block, just reject
  });
}

function makeBurstLimiter(burst: number) {
  return new RateLimiterMemory({
    points: burst,
    duration: 10, // burst window: 10 seconds
    blockDuration: 0,
  });
}

const limiters: Record<RateLimitTier, { main: RateLimiterMemory; burst: RateLimiterMemory }> = {
  free:       { main: makeLimiter(config.rateLimit.tiers.free.rpm),       burst: makeBurstLimiter(config.rateLimit.tiers.free.burst) },
  pro:        { main: makeLimiter(config.rateLimit.tiers.pro.rpm),        burst: makeBurstLimiter(config.rateLimit.tiers.pro.burst) },
  enterprise: { main: makeLimiter(config.rateLimit.tiers.enterprise.rpm), burst: makeBurstLimiter(config.rateLimit.tiers.enterprise.burst) },
  internal:   { main: makeLimiter(config.rateLimit.tiers.internal.rpm),   burst: makeBurstLimiter(config.rateLimit.tiers.internal.burst) },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

function setRateLimitHeaders(
  res: Response,
  tier: RateLimitTier,
  mainResult: RateLimiterRes,
  resetMs: number
): void {
  const tierCfg = config.rateLimit.tiers[tier];
  res.setHeader("X-RateLimit-Tier", tier);
  res.setHeader("X-RateLimit-Limit", tierCfg.rpm);
  res.setHeader("X-RateLimit-Remaining", Math.max(0, mainResult.remainingPoints ?? 0));
  res.setHeader("X-RateLimit-Reset", Math.ceil((Date.now() + resetMs) / 1000));
  res.setHeader("X-RateLimit-Burst-Limit", tierCfg.burst);
}

// ── Middleware factory ────────────────────────────────────────────────────────

/**
 * Creates a per-user rate limiter middleware.
 * @param endpoint  Label used in metrics (e.g. "verify", "general")
 * @param costPoints  How many points this endpoint costs (default 1)
 */
export function userRateLimit(endpoint: string, costPoints = 1) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    // Unauthenticated requests fall back to IP-based limiting (handled by
    // the existing generalLimiter / verifyLimiter). Skip user limiting.
    if (!req.auth) {
      next();
      return;
    }

    const userId = req.auth.sub;
    const tier: RateLimitTier = req.auth.tier ?? "free";
    const { main, burst } = limiters[tier];
    const tierCfg = config.rateLimit.tiers[tier];

    try {
      // Consume from main window limiter
      const mainResult = await main.consume(userId, costPoints);

      // Track usage analytics
      const usage = getUsage(userId);
      usage.requests += costPoints;

      const consumedRatio = (tierCfg.rpm - (mainResult.remainingPoints ?? 0)) / tierCfg.rpm;
      userRateLimitConsumed.observe({ tier }, Math.min(1, consumedRatio));

      setRateLimitHeaders(res, tier, mainResult, mainResult.msBeforeNext);

      // Warn at 80% consumption
      if (consumedRatio >= 0.8) {
        logger.warn("User approaching rate limit", { userId, tier, consumedRatio });
      }

      next();
    } catch (err) {
      if (err instanceof RateLimiterRes) {
        // Main window exhausted — try burst allowance
        try {
          const burstResult = await burst.consume(userId, costPoints);
          const usage = getUsage(userId);
          usage.requests += costPoints;
          usage.burstConsumed += costPoints;

          burstUsage.inc({ tier });
          setRateLimitHeaders(res, tier, burstResult, burstResult.msBeforeNext);

          logger.info("Request served from burst allowance", { userId, tier });
          next();
          return;
        } catch (burstErr) {
          if (burstErr instanceof RateLimiterRes) {
            // Both main and burst exhausted
            userRateLimitHits.inc({ tier, endpoint });

            const retryAfter = Math.ceil(burstErr.msBeforeNext / 1000);
            const tierCfgLocal = config.rateLimit.tiers[tier];

            res.setHeader("X-RateLimit-Tier", tier);
            res.setHeader("X-RateLimit-Limit", tierCfgLocal.rpm);
            res.setHeader("X-RateLimit-Remaining", "0");
            res.setHeader("X-RateLimit-Reset", Math.ceil((Date.now() + burstErr.msBeforeNext) / 1000));
            res.setHeader("Retry-After", retryAfter);

            logger.warn("User rate limit exceeded", { userId, tier, endpoint, retryAfter });

            sendError(
              res,
              429,
              "USER_RATE_LIMIT_EXCEEDED",
              `Rate limit exceeded for tier '${tier}'. Limit: ${tierCfgLocal.rpm} req/min, burst: ${tierCfgLocal.burst} req/10s.`,
              {
                tier,
                limit: tierCfgLocal.rpm,
                burstLimit: tierCfgLocal.burst,
                retryAfter,
                upgradeUrl: "https://strellerminds.com/pricing",
              },
              req.requestId
            );
            return;
          }
        }
      }
      // Unexpected error — fail open to avoid blocking legitimate traffic
      logger.error("Rate limiter error", { err });
      next();
    }
  };
}

// ── Usage analytics endpoint helper ──────────────────────────────────────────

export function getUserRateLimitStatus(userId: string, tier: RateLimitTier) {
  const tierCfg = config.rateLimit.tiers[tier];
  const usage = getUsage(userId);
  const now = Date.now();
  const windowElapsed = now - usage.windowStart;
  const windowRemaining = Math.max(0, 60_000 - windowElapsed);

  return {
    userId,
    tier,
    consumed: usage.requests,
    remaining: Math.max(0, tierCfg.rpm - usage.requests),
    limit: tierCfg.rpm,
    burstLimit: tierCfg.burst,
    burstConsumed: usage.burstConsumed,
    resetAt: Math.ceil((usage.windowStart + 60_000) / 1000),
    throttled: usage.requests >= tierCfg.rpm,
    windowRemainingMs: windowRemaining,
  };
}
