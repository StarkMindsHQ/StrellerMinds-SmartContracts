import { Request, Response, NextFunction } from "express";
import jwt, { SignOptions } from "jsonwebtoken";
import { config } from "../config";
import { sendLocalizedError } from "../utils/response";
import { storeCsrfToken } from "./csrf";
import type { JwtPayload } from "../types";

declare global {
  namespace Express {
    interface Request {
      sessionToken?: string;
      userSession?: {
        sub: string;
        scope: string[];
        sessionId: string;
        createdAt: number;
        lastActivity: number;
      };
    }
  }
}

/**
 * Secure Session Management Middleware
 * Handles HttpOnly cookies and session validation
 */
export function sessionMiddleware(req: Request, res: Response, next: NextFunction): void {
  // Extract session token from HttpOnly cookie or Authorization header
  const sessionToken = req.cookies?.session_token || req.headers.authorization?.replace('Bearer ', '');
  
  if (!sessionToken) {
    req.userSession = undefined;
    return next();
  }

  req.sessionToken = sessionToken;

  try {
    // Verify JWT token
    const decoded = jwt.verify(sessionToken, config.jwt.secret) as any;
    
    // Enhanced session validation
    if (!isValidSession(decoded)) {
      clearSessionCookie(res);
      sendLocalizedError(req, res, 401, "SESSION_INVALID", "Invalid session format");
      return;
    }

    // Check session expiry and activity
    const now = Date.now();
    const maxAge = 60 * 60 * 1000; // 1 hour
    const maxInactivity = 30 * 60 * 1000; // 30 minutes

    if (now - decoded.createdAt > maxAge) {
      clearSessionCookie(res);
      sendLocalizedError(req, res, 401, "SESSION_EXPIRED", "Session has expired");
      return;
    }

    if (now - decoded.lastActivity > maxInactivity) {
      clearSessionCookie(res);
      sendLocalizedError(req, res, 401, "SESSION_INACTIVE", "Session expired due to inactivity");
      return;
    }

    // Update last activity
    decoded.lastActivity = now;
    req.userSession = decoded;

    // Generate new CSRF token for this session
    const csrfToken = require('crypto').randomBytes(32).toString('hex');
    storeCsrfToken(sessionToken, csrfToken);
    
    // Set new CSRF token in response
    res.setHeader('X-CSRF-Token', csrfToken);
    res.cookie('csrf_token', csrfToken, {
      httpOnly: false,
      secure: config.nodeEnv === 'production',
      sameSite: 'strict',
      maxAge: 60 * 60 * 1000
    });

    next();
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      clearSessionCookie(res);
      sendLocalizedError(req, res, 401, "SESSION_EXPIRED", "Session has expired");
    } else if (error instanceof jwt.JsonWebTokenError) {
      clearSessionCookie(res);
      sendLocalizedError(req, res, 401, "SESSION_INVALID", "Invalid session token");
    } else {
      sendLocalizedError(req, res, 500, "SESSION_ERROR", "Session validation failed");
    }
  }
}

/**
 * Validate session structure
 */
function isValidSession(decoded: any): boolean {
  return (
    decoded &&
    typeof decoded.sub === 'string' &&
    Array.isArray(decoded.scope) &&
    typeof decoded.sessionId === 'string' &&
    typeof decoded.createdAt === 'number' &&
    typeof decoded.lastActivity === 'number'
  );
}

/**
 * Clear session cookie
 */
function clearSessionCookie(res: Response): void {
  res.clearCookie('session_token', {
    httpOnly: true,
    secure: config.nodeEnv === 'production',
    sameSite: 'strict'
  });
  res.clearCookie('csrf_token');
}

/**
 * Create secure session token
 */
export function createSessionToken(payload: {
  sub: string;
  scope: string[];
}): { token: string; sessionId: string } {
  const sessionId = require('crypto').randomBytes(16).toString('hex');
  const now = Date.now();

  const sessionPayload = {
    ...payload,
    sessionId,
    createdAt: now,
    lastActivity: now
  };

  const options: SignOptions = {
    expiresIn: config.jwt.expiresIn as "1h",
  };
  
  const token = jwt.sign(sessionPayload, config.jwt.secret, options);

  return { token, sessionId };
}

/**
 * Set secure session cookie
 */
export function setSessionCookie(res: Response, token: string): void {
  res.cookie('session_token', token, {
    httpOnly: true, // Prevent XSS attacks
    secure: config.nodeEnv === 'production', // HTTPS only in production
    sameSite: 'strict', // CSRF protection
    maxAge: 60 * 60 * 1000, // 1 hour
    path: '/',
    // Additional security headers
    domain: config.nodeEnv === 'production' ? process.env.DOMAIN : undefined
  });
}

/**
 * Enhanced authentication middleware that works with session cookies
 */
export function authenticateWithSession(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Check if user session exists from sessionMiddleware
  if (!req.userSession) {
    sendLocalizedError(req, res, 401, "AUTH_REQUIRED", "Authentication required");
    return;
  }

  // Attach user info to request for compatibility
  req.auth = {
    sub: req.userSession.sub,
    scope: req.userSession.scope,
    iat: Math.floor(req.userSession.createdAt / 1000),
    exp: Math.floor((req.userSession.createdAt + 60 * 60 * 1000) / 1000)
  };

  next();
}
