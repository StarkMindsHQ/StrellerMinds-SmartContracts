import { Router, Request, Response } from "express";
import { authenticate, requireScope } from "../middleware/auth";
import { complianceService } from "../services/complianceService";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { logger } from "../logger";

const router = Router();

/**
 * POST /api/v1/admin/compliance-report
 * 
 * Trigger manual compliance report generation.
 * Restricted to users with 'compliance' scope.
 */
router.post(
  "/compliance-report",
  authenticate,
  requireScope("compliance"),
  async (req: Request, res: Response) => {
    try {
      logger.info("Admin triggered manual compliance report", { 
        userId: req.auth?.sub,
        requestId: req.requestId 
      });

      const reportUrl = await complianceService.triggerReport();

      sendSuccess(
        res,
        {
          message: "Compliance report generation started successfully",
          reportUrl,
          timestamp: new Date().toISOString()
        },
        202,
        req.requestId
      );
    } catch (error) {
      logger.error("Failed to trigger compliance report", { error });
      sendLocalizedError(
        req, 
        res, 
        500, 
        "COMPLIANCE_TRIGGER_FAILED", 
        "Failed to trigger compliance report generation"
      );
    }
  }
);

export default router;
