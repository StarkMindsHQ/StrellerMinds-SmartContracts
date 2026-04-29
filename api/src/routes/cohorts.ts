// @ts-nocheck
/**
 * Cohort Management Routes
 * 
 * RESTful API for student cohort management:
 * - POST   /api/v1/cohorts                    - Create cohort
 * - GET    /api/v1/cohorts                    - List cohorts
 * - GET    /api/v1/cohorts/:id                - Get cohort details
 * - PUT    /api/v1/cohorts/:id                - Update cohort
 * - DELETE /api/v1/cohorts/:id                - Delete cohort
 * - POST   /api/v1/cohorts/:id/members        - Add students
 * - DELETE /api/v1/cohorts/:id/members        - Remove students
 * - GET    /api/v1/cohorts/:id/leaderboard    - Get leaderboard
 * - GET    /api/v1/cohorts/:id/messages       - Get messages
 * - POST   /api/v1/cohorts/:id/messages       - Send message
 */
import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { authenticate } from '../middleware/auth';
import { generalLimiter } from '../middleware/rateLimiter';
import { sendSuccess, sendLocalizedError } from '../utils/response';
import { cohortService } from '../services/cohortService';
import { stellarAddressSchema } from '../utils/validate';
import { logger } from '../logger';

const router = Router();

// Validation schemas
const createCohortSchema = z.object({
  name: z.string().min(1).max(200),
  description: z.string().max(2000),
  course: z.string().min(1).max(200),
  instructor: z.string().min(1).max(200),
  startDate: z.number().positive(),
  endDate: z.number().positive(),
  maxStudents: z.number().int().positive().max(1000),
  metadata: z.record(z.any()).optional(),
});

const updateCohortSchema = z.object({
  name: z.string().min(1).max(200).optional(),
  description: z.string().max(2000).optional(),
  course: z.string().min(1).max(200).optional(),
  instructor: z.string().min(1).max(200).optional(),
  startDate: z.number().positive().optional(),
  endDate: z.number().positive().optional(),
  maxStudents: z.number().int().positive().max(1000).optional(),
  status: z.enum(['active', 'completed', 'archived', 'draft']).optional(),
  metadata: z.record(z.any()).optional(),
});

const addStudentsSchema = z.object({
  studentAddresses: z.array(stellarAddressSchema).min(1).max(100),
});

const sendMessageSchema = z.object({
  content: z.string().min(1).max(5000),
  type: z.enum(['announcement', 'discussion', 'question', 'resource']).default('discussion'),
  replyTo: z.string().optional(),
});

/**
 * POST /api/v1/cohorts
 * Create a new cohort
 */
router.post(
  '/',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const validated = createCohortSchema.parse(req.body);

      const cohort = await cohortService.createCohort(validated);

      logger.info('Cohort created via API', {
        cohortId: cohort.id,
        instructor: cohort.instructor,
        requestId: req.requestId,
      });

      sendSuccess(res, cohort, 201, req.requestId);
    } catch (error) {
      if (error instanceof z.ZodError) {
        sendLocalizedError(req, res, 400, 'VALIDATION_ERROR', error.errors[0].message);
        return;
      }
      logger.error('Create cohort failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'CREATE_COHORT_ERROR', 'Failed to create cohort');
    }
  }
);

/**
 * GET /api/v1/cohorts
 * List all cohorts with optional filtering
 */
router.get(
  '/',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const status = req.query.status as string | undefined;
      const course = req.query.course as string | undefined;
      const instructor = req.query.instructor as string | undefined;
      const limit = parseInt(req.query.limit as string) || 50;
      const offset = parseInt(req.query.offset as string) || 0;

      const cohorts = await cohortService.listCohorts({
        status,
        course,
        instructor,
        limit: Math.min(limit, 100),
        offset,
      });

      sendSuccess(res, {
        cohorts,
        total: cohorts.length,
        limit,
        offset,
      }, 200, req.requestId);
    } catch (error) {
      logger.error('List cohorts failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'LIST_COHORTS_ERROR', 'Failed to list cohorts');
    }
  }
);

/**
 * GET /api/v1/cohorts/:id
 * Get cohort details
 */
router.get(
  '/:id',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const cohort = await cohortService.getCohort(req.params.id);

      if (!cohort) {
        sendLocalizedError(req, res, 404, 'COHORT_NOT_FOUND', 'Cohort not found');
        return;
      }

      sendSuccess(res, cohort, 200, req.requestId);
    } catch (error) {
      logger.error('Get cohort failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'GET_COHORT_ERROR', 'Failed to get cohort');
    }
  }
);

/**
 * PUT /api/v1/cohorts/:id
 * Update cohort
 */
router.put(
  '/:id',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const validated = updateCohortSchema.parse(req.body);

      const cohort = await cohortService.updateCohort(req.params.id, validated);

      logger.info('Cohort updated via API', {
        cohortId: cohort.id,
        requestId: req.requestId,
      });

      sendSuccess(res, cohort, 200, req.requestId);
    } catch (error) {
      if (error instanceof z.ZodError) {
        sendLocalizedError(req, res, 400, 'VALIDATION_ERROR', error.errors[0].message);
        return;
      }
      if (error instanceof Error && error.message === 'Cohort not found') {
        sendLocalizedError(req, res, 404, 'COHORT_NOT_FOUND', 'Cohort not found');
        return;
      }
      logger.error('Update cohort failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'UPDATE_COHORT_ERROR', 'Failed to update cohort');
    }
  }
);

/**
 * DELETE /api/v1/cohorts/:id
 * Delete cohort
 */
router.delete(
  '/:id',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      await cohortService.deleteCohort(req.params.id);

      logger.info('Cohort deleted via API', {
        cohortId: req.params.id,
        requestId: req.requestId,
      });

      sendSuccess(res, { message: 'Cohort deleted successfully' }, 200, req.requestId);
    } catch (error) {
      logger.error('Delete cohort failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'DELETE_COHORT_ERROR', 'Failed to delete cohort');
    }
  }
);

/**
 * POST /api/v1/cohorts/:id/members
 * Add students to cohort
 */
router.post(
  '/:id/members',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const validated = addStudentsSchema.parse(req.body);

      const members = await cohortService.addStudents(
        req.params.id,
        validated.studentAddresses
      );

      logger.info('Students added to cohort via API', {
        cohortId: req.params.id,
        count: members.length,
        requestId: req.requestId,
      });

      sendSuccess(res, { members, count: members.length }, 200, req.requestId);
    } catch (error) {
      if (error instanceof z.ZodError) {
        sendLocalizedError(req, res, 400, 'VALIDATION_ERROR', error.errors[0].message);
        return;
      }
      logger.error('Add students failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'ADD_STUDENTS_ERROR', 'Failed to add students to cohort');
    }
  }
);

/**
 * DELETE /api/v1/cohorts/:id/members
 * Remove students from cohort
 */
router.delete(
  '/:id/members',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const validated = addStudentsSchema.parse(req.body);

      await cohortService.removeStudents(
        req.params.id,
        validated.studentAddresses
      );

      logger.info('Students removed from cohort via API', {
        cohortId: req.params.id,
        count: validated.studentAddresses.length,
        requestId: req.requestId,
      });

      sendSuccess(res, { message: 'Students removed successfully' }, 200, req.requestId);
    } catch (error) {
      if (error instanceof z.ZodError) {
        sendLocalizedError(req, res, 400, 'VALIDATION_ERROR', error.errors[0].message);
        return;
      }
      logger.error('Remove students failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'REMOVE_STUDENTS_ERROR', 'Failed to remove students from cohort');
    }
  }
);

/**
 * GET /api/v1/cohorts/:id/leaderboard
 * Get cohort leaderboard
 */
router.get(
  '/:id/leaderboard',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const leaderboard = await cohortService.getLeaderboard(req.params.id);

      sendSuccess(res, {
        leaderboard,
        total: leaderboard.length,
      }, 200, req.requestId);
    } catch (error) {
      logger.error('Get leaderboard failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'GET_LEADERBOARD_ERROR', 'Failed to get leaderboard');
    }
  }
);

/**
 * GET /api/v1/cohorts/:id/messages
 * Get cohort messages
 */
router.get(
  '/:id/messages',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const limit = parseInt(req.query.limit as string) || 50;
      const offset = parseInt(req.query.offset as string) || 0;
      const type = req.query.type as string | undefined;

      const messages = await cohortService.getMessages(req.params.id, {
        limit: Math.min(limit, 100),
        offset,
        type,
      });

      sendSuccess(res, {
        messages,
        total: messages.length,
        limit,
        offset,
      }, 200, req.requestId);
    } catch (error) {
      logger.error('Get messages failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'GET_MESSAGES_ERROR', 'Failed to get messages');
    }
  }
);

/**
 * POST /api/v1/cohorts/:id/messages
 * Send message to cohort
 */
router.post(
  '/:id/messages',
  generalLimiter,
  authenticate,
  async (req: Request, res: Response) => {
    try {
      const validated = sendMessageSchema.parse(req.body);

      const senderAddress = req.auth?.sub || 'anonymous';

      const message = await cohortService.sendMessage(
        req.params.id,
        senderAddress,
        validated.content,
        validated.type,
        validated.replyTo
      );

      logger.info('Message sent to cohort via API', {
        cohortId: req.params.id,
        messageId: message.id,
        requestId: req.requestId,
      });

      sendSuccess(res, message, 201, req.requestId);
    } catch (error) {
      if (error instanceof z.ZodError) {
        sendLocalizedError(req, res, 400, 'VALIDATION_ERROR', error.errors[0].message);
        return;
      }
      logger.error('Send message failed', { error, requestId: req.requestId });
      sendLocalizedError(req, res, 500, 'SEND_MESSAGE_ERROR', 'Failed to send message');
    }
  }
);

export default router;
