import express from "express";
import helmet from "helmet";
import cors from "cors";
import swaggerUi from "swagger-ui-express";

import { config } from "./config";
import { requestId } from "./middleware/requestId";
import { metricsMiddleware } from "./middleware/metricsMiddleware";
import { analyticsConsent } from "./middleware/analyticsConsent";
import { openApiSpec } from "./openapi";
import { logger } from "./logger";

import authRouter from "./routes/auth";
import certificatesRouter from "./routes/certificates";
import studentsRouter from "./routes/students";
import analyticsRouter from "./routes/analytics";
import consentRouter from "./routes/consent";
import healthRouter from "./routes/health";

const app = express();

// ── Security headers ──────────────────────────────────────────────────────────
app.use(
  helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'", "'unsafe-inline'"], // needed for Swagger UI
        styleSrc: ["'self'", "'unsafe-inline'"],
        imgSrc: ["'self'", "data:"],
        // Allow GA4 Measurement Protocol calls from client-side code
        connectSrc: ["'self'", "https://www.google-analytics.com", "https://analytics.google.com"],
      },
    },
  })
);

// ── CORS ──────────────────────────────────────────────────────────────────────
app.use(
  cors({
    origin: config.cors.origins,
    methods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Authorization", "Content-Type", "X-Request-ID"],
    exposedHeaders: ["X-Request-ID", "RateLimit-Limit", "RateLimit-Remaining"],
  })
);

// ── Body parsing ──────────────────────────────────────────────────────────────
app.use(express.json({ limit: "16kb" }));

// ── Request ID + metrics ──────────────────────────────────────────────────────
app.use(requestId);
app.use(metricsMiddleware);

// ── GDPR analytics consent ────────────────────────────────────────────────────
// Must run before any route handler that fires GA4 events.
app.use(analyticsConsent);

// ── Request logging ───────────────────────────────────────────────────────────
app.use((req: express.Request, _res: express.Response, next: express.NextFunction) => {
  logger.debug("Incoming request", {
    method: req.method,
    path: req.path,
    requestId: req.requestId,
    ip: req.ip,
  });
  next();
});

// ── API docs ──────────────────────────────────────────────────────────────────
app.use(
  "/api/docs",
  swaggerUi.serve,
  swaggerUi.setup(openApiSpec, {
    customSiteTitle: "StrellerMinds Certificate API",
  })
);

// ── Routes ────────────────────────────────────────────────────────────────────
app.use("/health", healthRouter);
app.use("/api/v1/auth", authRouter);
app.use("/api/v1/certificates", certificatesRouter);
app.use("/api/v1/students", studentsRouter);
app.use("/api/v1/analytics", analyticsRouter);
app.use("/api/v1/analytics", consentRouter); // consent sub-routes

// ── 404 handler ───────────────────────────────────────────────────────────────
app.use((_req: express.Request, res: express.Response) => {
  res.status(404).json({
    success: false,
    data: null,
    error: { code: "NOT_FOUND", message: "Endpoint not found" },
    meta: { requestId: "unknown", timestamp: new Date().toISOString(), version: "1.0.0" },
  });
});

// ── Global error handler ──────────────────────────────────────────────────────
app.use(
  (
    err: Error,
    _req: express.Request,
    res: express.Response,
    _next: express.NextFunction
  ) => {
    logger.error("Unhandled error", { error: err });
    res.status(500).json({
      success: false,
      data: null,
      error: { code: "INTERNAL_ERROR", message: "An unexpected error occurred" },
      meta: { requestId: "unknown", timestamp: new Date().toISOString(), version: "1.0.0" },
    });
  }
);

export default app;
