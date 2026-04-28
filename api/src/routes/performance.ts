import { Router, Request, Response } from "express";
import { z } from "zod";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { logger } from "../logger";

const router = Router();

const invalidateQueryCacheSchema = z
  .object({
    query: z.string().min(1).max(100).optional(),
    keyPrefix: z.string().min(1).max(500).optional(),
  })
  .refine((value) => value.query !== undefined || value.keyPrefix !== undefined || Object.keys(value).length === 0, {
    message: "Provide query, keyPrefix, or an empty body to invalidate the full cache",
    path: ["query"],
  });

router.get(
  "/query-optimization",
  generalLimiter,
  (_req: Request, res: Response) => {
    try {
      const report = contractClient.getQueryOptimizationReport();
      sendSuccess(res, report, 200, _req.requestId);
    } catch (error) {
      logger.error("Get query optimization report failed", { error, requestId: _req.requestId });
      sendError(
        res,
        500,
        "QUERY_OPTIMIZATION_REPORT_ERROR",
        "Failed to build query optimization report",
        undefined,
        _req.requestId
      );
    }
  }
);

router.post(
  "/query-cache/invalidate",
  generalLimiter,
  authenticate,
  (req: Request, res: Response) => {
    const parsed = invalidateQueryCacheSchema.safeParse(req.body ?? {});
    if (!parsed.success) {
      sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
      return;
    }

    try {
      const result = contractClient.invalidateQueryCache({
        queryName: parsed.data.query,
        keyPrefix: parsed.data.keyPrefix,
      });

      logger.info("Query cache invalidated", {
        query: parsed.data.query ?? null,
        keyPrefix: parsed.data.keyPrefix ?? null,
        invalidated: result.invalidated,
        user: req.auth?.sub,
        requestId: req.requestId,
      });

      sendSuccess(
        res,
        {
          invalidated: result.invalidated,
          query: parsed.data.query ?? null,
          keyPrefix: parsed.data.keyPrefix ?? null,
          invalidatedAt: new Date().toISOString(),
        },
        200,
        req.requestId
      );
    } catch (error) {
      logger.error("Query cache invalidation failed", { error, requestId: req.requestId });
      sendError(
        res,
        500,
        "QUERY_CACHE_INVALIDATION_ERROR",
        "Failed to invalidate query cache",
        undefined,
        req.requestId
      );
    }
  }
);

export default router;
