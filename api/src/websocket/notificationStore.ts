/**
 * Notification Persistence Store
 *
 * Handles all database operations for notifications:
 * - Creating and persisting notifications
 * - Marking notifications as delivered / failed
 * - Fetching undelivered notifications for reconnecting clients
 * - Expiry-aware queries
 */
import { v4 as uuidv4 } from "uuid";
import { dbPool } from "../utils/dbPool";
import { logger } from "../logger";
import type {
  Notification,
  NotificationType,
  NotificationPriority,
  NotificationStatus,
  CreateNotificationInput,
} from "./types";

// ── Row → domain mapper ───────────────────────────────────────────────────────

function rowToNotification(row: Record<string, unknown>): Notification {
  return {
    id: row.id as string,
    type: row.type as NotificationType,
    priority: row.priority as NotificationPriority,
    recipientId: row.recipient_id as string,
    title: row.title as string,
    body: row.body as string,
    data: row.data as Record<string, unknown> | undefined,
    status: row.status as NotificationStatus,
    retryCount: row.retry_count as number,
    createdAt: (row.created_at as Date).toISOString(),
    expiresAt: row.expires_at ? (row.expires_at as Date).toISOString() : undefined,
    deliveredAt: row.delivered_at ? (row.delivered_at as Date).toISOString() : undefined,
  };
}

// ── Store ─────────────────────────────────────────────────────────────────────

export class NotificationStore {
  /**
   * Persist a new notification. Returns the saved notification with its DB-assigned ID.
   */
  async create(input: CreateNotificationInput): Promise<Notification> {
    const id = uuidv4();
    const expiresAt = input.ttlSeconds
      ? new Date(Date.now() + input.ttlSeconds * 1000)
      : null;

    const result = await dbPool.query<Record<string, unknown>>(
      `INSERT INTO notifications
         (id, type, priority, recipient_id, title, body, data, status, retry_count, expires_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', 0, $8)
       RETURNING *`,
      [
        id,
        input.type,
        input.priority ?? "normal",
        input.recipientId,
        input.title,
        input.body,
        input.data ? JSON.stringify(input.data) : null,
        expiresAt,
      ]
    );

    const notification = rowToNotification(result.rows[0]);
    logger.debug("Notification created", { id, type: input.type, recipientId: input.recipientId });
    return notification;
  }

  /**
   * Mark a notification as delivered.
   */
  async markDelivered(notificationId: string): Promise<void> {
    await dbPool.query(
      `UPDATE notifications
       SET status = 'delivered', delivered_at = NOW()
       WHERE id = $1 AND status != 'delivered'`,
      [notificationId]
    );
    logger.debug("Notification marked delivered", { notificationId });
  }

  /**
   * Mark a notification as failed and increment retry count.
   */
  async markFailed(notificationId: string): Promise<void> {
    await dbPool.query(
      `UPDATE notifications
       SET status = 'failed', retry_count = retry_count + 1
       WHERE id = $1`,
      [notificationId]
    );
    logger.debug("Notification marked failed", { notificationId });
  }

  /**
   * Increment retry count without changing status (for in-flight retries).
   */
  async incrementRetry(notificationId: string): Promise<void> {
    await dbPool.query(
      `UPDATE notifications SET retry_count = retry_count + 1 WHERE id = $1`,
      [notificationId]
    );
  }

  /**
   * Fetch all pending (undelivered, non-expired) notifications for a user.
   * Used when a client reconnects to replay missed notifications.
   */
  async getPendingForUser(
    userId: string,
    limit = 100
  ): Promise<Notification[]> {
    const result = await dbPool.query<Record<string, unknown>>(
      `SELECT * FROM notifications
       WHERE recipient_id = $1
         AND status = 'pending'
         AND (expires_at IS NULL OR expires_at > NOW())
       ORDER BY created_at ASC
       LIMIT $2`,
      [userId, limit]
    );

    return result.rows.map(rowToNotification);
  }

  /**
   * Fetch a single notification by ID.
   */
  async getById(notificationId: string): Promise<Notification | null> {
    const result = await dbPool.query<Record<string, unknown>>(
      `SELECT * FROM notifications WHERE id = $1`,
      [notificationId]
    );

    if (result.rows.length === 0) return null;
    return rowToNotification(result.rows[0]);
  }

  /**
   * Fetch recent notifications for a user (for history/inbox).
   */
  async getRecentForUser(
    userId: string,
    limit = 50,
    offset = 0
  ): Promise<Notification[]> {
    const result = await dbPool.query<Record<string, unknown>>(
      `SELECT * FROM notifications
       WHERE recipient_id = $1
         AND (expires_at IS NULL OR expires_at > NOW())
       ORDER BY created_at DESC
       LIMIT $2 OFFSET $3`,
      [userId, limit, offset]
    );

    return result.rows.map(rowToNotification);
  }

  /**
   * Delete expired notifications (maintenance task).
   * Returns the number of rows deleted.
   */
  async cleanupExpired(): Promise<number> {
    const result = await dbPool.query<{ cleanup_notifications: number }>(
      `SELECT cleanup_notifications(30) AS deleted`
    );
    const deleted = result.rows[0]?.deleted ?? 0;
    if (deleted > 0) {
      logger.info("Cleaned up expired notifications", { deleted });
    }
    return deleted;
  }
}

export const notificationStore = new NotificationStore();
