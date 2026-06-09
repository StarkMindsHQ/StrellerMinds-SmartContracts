/**
 * WebSocket Server
 *
 * Handles the full WebSocket lifecycle:
 *   1. HTTP upgrade → JWT authentication
 *   2. Connection registration
 *   3. Inbound message routing (ack, subscribe, unsubscribe, pong)
 *   4. Outbound notification delivery with ack tracking
 *   5. Reconnect replay of pending notifications
 *   6. Graceful shutdown
 *
 * Attach to an existing http.Server via wsServer.attach(httpServer).
 */
import { WebSocketServer, WebSocket } from "ws";
import type { IncomingMessage } from "http";
import type { Server as HttpServer } from "http";
import { URL } from "url";
import jwt from "jsonwebtoken";
import { config } from "../config";
import { logger } from "../logger";import { ConnectionManager } from "./connectionManager";
import { notificationStore } from "./notificationStore";
import { wsPubSub } from "./pubsub";
import { wsMetrics } from "./metrics";
import type {
  WsConnection,
  WsFrame,
  AckFrame,
  SubscribeFrame,
  NotificationFrame,
  ConnectedFrame,
  ErrorFrame,
  Notification,
  WsConfig,
} from "./types";
import type { JwtPayload } from "../types";

// ── Default config ────────────────────────────────────────────────────────────

const DEFAULT_WS_CONFIG: WsConfig = {
  path: "/ws/notifications",
  heartbeatIntervalMs: 30_000,   // ping every 30s
  heartbeatTimeoutMs: 60_000,    // terminate if no pong within 60s
  maxConnectionsPerUser: 5,
  ackTimeoutMs: 10_000,          // wait 10s for client ack before retry
  maxRetries: 3,
  maxQueueSize: 200,
};

// ── WS Server ─────────────────────────────────────────────────────────────────

export class WsNotificationServer {
  private wss: WebSocketServer | null = null;
  private readonly connManager: ConnectionManager;
  private readonly wsConfig: WsConfig;
  private cleanupInterval: NodeJS.Timeout | null = null;

  constructor(wsConfig: Partial<WsConfig> = {}) {
    this.wsConfig = { ...DEFAULT_WS_CONFIG, ...wsConfig };
    this.connManager = new ConnectionManager(this.wsConfig);
  }

  /**
   * Attach the WebSocket server to an existing HTTP server.
   * Call this once after the HTTP server is created.
   */
  attach(httpServer: HttpServer): void {
    this.wss = new WebSocketServer({
      server: httpServer,
      path: this.wsConfig.path,
    });

    this.wss.on("connection", (socket, req) => {
      this.handleUpgrade(socket, req).catch((err) => {
        logger.error("WS upgrade handler error", { error: err });
        socket.terminate();
      });
    });

    this.wss.on("error", (err) => {
      logger.error("WebSocketServer error", { error: err });
    });

    // Subscribe to pub/sub messages from other server instances
    wsPubSub.onMessage((msg) => this.handlePubSubMessage(msg));

    // Start heartbeat
    this.connManager.startHeartbeat();

    // Periodic cleanup of expired notifications (every 10 minutes)
    this.cleanupInterval = setInterval(() => {
      notificationStore.cleanupExpired().catch((err) =>
        logger.error("Notification cleanup error", { error: err })
      );
    }, 10 * 60 * 1000);

    logger.info("WS notification server attached", { path: this.wsConfig.path });
  }

  /**
   * Gracefully shut down the WebSocket server.
   */
  async shutdown(): Promise<void> {
    this.connManager.stopHeartbeat();

    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }

    if (this.wss) {
      // Close all connections
      for (const conn of this.connManager.getAll()) {
        conn.socket.close(1001, "Server shutting down");
      }
      await new Promise<void>((resolve) => this.wss!.close(() => resolve()));
      this.wss = null;
    }

    await wsPubSub.disconnect();
    logger.info("WS notification server shut down");
  }

  // ── Connection handling ─────────────────────────────────────────────────────

  private async handleUpgrade(
    socket: WebSocket,
    req: IncomingMessage
  ): Promise<void> {
    // Parse JWT from query string: ?token=<jwt>
    // (Authorization header is not available during WS upgrade in all clients)
    const url = new URL(req.url ?? "/", `http://${req.headers.host}`);
    const token = url.searchParams.get("token");

    if (!token) {
      wsMetrics.authFailures.inc({ reason: "missing_token" });
      this.sendError(socket, "AUTH_REQUIRED", "Missing token query parameter");
      socket.close(4001, "Authentication required");
      return;
    }

    let auth: JwtPayload;
    try {
      auth = jwt.verify(token, config.jwt.secret) as JwtPayload;
    } catch (err) {
      const reason = err instanceof jwt.TokenExpiredError ? "token_expired" : "token_invalid";
      wsMetrics.authFailures.inc({ reason });
      this.sendError(socket, "TOKEN_INVALID", "Invalid or expired token");
      socket.close(4001, "Authentication failed");
      return;
    }

    const conn = this.connManager.register(socket, auth);
    if (!conn) {
      this.sendError(socket, "CONNECTION_LIMIT", "Too many connections for this user");
      socket.close(4029, "Connection limit exceeded");
      return;
    }

    // Subscribe to this user's Redis channel
    await wsPubSub.subscribeUser(conn.userId);

    // Wire up socket events
    socket.on("message", (raw) => this.handleMessage(conn, raw.toString()));
    socket.on("close", () => this.handleClose(conn));
    socket.on("error", (err) => {
      logger.error("WS socket error", { connectionId: conn.id, error: err });
      this.connManager.remove(conn.id);
    });

    // Send connected frame
    const connectedFrame: ConnectedFrame = {
      type: "connected",
      payload: { connectionId: conn.id, userId: conn.userId },
      timestamp: new Date().toISOString(),
    };
    this.connManager.send(conn, connectedFrame);

    // Replay pending notifications from DB
    await this.replayPending(conn);

    logger.info("WS client connected", {
      connectionId: conn.id,
      userId: conn.userId,
    });
  }

  private handleClose(conn: WsConnection): void {
    // Cancel pending ack timers
    for (const [notifId, entry] of conn.pendingAcks) {
      logger.debug("WS connection closed with pending ack", {
        connectionId: conn.id,
        notificationId: notifId,
        retries: entry.retries,
      });
    }
    this.connManager.remove(conn.id);
  }

  // ── Inbound message routing ─────────────────────────────────────────────────

  private handleMessage(conn: WsConnection, raw: string): void {
    let frame: WsFrame;
    try {
      frame = JSON.parse(raw) as WsFrame;
    } catch {
      logger.warn("WS received malformed message", { connectionId: conn.id });
      return;
    }

    wsMetrics.messagesReceived.inc({ type: frame.type ?? "unknown" });

    switch (frame.type) {
      case "pong":
        this.connManager.recordPong(conn.id);
        break;

      case "ack":
        this.handleAck(conn, frame as AckFrame);
        break;

      case "subscribe":
        this.handleSubscribe(conn, frame as SubscribeFrame);
        break;

      case "unsubscribe":
        this.handleUnsubscribe(conn, frame as SubscribeFrame);
        break;

      default:
        logger.debug("WS unknown message type", { type: frame.type, connectionId: conn.id });
    }
  }

  private handleAck(conn: WsConnection, frame: AckFrame): void {
    const notificationId = frame.payload?.notificationId;
    if (!notificationId) return;

    conn.pendingAcks.delete(notificationId);

    notificationStore.markDelivered(notificationId).catch((err) =>
      logger.error("Failed to mark notification delivered", { notificationId, error: err })
    );

    logger.debug("WS notification acked", {
      connectionId: conn.id,
      notificationId,
    });
  }

  private handleSubscribe(conn: WsConnection, frame: SubscribeFrame): void {
    const topics = frame.payload?.topics ?? [];
    for (const topic of topics) {
      conn.topics.add(topic);
      wsPubSub.subscribeTopic(topic).catch((err) =>
        logger.error("Failed to subscribe to topic", { topic, error: err })
      );
    }
    logger.debug("WS client subscribed to topics", {
      connectionId: conn.id,
      topics,
    });
  }

  private handleUnsubscribe(conn: WsConnection, frame: SubscribeFrame): void {
    const topics = frame.payload?.topics ?? [];
    for (const topic of topics) {
      conn.topics.delete(topic);
    }
  }

  // ── Notification delivery ───────────────────────────────────────────────────

  /**
   * Deliver a notification to a specific connection.
   * Tracks pending acks and schedules retries.
   */
  async deliverToConnection(
    conn: WsConnection,
    notification: Notification
  ): Promise<boolean> {
    const frame: NotificationFrame = {
      type: "notification",
      id: notification.id,
      payload: notification,
      timestamp: new Date().toISOString(),
    };

    const sent = this.connManager.send(conn, frame);
    if (!sent) return false;

    // Track for ack
    conn.pendingAcks.set(notification.id, {
      notification,
      sentAt: new Date(),
      retries: 0,
    });

    // Schedule ack timeout / retry
    this.scheduleAckCheck(conn, notification);

    const latencyMs = Date.now() - new Date(notification.createdAt).getTime();
    wsMetrics.notificationsDelivered.inc({
      type: notification.type,
      priority: notification.priority,
    });
    wsMetrics.notificationLatencyMs.observe(
      { type: notification.type, priority: notification.priority },
      latencyMs
    );

    return true;
  }

  /**
   * Deliver a notification to all connections of a user.
   */
  async deliverToUser(
    userId: string,
    notification: Notification
  ): Promise<number> {
    const conns = this.connManager.getByUser(userId);
    let delivered = 0;
    for (const conn of conns) {
      if (await this.deliverToConnection(conn, notification)) delivered++;
    }
    return delivered;
  }

  /**
   * Deliver a notification to all connections subscribed to a topic.
   */
  async deliverToTopic(
    topic: string,
    notification: Notification
  ): Promise<number> {
    const conns = this.connManager.getByTopic(topic);
    let delivered = 0;
    for (const conn of conns) {
      if (await this.deliverToConnection(conn, notification)) delivered++;
    }
    return delivered;
  }

  /**
   * Broadcast a notification to all connected clients.
   */
  async broadcast(notification: Notification): Promise<void> {
    for (const conn of this.connManager.getAll()) {
      await this.deliverToConnection(conn, notification);
    }
  }

  // ── Ack / retry logic ───────────────────────────────────────────────────────

  private scheduleAckCheck(conn: WsConnection, notification: Notification): void {
    setTimeout(() => {
      const pending = conn.pendingAcks.get(notification.id);
      if (!pending) return; // already acked

      if (pending.retries >= this.wsConfig.maxRetries) {
        conn.pendingAcks.delete(notification.id);
        wsMetrics.ackTimeouts.inc();
        wsMetrics.notificationsFailed.inc({
          type: notification.type,
          reason: "ack_timeout",
        });
        notificationStore.markFailed(notification.id).catch(() => {});
        logger.warn("WS notification ack timeout — giving up", {
          connectionId: conn.id,
          notificationId: notification.id,
          retries: pending.retries,
        });
        return;
      }

      // Retry delivery
      pending.retries++;
      pending.sentAt = new Date();
      notificationStore.incrementRetry(notification.id).catch(() => {});

      const frame: NotificationFrame = {
        type: "notification",
        id: notification.id,
        payload: notification,
        timestamp: new Date().toISOString(),
      };
      this.connManager.send(conn, frame);
      wsMetrics.messagesSent.inc({ type: "notification" });

      logger.debug("WS notification retry", {
        connectionId: conn.id,
        notificationId: notification.id,
        retry: pending.retries,
      });

      // Schedule next check
      this.scheduleAckCheck(conn, notification);
    }, this.wsConfig.ackTimeoutMs);
  }

  // ── Reconnect replay ────────────────────────────────────────────────────────

  /**
   * On reconnect, replay all pending notifications from the DB.
   */
  private async replayPending(conn: WsConnection): Promise<void> {
    try {
      const pending = await notificationStore.getPendingForUser(conn.userId);
      if (pending.length === 0) return;

      logger.info("WS replaying pending notifications", {
        connectionId: conn.id,
        userId: conn.userId,
        count: pending.length,
      });

      for (const notification of pending) {
        await this.deliverToConnection(conn, notification);
      }
    } catch (err) {
      logger.error("WS replay failed", { connectionId: conn.id, error: err });
    }
  }

  // ── Pub/Sub handler ─────────────────────────────────────────────────────────

  private handlePubSubMessage(msg: import("./types").PubSubMessage): void {
    const { notification } = msg;

    if (msg.broadcast) {
      this.broadcast(notification).catch((err) =>
        logger.error("WS broadcast error", { error: err })
      );
      return;
    }

    if (msg.targetUserId) {
      this.deliverToUser(msg.targetUserId, notification).catch((err) =>
        logger.error("WS deliver to user error", { error: err })
      );
    }

    if (msg.targetTopic) {
      this.deliverToTopic(msg.targetTopic, notification).catch((err) =>
        logger.error("WS deliver to topic error", { error: err })
      );
    }
  }

  // ── Helpers ─────────────────────────────────────────────────────────────────

  private sendError(socket: WebSocket, code: string, message: string): void {
    if (socket.readyState !== WebSocket.OPEN) return;
    const frame: ErrorFrame = {
      type: "error",
      payload: { code, message },
      timestamp: new Date().toISOString(),
    };
    try {
      socket.send(JSON.stringify(frame));
    } catch {
      // ignore — socket may already be closing
    }
  }

  get connectionCount(): number {
    return this.connManager.totalConnections;
  }
}

// Singleton
export const wsNotificationServer = new WsNotificationServer(config.ws);
