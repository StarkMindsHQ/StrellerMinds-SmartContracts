import { Request, Response, NextFunction } from "express";
import { RateLimiterMemory } from "rate-limiter-flexible";
import { config } from "../config";
import { sendLocalizedError } from "../utils/response";
import { logger } from "../logger";

// Enhanced rate limiters for different security scenarios
const authLimiter = new RateLimiterMemory({
  points: 5, // Number of requests
  duration: 900, // Per 15 minutes
  blockDuration: 900, // Block for 15 minutes
});

const bruteForceLimiter = new RateLimiterMemory({
  points: 3, // Number of failed attempts
  duration: 3600, // Per hour
  blockDuration: 3600, // Block for 1 hour
});

const sessionCreationLimiter = new RateLimiterMemory({
  points: 10, // Number of session creations
  duration: 3600, // Per hour
  blockDuration: 1800, // Block for 30 minutes
});

const passwordResetLimiter = new RateLimiterMemory({
  points: 3, // Number of password reset attempts
  duration: 86400, // Per 24 hours
  blockDuration: 86400, // Block for 24 hours
});

/**
 * Enhanced rate limiting middleware for authentication endpoints
 */
export function authRateLimiter(req: Request, res: Response, next: NextFunction): void {
  authLimiter.consume(req.ip || 'unknown')
    .then(() => next())
    .catch((rejRes) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 900;
      res.set('Retry-After', String(secs));
      
      logger.warn("Rate limit exceeded for auth endpoint", {
        ip: req.ip,
        path: req.path,
        userAgent: req.get('User-Agent'),
        remainingPoints: rejRes.remainingPoints
      });
      
      sendLocalizedError(req, res, 429, "RATE_LIMIT_EXCEEDED", 
        `Too many authentication attempts. Try again in ${Math.ceil(secs / 60)} minutes.`,
        { retryAfter: secs }
      );
    });
}

/**
 * Brute force protection for failed login attempts
 */
export function bruteForceProtection(req: Request, res: Response, next: NextFunction): void {
  bruteForceLimiter.consume(req.ip || 'unknown')
    .then(() => next())
    .catch((rejRes) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 3600;
      res.set('Retry-After', String(secs));
      
      logger.warn("Brute force attack detected", {
        ip: req.ip,
        path: req.path,
        userAgent: req.get('User-Agent'),
        remainingPoints: rejRes.remainingPoints,
        blocked: true
      });
      
      sendLocalizedError(req, res, 429, "BRUTE_FORCE_DETECTED", 
        `Too many failed attempts. Account locked for ${Math.ceil(secs / 60)} minutes.`,
        { retryAfter: secs }
      );
    });
}

/**
 * Rate limiting for session creation
 */
export function sessionCreationRateLimiter(req: Request, res: Response, next: NextFunction): void {
  sessionCreationLimiter.consume(req.ip || 'unknown')
    .then(() => next())
    .catch((rejRes) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 1800;
      res.set('Retry-After', String(secs));
      
      logger.warn("Session creation rate limit exceeded", {
        ip: req.ip,
        path: req.path,
        userAgent: req.get('User-Agent'),
        remainingPoints: rejRes.remainingPoints
      });
      
      sendLocalizedError(req, res, 429, "SESSION_CREATION_LIMITED", 
        `Too many session creation attempts. Try again in ${Math.ceil(secs / 60)} minutes.`,
        { retryAfter: secs }
      );
    });
}

/**
 * Rate limiting for password reset operations
 */
export function passwordResetRateLimiter(req: Request, res: Response, next: NextFunction): void {
  passwordResetLimiter.consume(req.ip || 'unknown')
    .then(() => next())
    .catch((rejRes) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000) || 86400;
      res.set('Retry-After', String(secs));
      
      logger.warn("Password reset rate limit exceeded", {
        ip: req.ip,
        path: req.path,
        userAgent: req.get('User-Agent'),
        remainingPoints: rejRes.remainingPoints
      });
      
      sendLocalizedError(req, res, 429, "PASSWORD_RESET_LIMITED", 
        `Too many password reset attempts. Try again in ${Math.ceil(secs / 3600)} hours.`,
        { retryAfter: secs }
      );
    });
}

/**
 * Progressive rate limiting - gets stricter with repeated violations
 */
export function progressiveRateLimiter(req: Request, res: Response, next: NextFunction): void {
  const ip = req.ip || 'unknown';
  const key = `progressive:${ip}`;
  
  // Check existing violation count
  const violationCount = getViolationCount(ip);
  
  // Calculate rate limit based on violation history
  let points, duration, blockDuration;
  
  if (violationCount === 0) {
    // Normal rate limiting
    points = 60;
    duration = 60; // 1 minute
    blockDuration = 60;
  } else if (violationCount <= 2) {
    // First few violations - stricter limits
    points = 30;
    duration = 120; // 2 minutes
    blockDuration = 300; // 5 minutes
  } else if (violationCount <= 5) {
    // Multiple violations - much stricter
    points = 15;
    duration = 300; // 5 minutes
    blockDuration = 900; // 15 minutes
  } else {
    // Chronic violator - very strict
    points = 5;
    duration = 900; // 15 minutes
    blockDuration = 3600; // 1 hour
  }
  
  const limiter = new RateLimiterMemory({
    points,
    duration,
    blockDuration,
  });
  
  limiter.consume(key)
    .then(() => {
      next();
    })
    .catch((rejRes) => {
      const secs = Math.round(rejRes.msBeforeNext / 1000);
      res.set('Retry-After', String(secs));
      
      // Increment violation count
      incrementViolationCount(ip);
      
      logger.warn("Progressive rate limit exceeded", {
        ip,
        path: req.path,
        violationCount: violationCount + 1,
        remainingPoints: rejRes.remainingPoints,
        blockDuration: secs
      });
      
      sendLocalizedError(req, res, 429, "RATE_LIMIT_EXCEEDED", 
        `Rate limit exceeded. Try again in ${Math.ceil(secs / 60)} minutes.`,
        { retryAfter: secs, violationCount: violationCount + 1 }
      );
    });
}

/**
 * Simple in-memory store for violation tracking
 * In production, this should use Redis or another persistent store
 */
const violationStore = new Map<string, { count: number; lastReset: number }>();

function getViolationCount(ip: string): number {
  const record = violationStore.get(ip);
  if (!record) {
    return 0;
  }
  
  // Reset after 24 hours
  const now = Date.now();
  if (now - record.lastReset > 86400000) {
    violationStore.delete(ip);
    return 0;
  }
  
  return record.count;
}

function incrementViolationCount(ip: string): void {
  const record = violationStore.get(ip);
  if (!record) {
    violationStore.set(ip, { count: 1, lastReset: Date.now() });
  } else {
    record.count++;
    record.lastReset = Date.now();
  }
}

/**
 * Clean up old violation records periodically
 */
setInterval(() => {
  const now = Date.now();
  for (const [ip, record] of violationStore.entries()) {
    if (now - record.lastReset > 86400000) {
      violationStore.delete(ip);
    }
  }
}, 3600000); // Clean up every hour
