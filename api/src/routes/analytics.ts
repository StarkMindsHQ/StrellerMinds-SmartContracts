/**
 * GET /api/v1/analytics       — aggregate certificate analytics (auth required)
 * GET /api/v1/analytics/cache — cache hit/miss stats (auth required)
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { logger } from "../logger";
import { trackAnalyticsQueried, anonymizeClientId } from "../analytics";
import fs from "fs";
import path from "path";


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

// Lightweight event ingestion endpoint for search analytics
router.post(
  "/events",
  generalLimiter,
  async (req: Request, res: Response) => {
    try {
      const event = req.body;

      // Basic validation: require `type` and `query`
      if (!event || !event.type || !event.query) {
        return res.status(400).json({ success: false, error: { code: "INVALID_PAYLOAD", message: "Missing required fields" } });
      }

      const outDir = path.join(process.cwd(), "api", "data");
      if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });

      const file = path.join(outDir, "search_events.jsonl");
      const record = Object.assign({}, event, { received_at: new Date().toISOString() });

      fs.appendFile(file, JSON.stringify(record) + "\n", (err) => {
        if (err) {
          logger.error("Failed to write analytics event", { error: err });
        }
      });

      res.status(202).json({ success: true, data: null });
    } catch (err) {
      logger.error("Ingest analytics event failed", { error: err });
      res.status(500).json({ success: false, error: { code: "INGEST_ERROR", message: "Failed to ingest event" } });
    }
  }
);

// Serve computed suggestions (read-only, public)
router.get(
  "/suggestions",
  generalLimiter,
  async (_req: Request, res: Response) => {
    try {
      const file = path.join(process.cwd(), "api", "data", "suggestions.json");
      if (!fs.existsSync(file)) {
        return res.status(204).json({ success: true, data: {} });
      }

      const raw = fs.readFileSync(file, "utf8");
      const suggestions = JSON.parse(raw);

      res.status(200).json({ success: true, data: suggestions });
    } catch (err) {
      logger.error("Failed to read suggestions file", { error: err });
      res.status(500).json({ success: false, error: { code: "SUGGESTIONS_ERROR", message: "Failed to load suggestions" } });
    }
  }
);
