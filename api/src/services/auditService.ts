/**
 * Audit Logging Service
 * 
 * Comprehensive audit logging for all employer verification activities
 * Provides detailed tracking for compliance and security monitoring
 */
import { logger } from "../logger";
import { promises as fs } from 'fs';
import path from 'path';

export interface VerificationAttemptLog {
  employerId: string;
  certificateId: string;
  studentAddress: string;
  verificationLevel: string;
  timestamp: string;
  ipAddress: string;
  userAgent?: string;
}

export interface VerificationSuccessLog {
  employerId: string;
  certificateId: string;
  studentAddress: string;
  verificationLevel: string;
  result: string;
  duration: number;
  timestamp: string;
}

export interface VerificationFailureLog {
  employerId: string;
  certificateId: string;
  studentAddress: string;
  verificationLevel: string;
  error: string;
  duration: number;
  timestamp: string;
}

export interface BatchVerificationAttemptLog {
  employerId: string;
  batchSize: number;
  verificationLevel: string;
  timestamp: string;
  ipAddress: string;
  userAgent?: string;
}

export interface BatchVerificationCompletionLog {
  employerId: string;
  batchSize: number;
  successCount: number;
  failureCount: number;
  verificationLevel: string;
  duration: number;
  timestamp: string;
}

export interface AuthenticationSuccessLog {
  employerId: string;
  ip: string;
  userAgent?: string;
  duration: number;
  timestamp: string;
}

export interface AuthenticationFailureLog {
  ip: string;
  userAgent?: string;
  reason: string;
  timestamp: string;
}

export interface AuditLogEntry {
  type: 'verification_attempt' | 'verification_success' | 'verification_failure' | 
       'batch_attempt' | 'batch_completion' | 'auth_success' | 'auth_failure';
  data: any;
  timestamp: string;
  id: string;
}

class AuditLogger {
  private logDir: string;
  private maxFileSize: number = 10 * 1024 * 1024; // 10MB
  private maxFiles: number = 100;

  constructor() {
    this.logDir = process.env.AUDIT_LOG_DIR || path.join(process.cwd(), 'logs', 'audit');
    this.ensureLogDirectory();
  }

  private async ensureLogDirectory(): Promise<void> {
    try {
      await fs.mkdir(this.logDir, { recursive: true });
    } catch (error) {
      logger.error("Failed to create audit log directory", { error, logDir: this.logDir });
    }
  }

  private async writeLog(entry: AuditLogEntry): Promise<void> {
    const date = new Date(entry.timestamp);
    const fileName = `audit-${date.toISOString().split('T')[0]}.jsonl`;
    const filePath = path.join(this.logDir, fileName);

    try {
      // Check file size and rotate if necessary
      await this.rotateLogFileIfNeeded(filePath);

      const logLine = JSON.stringify(entry) + '\n';
      await fs.appendFile(filePath, logLine, 'utf8');

      // Also log to application logger for immediate visibility
      logger.info("Audit log entry", {
        type: entry.type,
        id: entry.id,
        timestamp: entry.timestamp,
        data: entry.data
      });
    } catch (error) {
      logger.error("Failed to write audit log", { error, entry, filePath });
    }
  }

  private async rotateLogFileIfNeeded(filePath: string): Promise<void> {
    try {
      const stats = await fs.stat(filePath);
      if (stats.size > this.maxFileSize) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const rotatedPath = filePath.replace('.jsonl', `-${timestamp}.jsonl`);
        await fs.rename(filePath, rotatedPath);
        
        // Clean up old files
        await this.cleanupOldFiles();
      }
    } catch (error) {
      // File doesn't exist, which is fine
      if ((error as any).code !== 'ENOENT') {
        logger.error("Failed to rotate log file", { error, filePath });
      }
    }
  }

  private async cleanupOldFiles(): Promise<void> {
    try {
      const files = await fs.readdir(this.logDir);
      const auditFiles = files
        .filter(f => f.startsWith('audit-') && f.endsWith('.jsonl'))
        .map(f => ({
          name: f,
          path: path.join(this.logDir, f)
        }));

      if (auditFiles.length > this.maxFiles) {
        // Sort by modification time and remove oldest
        const fileStats = await Promise.all(
          auditFiles.map(async f => ({
            ...f,
            stats: await fs.stat(f.path)
          }))
        );

        fileStats.sort((a, b) => a.stats.mtime.getTime() - b.stats.mtime.getTime());
        
        const filesToDelete = fileStats.slice(0, fileStats.length - this.maxFiles);
        await Promise.all(
          filesToDelete.map(f => fs.unlink(f.path))
        );
      }
    } catch (error) {
      logger.error("Failed to cleanup old audit files", { error });
    }
  }

  private generateLogId(): string {
    return `audit_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  async logVerificationAttempt(data: VerificationAttemptLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'verification_attempt',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logVerificationSuccess(data: VerificationSuccessLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'verification_success',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logVerificationFailure(data: VerificationFailureLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'verification_failure',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logBatchVerificationAttempt(data: BatchVerificationAttemptLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'batch_attempt',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logBatchVerificationCompletion(data: BatchVerificationCompletionLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'batch_completion',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logAuthenticationSuccess(data: AuthenticationSuccessLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'auth_success',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async logAuthenticationFailure(data: AuthenticationFailureLog): Promise<void> {
    const entry: AuditLogEntry = {
      type: 'auth_failure',
      data,
      timestamp: data.timestamp,
      id: this.generateLogId()
    };
    await this.writeLog(entry);
  }

  async getEmployerVerificationHistory(employerId: string, limit: number = 100, offset: number = 0): Promise<any[]> {
    const logs: any[] = [];
    const date = new Date();
    
    // Search through recent audit files
    for (let i = 0; i < 30; i++) { // Search last 30 days
      const searchDate = new Date(date);
      searchDate.setDate(date.getDate() - i);
      const fileName = `audit-${searchDate.toISOString().split('T')[0]}.jsonl`;
      const filePath = path.join(this.logDir, fileName);

      try {
        const content = await fs.readFile(filePath, 'utf8');
        const lines = content.trim().split('\n').filter(line => line);
        
        for (const line of lines) {
          try {
            const entry = JSON.parse(line);
            if (
              (entry.type === 'verification_success' || entry.type === 'verification_failure') &&
              entry.data.employerId === employerId
            ) {
              logs.push(entry);
            }
          } catch (parseError) {
            // Skip malformed lines
            continue;
          }
        }
      } catch (error) {
        // File doesn't exist, continue
        continue;
      }
    }

    // Sort by timestamp (newest first) and apply pagination
    logs.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
    return logs.slice(offset, offset + limit);
  }

  async getAuditStats(employerId?: string, startDate?: Date, endDate?: Date): Promise<any> {
    const stats = {
      totalVerifications: 0,
      successfulVerifications: 0,
      failedVerifications: 0,
      batchVerifications: 0,
      authenticationAttempts: 0,
      authenticationSuccesses: 0,
      authenticationFailures: 0,
      averageVerificationTime: 0,
      verificationByLevel: {} as Record<string, number>,
      errorsByType: {} as Record<string, number>
    };

    const date = new Date();
    const searchDays = 30; // Default to last 30 days
    
    for (let i = 0; i < searchDays; i++) {
      const searchDate = new Date(date);
      searchDate.setDate(date.getDate() - i);
      const fileName = `audit-${searchDate.toISOString().split('T')[0]}.jsonl`;
      const filePath = path.join(this.logDir, fileName);

      try {
        const content = await fs.readFile(filePath, 'utf8');
        const lines = content.trim().split('\n').filter(line => line);
        
        for (const line of lines) {
          try {
            const entry = JSON.parse(line);
            
            // Filter by employer if specified
            if (employerId && entry.data.employerId && entry.data.employerId !== employerId) {
              continue;
            }

            // Filter by date range if specified
            const entryDate = new Date(entry.timestamp);
            if (startDate && entryDate < startDate) continue;
            if (endDate && entryDate > endDate) continue;

            switch (entry.type) {
              case 'verification_success':
                stats.successfulVerifications++;
                stats.totalVerifications++;
                if (entry.data.verificationLevel) {
                  stats.verificationByLevel[entry.data.verificationLevel] = 
                    (stats.verificationByLevel[entry.data.verificationLevel] || 0) + 1;
                }
                break;
              case 'verification_failure':
                stats.failedVerifications++;
                stats.totalVerifications++;
                if (entry.data.error) {
                  stats.errorsByType[entry.data.error] = 
                    (stats.errorsByType[entry.data.error] || 0) + 1;
                }
                break;
              case 'batch_attempt':
                stats.batchVerifications++;
                break;
              case 'auth_success':
                stats.authenticationSuccesses++;
                stats.authenticationAttempts++;
                break;
              case 'auth_failure':
                stats.authenticationFailures++;
                stats.authenticationAttempts++;
                break;
            }
          } catch (parseError) {
            continue;
          }
        }
      } catch (error) {
        continue;
      }
    }

    return stats;
  }
}

export const auditLogger = new AuditLogger();
