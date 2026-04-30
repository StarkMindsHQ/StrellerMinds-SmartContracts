/**
 * WebSocket notification system types
 */
import type { WebSocket } from "ws";
import type { JwtPayload } from "../types";

// ── Notification types ────────────────────────────────────────────────────────

export type NotificationType =
  | "certificate_issued"
  | "certificate_revoked"
  | "certificate_verified"
  | "cohort_message"
  | "cohort_update"
  | "system_alert"
  | "system_info"
  | "custom";

export type NotificationPriority = "low" | "normal" | "high" | "critical";

export type NotificationStatus = "pending" | "delivered" | "failed" | "expired";

export interface Notification {
  id: string;
  type: NotificationType;
  priority: NotificationPriority;
  recipientId: string;       // user sub from JWT
  title: string;
  body: string;
  data?: Record<string, unknown>;
  createdAt: string;         // ISO-8601
  expiresAt?: string;        // ISO-8601, optional TTL
  status: NotificationStatus;
  deliveredAt?: string;
  retryCount: number;
}

// ── WebSocket message frames ──────────────────────────────────────────────────

export type WsMessageType =
  | "notification"           // server → client: push a notification
  | "ack"                    // client → server: acknowledge receipt
  | "ping"                   // server → client: heartbeat
  | "pong"                   // client → server: heartbeat reply
  | "subscribe"              // client → server: subscribe to a topic
  | "unsubscribe"            // client → server: unsubscribe from a topic
  | "error"                  // server → client: error frame
  | "connected";             // server → client: connection established

export interface WsFrame<T = unknown> {
  type: WsMessageType;
  id?: string;               // correlation ID
  payload?: T;
  timestamp: string;         // ISO-8601
}

export interface NotificationFrame extends WsFrame<Notification> {
  type: "notification";
}

export interface AckFrame extends WsFrame<{ notificationId: string }> {
  type: "ack";
}

export interface SubscribeFrame extends WsFrame<{ topics: string[] }> {
  type: "subscribe";
}

export interface ConnectedFrame extends WsFrame<{ connectionId: string; userId: string }> {
  type: "connected";
}

export interface ErrorFrame extends WsFrame<{ code: string; message: string }> {
  type: "error";
}

// ── Connection state ──────────────────────────────────────────────────────────

export interface WsConnection {
  id: string;                // unique connection ID (UUID)
  socket: WebSocket;
  userId: string;            // from JWT sub
  auth: JwtPayload;
  connectedAt: Date;
  lastPingAt: Date;
  lastPongAt: Date;
  topics: Set<string>;       // subscribed topics
  isAlive: boolean;
  pendingAcks: Map<string, { notification: Notification; sentAt: Date; retries: number }>;
}

// ── Pub/Sub message (Redis channel payload) ───────────────────────────────────

export interface PubSubMessage {
  notification: Notification;
  targetUserId?: string;     // if set, only deliver to this user
  targetTopic?: string;      // if set, deliver to all subscribers of this topic
  broadcast?: boolean;       // if true, deliver to all connected users
}

// ── Notification creation ─────────────────────────────────────────────────────

export interface CreateNotificationInput {
  type: NotificationType;
  priority?: NotificationPriority;
  recipientId: string;
  title: string;
  body: string;
  data?: Record<string, unknown>;
  ttlSeconds?: number;       // optional expiry
}

// ── WS config ─────────────────────────────────────────────────────────────────

export interface WsConfig {
  path: string;
  heartbeatIntervalMs: number;
  heartbeatTimeoutMs: number;
  maxConnectionsPerUser: number;
  ackTimeoutMs: number;
  maxRetries: number;
  maxQueueSize: number;
}
