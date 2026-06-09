/**
 * Employer Verification API Tests
 * 
 * Comprehensive test suite for employer verification endpoints
 * including authentication, rate limiting, and validation
 */
import request from 'supertest';
import express from 'express';
import app from '../src/app';
import { auditLogger } from '../src/services/auditService';

// Mock the audit logger to avoid file system operations during tests
jest.mock('../src/services/auditService', () => ({
  auditLogger: {
    logVerificationAttempt: jest.fn().mockResolvedValue(undefined),
    logVerificationSuccess: jest.fn().mockResolvedValue(undefined),
    logVerificationFailure: jest.fn().mockResolvedValue(undefined),
    logBatchVerificationAttempt: jest.fn().mockResolvedValue(undefined),
    logBatchVerificationCompletion: jest.fn().mockResolvedValue(undefined),
    logAuthenticationSuccess: jest.fn().mockResolvedValue(undefined),
    logAuthenticationFailure: jest.fn().mockResolvedValue(undefined),
    getEmployerVerificationHistory: jest.fn().mockResolvedValue([]),
  },
}));

// Mock the contract client
jest.mock('../src/soroban-client', () => ({
  contractClient: {
    verifyCertificate: jest.fn(),
    getCertificate: jest.fn(),
    getRevocationRecord: jest.fn(),
  },
}));

import { contractClient } from '../src/soroban-client';

describe('Employer Verification API', () => {
  const validCertificateId = '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
  const validStudentAddress = 'GABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ';
  const basicApiKey = 'emp_demo_key_basic';
  const premiumApiKey = 'emp_demo_key_premium';
  const enterpriseApiKey = 'emp_demo_key_enterprise';

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('POST /api/v1/employer/verify', () => {
    describe('Authentication', () => {
      it('should reject requests without authentication', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify')
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(401);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('UNAUTHORIZED');
      });

      it('should reject requests with invalid API key', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', 'invalid_key')
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(401);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('INVALID_AUTH');
      });

      it('should accept requests with valid API key', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: {
            id: validCertificateId,
            studentAddress: validStudentAddress,
            issuer: 'StrellerMinds',
            issueDate: '2024-01-15T10:30:00Z',
          },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(200);
        expect(response.body.success).toBe(true);
        expect(response.body.data.isValid).toBe(true);
      });
    });

    describe('Request Validation', () => {
      it('should reject invalid certificate ID format', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: 'invalid_id',
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(400);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('VALIDATION_ERROR');
      });

      it('should reject invalid student address format', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: 'invalid_address',
          });

        expect(response.status).toBe(400);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('VALIDATION_ERROR');
      });

      it('should reject invalid verification level', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'invalid_level',
          });

        expect(response.status).toBe(400);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('VALIDATION_ERROR');
      });

      it('should accept valid request with all optional fields', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: {
            id: validCertificateId,
            studentAddress: validStudentAddress,
          },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'comprehensive',
            includeMetadata: true,
          });

        expect(response.status).toBe(200);
        expect(response.body.success).toBe(true);
        expect(response.body.data.verificationMetadata).toBeDefined();
      });
    });

    describe('Verification Levels', () => {
      it('should perform basic verification', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: {
            id: validCertificateId,
            studentAddress: validStudentAddress,
          },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'basic',
          });

        expect(response.status).toBe(200);
        expect(response.body.data.isValid).toBe(true);
        expect(contractClient.verifyCertificate).toHaveBeenCalledWith(validCertificateId);
      });

      it('should perform enhanced verification', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: {
            id: validCertificateId,
            studentAddress: validStudentAddress,
          },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'enhanced',
          });

        expect(response.status).toBe(200);
        expect(response.body.data.enhancedVerification).toBeDefined();
        expect(response.body.data.enhancedVerification.studentAddressMatch).toBe(true);
      });

      it('should perform comprehensive verification', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: {
            id: validCertificateId,
            studentAddress: validStudentAddress,
          },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'comprehensive',
          });

        expect(response.status).toBe(200);
        expect(response.body.data.comprehensiveVerification).toBeDefined();
        expect(response.body.data.comprehensiveVerification.riskScore).toBeDefined();
      });
    });

    describe('Error Handling', () => {
      it('should handle contract client errors', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockRejectedValue(
          new Error('Blockchain query failed')
        );

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(502);
        expect(response.body.success).toBe(false);
        expect(response.body.error.code).toBe('CONTRACT_ERROR');
      });

      it('should handle certificate not found', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: false,
          certificate: null,
        });

        const response = await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(response.status).toBe(200);
        expect(response.body.success).toBe(true);
        expect(response.body.data.isValid).toBe(false);
      });
    });

    describe('Audit Logging', () => {
      it('should log verification attempts', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: { id: validCertificateId },
        });

        await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(auditLogger.logVerificationAttempt).toHaveBeenCalledWith(
          expect.objectContaining({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
            verificationLevel: 'basic',
          })
        );
      });

      it('should log successful verifications', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: { id: validCertificateId },
        });

        await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(auditLogger.logVerificationSuccess).toHaveBeenCalledWith(
          expect.objectContaining({
            certificateId: validCertificateId,
            result: 'valid',
          })
        );
      });

      it('should log failed verifications', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockRejectedValue(
          new Error('Blockchain error')
        );

        await request(app)
          .post('/api/v1/employer/verify')
          .set('Authorization', premiumApiKey)
          .send({
            certificateId: validCertificateId,
            studentAddress: validStudentAddress,
          });

        expect(auditLogger.logVerificationFailure).toHaveBeenCalledWith(
          expect.objectContaining({
            certificateId: validCertificateId,
            error: 'Blockchain error',
          })
        );
      });
    });
  });

  describe('POST /api/v1/employer/verify/batch', () => {
    describe('Request Validation', () => {
      it('should reject empty verification array', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [],
          });

        expect(response.status).toBe(400);
        expect(response.body.error.code).toBe('VALIDATION_ERROR');
      });

      it('should reject oversized batch', async () => {
        const verifications = Array(51).fill({
          certificateId: validCertificateId,
          studentAddress: validStudentAddress,
        });

        const response = await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications,
          });

        expect(response.status).toBe(400);
        expect(response.body.error.code).toBe('BATCH_TOO_LARGE');
      });

      it('should reject invalid items in batch', async () => {
        const response = await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [
              {
                certificateId: 'invalid_id',
                studentAddress: validStudentAddress,
              },
            ],
          });

        expect(response.status).toBe(400);
        expect(response.body.error.code).toBe('VALIDATION_ERROR');
      });
    });

    describe('Batch Processing', () => {
      it('should process valid batch successfully', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: { id: validCertificateId },
        });

        const response = await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [
              {
                certificateId: validCertificateId,
                studentAddress: validStudentAddress,
              },
              {
                certificateId: '0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456',
                studentAddress: 'GBCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXY',
              },
            ],
          });

        expect(response.status).toBe(200);
        expect(response.body.success).toBe(true);
        expect(response.body.data.results).toHaveLength(2);
        expect(response.body.data.summary.total).toBe(2);
        expect(response.body.data.summary.successful).toBe(2);
        expect(response.body.data.summary.failed).toBe(0);
      });

      it('should handle mixed success/failure batch', async () => {
        (contractClient.verifyCertificate as jest.Mock)
          .mockResolvedValueOnce({
            isValid: true,
            certificate: { id: validCertificateId },
          })
          .mockRejectedValueOnce(new Error('Certificate not found'));

        const response = await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [
              {
                certificateId: validCertificateId,
                studentAddress: validStudentAddress,
              },
              {
                certificateId: '0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456',
                studentAddress: 'GBCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXY',
              },
            ],
          });

        expect(response.status).toBe(200);
        expect(response.body.data.summary.successful).toBe(1);
        expect(response.body.data.summary.failed).toBe(1);
        expect(response.body.data.summary.successRate).toBe('50.00%');
      });
    });

    describe('Audit Logging', () => {
      it('should log batch verification attempts', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: { id: validCertificateId },
        });

        await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [
              {
                certificateId: validCertificateId,
                studentAddress: validStudentAddress,
              },
            ],
          });

        expect(auditLogger.logBatchVerificationAttempt).toHaveBeenCalledWith(
          expect.objectContaining({
            batchSize: 1,
            verificationLevel: 'basic',
          })
        );
      });

      it('should log batch completion', async () => {
        (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
          isValid: true,
          certificate: { id: validCertificateId },
        });

        await request(app)
          .post('/api/v1/employer/verify/batch')
          .set('Authorization', premiumApiKey)
          .send({
            verifications: [
              {
                certificateId: validCertificateId,
                studentAddress: validStudentAddress,
              },
            ],
          });

        expect(auditLogger.logBatchVerificationCompletion).toHaveBeenCalledWith(
          expect.objectContaining({
            batchSize: 1,
            successCount: 1,
            failureCount: 0,
          })
        );
      });
    });
  });

  describe('GET /api/v1/employer/verification-history', () => {
    it('should require authentication', async () => {
      const response = await request(app)
        .get('/api/v1/employer/verification-history');

      expect(response.status).toBe(401);
      expect(response.body.error.code).toBe('UNAUTHORIZED');
    });

    it('should return verification history', async () => {
      const mockHistory = [
        {
          type: 'verification_success',
          data: {
            employerId: 'emp_002',
            certificateId: validCertificateId,
            result: 'valid',
          },
          timestamp: '2024-04-28T15:30:00Z',
          id: 'audit_123',
        },
      ];

      (auditLogger.getEmployerVerificationHistory as jest.Mock).mockResolvedValue(mockHistory);

      const response = await request(app)
        .get('/api/v1/employer/verification-history')
        .set('Authorization', premiumApiKey);

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.data.history).toEqual(mockHistory);
      expect(auditLogger.getEmployerVerificationHistory).toHaveBeenCalledWith(
        'emp_002',
        100,
        0
      );
    });

    it('should respect pagination parameters', async () => {
      (auditLogger.getEmployerVerificationHistory as jest.Mock).mockResolvedValue([]);

      const response = await request(app)
        .get('/api/v1/employer/verification-history?limit=50&offset=10')
        .set('Authorization', premiumApiKey);

      expect(response.status).toBe(200);
      expect(auditLogger.getEmployerVerificationHistory).toHaveBeenCalledWith(
        'emp_002',
        50,
        10
      );
    });

    it('should reject invalid limit parameter', async () => {
      const response = await request(app)
        .get('/api/v1/employer/verification-history?limit=1001')
        .set('Authorization', premiumApiKey);

      expect(response.status).toBe(400);
      expect(response.body.error.code).toBe('INVALID_LIMIT');
    });
  });

  describe('Rate Limiting', () => {
    it('should enforce rate limits for basic tier', async () => {
      // Mock successful verification
      (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
        isValid: true,
        certificate: { id: validCertificateId },
      });

      // Make requests up to the limit
      const requests = [];
      for (let i = 0; i < 105; i++) {
        requests.push(
          request(app)
            .post('/api/v1/employer/verify')
            .set('Authorization', basicApiKey)
            .send({
              certificateId: validCertificateId,
              studentAddress: validStudentAddress,
            })
        );
      }

      const responses = await Promise.all(requests);
      
      // Some requests should be rate limited
      const rateLimitedResponses = responses.filter(r => r.status === 429);
      expect(rateLimitedResponses.length).toBeGreaterThan(0);
      
      rateLimitedResponses.forEach(response => {
        expect(response.body.error.code).toBe('RATE_LIMIT_EXCEEDED');
        expect(response.headers['retry-after']).toBeDefined();
      });
    });

    it('should include rate limit headers', async () => {
      (contractClient.verifyCertificate as jest.Mock).mockResolvedValue({
        isValid: true,
        certificate: { id: validCertificateId },
      });

      const response = await request(app)
        .post('/api/v1/employer/verify')
        .set('Authorization', premiumApiKey)
        .send({
          certificateId: validCertificateId,
          studentAddress: validStudentAddress,
        });

      expect(response.headers['x-ratelimit-limit']).toBeDefined();
      expect(response.headers['x-ratelimit-remaining']).toBeDefined();
      expect(response.headers['x-ratelimit-reset']).toBeDefined();
    });
  });
});
