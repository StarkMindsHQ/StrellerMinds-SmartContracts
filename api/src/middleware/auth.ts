import { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import { config } from "../config";
import { sendError } from "../utils/response";
import type { JwtPayload } from "../types";

declare global {
  namespace Express {
    interface Request {
      auth?: JwtPayload;
    }
  }
}

/**
 * Validates a Bearer JWT token in the Authorization header.
 * Attaches the decoded payload to req.auth.
 */
export function authenticate(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith("Bearer ")) {
    sendError(res, 401, "AUTH_REQUIRED", "Authorization header is required");
    return;
  }

  const token = authHeader.slice(7);
  try {
    const payload = jwt.verify(token, config.jwt.secret) as JwtPayload;
    req.auth = payload;
    next();
  } catch (err) {
    if (err instanceof jwt.TokenExpiredError) {
      sendError(res, 401, "TOKEN_EXPIRED", "Access token has expired");
    } else {
      sendError(res, 401, "TOKEN_INVALID", "Access token is invalid");
    }
  }
}

/**
 * Checks that the authenticated user has the required scope.
 */
export function requireScope(scope: string) {
  return (req: Request, res: Response, next: NextFunction): void => {
    if (!req.auth?.scope?.includes(scope)) {
      sendError(
        res,
        403,
        "INSUFFICIENT_SCOPE",
        `Required scope: ${scope}`
      );
      return;
    }
    next();
  };
}
