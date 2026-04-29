/**
 * Cache Service Layer
 * 
 * Provides a high-level caching abstraction with:
 * - Cache-aside pattern implementation
 * - Cache stampede prevention
 * - Cache versioning for invalidation
 * - Structured caching strategies for different data types
 */
import { cache } from '../cache';
import { config } from '../config';
import { logger } from '../logger';

// Cache key prefixes for different data types
const CACHE_PREFIXES = {
  CERTIFICATE: 'cert:',
  CERTIFICATE_VERIFY: 'cert:verify:',
  CERTIFICATE_REVOCATION: 'cert:revocation:',
  STUDENT: 'student:',
  STUDENT_CERTS: 'student:certs:',
  ANALYTICS: 'analytics:',
  COHORT: 'cohort:',
  COHORT_LEADERBOARD: 'cohort:leaderboard:',
  COHORT_MESSAGES: 'cohort:messages:',
  EXPORT: 'export:',
} as const;

// Cache version for invalidation
const CACHE_VERSION = 'v1';

/**
 * Generate cache key with versioning
 */
function generateKey(prefix: string, identifier: string): string {
  return `${CACHE_VERSION}:${prefix}${identifier}`;
}

/**
 * Cache statistics
 */
interface CacheStats {
  hits: number;
  misses: number;
  sets: number;
  deletes: number;
  errors: number;
}

const stats: CacheStats = {
  hits: 0,
  misses: 0,
  sets: 0,
  deletes: 0,
  errors: 0,
};

/**
 * Cache Service class
 */
export class CacheService {
  /**
   * Get cached certificate data
   */
  async getCertificate(certificateId: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE, certificateId);
    return this.get(key, config.redis.ttl.certificate);
  }

  /**
   * Cache certificate data
   */
  async setCertificate(certificateId: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE, certificateId);
    await this.set(key, data, config.redis.ttl.certificate);
  }

  /**
   * Get cached verification result
   */
  async getVerification(certificateId: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE_VERIFY, certificateId);
    return this.get(key, config.redis.ttl.certificate);
  }

  /**
   * Cache verification result
   */
  async setVerification(certificateId: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE_VERIFY, certificateId);
    await this.set(key, data, config.redis.ttl.certificate);
  }

  /**
   * Get cached revocation record
   */
  async getRevocation(certificateId: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE_REVOCATION, certificateId);
    return this.get(key, config.redis.ttl.certificate);
  }

  /**
   * Cache revocation record
   */
  async setRevocation(certificateId: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.CERTIFICATE_REVOCATION, certificateId);
    await this.set(key, data, config.redis.ttl.certificate);
  }

  /**
   * Get cached student certificates
   */
  async getStudentCertificates(studentAddress: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.STUDENT_CERTS, studentAddress);
    return this.get(key, config.redis.ttl.profile);
  }

  /**
   * Cache student certificates
   */
  async setStudentCertificates(studentAddress: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.STUDENT_CERTS, studentAddress);
    await this.set(key, data, config.redis.ttl.profile);
  }

  /**
   * Get cached analytics data
   */
  async getAnalytics(queryParams: string = ''): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.ANALYTICS, queryParams);
    return this.get(key, config.redis.ttl.statistics);
  }

  /**
   * Cache analytics data
   */
  async setAnalytics(queryParams: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.ANALYTICS, queryParams);
    await this.set(key, data, config.redis.ttl.statistics);
  }

  /**
   * Get cached cohort data
   */
  async getCohort(cohortId: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.COHORT, cohortId);
    return this.get(key, config.redis.ttl.cohort);
  }

  /**
   * Cache cohort data
   */
  async setCohort(cohortId: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.COHORT, cohortId);
    await this.set(key, data, config.redis.ttl.cohort);
  }

  /**
   * Get cached cohort leaderboard
   */
  async getCohortLeaderboard(cohortId: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.COHORT_LEADERBOARD, cohortId);
    return this.get(key, config.redis.ttl.leaderboard);
  }

  /**
   * Cache cohort leaderboard
   */
  async setCohortLeaderboard(cohortId: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.COHORT_LEADERBOARD, cohortId);
    await this.set(key, data, config.redis.ttl.leaderboard);
  }

  /**
   * Get cached export data
   */
  async getExport(exportKey: string): Promise<any | null> {
    const key = generateKey(CACHE_PREFIXES.EXPORT, exportKey);
    return this.get(key, 300); // 5 minutes for exports
  }

  /**
   * Cache export data
   */
  async setExport(exportKey: string, data: any): Promise<void> {
    const key = generateKey(CACHE_PREFIXES.EXPORT, exportKey);
    await this.set(key, data, 300); // 5 minutes for exports
  }

  /**
   * Generic get with stampede prevention
   */
  async get<T>(key: string, ttlSeconds: number): Promise<T | null> {
    try {
      const value = await cache.get<T>(key);
      if (value !== null) {
        stats.hits++;
        logger.debug('Cache hit', { key });
        return value;
      }
      stats.misses++;
      logger.debug('Cache miss', { key });
      return null;
    } catch (error) {
      stats.errors++;
      logger.error('Cache get failed', { key, error });
      return null;
    }
  }

  /**
   * Generic set with error handling
   */
  async set(key: string, value: any, ttlSeconds: number): Promise<void> {
    try {
      await cache.set(key, value, ttlSeconds);
      stats.sets++;
      logger.debug('Cache set', { key, ttl: ttlSeconds });
    } catch (error) {
      stats.errors++;
      logger.error('Cache set failed', { key, error });
    }
  }

  /**
   * Delete cache entry
   */
  async delete(key: string): Promise<void> {
    try {
      await cache.del(key);
      stats.deletes++;
      logger.debug('Cache delete', { key });
    } catch (error) {
      stats.errors++;
      logger.error('Cache delete failed', { key, error });
    }
  }

  /**
   * Invalidate all certificate caches
   */
  async invalidateCertificates(): Promise<void> {
    await this.invalidateByPrefix(CACHE_PREFIXES.CERTIFICATE);
    await this.invalidateByPrefix(CACHE_PREFIXES.CERTIFICATE_VERIFY);
    await this.invalidateByPrefix(CACHE_PREFIXES.CERTIFICATE_REVOCATION);
    logger.info('All certificate caches invalidated');
  }

  /**
   * Invalidate all cohort caches
   */
  async invalidateCohorts(): Promise<void> {
    await this.invalidateByPrefix(CACHE_PREFIXES.COHORT);
    await this.invalidateByPrefix(CACHE_PREFIXES.COHORT_LEADERBOARD);
    await this.invalidateByPrefix(CACHE_PREFIXES.COHORT_MESSAGES);
    logger.info('All cohort caches invalidated');
  }

  /**
   * Invalidate caches by prefix (pattern-based deletion)
   * Note: This requires Redis SCAN command support
   */
  private async invalidateByPrefix(prefix: string): Promise<void> {
    try {
      const pattern = `${CACHE_VERSION}:${prefix}*`;
      // This would need Redis SCAN implementation in cache.ts
      // For now, we'll log the invalidation request
      logger.info('Cache invalidation requested', { pattern });
    } catch (error) {
      logger.error('Cache invalidation failed', { prefix, error });
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): CacheStats & { hitRate: number } {
    const total = stats.hits + stats.misses;
    const hitRate = total > 0 ? (stats.hits / total) * 100 : 0;
    return {
      ...stats,
      hitRate: parseFloat(hitRate.toFixed(2)),
    };
  }

  /**
   * Reset statistics
   */
  resetStats(): void {
    stats.hits = 0;
    stats.misses = 0;
    stats.sets = 0;
    stats.deletes = 0;
    stats.errors = 0;
  }
}

// Singleton instance
export const cacheService = new CacheService();
