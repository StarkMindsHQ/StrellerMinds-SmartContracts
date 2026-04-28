/**
 * Database Connection Pool Manager
 * 
 * Implements a robust PostgreSQL connection pool with:
 * - Connection leak detection and prevention
 * - Pool monitoring and health checks
 * - Automatic connection recycling
 * - Graceful shutdown handling
 */
import { Pool, PoolConfig, PoolClient, QueryResult } from 'pg';
import { config } from '../config';
import { logger } from '../logger';

interface PoolMetrics {
  totalCount: number;
  idleCount: number;
  waitingCount: number;
  activeCount: number;
  maxUsesReached: number;
  leakDetected: number;
}

class DatabasePoolManager {
  private pool: Pool | null = null;
  private metrics: PoolMetrics = {
    totalCount: 0,
    idleCount: 0,
    waitingCount: 0,
    activeCount: 0,
    maxUsesReached: 0,
    leakDetected: 0,
  };
  private healthCheckInterval: NodeJS.Timeout | null = null;
  private isShuttingDown = false;

  /**
   * Initialize the connection pool with configuration
   */
  async initialize(): Promise<void> {
    if (this.pool) {
      logger.warn('Database pool already initialized');
      return;
    }

    const poolConfig: PoolConfig = {
      host: config.database.host,
      port: config.database.port,
      database: config.database.name,
      user: config.database.user,
      password: config.database.password,
      min: config.database.pool.min,
      max: config.database.pool.max,
      idleTimeoutMillis: config.database.pool.idleTimeoutMillis,
      connectionTimeoutMillis: config.database.pool.connectionTimeoutMillis,
      maxUses: config.database.pool.maxUses,
      // Enable leak detection
      acquireTimeoutMillis: config.database.pool.leakDetectionThreshold,
      // SSL configuration for production
      ssl: config.nodeEnv === 'production' ? { rejectUnauthorized: false } : false,
      // Statement timeout to prevent long-running queries
      statement_timeout: 30000, // 30 seconds
    };

    try {
      this.pool = new Pool(poolConfig);

      // Event handlers for pool monitoring
      this.pool.on('connect', () => {
        this.metrics.totalCount++;
        logger.debug('New database connection established');
      });

      this.pool.on('acquire', () => {
        this.metrics.activeCount++;
      });

      this.pool.on('remove', () => {
        this.metrics.totalCount--;
        this.metrics.activeCount = Math.max(0, this.metrics.activeCount - 1);
      });

      this.pool.on('error', (err) => {
        logger.error('Database pool error', { error: err });
      });

      // Test the connection
      const client = await this.pool.connect();
      await client.query('SELECT NOW()');
      client.release();

      logger.info('Database pool initialized successfully', {
        min: poolConfig.min,
        max: poolConfig.max,
        idleTimeout: poolConfig.idleTimeoutMillis,
        connectionTimeout: poolConfig.connectionTimeoutMillis,
      });

      // Start health check monitoring
      this.startHealthChecks();

    } catch (error) {
      logger.error('Failed to initialize database pool', { error });
      throw error;
    }
  }

  /**
   * Get a client from the pool with leak detection
   */
  async getClient(): Promise<PoolClient> {
    if (!this.pool) {
      throw new Error('Database pool not initialized');
    }

    if (this.isShuttingDown) {
      throw new Error('Database pool is shutting down');
    }

    const client = await this.pool.connect();
    const acquisitionTime = Date.now();

    // Set up leak detection timeout
    const leakTimeout = setTimeout(() => {
      this.metrics.leakDetected++;
      logger.warn('Potential connection leak detected', {
        acquisitionTime: new Date(acquisitionTime).toISOString(),
        heldFor: `${Date.now() - acquisitionTime}ms`,
      });
    }, config.database.pool.leakDetectionThreshold);

    // Override release to clear the timeout
    const originalRelease = client.release.bind(client);
    client.release = (err?: Error | boolean) => {
      clearTimeout(leakTimeout);
      this.metrics.activeCount = Math.max(0, this.metrics.activeCount - 1);
      return originalRelease(err as any);
    };

    return client;
  }

  /**
   * Execute a query with automatic client management
   */
  async query<T = any>(text: string, params?: any[]): Promise<QueryResult<T>> {
    const client = await this.getClient();
    try {
      const result = await client.query<T>(text, params);
      return result;
    } finally {
      client.release();
    }
  }

  /**
   * Execute a transaction with automatic rollback on error
   */
  async transaction<T>(callback: (client: PoolClient) => Promise<T>): Promise<T> {
    const client = await this.getClient();
    try {
      await client.query('BEGIN');
      const result = await callback(client);
      await client.query('COMMIT');
      return result;
    } catch (error) {
      await client.query('ROLLBACK');
      logger.error('Transaction failed, rolled back', { error });
      throw error;
    } finally {
      client.release();
    }
  }

  /**
   * Get current pool metrics
   */
  getMetrics(): PoolMetrics {
    if (!this.pool) {
      return this.metrics;
    }

    return {
      totalCount: this.pool.totalCount,
      idleCount: this.pool.idleCount,
      waitingCount: this.pool.waitingCount || 0,
      activeCount: this.pool.totalCount - this.pool.idleCount,
      maxUsesReached: this.metrics.maxUsesReached,
      leakDetected: this.metrics.leakDetected,
    };
  }

  /**
   * Health check - verify pool is operational
   */
  async healthCheck(): Promise<boolean> {
    if (!this.pool) {
      return false;
    }

    try {
      const result = await this.query('SELECT 1 as health_check');
      return result.rows.length === 1 && result.rows[0].health_check === 1;
    } catch (error) {
      logger.error('Database health check failed', { error });
      return false;
    }
  }

  /**
   * Start periodic health checks
   */
  private startHealthChecks(): void {
    this.healthCheckInterval = setInterval(async () => {
      const isHealthy = await this.healthCheck();
      const metrics = this.getMetrics();

      logger.debug('Database pool health check', {
        healthy: isHealthy,
        metrics,
      });

      // Alert if pool is nearly exhausted
      if (metrics.activeCount > config.database.pool.max * 0.9) {
        logger.warn('Database pool nearly exhausted', {
          activeCount: metrics.activeCount,
          max: config.database.pool.max,
          utilization: `${((metrics.activeCount / config.database.pool.max) * 100).toFixed(1)}%`,
        });
      }
    }, 30000); // Check every 30 seconds
  }

  /**
   * Gracefully shutdown the pool
   */
  async shutdown(): Promise<void> {
    this.isShuttingDown = true;

    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
      this.healthCheckInterval = null;
    }

    if (this.pool) {
      try {
        await this.pool.end();
        logger.info('Database pool shut down successfully');
      } catch (error) {
        logger.error('Error shutting down database pool', { error });
      } finally {
        this.pool = null;
      }
    }
  }

  /**
   * Get pool instance (for advanced usage)
   */
  getPool(): Pool | null {
    return this.pool;
  }
}

// Singleton instance
export const dbPool = new DatabasePoolManager();
