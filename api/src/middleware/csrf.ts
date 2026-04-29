import { Request, Response, NextFunction } from "express";
import { randomBytes } from "crypto";
import { config } from "../config";
import { sendLocalizedError } from "../utils/response";

declare global {
  namespace Express {
    interface Request {
      csrfToken?: string;
      sessionToken?: string;
    }
  }
}

/**
 * CSRF Protection Middleware
 * Generates and validates CSRF tokens for state-changing requests
 */
export function csrfProtection(req: Request, res: Response, next: NextFunction): void {
  // Skip CSRF for GET, HEAD, OPTIONS requests
  if (['GET', 'HEAD', 'OPTIONS'].includes(req.method)) {
    generateCsrfToken(req, res);
    return next();
  }

  // Validate CSRF token for state-changing requests
  const token = req.headers['x-csrf-token'] as string || req.body._csrf;
  const sessionToken = req.cookies?.session_token || req.headers.authorization?.replace('Bearer ', '');

  if (!token || !sessionToken) {
    sendLocalizedError(req, res, 403, "CSRF_TOKEN_MISSING", "CSRF token is required");
    return;
  }

  // In production, validate against stored session tokens
  if (config.nodeEnv === 'production') {
    validateStoredCsrfToken(sessionToken, token, req, res, next);
  } else {
    // In development, use a simpler validation
    if (token.length < 32) {
      sendLocalizedError(req, res, 403, "CSRF_TOKEN_INVALID", "Invalid CSRF token");
      return;
    }
    next();
  }
}

/**
 * Generate CSRF token and set it in response headers
 */
function generateCsrfToken(req: Request, res: Response): void {
  const token = randomBytes(32).toString('hex');
  req.csrfToken = token;
  
  // Set CSRF token in response header for client-side access
  res.setHeader('X-CSRF-Token', token);
  
  // Also set in cookie for fallback (HttpOnly=false for JavaScript access)
  res.cookie('csrf_token', token, {
    httpOnly: false, // Allow JavaScript access
    secure: config.nodeEnv === 'production',
    sameSite: 'strict',
    maxAge: 60 * 60 * 1000 // 1 hour
  });
}

/**
 * Validate CSRF token against stored session token (Redis in production)
 */
function validateStoredCsrfToken(
  sessionToken: string, 
  csrfToken: string, 
  req: Request, 
  res: Response, 
  next: NextFunction
): void {
  // Import here to avoid circular dependencies
  const { cache } = require("../cache");
  
  const key = `csrf:${sessionToken}`;
  
  cache.get(key)
    .then((storedToken: string) => {
      if (!storedToken || storedToken !== csrfToken) {
        sendLocalizedError(req, res, 403, "CSRF_TOKEN_INVALID", "Invalid CSRF token");
        return;
      }
      next();
    })
    .catch(() => {
      sendLocalizedError(req, res, 500, "CSRF_VALIDATION_ERROR", "CSRF validation failed");
    });
}

/**
 * Store CSRF token for session
 */
export function storeCsrfToken(sessionToken: string, csrfToken: string): void {
  if (config.nodeEnv === 'production') {
    const { cache } = require("../cache");
    const key = `csrf:${sessionToken}`;
    cache.set(key, csrfToken, { EX: 3600 }); // 1 hour expiry
  }
}
