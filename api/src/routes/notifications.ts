/**
 * Notifications REST routes
 *
 * GET  /api/v1/notifications          — list recent notifications for the authed user
 * POST /api/v1/notifications          — send a notification (internal/admin scope)
 * GET  /api/v1/notifications/:id      — get a single notification
 * POST /api/v1/notifications/:id/ack  — mark a notification as delivered (REST fallback)
 */
import { Router, Request, Response } from "express";
import { z } from "zod";
import { authenticate, requireScope } from "../middleware/auth";
import { notificationStore } from "../websocket/notificationStore";
import { notificationService } from "../websocket/notificationService";
import { wsNotificationServer } from "../websocket/wsServer";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { logger } from "../logger";

const router = Router();

// ── Validation schemas ────────────────────────────────────────────────────────

const listQuerySchema = z.object({
  limit: z.coerce.number().int().min(1).max(100).default(50),
  offset: z.coerce.number().int().min(0).default(0),
});

const sendBodySchema = z.object({
  type: z.enum([
    "certificate_issued",
    "certificate_revoked",
    "certificate_verified",
    "cohort_message",
    "cohort_update",
    "system_alert",
    "system_info",
    "custom",
  ]),
  priority: z.enum(["low", "normal", "high", "critical"]).default("normal"),
  recipientId: z.string().min(1).max(256),
  title: z.string().min(1).max(256),
  body: z.string().min(1).max(2048),
  data: z.record(z.unknown()).optional(),
  ttlSeconds: z.number().int().min(60).max(86400).optional(),
});

// ── Routes ────────────────────────────────────────────────────────────────────

/**
 * GET /api/v1/notifications
 * Returns recent notifications for the authenticated user.
 */
router.get(
  "/",
  authenticate,
  async (req: Request, res: Response): Promise<void> => {
    const parsed = listQuerySchema.safeParse(req.query);
    if (!parsed.success) {
      sendLocalizedError(req, res, 400, "VALIDATION_ERROR", "Invalid query parameters", parsed.error.flatten());
      return;
    }

    const { limit, offset } = parsed.data;
    const userId = req.auth!.sub;

    try {
      const notifications = await notificationStore.getRecentForUser(userId, limit, offset);
      sendSuccess(res, { notifications, count: notifications.length }, 200, req.requestId);
    } catch (err) {
      logger.error("Failed to fetch notifications", { userId, error: err });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "Failed to fetch notifications");
    }
  }
);

/**
 * POST /api/v1/notifications
 * Send a notification. Requires 'notify' scope (internal/admin use).
 */
router.post(
  "/",
  authenticate,
  requireScope("notify"),
  async (req: Request, res: Response): Promise<void> => {
    const parsed = sendBodySchema.safeParse(req.body);
    if (!parsed.success) {
      sendLocalizedError(req, res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten());
      return;
    }

    try {
      const notification = await notificationService.send(parsed.data);
      sendSuccess(res, { notification }, 201, req.requestId);
    } catch (err) {
      logger.error("Failed to send notification", { error: err });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "Failed to send notification");
    }
  }
);

/**
 * GET /api/v1/notifications/:id
 * Fetch a single notification (must belong to the authed user).
 */
router.get(
  "/:id",
  authenticate,
  async (req: Request, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.auth!.sub;

    try {
      const notification = await notificationStore.getById(id);
      if (!notification) {
        sendLocalizedError(req, res, 404, "NOT_FOUND", "Notification not found");
        return;
      }
      if (notification.recipientId !== userId) {
        sendLocalizedError(req, res, 403, "INSUFFICIENT_SCOPE", "Access denied");
        return;
      }
      sendSuccess(res, { notification }, 200, req.requestId);
    } catch (err) {
      logger.error("Failed to fetch notification", { id, error: err });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "Failed to fetch notification");
    }
  }
);

/**
 * POST /api/v1/notifications/:id/ack
 * REST fallback for acknowledging a notification (for clients that can't use WS).
 */
router.post(
  "/:id/ack",
  authenticate,
  async (req: Request, res: Response): Promise<void> => {
    const { id } = req.params;
    const userId = req.auth!.sub;

    try {
      const notification = await notificationStore.getById(id);
      if (!notification) {
        sendLocalizedError(req, res, 404, "NOT_FOUND", "Notification not found");
        return;
      }
      if (notification.recipientId !== userId) {
        sendLocalizedError(req, res, 403, "INSUFFICIENT_SCOPE", "Access denied");
        return;
      }
      await notificationStore.markDelivered(id);
      sendSuccess(res, { acknowledged: true }, 200, req.requestId);
    } catch (err) {
      logger.error("Failed to ack notification", { id, error: err });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "Failed to acknowledge notification");
    }
  }
);

/**
 * GET /api/v1/notifications/status
 * Returns WebSocket server stats (admin/internal scope).
 */
router.get(
  "/status",
  authenticate,
  requireScope("internal"),
  (_req: Request, res: Response): void => {
    sendSuccess(res, {
      activeConnections: wsNotificationServer.connectionCount,
    });
  }
);

export default router;
