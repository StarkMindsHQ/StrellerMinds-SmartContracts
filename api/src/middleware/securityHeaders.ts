import { Request, Response, NextFunction } from 'express';
import { logger } from '../logger';

/**
 * Middleware to validate the presence of required security headers.
 * Attaches a listener to the response's 'finish' event to check headers
 * after the response is sent (i.e., after all middleware and routes have run).
 */
export const securityHeadersValidator = (req: Request, res: Response, next: NextFunction) => {
  // Define required security headers and their warning messages
  const requiredHeaders: Record<string, string> = {
    'strict-transport-security': 'HSTS header is missing',
    'content-security-policy': 'CSP header is missing',
    'x-frame-options': 'X-Frame-Options header is missing',
    'x-content-type-options': 'X-Content-Type-Options header is missing',
    'referrer-policy': 'Referrer-Policy header is missing',
    // Note: X-XSS-Protection is deprecated but still checked for completeness
    'x-xss-protection': 'X-XSS-Protection header is missing (deprecated but recommended for legacy browsers)',
  };

  // Listen for the 'finish' event which is emitted after the response is sent
  res.on('finish', () => {
    const headers = res.getHeaders();

    // Check each required header
    for (const [header, message] of Object.entries(requiredHeaders)) {
      if (!headers[header]) {
        logger.warn(`Security header validation: ${message} for path ${req.path}`);
      }
    }
  });

  next();
};

export default securityHeadersValidator;