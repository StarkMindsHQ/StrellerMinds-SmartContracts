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
import { userRateLimit } from "../middleware/userRateLimiter";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { certificateIdSchema, stellarAddressSchema, normalizeCertId } from "../utils/validate";
import { verificationTotal } from "../metrics";
import { logger } from "../logger";

const router = Router();

// ── GET /certificates/:id/verify ─────────────────────────────────────────────
// Public endpoint — no auth required, stricter rate limit
// Authenticated users get per-user limits on top of the IP-based limit
router.get(
  "/:id/verify",
  verifyLimiter,
  userRateLimit("verify"),
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendLocalizedError(req, res, 400, "INVALID_CERTIFICATE_ID", "Certificate ID must be a 64-character hex string");
      return;
    }

    const certId = normalizeCertId(parsed.data);

    try {
      const result = await contractClient.verifyCertificate(certId);

      verificationTotal.inc({
        result: result.isValid
          ? "valid"
          : result.certificate === null
          ? "not_found"
          : "invalid",
      });

      logger.info("Certificate verified", {
        certificateId: certId,
        isValid: result.isValid,
        status: result.status,
        requestId: req.requestId,
      });

      sendSuccess(res, result, 200, req.requestId);
    } catch (err) {
      logger.error("Verification failed", { certId, error: err, requestId: req.requestId });
      verificationTotal.inc({ result: "error" });
      sendLocalizedError(req, res, 502, "CONTRACT_ERROR", "Failed to query the blockchain. Please try again.");
    }
  }
);

// ── GET /certificates/:id ─────────────────────────────────────────────────────
router.get(
  "/:id",
  generalLimiter,
  authenticate,
  userRateLimit("read"),
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendLocalizedError(req, res, 400, "INVALID_CERTIFICATE_ID", "Invalid certificate ID");
      return;
    }

    const certId = normalizeCertId(parsed.data);

    try {
      const cert = await contractClient.getCertificate(certId);
      if (!cert) {
        sendLocalizedError(req, res, 404, "CERTIFICATE_NOT_FOUND", "Certificate not found");
        return;
      }
      sendSuccess(res, cert, 200, req.requestId);
    } catch (err) {
      logger.error("Get certificate failed", { certId, error: err });
      sendLocalizedError(req, res, 502, "CONTRACT_ERROR", "Failed to query the blockchain");
    }
  }
);

// ── GET /certificates/:id/revocation ─────────────────────────────────────────
router.get(
  "/:id/revocation",
  generalLimiter,
  authenticate,
  userRateLimit("read"),
  async (req: Request, res: Response) => {
    const parsed = certificateIdSchema.safeParse(req.params.id);
    if (!parsed.success) {
      sendLocalizedError(req, res, 400, "INVALID_CERTIFICATE_ID", "Invalid certificate ID");
      return;
    }

    const certId = normalizeCertId(parsed.data);

    try {
      const record = await contractClient.getRevocationRecord(certId);
      if (!record) {
        sendLocalizedError(req, res, 404, "REVOCATION_NOT_FOUND", "No revocation record found for this certificate");
        return;
      }
      sendSuccess(res, record, 200, req.requestId);
    } catch (err) {
      logger.error("Get revocation failed", { certId, error: err });
      sendLocalizedError(req, res, 502, "CONTRACT_ERROR", "Failed to query the blockchain");
    }
  }
);

export default router;
