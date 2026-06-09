/**
 * WebSocket Connection Manager
 *
 * Tracks all active connections, handles heartbeats, and manages
 * per-user connection limits. Provides lookup helpers used by the
 * notification delivery layer.
 *
 * Heartbeat protocol:
 *   Server sends { type: "ping" } every heartbeatIntervalMs.
 *   Client must reply with { type: "pong" } within heartbeatTimeoutMs.
 *   Connections that miss a pong are terminated and cleaned up.
 */
import { v4 as uuidv4 } from "uuid";
import type { WebSocket } from "ws";
import { logger } from "../logger";
import { wsMetrics } from "./metrics";
import type { WsConnection, WsFrame, WsConfig } from "./types";

export class ConnectionManager {
  /** connectionId → WsConnection */
  private connections = new Map<string, WsConnection>();
  /** userId → Set<connectionId> */
  private userIndex = new Map<string, Set<string>>();

  private heartbeatTimer: NodeJS.Timeout | null = null;
  private readonly config: WsConfig;

  constructor(config: WsConfig) {
    this.config = config;
  }

  // ── Lifecycle ───────────────────────────────────────────────────────────────

  startHeartbeat(): void {
    this.heartbeatTimer = setInterval(() => {
      this.runHeartbeat();
    }, this.config.heartbeatIntervalMs);
    logger.info("WS heartbeat started", { intervalMs: this.config.heartbeatIntervalMs });
  }

  stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  // ── Connection registration ─────────────────────────────────────────────────

  /**
   * Register a new authenticated WebSocket connection.
   * Returns the connection object (or null if the per-user limit is exceeded).
   */
  register(
    socket: WebSocket,
    auth: WsConnection["auth"]
  ): WsConnection | null {
    const userId = auth.sub;

    // Enforce per-user connection limit
    const userConns = this.userIndex.get(userId) ?? new Set();
    if (userConns.size >= this.config.maxConnectionsPerUser) {
      logger.warn("WS connection limit reached for user", {
        userId,
        limit: this.config.maxConnectionsPerUser,
      });
      return null;
    }

    const conn: WsConnection = {
      id: uuidv4(),
      socket,
      userId,
      auth,
      connectedAt: new Date(),
      lastPingAt: new Date(),
      lastPongAt: new Date(),
      topics: new Set(),
      isAlive: true,
      pendingAcks: new Map(),
    };

    this.connections.set(conn.id, conn);
    userConns.add(conn.id);
    this.userIndex.set(userId, userConns);

    wsMetrics.connectionsTotal.inc();
    wsMetrics.connectionsActive.inc();
    logger.info("WS connection registered", { connectionId: conn.id, userId });

    return conn;
  }

  /**
   * Remove a connection (on close or error).
   */
  remove(connectionId: string): void {
    const conn = this.connections.get(connectionId);
    if (!conn) return;

    this.connections.delete(connectionId);

    const userConns = this.userIndex.get(conn.userId);
    if (userConns) {
      userConns.delete(connectionId);
      if (userConns.size === 0) {
        this.userIndex.delete(conn.userId);
      }
    }

    wsMetrics.connectionsActive.dec();
    logger.info("WS connection removed", { connectionId, userId: conn.userId });
  }

  // ── Lookup ──────────────────────────────────────────────────────────────────

  get(connectionId: string): WsConnection | undefined {
    return this.connections.get(connectionId);
  }

  /** All connections for a given user. */
  getByUser(userId: string): WsConnection[] {
    const ids = this.userIndex.get(userId);
    if (!ids) return [];
    return Array.from(ids)
      .map((id) => this.connections.get(id))
      .filter((c): c is WsConnection => c !== undefined);
  }

  /** All connections subscribed to a topic. */
  getByTopic(topic: string): WsConnection[] {
    return Array.from(this.connections.values()).filter((c) =>
      c.topics.has(topic)
    );
  }

  /** All active connections. */
  getAll(): WsConnection[] {
    return Array.from(this.connections.values());
  }

  get totalConnections(): number {
    return this.connections.size;
  }

  // ── Heartbeat ───────────────────────────────────────────────────────────────

  /** Mark a connection as having received a pong. */
  recordPong(connectionId: string): void {
    const conn = this.connections.get(connectionId);
    if (conn) {
      conn.isAlive = true;
      conn.lastPongAt = new Date();
    }
  }

  private runHeartbeat(): void {
    const now = Date.now();
    const timeout = this.config.heartbeatTimeoutMs;

    for (const conn of this.connections.values()) {
      const msSinceLastPong = now - conn.lastPongAt.getTime();

      if (!conn.isAlive || msSinceLastPong > timeout) {
        logger.warn("WS connection timed out — terminating", {
          connectionId: conn.id,
          userId: conn.userId,
          msSinceLastPong,
        });
        wsMetrics.heartbeatTimeouts.inc();
        conn.socket.terminate();
        this.remove(conn.id);
        continue;
      }

      // Send ping
      conn.isAlive = false; // will be reset when pong arrives
      conn.lastPingAt = new Date();
      this.send(conn, { type: "ping", timestamp: new Date().toISOString() });
    }
  }

  // ── Message sending ─────────────────────────────────────────────────────────

  /**
   * Send a typed frame to a single connection.
   * Returns false if the socket is not open.
   */
  send<T>(conn: WsConnection, frame: WsFrame<T>): boolean {
    if (conn.socket.readyState !== 1 /* OPEN */) return false;
    try {
      conn.socket.send(JSON.stringify(frame));
      wsMetrics.messagesSent.inc({ type: frame.type });
      return true;
    } catch (err) {
      logger.error("WS send failed", { connectionId: conn.id, error: err });
      return false;
    }
  }

  /**
   * Send a frame to all connections of a user.
   * Returns the number of connections successfully sent to.
   */
  sendToUser<T>(userId: string, frame: WsFrame<T>): number {
    const conns = this.getByUser(userId);
    let sent = 0;
    for (const conn of conns) {
      if (this.send(conn, frame)) sent++;
    }
    return sent;
  }

  /**
   * Broadcast a frame to all connected clients.
   */
  broadcast<T>(frame: WsFrame<T>): void {
    for (const conn of this.connections.values()) {
      this.send(conn, frame);
    }
  }
}
