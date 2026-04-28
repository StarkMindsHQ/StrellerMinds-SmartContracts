/**
 * GET /api/v1/analytics — aggregate certificate analytics (auth required)
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { logger } from "../logger";
import { trackAnalyticsQueried, anonymizeClientId } from "../analytics";


const router = Router();

router.get(
  "/",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const analytics = await contractClient.getAnalytics();

      // ── GA4: analytics_queried ──────────────────────────────────────────────
      trackAnalyticsQueried(
        anonymizeClientId(req.auth?.sub ?? "anonymous"),
        req.analyticsOptOut
      );

      sendSuccess(res, analytics, 200, req.requestId);
    } catch (err) {
      logger.error("Get analytics failed", { error: err });
      sendLocalizedError(req, res, 502, "CONTRACT_ERROR", "Failed to query the blockchain");
    }
  }
);

export default router;