// @ts-nocheck
/**
 * analyticsConsent.ts
 *
 * GDPR-compliant analytics consent middleware.
 *
 * Reads the `X-Analytics-Consent` request header set by API consumers or
 * client-side applications to indicate whether the end-user has consented
 * to event tracking.
 *
 * Header values:
 *   "granted" (or absent) → tracking is permitted (default for M2M API calls)
 *   "denied"              → tracking is suppressed for this request
 *
 * The resolved preference is written to `req.analyticsOptOut` (boolean) so
 * that route handlers can pass it straight to `sendGa4Event(..., optOut)`.
 *
 * The middleware also attaches a `X-Analytics-Opt-Out-Instructions` header
 * to every response so API consumers can discover the opt-out mechanism.
 */

import { Request, Response, NextFunction } from "express";

// ── Module augmentation ───────────────────────────────────────────────────────

declare global {
  namespace Express {
    interface Request {
      /**
       * True when the request carries an explicit opt-out signal
       * (`X-Analytics-Consent: denied`).
       * Route handlers should forward this flag to all `sendGa4Event` calls.
       */
      analyticsOptOut: boolean;
    }
  }
}

// ── Middleware ────────────────────────────────────────────────────────────────

const OPT_OUT_INSTRUCTIONS_URL =
  "https://strellerminds.com/privacy#analytics-opt-out";

/**
 * Parses the `X-Analytics-Consent` header and sets `req.analyticsOptOut`.
 *
 * This middleware MUST be registered before any route handlers that fire
 * GA4 events.
 */
export function analyticsConsent(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const consentHeader = req.headers["x-analytics-consent"];

  // Explicit opt-out only — any other value (including absent) is opt-in.
  // For pure machine-to-machine API calls there are no cookies and no PII,
  // so defaulting to opt-in is GDPR-safe per Recital 26 (anonymous data).
  req.analyticsOptOut = consentHeader === "denied";

  // Let consumers discover how to opt out via response headers.
  res.setHeader(
    "X-Analytics-Opt-Out-Instructions",
    `Send header: X-Analytics-Consent: denied — details: ${OPT_OUT_INSTRUCTIONS_URL}`
  );

  next();
}
