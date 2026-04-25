/**
 * GET /health        — liveness probe
 * GET /health/ready  — readiness probe (checks contract connectivity)
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { registry } from "../metrics";
import { logger } from "../logger";
import { config } from "../config";

const router = Router();

router.get("/", (_req: Request, res: Response) => {
  res.json({
    status: "ok",
    service: "certificate-verification-api",
    version: "1.0.0",
    timestamp: new Date().toISOString(),
  });
});

router.get("/ready", async (_req: Request, res: Response) => {
  try {
    // Light check: fetch analytics to confirm contract is reachable
    await contractClient.getAnalytics();
    res.json({
      status: "ready",
      contract: config.stellar.contractId,
      network: config.stellar.rpcUrl,
      timestamp: new Date().toISOString(),
    });
  } catch (err) {
    logger.warn("Readiness check failed", { error: err });
    res.status(503).json({
      status: "not_ready",
      reason: "Cannot reach Soroban contract",
      timestamp: new Date().toISOString(),
    });
  }
});

// Prometheus metrics scrape endpoint
router.get("/metrics", async (_req: Request, res: Response) => {
  res.set("Content-Type", registry.contentType);
  res.end(await registry.metrics());
});

export default router;
