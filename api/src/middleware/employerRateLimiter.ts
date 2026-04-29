// @ts-nocheck
/**
 * Employer Rate Limiting Middleware
 * 
 * Provides tiered rate limiting based on employer subscription levels
 * Implements different limits for different operation types
 */
import { RateLimiterMemory, RateLimiterRedis } from "rate-limiter-flexible";
import { Request, Response, NextFunction } from "express";
import { EmployerAuth, AuthenticatedRequest } from "./employerAuth";
import { config } from "../config";
import { logger } from "../logger";

// Rate limit configurations by subscription tier
const RATE_LIMITS = {
  standard: {
    // Basic tier limits
    verify: { points: 100, duration: 3600 }, // 100 verifications per hour
    batch: { points: 10, duration: 3600 }, // 10 batch verifications per hour
    read: { points: 1000, duration: 3600 }, // 1000 reads per hour
  },
  enhanced: {
    // Premium tier limits
    verify: { points: 500, duration: 3600 }, // 500 verifications per hour
    batch: { points: 50, duration: 3600 }, // 50 batch verifications per hour
    read: { points: 5000, duration: 3600 }, // 5000 reads per hour
  },
  unlimited: {
    // Enterprise tier (very high limits)
    verify: { points: 10000, duration: 3600 }, // 10k verifications per hour
    batch: { points: 1000, duration: 3600 }, // 1k batch verifications per hour
    read: { points: 100000, duration: 3600 }, // 100k reads per hour
  },
};

// In-memory rate limiters (for development)
const memoryLimiters: Record<string, RateLimiterMemory> = {};

// Redis rate limiters (for production)
let redisLimiters: Record<string, RateLimiterRedis> = {};

// Initialize rate limiters
function initializeLimiters(): void {
  Object.keys(RATE_LIMITS).forEach(tier => {
    Object.keys(RATE_LIMITS[tier as keyof typeof RATE_LIMITS]).forEach(operation => {
      const limit = RATE_LIMITS[tier as keyof typeof RATE_LIMITS][operation as keyof typeof RATE_LIMITS.standard];
      const key = `${tier}_${operation}`;
      
      if (config.redis.enabled) {
        redisLimiters[key] = new RateLimiterRedis({
          storeClient: require('ioredis'),
          keyPrefix: `employer_rl_${key}`,
          points: limit.points,
          duration: limit.duration,
          blockDuration: Math.floor(limit.duration / 4), // Block for 1/4 of duration
        });
      } else {
        memoryLimiters[key] = new RateLimiterMemory({
          keyPrefix: `employer_rl_${key}`,
          points: limit.points,
          duration: limit.duration,
          blockDuration: Math.floor(limit.duration / 4),
        });
      }
    });
  });
}

// Initialize on module load
initializeLimiters();

/**
 * Get rate limiter for specific tier and operation
 */
function getRateLimiter(tier: string, operation: string): RateLimiterMemory | RateLimiterRedis {
  const key = `${tier}_${operation}`;
  
  if (config.redis.enabled) {
    return redisLimiters[key];
  }
  
  return memoryLimiters[key];
}

/**
 * Create rate limit key for employer
 */
function createRateLimitKey(employer: EmployerAuth, operation: string): string {
  return `${employer.id}_${operation}`;
}

/**
 * Standard employer rate limiter for verification operations
 */
export function employerRateLimiter(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
  if (!req.employer) {
    res.status(401).json({
      success: false,
      data: null,
      error: { code: "UNAUTHORIZED", message: "Employer authentication required" },
      meta: {
        requestId: (req as any).requestId,
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
    return;
  }

  const limiter = getRateLimiter(req.employer.rateLimitTier, 'verify');
  const key = createRateLimitKey(req.employer, 'verify');

  limiter.consume(key)
    .then(() => {
      // Add rate limit headers
      const resHeaders = res.getHeaders();
      res.set({
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': Math.max(0, limiter.points - (limiter.currentRes?.consumedPoints || 0)).toString(),
        'X-RateLimit-Reset': new Date(Date.now() + limiter.duration * 1000).toISOString(),
      });
      
      next();
    })
    .catch((rejRes: any) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 1;
      res.set({
        'Retry-After': secs.toString(),
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': '0',
        'X-RateLimit-Reset': new Date(Date.now() + rejRes.msBeforeNext).toISOString(),
      });

      logger.warn("Employer rate limit exceeded", {
        employerId: req.employer?.id,
        operation: 'verify',
        key,
        secs,
        requestId: (req as any).requestId,
      });

      res.status(429).json({
        success: false,
        data: null,
        error: { 
          code: "RATE_LIMIT_EXCEEDED", 
          message: `Rate limit exceeded. Try again in ${secs} seconds.`,
          retryAfter: secs
        },
        meta: {
          requestId: (req as any).requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
    });
}

/**
 * Batch verification rate limiter with stricter limits
 */
export function batchVerificationLimiter(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
  if (!req.employer) {
    res.status(401).json({
      success: false,
      data: null,
      error: { code: "UNAUTHORIZED", message: "Employer authentication required" },
      meta: {
        requestId: (req as any).requestId,
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
    return;
  }

  const limiter = getRateLimiter(req.employer.rateLimitTier, 'batch');
  const key = createRateLimitKey(req.employer, 'batch');

  // Additional check for batch size
  const batchSize = req.body?.verifications?.length || 1;
  if (batchSize > 50) {
    res.status(400).json({
      success: false,
      data: null,
      error: { code: "BATCH_TOO_LARGE", message: "Maximum batch size is 50 certificates" },
      meta: {
        requestId: (req as any).requestId,
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
    return;
  }

  // Consume points based on batch size (larger batches cost more)
  const pointsToConsume = Math.ceil(batchSize / 10); // 1 point per 10 certificates

  limiter.consume(key, pointsToConsume)
    .then(() => {
      // Add rate limit headers
      res.set({
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': Math.max(0, limiter.points - (limiter.currentRes?.consumedPoints || 0)).toString(),
        'X-RateLimit-Reset': new Date(Date.now() + limiter.duration * 1000).toISOString(),
      });
      
      next();
    })
    .catch((rejRes: any) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 1;
      res.set({
        'Retry-After': secs.toString(),
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': '0',
        'X-RateLimit-Reset': new Date(Date.now() + rejRes.msBeforeNext).toISOString(),
      });

      logger.warn("Employer batch rate limit exceeded", {
        employerId: req.employer?.id,
        operation: 'batch',
        batchSize,
        pointsToConsume,
        key,
        secs,
        requestId: (req as any).requestId,
      });

      res.status(429).json({
        success: false,
        data: null,
        error: { 
          code: "RATE_LIMIT_EXCEEDED", 
          message: `Batch rate limit exceeded. Try again in ${secs} seconds.`,
          retryAfter: secs
        },
        meta: {
          requestId: (req as any).requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
    });
}

/**
 * Read operations rate limiter (for history, analytics, etc.)
 */
export function readRateLimiter(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
  if (!req.employer) {
    res.status(401).json({
      success: false,
      data: null,
      error: { code: "UNAUTHORIZED", message: "Employer authentication required" },
      meta: {
        requestId: (req as any).requestId,
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
    return;
  }

  const limiter = getRateLimiter(req.employer.rateLimitTier, 'read');
  const key = createRateLimitKey(req.employer, 'read');

  limiter.consume(key)
    .then(() => {
      // Add rate limit headers
      res.set({
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': Math.max(0, limiter.points - (limiter.currentRes?.consumedPoints || 0)).toString(),
        'X-RateLimit-Reset': new Date(Date.now() + limiter.duration * 1000).toISOString(),
      });
      
      next();
    })
    .catch((rejRes: any) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 1;
      res.set({
        'Retry-After': secs.toString(),
        'X-RateLimit-Limit': limiter.points.toString(),
        'X-RateLimit-Remaining': '0',
        'X-RateLimit-Reset': new Date(Date.now() + rejRes.msBeforeNext).toISOString(),
      });

      logger.warn("Employer read rate limit exceeded", {
        employerId: req.employer?.id,
        operation: 'read',
        key,
        secs,
        requestId: (req as any).requestId,
      });

      res.status(429).json({
        success: false,
        data: null,
        error: { 
          code: "RATE_LIMIT_EXCEEDED", 
          message: `Read rate limit exceeded. Try again in ${secs} seconds.`,
          retryAfter: secs
        },
        meta: {
          requestId: (req as any).requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
    });
}

/**
 * Get current rate limit status for an employer
 */
export async function getRateLimitStatus(employer: EmployerAuth, operation: string): Promise<{
  limit: number;
  remaining: number;
  resetTime: Date;
}> {
  const limiter = getRateLimiter(employer.rateLimitTier, operation);
  const key = createRateLimitKey(employer, operation);
  
  try {
    const res = await limiter.get(key);
    return {
      limit: limiter.points,
      remaining: Math.max(0, limiter.points - (res?.consumedPoints || 0)),
      resetTime: new Date(Date.now() + limiter.duration * 1000),
    };
  } catch (error) {
    return {
      limit: limiter.points,
      remaining: limiter.points,
      resetTime: new Date(Date.now() + limiter.duration * 1000),
    };
  }
}

/**
 * Reset rate limits for an employer (admin function)
 */
export async function resetRateLimits(employer: EmployerAuth, operation: string): Promise<void> {
  const limiter = getRateLimiter(employer.rateLimitTier, operation);
  const key = createRateLimitKey(employer, operation);
  
  try {
    await limiter.delete(key);
    logger.info("Rate limits reset for employer", {
      employerId: employer.id,
      operation,
    });
  } catch (error) {
    logger.error("Failed to reset rate limits", {
      employerId: employer.id,
      operation,
      error,
    });
  }
}
