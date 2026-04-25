/**
 * POST /api/v1/auth/token
 *
 * Issues a short-lived JWT for API consumers.
 * In production this would validate against a real credential store.
 * Here we accept a pre-shared API key and return a signed JWT.
 */
import { Router, Request, Response } from "express";
import jwt from "jsonwebtoken";
import { z } from "zod";
import { config } from "../config";
import { sendSuccess, sendError } from "../utils/response";
import { generalLimiter } from "../middleware/rateLimiter";

const router = Router();

const tokenRequestSchema = z.object({
  apiKey: z.string().min(16, "API key must be at least 16 characters"),
});

// In production: look up apiKey in a database and validate a hashed secret.
// For this implementation we use a single env-configured key for demonstration.
const DEMO_API_KEY = process.env.DEMO_API_KEY ?? "demo-api-key-change-in-prod";

router.post("/token", generalLimiter, (req: Request, res: Response) => {
  const parsed = tokenRequestSchema.safeParse(req.body);
  if (!parsed.success) {
    sendError(
      res,
      400,
      "VALIDATION_ERROR",
      "Invalid request body",
      parsed.error.flatten(),
      req.requestId
    );
    return;
  }

  const { apiKey } = parsed.data;

  // Constant-time comparison to prevent timing attacks
  const expected = Buffer.from(DEMO_API_KEY);
  const provided = Buffer.from(apiKey);
  const valid =
    expected.length === provided.length &&
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    require("crypto").timingSafeEqual(expected, provided);

  if (!valid) {
    sendError(res, 401, "INVALID_API_KEY", "Invalid API key", undefined, req.requestId);
    return;
  }

  const payload = {
    sub: "api-consumer",
    scope: ["verify", "read"],
  };

  const token = jwt.sign(payload, config.jwt.secret, {
    expiresIn: config.jwt.expiresIn as jwt.SignOptions["expiresIn"],
  });

  sendSuccess(
    res,
    {
      accessToken: token,
      tokenType: "Bearer",
      expiresIn: config.jwt.expiresIn,
      scope: payload.scope,
    },
    200,
    req.requestId
  );
});

export default router;
