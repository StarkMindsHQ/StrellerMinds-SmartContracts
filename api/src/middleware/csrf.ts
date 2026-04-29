/**
 * CSRF Token Validation Middleware
 *
 * Implements CSRF (Cross-Site Request Forgery) protection by validating
 * CSRF tokens on all state-changing requests (POST, PUT, PATCH, DELETE).
 *
 * Security Model:
 * - Tokens are generated per session/request ID
 * - Tokens are one-time use (consumed after validation)
 * - Tokens are validated before processing state-changing requests
 * - Safe methods (GET, HEAD, OPTIONS) bypass validation
 *
 * Production Deployment:
 * - Replace in-memory token storage with Redis or session store
 * - Implement token rotation on each request
 * - Set SameSite=Strict on session cookies (if using cookies)
 * - Add Origin/Referer header validation as defense-in-depth
 */

import { Request, Response, NextFunction } from "express";
import crypto from "crypto";
import { sendLocalizedError } from "../utils/response";
import { logger } from "../logger";

/**
 * CSRF token storage (in-memory for development)
 * In production, use Redis or a session store
 */
const csrfTokens = new Map<string, Set<string>>();

/**
 * Generate a CSRF token for the current session
 * 
 * @param req Express request object
 * @param res Express response object
 * @returns Generated CSRF token
 */
export function generateCsrfToken(req: Request, res: Response): string {
  const token = crypto.randomBytes(32).toString("hex");
  const sessionId = req.requestId || "unknown";

  if (!csrfTokens.has(sessionId)) {
    csrfTokens.set(sessionId, new Set());
  }
  csrfTokens.get(sessionId)!.add(token);

  // Set token in response header for client to use
  res.setHeader("X-CSRF-Token", token);

  logger.debug("CSRF token generated", {
    sessionId,
    tokenLength: token.length,
  });

  return token;
}

/**
 * Validate CSRF token on state-changing requests
 * 
 * Middleware that validates CSRF tokens for POST, PUT, PATCH, DELETE requests.
 * Safe methods (GET, HEAD, OPTIONS) bypass validation.
 * 
 * @param req Express request object
 * @param res Express response object
 * @param next Express next function
 */
export function validateCsrfToken(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Skip validation for safe methods
  if (["GET", "HEAD", "OPTIONS"].includes(req.method)) {
    return next();
  }

  const token = req.headers["x-csrf-token"] as string;
  const sessionId = req.requestId || "unknown";

  if (!token) {
    logger.warn("CSRF token missing", {
      method: req.method,
      path: req.path,
      sessionId,
    });

    sendLocalizedError(
      req,
      res,
      403,
      "CSRF_TOKEN_MISSING",
      "CSRF token is required for state-changing requests"
    );
    return;
  }

  const validTokens = csrfTokens.get(sessionId);
  if (!validTokens || !validTokens.has(token)) {
    logger.warn("CSRF token invalid or expired", {
      method: req.method,
      path: req.path,
      sessionId,
      tokenProvided: !!token,
      tokenValid: validTokens?.has(token) ?? false,
    });

    sendLocalizedError(
      req,
      res,
      403,
      "CSRF_TOKEN_INVALID",
      "CSRF token is invalid or expired"
    );
    return;
  }

  // Consume the token (one-time use)
  validTokens.delete(token);

  logger.debug("CSRF token validated", {
    method: req.method,
    path: req.path,
    sessionId,
  });

  next();
}

/**
 * Cleanup expired CSRF tokens
 * 
 * Removes CSRF tokens for sessions that have been inactive for a specified duration.
 * This prevents memory leaks from accumulating tokens.
 * 
 * @param maxAgeMs Maximum age of tokens in milliseconds (default: 1 hour)
 */
export function cleanupExpiredTokens(maxAgeMs: number = 3600000): void {
  // In production, implement proper session expiry tracking
  // For now, this is a placeholder for cleanup logic
  logger.debug("CSRF token cleanup triggered", {
    tokensStored: csrfTokens.size,
  });
}

/**
 * Clear all CSRF tokens (for testing purposes)
 */
export function clearAllTokens(): void {
  csrfTokens.clear();
  logger.debug("All CSRF tokens cleared");
}

/**
 * Get token count (for monitoring/debugging)
 */
export function getTokenCount(): number {
  return csrfTokens.size;
}
