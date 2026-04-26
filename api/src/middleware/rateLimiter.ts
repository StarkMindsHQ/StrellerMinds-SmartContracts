import rateLimit from "express-rate-limit";
import { config } from "../config";
import { rateLimitHits } from "../metrics";
import { sendError } from "../utils/response";
import type { Request, Response } from "express";

function onLimitReached(endpoint: string) {
  return (_req: Request, res: Response): void => {
    rateLimitHits.inc({ endpoint });
    sendError(
      res,
      429,
      "RATE_LIMIT_EXCEEDED",
      "Too many requests. Please slow down and try again later.",
      { retryAfter: Math.ceil(config.rateLimit.windowMs / 1000) }
    );
  };
}

/** General API rate limiter */
export const generalLimiter = rateLimit({
  windowMs: config.rateLimit.windowMs,
  max: config.rateLimit.maxRequests,
  standardHeaders: true,
  legacyHeaders: false,
  handler: onLimitReached("general"),
});

/** Stricter limiter for the verify endpoint (public, higher volume) */
export const verifyLimiter = rateLimit({
  windowMs: config.rateLimit.windowMs,
  max: config.rateLimit.verifyMax,
  standardHeaders: true,
  legacyHeaders: false,
  handler: onLimitReached("verify"),
});
