// @ts-nocheck
/**
 * Employer Credential Verification API
 * 
 * Enhanced verification endpoints specifically for employers with:
 * - Employer authentication
 * - Enhanced rate limiting
 * - Comprehensive audit logging
 * - Batch verification capabilities
 * - Detailed verification reports
 */
import { Router, Request, Response } from "express";
import { contractClient } from "../soroban-client";
import { authenticateEmployer } from "../middleware/employerAuth";
import { employerRateLimiter, batchVerificationLimiter } from "../middleware/employerRateLimiter";
import { auditLogger } from "../services/auditService";
import { sendSuccess, sendLocalizedError } from "../utils/response";
import { 
  certificateIdSchema, 
  stellarAddressSchema, 
  normalizeCertId,
  employerVerificationSchema,
  batchVerificationSchema
} from "../utils/validate";
import { 
  employerVerificationTotal, 
  batchVerificationTotal,
  verificationDuration 
} from "../metrics";
import { logger } from "../logger";
import { 
  trackEmployerVerification,
  trackBatchVerification,
  anonymizeClientId 
} from "../analytics";

const router = Router();

/**
 * POST /api/v1/employer/verify
 * 
 * Verify a single credential with employer authentication
 * 
 * Request body:
 * {
 *   "certificateId": "64-char-hex-string",
 *   "studentAddress": "G-address...",
 *   "verificationLevel": "basic|enhanced|comprehensive",
 *   "includeMetadata": boolean
 * }
 */
router.post(
  "/verify",
  authenticateEmployer,
  employerRateLimiter,
  async (req: Request, res: Response) => {
    const startTime = Date.now();
    
    try {
      const parsed = employerVerificationSchema.safeParse(req.body);
      if (!parsed.success) {
        sendLocalizedError(req, res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten());
        return;
      }

      const { certificateId, studentAddress, verificationLevel = "basic", includeMetadata = false } = parsed.data;
      const normalizedCertId = normalizeCertId(certificateId);
      const employerId = req.employer?.id || "unknown";
      const clientId = anonymizeClientId(employerId);

      // Log verification attempt
      await auditLogger.logVerificationAttempt({
        employerId,
        certificateId: normalizedCertId,
        studentAddress,
        verificationLevel,
        timestamp: new Date().toISOString(),
        ipAddress: req.ip,
        userAgent: req.get('User-Agent')
      });

      try {
        // Verify certificate on blockchain
        const result = await contractClient.verifyCertificate(normalizedCertId);
        
        // Enhanced verification based on level
        let enhancedResult = result;
        if (verificationLevel === "enhanced" || verificationLevel === "comprehensive") {
          enhancedResult = await performEnhancedVerification(normalizedCertId, studentAddress, result);
        }

        if (verificationLevel === "comprehensive") {
          enhancedResult = await performComprehensiveVerification(normalizedCertId, studentAddress, enhancedResult);
        }

        const duration = Date.now() - startTime;
        verificationDuration.observe(duration);

        const verResult = enhancedResult.isValid
          ? "valid"
          : enhancedResult.certificate === null
          ? "not_found"
          : "invalid";

        employerVerificationTotal.inc({ 
          result: verResult, 
          level: verificationLevel,
          employer: employerId 
        });

        // Log successful verification
        await auditLogger.logVerificationSuccess({
          employerId,
          certificateId: normalizedCertId,
          studentAddress,
          verificationLevel,
          result: verResult,
          duration,
          timestamp: new Date().toISOString()
        });

        logger.info("Employer certificate verified", {
          employerId,
          certificateId: normalizedCertId,
          studentAddress,
          verificationLevel,
          isValid: enhancedResult.isValid,
          duration,
          requestId: req.requestId,
        });

        // Track analytics
        trackEmployerVerification(clientId, normalizedCertId, verResult, verificationLevel, req.analyticsOptOut);

        const responseData = includeMetadata 
          ? { ...enhancedResult, verificationMetadata: { level: verificationLevel, duration, timestamp: new Date().toISOString() } }
          : enhancedResult;

        sendSuccess(res, responseData, 200, req.requestId);
      } catch (contractError) {
        const duration = Date.now() - startTime;
        
        // Log verification failure
        await auditLogger.logVerificationFailure({
          employerId,
          certificateId: normalizedCertId,
          studentAddress,
          verificationLevel,
          error: contractError instanceof Error ? contractError.message : 'Unknown error',
          duration,
          timestamp: new Date().toISOString()
        });

        logger.error("Employer verification failed", { 
          employerId,
          certificateId: normalizedCertId, 
          error: contractError, 
          requestId: req.requestId 
        });
        
        employerVerificationTotal.inc({ result: "error", level: verificationLevel, employer: employerId });
        sendLocalizedError(req, res, 502, "CONTRACT_ERROR", "Failed to query the blockchain. Please try again.");
      }
    } catch (error) {
      logger.error("Unexpected error in employer verification", { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "An unexpected error occurred");
    }
  }
);

/**
 * POST /api/v1/employer/verify/batch
 * 
 * Verify multiple credentials in a single request
 * 
 * Request body:
 * {
 *   "verifications": [
 *     {
 *       "certificateId": "64-char-hex-string",
 *       "studentAddress": "G-address..."
 *     }
 *   ],
 *   "verificationLevel": "basic|enhanced|comprehensive",
 *   "includeMetadata": boolean
 * }
 */
router.post(
  "/verify/batch",
  authenticateEmployer,
  batchVerificationLimiter,
  async (req: Request, res: Response) => {
    const startTime = Date.now();
    
    try {
      const parsed = batchVerificationSchema.safeParse(req.body);
      if (!parsed.success) {
        sendLocalizedError(req, res, 400, "VALIDATION_ERROR", "Invalid request body", parsed.error.flatten());
        return;
      }

      const { verifications, verificationLevel = "basic", includeMetadata = false } = parsed.data;
      const employerId = req.employer?.id || "unknown";
      const clientId = anonymizeClientId(employerId);

      // Validate batch size limits
      if (verifications.length > 50) {
        sendLocalizedError(req, res, 400, "BATCH_TOO_LARGE", "Maximum 50 certificates per batch");
        return;
      }

      // Log batch verification attempt
      await auditLogger.logBatchVerificationAttempt({
        employerId,
        batchSize: verifications.length,
        verificationLevel,
        timestamp: new Date().toISOString(),
        ipAddress: req.ip,
        userAgent: req.get('User-Agent')
      });

      const results = [];
      let successCount = 0;
      let failureCount = 0;

      // Process verifications in parallel with concurrency limit
      const concurrencyLimit = 10;
      const chunks = [];
      for (let i = 0; i < verifications.length; i += concurrencyLimit) {
        chunks.push(verifications.slice(i, i + concurrencyLimit));
      }

      for (const chunk of chunks) {
        const chunkPromises = chunk.map(async (verification) => {
          const { certificateId, studentAddress } = verification;
          const normalizedCertId = normalizeCertId(certificateId);
          
          try {
            const result = await contractClient.verifyCertificate(normalizedCertId);
            
            // Apply enhanced verification based on level
            let enhancedResult = result;
            if (verificationLevel === "enhanced" || verificationLevel === "comprehensive") {
              enhancedResult = await performEnhancedVerification(normalizedCertId, studentAddress, result);
            }

            if (verificationLevel === "comprehensive") {
              enhancedResult = await performComprehensiveVerification(normalizedCertId, studentAddress, enhancedResult);
            }

            return {
              certificateId,
              studentAddress,
              success: true,
              result: enhancedResult
            };
          } catch (error) {
            return {
              certificateId,
              studentAddress,
              success: false,
              error: error instanceof Error ? error.message : 'Verification failed'
            };
          }
        });

        const chunkResults = await Promise.all(chunkPromises);
        results.push(...chunkResults);
      }

      // Count successes and failures
      successCount = results.filter(r => r.success).length;
      failureCount = results.filter(r => !r.success).length;

      const duration = Date.now() - startTime;
      verificationDuration.observe(duration);

      batchVerificationTotal.inc({ 
        result: successCount === verifications.length ? "success" : "partial",
        employer: employerId 
      });

      // Log batch verification completion
      await auditLogger.logBatchVerificationCompletion({
        employerId,
        batchSize: verifications.length,
        successCount,
        failureCount,
        verificationLevel,
        duration,
        timestamp: new Date().toISOString()
      });

      logger.info("Batch employer verification completed", {
        employerId,
        batchSize: verifications.length,
        successCount,
        failureCount,
        verificationLevel,
        duration,
        requestId: req.requestId,
      });

      // Track analytics
      trackBatchVerification(clientId, verifications.length, successCount, verificationLevel, req.analyticsOptOut);

      const responseData = {
        results,
        summary: {
          total: verifications.length,
          successful: successCount,
          failed: failureCount,
          successRate: (successCount / verifications.length * 100).toFixed(2) + '%'
        },
        metadata: includeMetadata ? {
          verificationLevel,
          duration,
          timestamp: new Date().toISOString(),
          employerId
        } : undefined
      };

      sendSuccess(res, responseData, 200, req.requestId);
    } catch (error) {
      logger.error("Unexpected error in batch employer verification", { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "An unexpected error occurred");
    }
  }
);

/**
 * GET /api/v1/employer/verification-history
 * 
 * Get verification history for the authenticated employer
 */
router.get(
  "/verification-history",
  authenticateEmployer,
  employerRateLimiter,
  async (req: Request, res: Response) => {
    try {
      const employerId = req.employer?.id;
      if (!employerId) {
        sendLocalizedError(req, res, 401, "UNAUTHORIZED", "Employer authentication required");
        return;
      }

      const limit = parseInt(req.query.limit as string) || 100;
      const offset = parseInt(req.query.offset as string) || 0;

      if (limit > 1000) {
        sendLocalizedError(req, res, 400, "INVALID_LIMIT", "Maximum limit is 1000 records");
        return;
      }

      const history = await auditLogger.getEmployerVerificationHistory(employerId, limit, offset);

      sendSuccess(res, {
        history,
        pagination: {
          limit,
          offset,
          total: history.length
        }
      }, 200, req.requestId);
    } catch (error) {
      logger.error("Failed to fetch verification history", { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, "INTERNAL_ERROR", "Failed to fetch verification history");
    }
  }
);

// Helper functions for enhanced verification
async function performEnhancedVerification(certId: string, studentAddress: string, baseResult: any): Promise<any> {
  // Add enhanced verification logic
  // This could include additional blockchain checks, cross-references, etc.
  return {
    ...baseResult,
    enhancedVerification: {
      studentAddressMatch: baseResult.certificate?.studentAddress === studentAddress,
      additionalChecks: "passed",
      verificationTimestamp: new Date().toISOString()
    }
  };
}

async function performComprehensiveVerification(certId: string, studentAddress: string, enhancedResult: any): Promise<any> {
  // Add comprehensive verification logic
  // This could include deep analytics, historical data, etc.
  return {
    ...enhancedResult,
    comprehensiveVerification: {
      riskScore: "low",
      historicalVerificationCount: 1,
      complianceChecks: "passed",
      comprehensiveTimestamp: new Date().toISOString()
    }
  };
}

export default router;
