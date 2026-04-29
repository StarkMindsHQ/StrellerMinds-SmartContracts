/**
 * consent.ts
 *
 * POST /api/v1/analytics/consent  — store tracking consent preference
 * GET  /api/v1/analytics/consent  — retrieve current consent preference
 * DELETE /api/v1/analytics/consent — withdraw consent (opt out)
 *
 * Intended for use by browser-based frontends that need to relay a user's
 * cookie-consent-banner choice to the server before events are dispatched.
 *
 * Storage: in-memory Map keyed by anonymized client ID.
 * In production this should be backed by Redis or a DB for persistence
 * across restarts and horizontal scale.
 *
 * GDPR notes:
 *  - No PII is stored — only the anonymized client_id and a boolean.
 *  - The client_id is derived from the Authorization JWT `sub` claim.
 *  - Unauthenticated callers receive a 401; the consent store is not public.
 */

import { Router, Request, Response } from "express";
import { z } from "zod";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { anonymizeClientId } from "../analytics/ga4Client";
import { logger } from "../logger";

const router = Router();

// ── In-memory consent store ───────────────────────────────────────────────────

/**
 * Map<anonymizedClientId, "granted" | "denied">
 * Replace with Redis SET/GET in production for persistence.
 */
const consentStore = new Map<string, "granted" | "denied">();

// ── Schemas ───────────────────────────────────────────────────────────────────

const consentBodySchema = z.object({
  consent: z.enum(["granted", "denied"]),
});

// ── Helpers ───────────────────────────────────────────────────────────────────

function getClientId(req: Request): string {
  const sub = req.auth?.sub ?? req.ip ?? "anonymous";
  return anonymizeClientId(sub);
}

// ── POST /api/v1/analytics/consent ───────────────────────────────────────────
router.post(
  "/consent",
  generalLimiter,
  authenticate,
  (req: Request, res: Response) => {
    const parsed = consentBodySchema.safeParse(req.body);
    if (!parsed.success) {
      sendError(
        res,
        400,
        "VALIDATION_ERROR",
        "Body must be: { \"consent\": \"granted\" | \"denied\" }",
        parsed.error.flatten(),
        req.requestId
      );
      return;
    }

    const clientId = getClientId(req);
    consentStore.set(clientId, parsed.data.consent);

    logger.info("Analytics consent preference updated", {
      clientId,
      consent: parsed.data.consent,
      requestId: req.requestId,
    });

    sendSuccess(
      res,
      { consent: parsed.data.consent, updatedAt: new Date().toISOString() },
      200,
      req.requestId
    );
  }
);

// ── GET /api/v1/analytics/consent ────────────────────────────────────────────
router.get(
  "/consent",
  generalLimiter,
  authenticate,
  (req: Request, res: Response) => {
    const clientId = getClientId(req);
    const preference = consentStore.get(clientId) ?? "granted"; // default: opt-in

    sendSuccess(
      res,
      {
        consent: preference,
        note:
          preference === "granted"
            ? "Tracking is enabled. Send X-Analytics-Consent: denied to opt out."
            : "Tracking is disabled. Send X-Analytics-Consent: granted to opt in.",
      },
      200,
      req.requestId
    );
  }
);

// ── DELETE /api/v1/analytics/consent (withdraw) ───────────────────────────────
router.delete(
  "/consent",
  generalLimiter,
  authenticate,
  (req: Request, res: Response) => {
    const clientId = getClientId(req);
    consentStore.delete(clientId);

    logger.info("Analytics consent preference withdrawn", {
      clientId,
      requestId: req.requestId,
    });

    sendSuccess(
      res,
      {
        consent: "denied",
        note: "Your tracking preference has been removed. Tracking is now disabled.",
      },
      200,
      req.requestId
    );
  }
);

export default router;
