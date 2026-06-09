/**
 * Express type extensions for employer verification API
 * 
 * Extends the Express Request interface to include employer-specific properties
 * and other custom properties used throughout the application
 */
import { Request } from 'express';
import { EmployerAuth } from '../middleware/employerAuth';

declare global {
  namespace Express {
    interface Request {
      // Employer authentication information
      employer?: EmployerAuth;
      
      // Request ID for tracing
      requestId?: string;
      
      // Analytics opt-out flag
      analyticsOptOut?: boolean;
      
      // IP address (from trust proxy or direct connection)
      ip?: string;
      
      // Socket connection for IP fallback
      socket?: {
        remoteAddress?: string;
      };
    }
  }
}

export interface AuthenticatedRequest extends Request {
  employer?: EmployerAuth;
}
