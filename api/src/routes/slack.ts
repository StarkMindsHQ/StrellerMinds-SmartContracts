// @ts-nocheck
/**
 * Slack notification routes
 *
 * POST /api/v1/slack/notify          — send a custom notification
 * POST /api/v1/slack/webhooks        — register a new webhook
 * DELETE /api/v1/slack/webhooks/:key — remove a webhook
 * GET  /api/v1/slack/webhooks        — list registered webhook keys
 * POST /api/v1/slack/test            — send a test message
 */
import { Router, Request, Response } from "express";
import { z } from "zod";
import { authenticate } from "../middleware/auth";
import { generalLimiter } from "../middleware/rateLimiter";
import { sendSuccess, sendError } from "../utils/response";
import { getSlackNotifier } from "../notifications/slack";
import { logger } from "../logger";

const router = Router();

// All Slack management endpoints require authentication
router.use(authenticate);
router.use(generalLimiter);

// ── Input schemas ─────────────────────────────────────────────────────────────

const notifySchema = z.object({
  type: z.enum(["certificate_issued", "system_alert", "system_warning", "custom"]),
  title: z.string().min(1).max(200),
  message: z.string().min(1).max(2000),
  channel: z.string().max(100).optional(),
  fields: z
    .array(
      z.object({
        title: z.string().max(100),
        value: z.string().max(500),
        short: z.boolean().optional(),
      })
    )
    .max(10)
    .optional(),
});

const webhookSchema = z.object({
  key: z.string().min(1).max(50).regex(/^[a-z0-9_-]+$/),
  url: z.string().url().startsWith("https://hooks.slack.com/"),
  channel: z.string().max(100).optional(),
  username: z.string().max(80).optional(),
});

// ── POST /slack/notify ────────────────────────────────────────────────────────
router.post("/notify", async (req: Request, res: Response) => {
  const parsed = notifySchema.safeParse(req.body);
  if (!parsed.success) {
    sendError(res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten(), req.requestId);
    return;
  }

  const notifier = getSlackNotifier();
  if (!notifier) {
    sendError(res, 503, "SLACK_NOT_CONFIGURED", "Slack notifications are not configured", undefined, req.requestId);
    return;
  }

  const ok = await notifier.notify(parsed.data);
  if (!ok) {
    logger.warn("Slack notify failed", { type: parsed.data.type, requestId: req.requestId });
    sendError(res, 502, "SLACK_DELIVERY_FAILED", "Failed to deliver Slack notification", undefined, req.requestId);
    return;
  }

  logger.info("Slack notification sent", { type: parsed.data.type, requestId: req.requestId });
  sendSuccess(res, { delivered: true }, 200, req.requestId);
});

// ── POST /slack/webhooks ──────────────────────────────────────────────────────
router.post("/webhooks", (req: Request, res: Response) => {
  const parsed = webhookSchema.safeParse(req.body);
  if (!parsed.success) {
    sendError(res, 400, "VALIDATION_ERROR", "Invalid webhook configuration", parsed.error.flatten(), req.requestId);
    return;
  }

  const notifier = getSlackNotifier();
  if (!notifier) {
    sendError(res, 503, "SLACK_NOT_CONFIGURED", "Slack notifications are not configured", undefined, req.requestId);
    return;
  }

  const { key, url, channel, username } = parsed.data;
  notifier.setWebhook(key, { url, channel, username });

  logger.info("Slack webhook registered", { key, requestId: req.requestId });
  sendSuccess(res, { key, registered: true }, 201, req.requestId);
});

// ── DELETE /slack/webhooks/:key ───────────────────────────────────────────────
router.delete("/webhooks/:key", (req: Request, res: Response) => {
  const key = req.params.key;
  if (!key || !/^[a-z0-9_-]+$/.test(key)) {
    sendError(res, 400, "INVALID_KEY", "Invalid webhook key", undefined, req.requestId);
    return;
  }

  const notifier = getSlackNotifier();
  if (!notifier) {
    sendError(res, 503, "SLACK_NOT_CONFIGURED", "Slack notifications are not configured", undefined, req.requestId);
    return;
  }

  const removed = notifier.removeWebhook(key);
  if (!removed) {
    sendError(res, 404, "WEBHOOK_NOT_FOUND", "Webhook not found", undefined, req.requestId);
    return;
  }

  logger.info("Slack webhook removed", { key, requestId: req.requestId });
  sendSuccess(res, { key, removed: true }, 200, req.requestId);
});

// ── GET /slack/webhooks ───────────────────────────────────────────────────────
router.get("/webhooks", (_req: Request, res: Response) => {
  const notifier = getSlackNotifier();
  if (!notifier) {
    sendError(res, 503, "SLACK_NOT_CONFIGURED", "Slack notifications are not configured", undefined, _req.requestId);
    return;
  }

  sendSuccess(res, { webhooks: notifier.listWebhooks() }, 200, _req.requestId);
});

// ── POST /slack/test ──────────────────────────────────────────────────────────
router.post("/test", async (req: Request, res: Response) => {
  const notifier = getSlackNotifier();
  if (!notifier) {
    sendError(res, 503, "SLACK_NOT_CONFIGURED", "Slack notifications are not configured", undefined, req.requestId);
    return;
  }

  const channel = typeof req.body?.channel === "string" ? req.body.channel : undefined;
  const ok = await notifier.notify({
    type: "custom",
    title: ":white_check_mark: Test Notification",
    message: "This is a test message from the StrellerMinds API.",
    channel,
  });

  if (!ok) {
    sendError(res, 502, "SLACK_DELIVERY_FAILED", "Test message delivery failed", undefined, req.requestId);
    return;
  }

  sendSuccess(res, { delivered: true }, 200, req.requestId);
});

export default router;
