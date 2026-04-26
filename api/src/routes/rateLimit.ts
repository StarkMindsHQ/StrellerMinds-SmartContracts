/**
 * GET  /api/v1/rate-limit/status  — current user's rate limit status
 * GET  /api/v1/rate-limit/tiers   — public tier definitions
 */
import { Router, Request, Response } from "express";
import { authenticate } from "../middleware/auth";
import { getUserRateLimitStatus } from "../middleware/userRateLimiter";
import { sendSuccess } from "../utils/response";
import { config } from "../config";
import type { RateLimitTier } from "../types";

const router = Router();

// ── GET /rate-limit/status ────────────────────────────────────────────────────
router.get("/status", authenticate, (req: Request, res: Response) => {
  const userId = req.auth!.sub;
  const tier: RateLimitTier = req.auth!.tier ?? "free";
  const status = getUserRateLimitStatus(userId, tier);
  sendSuccess(res, status, 200, req.requestId);
});

// ── GET /rate-limit/tiers ─────────────────────────────────────────────────────
router.get("/tiers", (req: Request, res: Response) => {
  const tiers = Object.entries(config.rateLimit.tiers).map(([name, cfg]) => ({
    tier: name,
    requestsPerMinute: cfg.rpm,
    burstAllowance: cfg.burst,
  }));
  sendSuccess(res, { tiers }, 200, req.requestId);
});

export default router;
