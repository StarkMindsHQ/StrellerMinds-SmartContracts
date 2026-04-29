/**
 * Notification Service — public API for emitting notifications
 *
 * This is the single entry point for all code that wants to send a
 * real-time notification. It:
 *   1. Persists the notification to PostgreSQL
 *   2. Publishes it to Redis pub/sub (fan-out to all server instances)
 *   3. Delivers directly to locally-connected clients as a fast path
 *
 * Usage:
 *   import { notificationService } from './websocket/notificationService';
 *
 *   await notificationService.send({
 *     type: 'certificate_issued',
 *     priority: 'high',
 *     recipientId: userId,
 *     title: 'Certificate Issued',
 *     body: 'Your certificate for "Intro to Stellar" is ready.',
 *     data: { certificateId: '...' },
 *   });
 */
import { notificationStore } from "./notificationStore";
import { wsPubSub } from "./pubsub";
import { wsNotificationServer } from "./wsServer";
import { logger } from "../logger";
import type { CreateNotificationInput, Notification } from "./types";

export class NotificationService {
  /**
   * Send a notification to a specific user.
   * Persists to DB, publishes to Redis, and delivers to local connections.
   */
  async send(input: CreateNotificationInput): Promise<Notification> {
    // 1. Persist
    const notification = await notificationStore.create(input);

    // 2. Publish to Redis (reaches all server instances)
    await wsPubSub.publish({
      notification,
      targetUserId: input.recipientId,
    });

    // 3. Fast-path: deliver directly to locally-connected clients
    //    (the pub/sub handler will also fire, but deduplication is handled
    //    by the ack map — a second delivery attempt for an already-acked
    //    notification is a no-op)
    await wsNotificationServer.deliverToUser(input.recipientId, notification);

    logger.debug("Notification sent", {
      id: notification.id,
      type: notification.type,
      recipientId: input.recipientId,
    });

    return notification;
  }

  /**
   * Send a notification to all subscribers of a topic.
   * Not persisted per-user — use for ephemeral topic broadcasts.
   */
  async sendToTopic(
    topic: string,
    input: Omit<CreateNotificationInput, "recipientId">
  ): Promise<void> {
    // For topic notifications we create a single DB record with recipientId = topic
    // so there's an audit trail, but delivery is fan-out to all topic subscribers.
    const notification = await notificationStore.create({
      ...input,
      recipientId: `topic:${topic}`,
    });

    await wsPubSub.publish({ notification, targetTopic: topic });
    await wsNotificationServer.deliverToTopic(topic, notification);
  }

  /**
   * Broadcast a notification to every connected user.
   * Not persisted — use for system-wide announcements.
   */
  async broadcast(
    input: Omit<CreateNotificationInput, "recipientId">
  ): Promise<void> {
    const notification = await notificationStore.create({
      ...input,
      recipientId: "broadcast",
    });

    await wsPubSub.publish({ notification, broadcast: true });
    await wsNotificationServer.broadcast(notification);
  }

  /**
   * Convenience: notify about a certificate being issued.
   */
  async notifyCertificateIssued(params: {
    userId: string;
    certificateId: string;
    courseTitle: string;
    issuer: string;
  }): Promise<Notification> {
    return this.send({
      type: "certificate_issued",
      priority: "high",
      recipientId: params.userId,
      title: "Certificate Issued",
      body: `Your certificate for "${params.courseTitle}" has been issued.`,
      data: {
        certificateId: params.certificateId,
        courseTitle: params.courseTitle,
        issuer: params.issuer,
      },
    });
  }

  /**
   * Convenience: notify about a certificate being revoked.
   */
  async notifyCertificateRevoked(params: {
    userId: string;
    certificateId: string;
    reason: string;
  }): Promise<Notification> {
    return this.send({
      type: "certificate_revoked",
      priority: "critical",
      recipientId: params.userId,
      title: "Certificate Revoked",
      body: `A certificate has been revoked. Reason: ${params.reason}`,
      data: {
        certificateId: params.certificateId,
        reason: params.reason,
      },
    });
  }

  /**
   * Convenience: send a cohort message notification.
   */
  async notifyCohortMessage(params: {
    userId: string;
    cohortId: string;
    cohortName: string;
    senderName: string;
    preview: string;
  }): Promise<Notification> {
    return this.send({
      type: "cohort_message",
      priority: "normal",
      recipientId: params.userId,
      title: `New message in ${params.cohortName}`,
      body: `${params.senderName}: ${params.preview}`,
      data: {
        cohortId: params.cohortId,
        cohortName: params.cohortName,
        senderName: params.senderName,
      },
    });
  }
}

export const notificationService = new NotificationService();
