/**
 * Certificate verification routes
 *
 * GET  /api/v1/certificates/:id/verify   — verify a certificate (public, rate-limited)
 * GET  /api/v1/certificates/:id          — get full certificate details (auth required)
 * GET  /api/v1/certificates/:id/revocation — get revocation record (auth required)
 * GET  /api/v1/students/:address/certificates — list student certificates (auth required)
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { verifyLimiter, generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { certificateIdSchema, stellarAddressSchema, normalizeCertId } from "../utils/validate";
import { verificationTotal } from "../metrics";
import { logger } from "../logger";
import {
  trackCertificateVerified,
  trackCertificateDetailFetched,
  trackRevocationChecked,
  anonymizeClientId,
} from "../analytics";

const router = Router();

// ── GET /certificates/:id/verify ─────────────────────────────────────────────
// Public endpoint — no auth required, stricter rate limit
router.get(
  "/:id/verify",
  verifyLimiter,
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendError(
        res,
        400,
        "INVALID_CERTIFICATE_ID",
        "Certificate ID must be a 64-character hex string",
        undefined,
        req.requestId
      );
      return;
    }

    const certId = normalizeCertId(parsed.data);

    // Use the request IP as the anonymous client identifier for public endpoints
    const clientId = anonymizeClientId(req.ip ?? req.socket.remoteAddress ?? "unknown");

    try {
      const result = await contractClient.verifyCertificate(certId);

      const verResult = result.isValid
        ? "valid"
        : result.certificate === null
        ? "not_found"
        : "invalid";

      verificationTotal.inc({ result: verResult });

      logger.info("Certificate verified", {
        certificateId: certId,
        isValid: result.isValid,
        status: result.status,
        requestId: req.requestId,
      });

      // ── GA4: certificate_verified (primary conversion) ──────────────────
      trackCertificateVerified(clientId, certId, verResult, req.analyticsOptOut);

      sendSuccess(res, result, 200, req.requestId);
    } catch (err) {
      logger.error("Verification failed", { certId, error: err, requestId: req.requestId });
      verificationTotal.inc({ result: "error" });

      // ── GA4: track error outcome too ────────────────────────────────────
      trackCertificateVerified(clientId, certId, "error", req.analyticsOptOut);

      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query the blockchain. Please try again.",
        undefined,
        req.requestId
      );
    }
  }
);

// ── GET /certificates/:id ─────────────────────────────────────────────────────
router.get(
  "/:id",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendError(res, 400, "INVALID_CERTIFICATE_ID", "Invalid certificate ID", undefined, req.requestId);
      return;
    }

    const certId = normalizeCertId(parsed.data);
    const clientId = anonymizeClientId(req.auth?.sub ?? "anonymous");

    try {
      const cert = await contractClient.getCertificate(certId);
      if (!cert) {
        sendError(res, 404, "CERTIFICATE_NOT_FOUND", "Certificate not found", undefined, req.requestId);
        return;
      }

      // ── GA4: certificate_detail_fetched ─────────────────────────────────
      trackCertificateDetailFetched(clientId, certId, req.analyticsOptOut);

      sendSuccess(res, cert, 200, req.requestId);
    } catch (err) {
      logger.error("Get certificate failed", { certId, error: err });
      sendError(res, 502, "CONTRACT_ERROR", "Failed to query the blockchain", undefined, req.requestId);
    }
  }
);

// ── GET /certificates/:id/revocation ─────────────────────────────────────────
router.get(
  "/:id/revocation",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendError(res, 400, "INVALID_CERTIFICATE_ID", "Invalid certificate ID", undefined, req.requestId);
      return;
    }

    const certId = normalizeCertId(parsed.data);
    const clientId = anonymizeClientId(req.auth?.sub ?? "anonymous");

    try {
      const record = await contractClient.getRevocationRecord(certId);
      if (!record) {
        // ── GA4: revocation_checked (not found) ─────────────────────────
        trackRevocationChecked(clientId, certId, false, req.analyticsOptOut);
        sendError(res, 404, "REVOCATION_NOT_FOUND", "No revocation record found for this certificate", undefined, req.requestId);
        return;
      }

      // ── GA4: revocation_checked (found) ─────────────────────────────────
      trackRevocationChecked(clientId, certId, true, req.analyticsOptOut);

      sendSuccess(res, record, 200, req.requestId);
    } catch (err) {
      logger.error("Get revocation failed", { certId, error: err });
      sendError(res, 502, "CONTRACT_ERROR", "Failed to query the blockchain", undefined, req.requestId);
    }
  }
);

export default router;
