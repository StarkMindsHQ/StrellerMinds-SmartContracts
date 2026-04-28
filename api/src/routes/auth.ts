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
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { generalLimiter } from "../middleware/rateLimiter";
import { 
  authRateLimiter, 
  bruteForceProtection, 
  sessionCreationRateLimiter 
} from "../middleware/enhancedRateLimiter";
import { trackAuthTokenIssued, anonymizeClientId } from "../analytics";
import { createSessionToken, setSessionCookie, sessionMiddleware } from "../middleware/session";
import { csrfProtection } from "../middleware/csrf";


const router = Router();

const tokenRequestSchema = z.object({
  apiKey: z.string().min(16, "API key must be at least 16 characters"),
});

// In production: look up apiKey in a database and validate a hashed secret.
// For this implementation we use a single env-configured key for demonstration.
const DEMO_API_KEY = process.env.DEMO_API_KEY ?? "demo-api-key-change-in-prod";

router.post("/token", authRateLimiter, sessionCreationRateLimiter, (req: Request, res: Response) => {
  const parsed = tokenRequestSchema.safeParse(req.body);
  if (!parsed.success) {
    sendLocalizedError(req, res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten());
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
    // Apply brute force protection for failed attempts
    bruteForceProtection(req, res, () => {
      sendLocalizedError(req, res, 401, "INVALID_API_KEY", "Invalid API key");
    });
    return;
  }

  const payload = {
    sub: "api-consumer",
    scope: ["verify", "read"],
  };

  // Create secure session token with enhanced security
  const { token, sessionId } = createSessionToken(payload);

  // Set HttpOnly cookie for session token (prevents XSS attacks)
  setSessionCookie(res, token);

  // ── GA4: auth conversion event ─────────────────────────────────────────────
  // Fire-and-forget — never awaited, cannot delay or fail the response.
  trackAuthTokenIssued(
    anonymizeClientId(payload.sub),
    payload.scope,
    req.analyticsOptOut
  );

  // Return minimal response (token is in HttpOnly cookie)
  sendSuccess(
    res,
    {
      sessionId: sessionId,
      tokenType: "Bearer",
      expiresIn: config.jwt.expiresIn,
      scope: payload.scope,
      message: "Session token set in HttpOnly cookie for security"
    },
    200,
    req.requestId
  );
});

/**
 * POST /api/v1/auth/logout
 * 
 * Clears the session cookie and invalidates the session
 */
router.post("/logout", csrfProtection, (req: Request, res: Response) => {
  // Clear session cookie
  res.clearCookie('session_token', {
    httpOnly: true,
    secure: config.nodeEnv === 'production',
    sameSite: 'strict'
  });
  
  // Clear CSRF token cookie
  res.clearCookie('csrf_token');
  
  // Invalidate session in Redis (if production)
  if (config.nodeEnv === 'production' && req.sessionToken) {
    const { cache } = require("../cache");
    const keys = [
      `csrf:${req.sessionToken}`,
      `session:${req.sessionToken}`
    ];
    
    // Delete session-related keys
    Promise.all(keys.map(key => cache.del(key))).catch(() => {
      // Log error but don't fail the logout
      console.warn('Failed to invalidate session in cache');
    });
  }
  
  sendSuccess(
    res,
    {
      message: "Successfully logged out",
      timestamp: new Date().toISOString()
    },
    200,
    req.requestId
  );
});

/**
 * GET /api/v1/auth/csrf-token
 * 
 * Returns a fresh CSRF token for the current session
 */
router.get("/csrf-token", sessionMiddleware, (req: Request, res: Response) => {
  if (!req.userSession) {
    sendLocalizedError(req, res, 401, "AUTH_REQUIRED", "Authentication required");
    return;
  }
  
  sendSuccess(
    res,
    {
      csrfToken: req.csrfToken,
      sessionId: req.userSession.sessionId
    },
    200,
    req.requestId
  );
});

export default router;
