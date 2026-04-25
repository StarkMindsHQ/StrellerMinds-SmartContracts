/**
 * GET /api/v1/analytics — aggregate certificate analytics (auth required)
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { logger } from "../logger";

const router = Router();

router.get(
  "/",
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const analytics = await contractClient.getAnalytics();
      sendSuccess(res, analytics, 200, req.requestId);
    } catch (err) {
      logger.error("Get analytics failed", { error: err });
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
