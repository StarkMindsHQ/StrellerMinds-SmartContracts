/**
 * Employer Authentication Middleware
 * 
 * Provides authentication and authorization for employer API access
 * Supports multiple authentication methods:
 * - API Key authentication
 * - JWT token authentication
 * - OAuth2 Bearer tokens
 */
import { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import { RateLimiterMemory } from "rate-limiter-flexible";
import { config } from "../config";
import { logger } from "../logger";
import { auditLogger } from "../services/auditService";

// Rate limiter for authentication attempts
const authRateLimiter = new RateLimiterMemory({
  keyPrefix: "employer_auth",
  points: 10, // Number of requests
  duration: 900, // Per 15 minutes
  blockDuration: 900, // Block for 15 minutes
});

export interface EmployerAuth {
  id: string;
  name: string;
  email: string;
  organizationId: string;
  permissions: string[];
  subscriptionTier: "basic" | "premium" | "enterprise";
  rateLimitTier: "standard" | "enhanced" | "unlimited";
}

export interface AuthenticatedRequest extends Request {
  employer?: EmployerAuth;
}

/**
 * Main employer authentication middleware
 * Supports multiple authentication methods
 */
export async function authenticateEmployer(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> {
  const startTime = Date.now();
  const clientIp = req.ip || req.socket.remoteAddress || "unknown";
  
  try {
    // Check rate limiting for authentication attempts
    await authRateLimiter.consume(clientIp);
    
    const authHeader = req.headers.authorization;
    
    if (!authHeader) {
      res.status(401).json({
        success: false,
        data: null,
        error: { code: "MISSING_AUTH_HEADER", message: "Authorization header required" },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    // Try different authentication methods
    const authResult = await tryAuthenticationMethods(authHeader, req);
    
    if (!authResult.success) {
      // Log failed authentication attempt
      await auditLogger.logAuthenticationFailure({
        ip: clientIp,
        userAgent: req.get('User-Agent'),
        reason: authResult.reason || "Authentication failed",
        timestamp: new Date().toISOString()
      });
      
      res.status(401).json({
        success: false,
        data: null,
        error: { code: "INVALID_AUTH", message: authResult.reason || "Authentication failed" },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    // Attach employer info to request
    req.employer = authResult.employer;

    // Log successful authentication
    await auditLogger.logAuthenticationSuccess({
      employerId: authResult.employer.id,
      ip: clientIp,
      userAgent: req.get('User-Agent'),
      duration: Date.now() - startTime,
      timestamp: new Date().toISOString()
    });

    logger.info("Employer authenticated", {
      employerId: authResult.employer.id,
      organizationId: authResult.employer.organizationId,
      subscriptionTier: authResult.employer.subscriptionTier,
      requestId: req.requestId,
    });

    next();
  } catch (error) {
    if (error instanceof Error && error.message.includes("Rate limit")) {
      res.status(429).json({
        success: false,
        data: null,
        error: { code: "RATE_LIMIT_EXCEEDED", message: "Too many authentication attempts" },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    logger.error("Authentication error", { error, requestId: req.requestId });
    res.status(500).json({
      success: false,
      data: null,
      error: { code: "AUTH_ERROR", message: "Authentication service error" },
      meta: {
        requestId: req.requestId,
        timestamp: new Date().toISOString(),
        version: "1.0.0",
      },
    });
  }
}

/**
 * Try different authentication methods in order
 */
async function tryAuthenticationMethods(authHeader: string, req: Request): Promise<{
  success: boolean;
  employer?: EmployerAuth;
  reason?: string;
}> {
  // Remove 'Bearer ' prefix if present
  const token = authHeader.startsWith("Bearer ") 
    ? authHeader.slice(7) 
    : authHeader;

  // Try JWT authentication first
  const jwtResult = await authenticateWithJWT(token);
  if (jwtResult.success) {
    return jwtResult;
  }

  // Try API Key authentication
  const apiKeyResult = await authenticateWithAPIKey(token);
  if (apiKeyResult.success) {
    return apiKeyResult;
  }

  // Try OAuth2 Bearer token (for future integration)
  const oauthResult = await authenticateWithOAuth(token);
  if (oauthResult.success) {
    return oauthResult;
  }

  return { success: false, reason: "Invalid authentication token" };
}

/**
 * Authenticate with JWT token
 */
async function authenticateWithJWT(token: string): Promise<{
  success: boolean;
  employer?: EmployerAuth;
  reason?: string;
}> {
  try {
    const decoded = jwt.verify(token, config.jwt.secret) as any;
    
    if (!decoded.sub || !decoded.employerId) {
      return { success: false, reason: "Invalid JWT structure" };
    }

    // In production, fetch employer details from database
    // For now, use a mock implementation
    const employer = await getEmployerFromDatabase(decoded.employerId);
    
    if (!employer) {
      return { success: false, reason: "Employer not found" };
    }

    return { success: true, employer };
  } catch (error) {
    if (error instanceof jwt.TokenExpiredError) {
      return { success: false, reason: "Token expired" };
    }
    if (error instanceof jwt.JsonWebTokenError) {
      return { success: false, reason: "Invalid token" };
    }
    return { success: false, reason: "JWT verification failed" };
  }
}

/**
 * Authenticate with API Key
 */
async function authenticateWithAPIKey(apiKey: string): Promise<{
  success: boolean;
  employer?: EmployerAuth;
  reason?: string;
}> {
  try {
    // In production, validate API key against database
    // For demo, use environment variable or hardcoded keys
    const validApiKeys = {
      "emp_demo_key_basic": { id: "emp_001", tier: "basic" },
      "emp_demo_key_premium": { id: "emp_002", tier: "premium" },
      "emp_demo_key_enterprise": { id: "emp_003", tier: "enterprise" },
    };

    const keyInfo = validApiKeys[apiKey as keyof typeof validApiKeys];
    if (!keyInfo) {
      return { success: false, reason: "Invalid API key" };
    }

    const employer = await getEmployerFromDatabase(keyInfo.id);
    if (!employer) {
      return { success: false, reason: "Employer not found" };
    }

    return { success: true, employer };
  } catch (error) {
    return { success: false, reason: "API key verification failed" };
  }
}

/**
 * Authenticate with OAuth2 (placeholder for future implementation)
 */
async function authenticateWithOAuth(token: string): Promise<{
  success: boolean;
  employer?: EmployerAuth;
  reason?: string;
}> {
  // Placeholder for OAuth2 integration
  // This would integrate with OAuth2 providers like LinkedIn, etc.
  return { success: false, reason: "OAuth2 not implemented yet" };
}

/**
 * Get employer details from database
 * In production, this would query your employer database
 */
async function getEmployerFromDatabase(employerId: string): Promise<EmployerAuth | null> {
  // Mock database implementation
  const employers: Record<string, EmployerAuth> = {
    "emp_001": {
      id: "emp_001",
      name: "Demo Company Basic",
      email: "contact@demo-basic.com",
      organizationId: "org_001",
      permissions: ["verify:basic", "read:own"],
      subscriptionTier: "basic",
      rateLimitTier: "standard",
    },
    "emp_002": {
      id: "emp_002",
      name: "Demo Company Premium",
      email: "contact@demo-premium.com",
      organizationId: "org_002",
      permissions: ["verify:basic", "verify:enhanced", "read:own", "read:analytics"],
      subscriptionTier: "premium",
      rateLimitTier: "enhanced",
    },
    "emp_003": {
      id: "emp_003",
      name: "Demo Company Enterprise",
      email: "contact@demo-enterprise.com",
      organizationId: "org_003",
      permissions: ["verify:basic", "verify:enhanced", "verify:comprehensive", "read:own", "read:analytics", "admin:org"],
      subscriptionTier: "enterprise",
      rateLimitTier: "unlimited",
    },
  };

  return employers[employerId] || null;
}

/**
 * Check if employer has required permission
 */
export function hasPermission(employer: EmployerAuth, permission: string): boolean {
  return employer.permissions.includes(permission);
}

/**
 * Middleware to check specific permissions
 */
export function requirePermission(permission: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.employer) {
      res.status(401).json({
        success: false,
        data: null,
        error: { code: "UNAUTHORIZED", message: "Employer authentication required" },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    if (!hasPermission(req.employer, permission)) {
      res.status(403).json({
        success: false,
        data: null,
        error: { code: "INSUFFICIENT_PERMISSIONS", message: `Permission '${permission}' required` },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    next();
  };
}

/**
 * Check subscription tier requirements
 */
export function requireSubscriptionTier(minTier: "basic" | "premium" | "enterprise") {
  const tierHierarchy = { basic: 0, premium: 1, enterprise: 2 };
  
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.employer) {
      res.status(401).json({
        success: false,
        data: null,
        error: { code: "UNAUTHORIZED", message: "Employer authentication required" },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    const employerTier = tierHierarchy[req.employer.subscriptionTier];
    const requiredTier = tierHierarchy[minTier];

    if (employerTier < requiredTier) {
      res.status(403).json({
        success: false,
        data: null,
        error: { code: "INSUFFICIENT_SUBSCRIPTION", message: `${minTier} subscription required` },
        meta: {
          requestId: req.requestId,
          timestamp: new Date().toISOString(),
          version: "1.0.0",
        },
      });
      return;
    }

    next();
  };
}
