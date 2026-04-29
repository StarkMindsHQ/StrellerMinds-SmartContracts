import app from "./app";
import { config } from "./config";
import { logger } from "./logger";
import { cache } from "./cache";
import { wsPubSub } from "./websocket/pubsub";
import { wsNotificationServer } from "./websocket/wsServer";

cache.connect();

// Connect Redis pub/sub for multi-instance WebSocket fan-out
wsPubSub.connect().catch((err) => {
  logger.error("WS pub/sub connect failed", { error: err });
});

const server = app.listen(config.port, () => {
  logger.info("Certificate Verification API started", {
    port: config.port,
    env: config.nodeEnv,
    contractId: config.stellar.contractId || "(not configured)",
    docs: `http://localhost:${config.port}/api/docs`,
    ws: `ws://localhost:${config.port}${config.ws.path}`,
  });
});

// Attach WebSocket server to the same HTTP server
wsNotificationServer.attach(server);

// Graceful shutdown
async function shutdown(signal: string) {
  logger.info(`Received ${signal}, shutting down gracefully`);
  await wsNotificationServer.shutdown();
  server.close(async () => {
    await cache.disconnect();
    logger.info("Server closed");
    process.exit(0);
  });
  // Force exit after 10s
  setTimeout(() => process.exit(1), 10_000).unref();
}

process.on("SIGTERM", () => shutdown("SIGTERM"));
process.on("SIGINT", () => shutdown("SIGINT"));

export default server;
