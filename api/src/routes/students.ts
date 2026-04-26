/**
 * GET /api/v1/students/:address/certificates
 * Lists all certificate IDs issued to a student.
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { stellarAddressSchema } from "../utils/validate";
import { logger } from "../logger";
import { trackStudentCertsListed, anonymizeClientId } from "../analytics";


const router = Router();

router.get(
  "/:address/certificates",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    const parsed = stellarAddressSchema.safeParse(req.params.address);
    if (!parsed.success) {
      sendError(
        res,
        400,
        "INVALID_ADDRESS",
        "Invalid Stellar address",
        undefined,
        req.requestId
      );
      return;
    }

    try {
      const ids = await contractClient.getStudentCertificates(parsed.data);

      // ── GA4: student_certs_listed ──────────────────────────────────────────
      trackStudentCertsListed(
        anonymizeClientId(req.auth?.sub ?? "anonymous"),
        parsed.data,
        req.analyticsOptOut
      );

      sendSuccess(
        res,
        { student: parsed.data, certificateIds: ids, total: ids.length },
        200,
        req.requestId
      );
    } catch (err) {
      logger.error("Get student certificates failed", {
        address: parsed.data,
        error: err,
      });
      sendError(
        res,
        502,
        "CONTRACT_ERROR",
        "Failed to query the blockchain",
        undefined,
        req.requestId
      );
    }
  }
);

export default router;
