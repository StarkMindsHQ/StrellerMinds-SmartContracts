# Migration Guide: StrellerMinds Smart Contracts v1 to v2

## Overview

This comprehensive migration guide provides step-by-step instructions for upgrading from StrellerMinds Smart Contracts version 1 to version 2. This migration includes significant architectural improvements, enhanced security features, and new functionality for blockchain-based education platforms.

## Table of Contents

1. [Breaking Changes](#breaking-changes)
2. [Data Migration Steps](#data-migration-steps)
3. [API Changes](#api-changes)
4. [Configuration Updates](#configuration-updates)
5. [Rollback Procedures](#rollback-procedures)
6. [Testing and Validation](#testing-and-validation)
7. [Troubleshooting](#troubleshooting)

---

## Breaking Changes

### 🔥 Critical Breaking Changes

#### 1. **User Management System Restructure**
- **Impact**: High - Core user authentication and management
- **Change**: Complete overhaul of user table structure with enhanced security fields
- **Migration Required**: Yes - Data migration script provided

**Before (v1):**
```sql
users table with basic fields (id, email, username, password, basic profile)
```

**After (v2):**
```sql
users table with enhanced fields:
- Two-factor authentication support
- Advanced permission system (JSON field)
- Account locking mechanisms
- Comprehensive audit trails
- Enhanced metadata support
```

#### 2. **Learning Path Architecture**
- **Impact**: High - New learning management system
- **Change**: Introduction of comprehensive learning path system with node-based structure
- **Migration Required**: Yes - New tables and relationships

**New Features in v2:**
- Learning path templates
- Node dependencies and prerequisites
- Progress tracking with detailed metrics
- Adaptive learning paths
- Assessment integration

#### 3. **Payment System Enhancement**
- **Impact**: Medium - Financial transactions
- **Change**: Enhanced payment processing with dispute resolution and tax handling
- **Migration Required**: Yes - New payment-related tables

#### 4. **Database Schema Changes**
- **Impact**: High - All database operations
- **Change**: Migration from basic schema to comprehensive educational platform schema
- **Migration Required**: Yes - Full schema migration

---

## Data Migration Steps

### 📋 Prerequisites

1. **Backup Current Database**
   ```bash
   # Create full database backup
   pg_dump strellerminds_v1 > strellerminds_v1_backup_$(date +%Y%m%d_%H%M%S).sql
   
   # Verify backup integrity
   pg_restore --list strellerminds_v1_backup_*.sql | head -20
   ```

2. **Environment Preparation**
   ```bash
   # Stop v1 services
   npm run stop:production
   
   # Set maintenance mode
   export MAINTENANCE_MODE=true
   
   # Verify system requirements
   node --version  # Should be v18+
   npm --version   # Should be v8+
   psql --version  # Should be v12+
   ```

3. **Install Migration Dependencies**
   ```bash
   # Install migration tools
   npm install -g typeorm-cli
   npm install @types/node typescript ts-node
   
   # Verify migration scripts
   ls -la src/migrations/
   ```

### 🔄 Step-by-Step Migration

#### Phase 1: Core System Migration

1. **User Tables Migration**
   ```bash
   # Run user table migrations
   npm run migration:run -- 1700000000000-create-user-tables.ts
   
   # Verify user data integrity
   npm run migration:verify -- users
   ```

2. **User Profiles Enhancement**
   ```bash
   # Migrate user profiles
   npm run migration:run -- 1704800000000-create-user-profiles.ts
   
   # Validate profile data
   npm run migration:validate -- user_profiles
   ```

#### Phase 2: Educational Content Migration

3. **Course System Migration**
   ```bash
   # Migrate course tables
   npm run migration:run -- 1735000000000-create-course-tables.ts
   
   # Verify course data
   npm run migration:verify -- courses
   ```

4. **Learning Path System**
   ```bash
   # Create learning path infrastructure
   npm run migration:run -- 1735400000000-create-learning-path-tables.ts
   
   # Migrate existing course progress to learning paths
   npm run migration:custom -- migrate-course-progress-to-learning-paths
   ```

#### Phase 3: Advanced Features Migration

5. **Payment and Financial Systems**
   ```bash
   # Migrate payment tables
   npm run migration:run -- 1704890000000-create-payment-tables.ts
   npm run migration:run -- 1704900000000-create-refund-dispute-tax-tables.ts
   
   # Validate financial data
   npm run migration:audit -- payments
   ```

6. **Gamification and Engagement**
   ```bash
   # Migrate gamification system
   npm run migration:run -- 1735100000000-create-gamification-tables.ts
   
   # Initialize gamification data
   npm run migration:seed -- gamification
   ```

#### Phase 4: Integration and Monitoring

7. **Integration Tables**
   ```bash
   # Create integration infrastructure
   npm run migration:run -- 1704900000000-create-integration-tables.ts
   ```

8. **Performance Monitoring**
   ```bash
   # Add monitoring tables
   npm run migration:run -- 1709000000000-CreatePerformanceMonitoringTables.ts
   npm run migration:run -- 1735300000000-add-performance-indexes.ts
   ```

### 📊 Data Validation

After each migration phase, run validation scripts:

```bash
# Comprehensive data validation
npm run migration:validate-all

# Check data integrity
npm run migration:integrity-check

# Generate migration report
npm run migration:report > migration_report_$(date +%Y%m%d).json
```

---

## API Changes

### 🔄 Authentication API Changes

#### v1 Endpoints (Deprecated)
```http
POST /api/v1/auth/login
POST /api/v1/auth/register
GET  /api/v1/auth/profile
```

#### v2 Endpoints (Enhanced)
```http
POST /api/v2/auth/login
POST /api/v2/auth/register
POST /api/v2/auth/2fa/setup
POST /api/v2/auth/2fa/verify
GET  /api/v2/auth/profile
PUT  /api/v2/auth/profile
POST /api/v2/auth/logout
```

**Breaking Changes:**
- Two-factor authentication now required for admin accounts
- Enhanced password validation (minimum 12 characters, complexity requirements)
- Session management with refresh tokens
- Account locking after failed attempts

### 👥 User Management API Changes

#### Enhanced User Endpoints
```http
GET    /api/v2/users                    # Paginated user listing
GET    /api/v2/users/:id               # Enhanced user profile
PUT    /api/v2/users/:id               # Update user with validation
DELETE /api/v2/users/:id               # Soft delete with audit
POST   /api/v2/users/bulk-update       # Bulk user operations
GET    /api/v2/users/activities        # User activity logs
```

**New Features:**
- Advanced filtering and sorting
- Bulk operations support
- Activity tracking
- Permission management

### 📚 Education API Changes

#### New Learning Path Endpoints
```http
GET    /api/v2/learning-paths          # List learning paths
POST   /api/v2/learning-paths          # Create learning path
GET    /api/v2/learning-paths/:id      # Get learning path details
PUT    /api/v2/learning-paths/:id      # Update learning path
DELETE /api/v2/learning-paths/:id      # Delete learning path
POST   /api/v2/learning-paths/:id/enroll # Enroll in learning path
GET    /api/v2/learning-paths/:id/progress # Get progress
```

#### Enhanced Course Management
```http
GET    /api/v2/courses                 # Enhanced course listing
POST   /api/v2/courses                 # Create course with validation
PUT    /api/v2/courses/:id             # Update course
POST   /api/v2/courses/:id/publish     # Publish course
GET    /api/v2/courses/:id/analytics   # Course analytics
```

### 💳 Payment API Changes

#### Enhanced Payment Processing
```http
POST   /api/v2/payments/process        # Process payment with enhanced validation
GET    /api/v2/payments/:id            # Get payment details
POST   /api/v2/payments/:id/refund     # Process refunds
GET    /api/v2/payments/history        # Payment history with filtering
POST   /api/v2/disputes/create         # Create dispute
GET    /api/v2/disputes/:id            # Get dispute details
```

---

## Configuration Updates

### 🔧 Environment Variables

#### New Required Variables (v2)
```env
# Enhanced Security
TWO_FACTOR_AUTH_ENABLED=true
SESSION_SECRET_MIN_LENGTH=32
ACCOUNT_LOCKOUT_THRESHOLD=5
ACCOUNT_LOCKOUT_DURATION=900

# Learning Path Configuration
LEARNING_PATH_MAX_NODES=50
LEARNING_PATH_DEFAULT_TYPE=linear
ADAPTIVE_LEARNING_ENABLED=true

# Performance Monitoring
PERFORMANCE_MONITORING_ENABLED=true
METRICS_RETENTION_DAYS=30
SLOW_QUERY_THRESHOLD_MS=1000

# Enhanced Database Configuration
DATABASE_POOL_MIN=5
DATABASE_POOL_MAX=20
DATABASE_QUERY_TIMEOUT=30000
DATABASE_CONNECTION_TIMEOUT=10000

# File Storage Enhancement
MAX_FILE_SIZE_MB=100
ALLOWED_FILE_TYPES=pdf,doc,docx,ppt,pptx,mp4,avi
FILE_STORAGE_ENCRYPTION=true

# Integration Settings
WEBHOOK_SECRET_REQUIRED=true
INTEGRATION_TIMEOUT_MS=5000
RATE_LIMITING_ENABLED=true
```

#### Updated Variable Formats
```env
# v1 Format
JWT_SECRET=mysecret

# v2 Format (Enhanced)
JWT_SECRET=your_super_secure_jwt_key_here_minimum_32_characters
JWT_EXPIRES_IN=24h
JWT_REFRESH_SECRET=your_refresh_token_secret_here
JWT_REFRESH_EXPIRES_IN=7d
```

### 🗄️ Database Configuration Updates

#### Connection Pool Settings
```typescript
// v1 Configuration
{
  type: 'postgres',
  host: 'localhost',
  port: 5432,
  username: 'postgres',
  password: 'password',
  database: 'strellerminds'
}

// v2 Configuration (Enhanced)
{
  type: 'postgres',
  host: process.env.DATABASE_HOST,
  port: parseInt(process.env.DATABASE_PORT) || 5432,
  username: process.env.DATABASE_USER,
  password: process.env.DATABASE_PASSWORD,
  database: process.env.DATABASE_NAME,
  ssl: process.env.NODE_ENV === 'production',
  logging: process.env.NODE_ENV === 'development',
  entities: ['dist/**/*.entity{.ts,.js}'],
  migrations: ['src/migrations/*.ts'],
  synchronize: false,
  migrationsRun: false,
  dropSchema: false,
  extra: {
    max: parseInt(process.env.DATABASE_POOL_MAX) || 20,
    min: parseInt(process.env.DATABASE_POOL_MIN) || 5,
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 10000,
  }
}
```

### 🚀 Application Configuration

#### NestJS Module Updates
```typescript
// v1 App Module
@Module({
  imports: [
    AuthModule,
    UserModule,
    CourseModule,
  ],
})

// v2 App Module (Enhanced)
@Module({
  imports: [
    AuthModule,
    UserModule,
    CourseModule,
    LearningPathModule,
    PaymentModule,
    GamificationModule,
    MonitoringModule,
    IntegrationModule,
    AccessibilityModule,
    VideoModule,
  ],
  providers: [
    {
      provide: APP_GUARD,
      useClass: ThrottlerGuard,
    },
    {
      provide: APP_FILTER,
      useClass: AllExceptionsFilter,
    },
  ],
})
```

---

## Rollback Procedures

### ⚠️ Emergency Rollback

#### Immediate Rollback (Critical Issues)
```bash
# 1. Stop v2 services immediately
npm run stop:production

# 2. Restore v1 database
pg_restore --clean --if-exists --verbose \
  -h localhost -U postgres -d strellerminds_v1 \
  strellerminds_v1_backup_YYYYMMDD_HHMMSS.sql

# 3. Switch to v1 codebase
git checkout v1.0.0
npm install
npm run build

# 4. Start v1 services
npm run start:prod

# 5. Verify system functionality
curl -f http://localhost:3000/health || exit 1
```

#### Controlled Rollback (Planned)
```bash
# 1. Create v2 data backup
pg_dump strellerminds_v2 > rollback_backup_v2_$(date +%Y%m%d_%H%M%S).sql

# 2. Run rollback migrations in reverse order
npm run migration:revert -- 1769400000000-create-backup-tables.ts
npm run migration:revert -- 1769345000000-create-video-tables.ts
npm run migration:revert -- 1735400000000-create-learning-path-tables.ts
# ... continue in reverse chronological order

# 3. Verify rollback success
npm run migration:verify-rollback

# 4. Restart services
npm run restart:production
```

### 🔄 Point-in-Time Recovery

#### Using Database Backups
```bash
# 1. Identify rollback point
ls -la backups/ | grep strellerminds_v1_backup

# 2. Restore specific backup
pg_restore --clean --if-exists \
  -h localhost -U postgres -d strellerminds \
  backups/strellerminds_v1_backup_20240101_120000.sql

# 3. Verify data integrity
npm run migration:integrity-check
```

#### Application Rollback
```bash
# 1. Checkout v1 tag
git checkout tags/v1.0.0

# 2. Install v1 dependencies
npm ci --production

# 3. Build v1 application
npm run build

# 4. Start v1 services
npm run start:prod
```

### 📋 Rollback Validation Checklist

- [ ] Database restored successfully
- [ ] All v1 services running
- [ ] API endpoints responding correctly
- [ ] User authentication working
- [ ] Data integrity verified
- [ ] Performance metrics within acceptable range
- [ ] No error logs in system
- [ ] Monitoring dashboards showing normal activity

---

## Testing and Validation

### 🧪 Pre-Migration Testing

#### 1. **Environment Setup Testing**
```bash
# Test v2 environment setup
npm run test:environment

# Verify database connectivity
npm run test:database-connection

# Validate configuration
npm run test:config-validation
```

#### 2. **Migration Script Testing**
```bash
# Test migration scripts on staging database
npm run test:migration -- --env=staging

# Validate migration data integrity
npm run test:migration-integrity

# Performance test migration scripts
npm run test:migration-performance
```

#### 3. **API Compatibility Testing**
```bash
# Test v2 API endpoints
npm run test:api:v2

# Test backward compatibility
npm run test:api:compatibility

# Load testing new endpoints
npm run test:api:load
```

### 🔄 Post-Migration Validation

#### 1. **Data Integrity Validation**
```bash
# Run comprehensive data validation
npm run validate:data-integrity

# Compare pre and post-migration data
npm run validate:data-comparison

# Validate foreign key constraints
npm run validate:foreign-keys
```

#### 2. **Functionality Testing**
```bash
# Test user authentication
npm run test:auth:full-suite

# Test learning path functionality
npm run test:learning-paths

# Test payment processing
npm run test:payments:end-to-end
```

#### 3. **Performance Validation**
```bash
# Database performance testing
npm run test:database:performance

# API response time testing
npm run test:api:response-times

# Load testing
npm run test:load:comprehensive
```

### 📊 Acceptance Criteria Validation

#### Migration Completion Checklist
- [ ] All migration scripts executed successfully
- [ ] Data integrity verified
- [ ] No data loss detected
- [ ] All v2 features functional
- [ ] Performance benchmarks met
- [ ] Security measures validated
- [ ] Monitoring systems operational
- [ ] Documentation updated

#### Testing Coverage Requirements
- [ ] Unit tests: >90% coverage
- [ ] Integration tests: >80% coverage
- [ ] E2E tests: All critical paths covered
- [ ] Performance tests: All SLAs met
- [ ] Security tests: All vulnerabilities addressed

---

## Troubleshooting

### 🚨 Common Issues and Solutions

#### 1. **Migration Failures**

**Issue**: Migration script fails with constraint violation
```bash
Error: duplicate key value violates unique constraint
```

**Solution**:
```bash
# 1. Identify conflicting data
npm run migration:identify-conflicts

# 2. Clean up conflicting data
npm run migration:cleanup-conflicts

# 3. Re-run migration
npm run migration:retry
```

#### 2. **Performance Degradation**

**Issue**: Slow queries after migration
```bash
Warning: Query execution time exceeded threshold
```

**Solution**:
```bash
# 1. Analyze slow queries
npm run performance:analyze-slow-queries

# 2. Update database statistics
ANALYZE;

# 3. Rebuild indexes
npm run migration:rebuild-indexes

# 4. Optimize configuration
npm run performance:tune-config
```

#### 3. **Authentication Issues**

**Issue**: Users cannot log in after migration
```bash
Error: Invalid credentials
```

**Solution**:
```bash
# 1. Verify password hashing migration
npm run auth:verify-password-migration

# 2. Check user account status
npm run auth:check-user-status

# 3. Reset affected user passwords
npm run auth:reset-affected-users
```

#### 4. **Data Corruption**

**Issue**: Data integrity check failures
```bash
Error: Foreign key constraint violation
```

**Solution**:
```bash
# 1. Identify corrupted data
npm run data:identify-corruption

# 2. Repair data relationships
npm run data:repair-relationships

# 3. Validate repair
npm run data:validate-repair
```

### 📞 Support and Escalation

#### Emergency Contacts
- **Technical Lead**: [Contact Information]
- **Database Administrator**: [Contact Information]
- **DevOps Engineer**: [Contact Information]

#### Escalation Procedures
1. **Level 1**: Basic troubleshooting (first 30 minutes)
2. **Level 2**: Technical team escalation (30-60 minutes)
3. **Level 3**: Emergency rollback decision (60+ minutes)

#### Documentation and Reporting
- Document all issues in incident tracking system
- Create post-mortem report for critical issues
- Update troubleshooting guide with new solutions

---

## Migration Timeline

### 📅 Recommended Schedule

#### Phase 1: Preparation (1-2 weeks)
- Environment setup
- Backup procedures
- Team training
- Testing on staging environment

#### Phase 2: Migration Execution (1-2 days)
- Data migration
- Application deployment
- Initial validation
- Performance tuning

#### Phase 3: Validation (1 week)
- Comprehensive testing
- User acceptance testing
- Performance monitoring
- Issue resolution

#### Phase 4: Stabilization (1-2 weeks)
- Monitoring
- Optimization
- Documentation updates
- Team training

### 🎯 Success Metrics

#### Technical Metrics
- Migration completion time: <48 hours
- Data integrity: 100%
- Performance impact: <10%
- Zero critical bugs

#### Business Metrics
- User experience: No degradation
- Feature availability: All v2 features operational
- System uptime: >99.9%
- Customer satisfaction: No complaints

---

## Conclusion

This migration guide provides a comprehensive approach to upgrading from StrellerMinds Smart Contracts v1 to v2. The migration introduces significant enhancements in security, functionality, and performance while maintaining data integrity and system reliability.

### Key Takeaways

1. **Thorough Preparation**: Complete all prerequisite steps before starting migration
2. **Comprehensive Testing**: Validate all aspects of the migration in staging
3. **Incremental Approach**: Execute migration in phases with validation at each step
4. **Rollback Planning**: Have clear rollback procedures for emergency situations
5. **Post-Migration Support**: Monitor system performance and address issues promptly

### Next Steps

1. Review this guide with your technical team
2. Schedule migration planning meeting
3. Set up staging environment for testing
4. Prepare migration timeline and resource allocation
5. Execute migration following this guide

For additional support or questions, contact the technical team or refer to the project documentation.

---

**Migration Version**: 2.0.0  
**Last Updated**: $(date)  
**Document Version**: 1.0  
**Next Review Date**: $(date -d "+6 months")
