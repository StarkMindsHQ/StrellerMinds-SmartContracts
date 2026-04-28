/**
 * Cohort Service
 * 
 * Business logic for student cohort management:
 * - Cohort CRUD operations
 * - Student enrollment/withdrawal
 * - Leaderboard calculation
 * - Group messaging system
 */
import { cacheService } from './cacheService';
import { dbPool } from '../utils/dbPool';
import { logger } from '../logger';
import type {
  Cohort,
  CohortMember,
  CohortLeaderboardEntry,
  CohortMessage,
  CreateCohortRequest,
  UpdateCohortRequest,
  CohortStats,
  CohortAnalytics,
} from '../types';

export class CohortService {
  /**
   * Create a new cohort
   */
  async createCohort(request: CreateCohortRequest): Promise<Cohort> {
    const cohort: Cohort = {
      id: this.generateId(),
      name: request.name,
      description: request.description,
      course: request.course,
      instructor: request.instructor,
      startDate: request.startDate,
      endDate: request.endDate,
      maxStudents: request.maxStudents,
      currentStudents: 0,
      status: 'draft',
      createdAt: Date.now(),
      updatedAt: Date.now(),
      metadata: request.metadata,
    };

    // Insert into database
    const query = `
      INSERT INTO cohorts (
        id, name, description, course, instructor, 
        start_date, end_date, max_students, current_students,
        status, created_at, updated_at, metadata
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    `;

    await dbPool.query(query, [
      cohort.id,
      cohort.name,
      cohort.description,
      cohort.course,
      cohort.instructor,
      cohort.startDate,
      cohort.endDate,
      cohort.maxStudents,
      cohort.currentStudents,
      cohort.status,
      cohort.createdAt,
      cohort.updatedAt,
      JSON.stringify(cohort.metadata || {}),
    ]);

    logger.info('Cohort created', { cohortId: cohort.id, name: cohort.name });

    // Invalidate cohort cache
    await cacheService.invalidateCohorts();

    return cohort;
  }

  /**
   * Get cohort by ID
   */
  async getCohort(cohortId: string): Promise<Cohort | null> {
    // Check cache first
    const cached = await cacheService.getCohort(cohortId);
    if (cached) {
      return cached;
    }

    const query = 'SELECT * FROM cohorts WHERE id = $1';
    const result = await dbPool.query(query, [cohortId]);

    if (result.rows.length === 0) {
      return null;
    }

    const cohort = this.mapRowToCohort(result.rows[0]);

    // Cache the result
    await cacheService.setCohort(cohortId, cohort);

    return cohort;
  }

  /**
   * List all cohorts with optional filtering
   */
  async listCohorts(options?: {
    status?: string;
    course?: string;
    instructor?: string;
    limit?: number;
    offset?: number;
  }): Promise<Cohort[]> {
    let query = 'SELECT * FROM cohorts WHERE 1=1';
    const params: any[] = [];
    let paramIndex = 1;

    if (options?.status) {
      query += ` AND status = $${paramIndex++}`;
      params.push(options.status);
    }

    if (options?.course) {
      query += ` AND course = $${paramIndex++}`;
      params.push(options.course);
    }

    if (options?.instructor) {
      query += ` AND instructor = $${paramIndex++}`;
      params.push(options.instructor);
    }

    query += ` ORDER BY created_at DESC`;

    if (options?.limit) {
      query += ` LIMIT $${paramIndex++}`;
      params.push(options.limit);
    }

    if (options?.offset) {
      query += ` OFFSET $${paramIndex++}`;
      params.push(options.offset);
    }

    const result = await dbPool.query(query, params);
    return result.rows.map(this.mapRowToCohort);
  }

  /**
   * Update cohort
   */
  async updateCohort(cohortId: string, updates: UpdateCohortRequest): Promise<Cohort> {
    const existing = await this.getCohort(cohortId);
    if (!existing) {
      throw new Error('Cohort not found');
    }

    const updated: Cohort = {
      ...existing,
      ...updates,
      updatedAt: Date.now(),
    };

    const query = `
      UPDATE cohorts SET
        name = $1,
        description = $2,
        course = $3,
        instructor = $4,
        start_date = $5,
        end_date = $6,
        max_students = $7,
        status = $8,
        updated_at = $9,
        metadata = $10
      WHERE id = $11
    `;

    await dbPool.query(query, [
      updated.name,
      updated.description,
      updated.course,
      updated.instructor,
      updated.startDate,
      updated.endDate,
      updated.maxStudents,
      updated.status,
      updated.updatedAt,
      JSON.stringify(updated.metadata || {}),
      cohortId,
    ]);

    logger.info('Cohort updated', { cohortId });

    // Update cache
    await cacheService.setCohort(cohortId, updated);

    return updated;
  }

  /**
   * Delete cohort
   */
  async deleteCohort(cohortId: string): Promise<void> {
    // First delete all members
    await dbPool.query('DELETE FROM cohort_members WHERE cohort_id = $1', [cohortId]);
    
    // Delete all messages
    await dbPool.query('DELETE FROM cohort_messages WHERE cohort_id = $1', [cohortId]);
    
    // Delete cohort
    await dbPool.query('DELETE FROM cohorts WHERE id = $1', [cohortId]);

    logger.info('Cohort deleted', { cohortId });

    // Invalidate cache
    await cacheService.invalidateCohorts();
  }

  /**
   * Add students to cohort
   */
  async addStudents(cohortId: string, studentAddresses: string[]): Promise<CohortMember[]> {
    const cohort = await this.getCohort(cohortId);
    if (!cohort) {
      throw new Error('Cohort not found');
    }

    if (cohort.currentStudents + studentAddresses.length > cohort.maxStudents) {
      throw new Error('Cohort is full');
    }

    const members: CohortMember[] = [];
    const now = Date.now();

    // Use transaction for atomicity
    await dbPool.transaction(async (client) => {
      for (const address of studentAddresses) {
        const member: CohortMember = {
          cohortId,
          studentAddress: address,
          enrolledAt: now,
          status: 'active',
          progress: 0,
          certificatesEarned: 0,
          lastActivity: now,
        };

        const query = `
          INSERT INTO cohort_members (
            cohort_id, student_address, enrolled_at, status,
            progress, certificates_earned, last_activity
          ) VALUES ($1, $2, $3, $4, $5, $6, $7)
          ON CONFLICT (cohort_id, student_address) DO UPDATE SET
            status = EXCLUDED.status,
            last_activity = EXCLUDED.last_activity
        `;

        await client.query(query, [
          member.cohortId,
          member.studentAddress,
          member.enrolledAt,
          member.status,
          member.progress,
          member.certificatesEarned,
          member.lastActivity,
        ]);

        members.push(member);
      }

      // Update cohort student count
      await client.query(
        'UPDATE cohorts SET current_students = current_students + $1, updated_at = $2 WHERE id = $3',
        [studentAddresses.length, now, cohortId]
      );
    });

    logger.info('Students added to cohort', {
      cohortId,
      count: studentAddresses.length,
    });

    // Invalidate cache
    await cacheService.invalidateCohorts();

    return members;
  }

  /**
   * Remove students from cohort
   */
  async removeStudents(cohortId: string, studentAddresses: string[]): Promise<void> {
    await dbPool.transaction(async (client) => {
      const query = `
        DELETE FROM cohort_members 
        WHERE cohort_id = $1 AND student_address = ANY($2)
      `;
      await client.query(query, [cohortId, studentAddresses]);

      // Update cohort student count
      await client.query(
        'UPDATE cohorts SET current_students = GREATEST(current_students - $1, 0), updated_at = $2 WHERE id = $3',
        [studentAddresses.length, Date.now(), cohortId]
      );
    });

    logger.info('Students removed from cohort', {
      cohortId,
      count: studentAddresses.length,
    });

    // Invalidate cache
    await cacheService.invalidateCohorts();
  }

  /**
   * Get cohort leaderboard
   */
  async getLeaderboard(cohortId: string): Promise<CohortLeaderboardEntry[]> {
    // Check cache first
    const cached = await cacheService.getCohortLeaderboard(cohortId);
    if (cached) {
      return cached;
    }

    const query = `
      SELECT 
        student_address,
        progress,
        certificates_earned,
        last_activity
      FROM cohort_members
      WHERE cohort_id = $1 AND status = 'active'
      ORDER BY progress DESC, certificates_earned DESC, last_activity DESC
    `;

    const result = await dbPool.query(query, [cohortId]);

    const leaderboard: CohortLeaderboardEntry[] = result.rows.map((row, index) => ({
      rank: index + 1,
      studentAddress: row.student_address,
      points: Math.round(row.progress * 10 + row.certificates_earned * 100),
      certificatesCompleted: row.certificates_earned,
      assignmentsCompleted: Math.round(row.progress),
      participationScore: Math.min(100, Math.round((Date.now() - row.last_activity) / 86400000)),
      badges: [],
    }));

    // Cache the result
    await cacheService.setCohortLeaderboard(cohortId, leaderboard);

    return leaderboard;
  }

  /**
   * Send message to cohort
   */
  async sendMessage(
    cohortId: string,
    senderAddress: string,
    content: string,
    type: CohortMessage['type'] = 'discussion',
    replyTo?: string
  ): Promise<CohortMessage> {
    const message: CohortMessage = {
      id: this.generateId(),
      cohortId,
      senderAddress,
      content,
      timestamp: Date.now(),
      type,
      replyTo,
      reactions: {},
    };

    const query = `
      INSERT INTO cohort_messages (
        id, cohort_id, sender_address, content, timestamp, type, reply_to
      ) VALUES ($1, $2, $3, $4, $5, $6, $7)
    `;

    await dbPool.query(query, [
      message.id,
      message.cohortId,
      message.senderAddress,
      message.content,
      message.timestamp,
      message.type,
      message.replyTo || null,
    ]);

    logger.info('Message sent to cohort', {
      cohortId,
      messageId: message.id,
      type,
    });

    return message;
  }

  /**
   * Get cohort messages
   */
  async getMessages(cohortId: string, options?: {
    limit?: number;
    offset?: number;
    type?: string;
  }): Promise<CohortMessage[]> {
    let query = 'SELECT * FROM cohort_messages WHERE cohort_id = $1';
    const params: any[] = [cohortId];
    let paramIndex = 2;

    if (options?.type) {
      query += ` AND type = $${paramIndex++}`;
      params.push(options.type);
    }

    query += ` ORDER BY timestamp DESC`;

    if (options?.limit) {
      query += ` LIMIT $${paramIndex++}`;
      params.push(options.limit);
    }

    if (options?.offset) {
      query += ` OFFSET $${paramIndex++}`;
      params.push(options.offset);
    }

    const result = await dbPool.query(query, params);
    return result.rows.map(this.mapRowToMessage);
  }

  /**
   * Get cohort statistics
   */
  async getStats(): Promise<CohortStats> {
    const query = `
      SELECT 
        COUNT(*) as total_cohorts,
        COUNT(*) FILTER (WHERE status = 'active') as active_cohorts,
        SUM(current_students) as total_students,
        AVG(
          (SELECT AVG(progress) FROM cohort_members WHERE cohort_members.cohort_id = cohorts.id)
        ) as average_progress
      FROM cohorts
    `;

    const result = await dbPool.query(query);
    const row = result.rows[0];

    return {
      totalCohorts: parseInt(row.total_cohorts) || 0,
      activeCohorts: parseInt(row.active_cohorts) || 0,
      totalStudents: parseInt(row.total_students) || 0,
      averageProgress: parseFloat(row.average_progress) || 0,
      completionRate: 0, // Would need historical data
      engagementRate: 0, // Would need activity tracking
    };
  }

  /**
   * Get comprehensive cohort analytics
   */
  async getCohortAnalytics(cohortId: string): Promise<CohortAnalytics> {
    const cohort = await this.getCohort(cohortId);
    if (!cohort) {
      throw new Error('Cohort not found');
    }

    const [stats, leaderboard, recentActivity] = await Promise.all([
      this.getStats(),
      this.getLeaderboard(cohortId),
      this.getMessages(cohortId, { limit: 10 }),
    ]);

    return {
      cohort,
      stats,
      leaderboard,
      recentActivity,
    };
  }

  // Helper methods

  private generateId(): string {
    return `cohort_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private mapRowToCohort(row: any): Cohort {
    return {
      id: row.id,
      name: row.name,
      description: row.description,
      course: row.course,
      instructor: row.instructor,
      startDate: row.start_date,
      endDate: row.end_date,
      maxStudents: row.max_students,
      currentStudents: row.current_students,
      status: row.status,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
      metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
    };
  }

  private mapRowToMessage(row: any): CohortMessage {
    return {
      id: row.id,
      cohortId: row.cohort_id,
      senderAddress: row.sender_address,
      content: row.content,
      timestamp: row.timestamp,
      type: row.type,
      replyTo: row.reply_to,
      reactions: row.reactions ? JSON.parse(row.reactions) : {},
    };
  }
}

// Singleton instance
export const cohortService = new CohortService();
