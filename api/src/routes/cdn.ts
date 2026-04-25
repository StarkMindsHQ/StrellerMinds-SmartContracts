/**
 * POST /api/v1/cdn/invalidate  — trigger cache invalidation for a path pattern
 * GET  /api/v1/cdn/status      — CDN configuration and invalidation store
 *
 * The invalidation endpoint is secured with an HMAC-SHA256 signature
 * (same pattern as GitHub webhooks) so it can be called from CI/CD pipelines
 * or deployment hooks without exposing a JWT.
 */
import { Router, Request, Response } from "express";
import { z } from "zod";
import {
  invalidateCache,
  validateInvalidationSignature,
  getInvalidationStore,
} from "../middleware/cdn";
import { sendSuccess, sendError } from "../utils/response";
import { config } from "../config";
import { logger } from "../logger";

const router = Router();

const invalidateSchema = z.object({
  patterns: z
    .array(z.string().min(1).max(200))
    .min(1)
    .max(50, "Max 50 patterns per request"),
  reason: z.string().max(200).optional(),
});

// ── POST /cdn/invalidate ──────────────────────────────────────────────────────
router.post("/invalidate", (req: Request, res: Response) => {
  const signature = req.headers["x-cdn-signature"] as string | undefined;
  if (!signature) {
    sendError(res, 401, "MISSING_SIGNATURE", "X-CDN-Signature header is required", undefined, req.requestId);
    return;
  }

  const rawBody = JSON.stringify(req.body);
  if (!validateInvalidationSignature(rawBody, signature)) {
    sendError(res, 401, "INVALID_SIGNATURE", "Signature verification failed", undefined, req.requestId);
    return;
  }

  const parsed = invalidateSchema.safeParse(req.body);
  if (!parsed.success) {
    sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
    return;
  }

  const { patterns, reason } = parsed.data;
  const invalidatedAt = Date.now();

  for (const pattern of patterns) {
    invalidateCache(pattern);
  }

  logger.info("Cache invalidation triggered", { patterns, reason, requestId: req.requestId });

  sendSuccess(
    res,
    {
      invalidated: patterns,
      invalidatedAt,
      reason: reason ?? null,
      cdnOrigin: config.cdn.origin || null,
    },
    200,
    req.requestId
  );
});

// ── GET /cdn/status ───────────────────────────────────────────────────────────
router.get("/status", (req: Request, res: Response) => {
  sendSuccess(
    res,
    {
      cdnOrigin: config.cdn.origin || null,
      maxAge: config.cdn.maxAge,
      sMaxAge: config.cdn.sMaxAge,
      staleWhileRevalidate: config.cdn.staleWhileRevalidate,
      activeInvalidations: getInvalidationStore(),
    },
    200,
    req.requestId
  );
});

export default router;
