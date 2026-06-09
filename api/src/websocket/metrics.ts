/**
 * Prometheus metrics for the WebSocket notification system
 */
import client from "prom-client";

export const wsMetrics = {
  connectionsTotal: new client.Counter({
    name: "cert_api_ws_connections_total",
    help: "Total WebSocket connections established",
  }),

  connectionsActive: new client.Gauge({
    name: "cert_api_ws_connections_active",
    help: "Currently active WebSocket connections",
  }),

  messagesSent: new client.Counter({
    name: "cert_api_ws_messages_sent_total",
    help: "Total WebSocket messages sent to clients",
    labelNames: ["type"],
  }),

  messagesReceived: new client.Counter({
    name: "cert_api_ws_messages_received_total",
    help: "Total WebSocket messages received from clients",
    labelNames: ["type"],
  }),

  notificationsDelivered: new client.Counter({
    name: "cert_api_ws_notifications_delivered_total",
    help: "Total notifications successfully delivered via WebSocket",
    labelNames: ["type", "priority"],
  }),

  notificationsFailed: new client.Counter({
    name: "cert_api_ws_notifications_failed_total",
    help: "Total notifications that failed delivery",
    labelNames: ["type", "reason"],
  }),

  notificationLatencyMs: new client.Histogram({
    name: "cert_api_ws_notification_latency_ms",
    help: "Time from notification creation to delivery (ms)",
    labelNames: ["type", "priority"],
    buckets: [5, 10, 25, 50, 100, 250, 500, 1000, 2500],
  }),

  heartbeatTimeouts: new client.Counter({
    name: "cert_api_ws_heartbeat_timeouts_total",
    help: "Total connections terminated due to heartbeat timeout",
  }),

  authFailures: new client.Counter({
    name: "cert_api_ws_auth_failures_total",
    help: "Total WebSocket authentication failures",
    labelNames: ["reason"],
  }),

  queueDepth: new client.Gauge({
    name: "cert_api_ws_queue_depth",
    help: "Current in-memory message queue depth across all connections",
  }),

  ackTimeouts: new client.Counter({
    name: "cert_api_ws_ack_timeouts_total",
    help: "Total notification acknowledgement timeouts",
  }),
};
