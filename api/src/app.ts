import express from "express";
import helmet from "helmet";
import cors from "cors";
import swaggerUi from "swagger-ui-express";

import { config } from "./config";
import { requestId } from "./middleware/requestId";
import { metricsMiddleware } from "./middleware/metricsMiddleware";
import { analyticsConsent } from "./middleware/analyticsConsent";
import { cdnMiddleware } from "./middleware/cdn";
import { securityHeadersValidator } from "./middleware/securityHeaders";
import { openApiSpec } from "./openapi";
import { logger } from "./logger";
import { preloadLocales } from "./i18n";

// Pre-load all locale files into memory at startup
preloadLocales();

import authRouter from "./routes/auth";
import certificatesRouter from "./routes/certificates";
import studentsRouter from "./routes/students";
import analyticsRouter from "./routes/analytics";
import socialSharingRouter from "./routes/social-sharing";
import healthRouter from "./routes/health";
import rateLimitRouter from "./routes/rateLimit";
import cdnRouter from "./routes/cdn";
import certificateTemplatesRouter from "./certificate-templates/certificate-templates.route";
import performanceRouter from "./routes/performance";
import slackRouter from "./routes/slack";

const app = express();

// ── Security headers ──────────────────────────────────────────────────────────
app.use(
  helmet({
    hsts: {
      maxAge: 31536000,
      includeSubDomains: true,
      preload: true,
    },
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'", "'unsafe-inline'"], // needed for Swagger UI
        styleSrc: ["'self'", "'unsafe-inline'"],
        imgSrc: ["'self'", "data:"],
      },
    },
    frameguard: {
      action: "sameorigin",
    },
  })
);

// ── CORS ──────────────────────────────────────────────────────────────────────
app.use(
  cors({
    origin: config.cors.origins,
    methods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Authorization", "Content-Type", "X-Request-ID", "X-Language", "Accept-Language"],
    exposedHeaders: ["X-Request-ID", "RateLimit-Limit", "RateLimit-Remaining", "Content-Language", "X-Text-Direction"],
  })
);

// ── Body parsing ──────────────────────────────────────────────────────────────
app.use(express.json({ limit: "16kb" }));

// ── Request ID + metrics ──────────────────────────────────────────────────────
app.use(requestId);
app.use(metricsMiddleware);
app.use(cdnMiddleware);

// ── i18n: language detection ──────────────────────────────────────────────────
app.use(i18nMiddleware);

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

// ── Security headers validator ────────────────────────────────────────────────
app.use(securityHeadersValidator);

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
app.use("/api/v1/rate-limit", rateLimitRouter);
app.use("/api/v1/cdn", cdnRouter);
app.use("/api/v1/certificate-templates", certificateTemplatesRouter);
app.use("/api/v1/performance", performanceRouter);
app.use("/api/v1/slack", slackRouter);

// ── CSP Violation Reporter ─────────────────────────────────────────────────────
app.post(
  "/api/v1/security/csp-report",
  express.json({ type: "application/csp-report" }),
  (req: express.Request, res: express.Response) => {
    const violation = req.body["csp-report"];
    if (violation) {
      logger.warn("CSP violation detected", {
        documentUri: violation["document-uri"],
        violatedDirective: violation["violated-directive"],
        effectiveDirective: violation["effective-directive"],
        originalPolicy: violation["original-policy"],
        sourceFile: violation["source-file"],
        lineNumber: violation["line-number"],
        columnNumber: violation["column-number"],
        statusCode: violation["status-code"],
      });
    }
    res.status(204).send();
  }
);

// ── 404 handler ───────────────────────────────────────────────────────────────
app.use((_req: express.Request, res: express.Response) => {
  res.status(404).json({
    success: false,
    data: null,
    error: { code: "NOT_FOUND", message: "Endpoint not found" },
    meta: {
      requestId: "unknown",
      timestamp: new Date().toISOString(),
      version: "1.0.0",
    },
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
      meta: {
        requestId: "unknown",
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
  }
);

export default app;