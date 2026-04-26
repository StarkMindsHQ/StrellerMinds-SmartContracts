import express from "express";
import helmet from "helmet";
import cors from "cors";
import swaggerUi from "swagger-ui-express";
import { randomBytes } from "crypto";

import { config } from "./config";
import { requestId } from "./middleware/requestId";
import { metricsMiddleware } from "./middleware/metricsMiddleware";
import { cdnMiddleware } from "./middleware/cdn";
import { openApiSpec } from "./openapi";
import { logger } from "./logger";

import authRouter from "./routes/auth";
import certificatesRouter from "./routes/certificates";
import studentsRouter from "./routes/students";
import analyticsRouter from "./routes/analytics";
import socialSharingRouter from "./routes/social-sharing";
import healthRouter from "./routes/health";
import rateLimitRouter from "./routes/rateLimit";
import cdnRouter from "./routes/cdn";

const app = express();

// ── CSP Nonce Middleware ──────────────────────────────────────────────────────
const cspNonceMiddleware = (req: express.Request, _res: express.Response, next: express.NextFunction) => {
  const nonce = randomBytes(16).toString("hex");
  (req as any).cspNonce = nonce;
  next();
};
app.use(cspNonceMiddleware);

// ── Security headers ──────────────────────────────────────────────────────────
app.use(
  helmet({
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'", (req: any) => `'nonce-${req.cspNonce}'`, "https://cdn.jsdelivr.net"],
        styleSrc: ["'self'", (req: any) => `'nonce-${req.cspNonce}'`, "https://cdn.jsdelivr.net"],
        imgSrc: ["'self'", "data:", "https:"],
        fontSrc: ["'self'", "https://fonts.googleapis.com", "https://fonts.gstatic.com"],
        connectSrc: ["'self'", "https://api.stellar.org"],
        frameSrc: ["'self'"],
        objectSrc: ["'none'"],
        mediaSrc: ["'self'"],
        childSrc: ["'self'"],
        formAction: ["'self'"],
        upgradeInsecureRequests: [],
        reportUri: ["/api/v1/security/csp-report"],
      },
      blockAllMixedContent: true,
    },
    crossOriginEmbedderPolicy: true,
    crossOriginOpenerPolicy: true,
    crossOriginResourcePolicy: { policy: "cross-origin" },
    dnsPrefetchControl: true,
    frameguard: { action: "deny" },
    hidePoweredBy: true,
    hsts: { maxAge: 31536000, includeSubDomains: true, preload: true },
    ieNoOpen: true,
    noSniff: true,
    referrerPolicy: { policy: "strict-no-referrer" },
    xssFilter: true,
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
app.use(cdnMiddleware);

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
app.use("/api/v1/social-sharing", socialSharingRouter);

// ── CSP Violation Reporter ─────────────────────────────────────────────────────
app.post("/api/v1/security/csp-report", express.json({ type: "application/csp-report" }), (req: express.Request, res: express.Response) => {
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
});

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
