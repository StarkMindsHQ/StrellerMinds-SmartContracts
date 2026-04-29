# ADR-004: Caching Strategy

## Status
Accepted

## Context
The StrellerMinds platform requires an efficient caching strategy to handle high-volume educational data and optimize performance across multiple layers. The system needs to address:

1. **Smart Contract Data**: Frequently accessed on-chain data (balances, permissions, achievements)
2. **Analytics Data**: Computed metrics and reports that are expensive to regenerate
3. **Educational Content**: Course materials and metadata that change infrequently
4. **User Sessions**: Active learning sessions and real-time progress data
5. **API Responses**: Repeated API calls and database queries

Key challenges include:
- High read-to-write ratios for educational content
- Expensive blockchain transactions for data retrieval
- Real-time requirements for progress tracking
- Multi-tenant data isolation requirements
- Cache invalidation across distributed systems

## Decision
We adopted a **multi-layered caching strategy** with different cache types and invalidation strategies for different data categories. The approach includes:

### 1. Cache Architecture Layers

#### L1: Application Memory Cache
```typescript
// In-memory cache for frequently accessed data
interface MemoryCache {
  userSessions: Map<string, UserSession>;     // Active user sessions
  courseMetadata: Map<string, Course>;       // Course information
  permissions: Map<string, Permissions>;     // User permissions
  rateLimitState: Map<string, RateLimit>;    // Rate limiting data
}
```

#### L2: Distributed Cache (Redis)
```typescript
// Redis cluster for shared caching
interface RedisCache {
  // User data
  userProfile: string;           // User profile data (TTL: 1 hour)
  userProgress: string;          // Progress analytics (TTL: 15 minutes)
  userAchievements: string;      // User achievements (TTL: 30 minutes)
  
  // Course data
  courseContent: string;         // Course content (TTL: 24 hours)
  courseAnalytics: string;       // Course analytics (TTL: 1 hour)
  moduleList: string;            // Module lists (TTL: 12 hours)
  
  // System data
  contractState: string;         // Smart contract state (TTL: 5 minutes)
  apiResponses: string;          // API response cache (TTL: 5 minutes)
  configData: string;            // Configuration data (TTL: 1 hour)
}
```

#### L3: Database Query Cache
```typescript
// Database-level query result caching
interface QueryCache {
  analyticsReports: string;      // Pre-computed reports (TTL: 1 hour)
  leaderboardData: string;       // Leaderboard rankings (TTL: 10 minutes)
  aggregateMetrics: string;      // Aggregated analytics (TTL: 30 minutes)
  historicalData: string;        // Historical analytics (TTL: 24 hours)
}
```

### 2. Cache Strategies by Data Type

#### Read-Heavy Data (Educational Content)
```typescript
// Cache-Aside Pattern for course content
class CourseContentCache {
  async getCourse(courseId: string): Promise<Course> {
    // 1. Try cache first
    let course = await redis.get(`course:${courseId}`);
    if (course) return JSON.parse(course);
    
    // 2. Load from database
    course = await database.getCourse(courseId);
    
    // 3. Cache for 24 hours
    await redis.setex(`course:${courseId}`, 86400, JSON.stringify(course));
    return course;
  }
  
  async updateCourse(courseId: string, updates: Partial<Course>): Promise<Course> {
    // 1. Update database
    const course = await database.updateCourse(courseId, updates);
    
    // 2. Invalidate cache
    await redis.del(`course:${courseId}`);
    await redis.del(`course:${courseId}:modules`);
    
    // 3. Publish cache invalidation event
    await eventBus.publish('course.updated', { courseId });
    return course;
  }
}
```

#### Write-Through Cache (User Progress)
```typescript
// Write-Through Pattern for real-time progress
class ProgressCache {
  async updateProgress(userId: string, courseId: string, progress: Progress): Promise<void> {
    // 1. Update database immediately
    await database.updateProgress(userId, courseId, progress);
    
    // 2. Update cache synchronously
    await redis.setex(`progress:${userId}:${courseId}`, 900, JSON.stringify(progress));
    
    // 3. Update in-memory cache for active sessions
    if (memoryCache.has(`session:${userId}`)) {
      memoryCache.set(`session:${userId}`, progress);
    }
  }
}
```

#### Write-Behind Cache (Analytics)
```typescript
// Write-Behind Pattern for analytics aggregation
class AnalyticsCache {
  private batchQueue: AnalyticsEvent[] = [];
  
  async recordEvent(event: AnalyticsEvent): Promise<void> {
    // 1. Add to batch queue
    this.batchQueue.push(event);
    
    // 2. Update cache immediately for reads
    await this.updateAnalyticsCache(event);
    
    // 3. Batch write to database (every 10 seconds or 100 events)
    if (this.batchQueue.length >= 100 || Date.now() % 10000 < 100) {
      await this.flushBatch();
    }
  }
  
  private async flushBatch(): Promise<void> {
    if (this.batchQueue.length === 0) return;
    
    const batch = this.batchQueue.splice(0);
    await database.batchInsertAnalytics(batch);
  }
}
```

### 3. Cache Invalidation Strategies

#### Time-Based Expiration (TTL)
```typescript
const CACHE_TTL = {
  // User data
  USER_PROFILE: 3600,        // 1 hour
  USER_PROGRESS: 900,        // 15 minutes
  USER_PERMISSIONS: 1800,    // 30 minutes
  
  // Course data
  COURSE_CONTENT: 86400,     // 24 hours
  COURSE_ANALYTICS: 3600,    // 1 hour
  MODULE_LIST: 43200,       // 12 hours
  
  // System data
  CONTRACT_STATE: 300,       // 5 minutes
  API_RESPONSES: 300,        // 5 minutes
  CONFIG_DATA: 3600,         // 1 hour
  
  // Analytics data
  REALTIME_ANALYTICS: 600,   // 10 minutes
  LEADERBOARD: 600,          // 10 minutes
  AGGREGATE_REPORTS: 3600,   // 1 hour
};
```

#### Event-Driven Invalidation
```typescript
class CacheInvalidationService {
  constructor(private eventBus: EventBus, private cache: CacheService) {
    this.setupInvalidationListeners();
  }
  
  private setupInvalidationListeners(): void {
    // Course updates
    this.eventBus.on('course.updated', (event) => {
      this.invalidateCourseData(event.courseId);
    });
    
    // User progress updates
    this.eventBus.on('progress.updated', (event) => {
      this.invalidateUserProgress(event.userId, event.courseId);
    });
    
    // Smart contract state changes
    this.eventBus.on('contract.state_changed', (event) => {
      this.invalidateContractState(event.contractId);
    });
  }
  
  private async invalidateCourseData(courseId: string): Promise<void> {
    const patterns = [
      `course:${courseId}`,
      `course:${courseId}:modules`,
      `course:${courseId}:analytics`,
      `course:${courseId}:leaderboard`
    ];
    
    await Promise.all(patterns.map(pattern => this.cache.del(pattern)));
  }
}
```

#### Tag-Based Invalidation
```typescript
class TaggedCache {
  private tagIndex: Map<string, Set<string>> = new Map();
  
  async set(key: string, value: any, tags: string[], ttl?: number): Promise<void> {
    // Store value
    await redis.setex(key, ttl || 3600, JSON.stringify(value));
    
    // Update tag index
    for (const tag of tags) {
      if (!this.tagIndex.has(tag)) {
        this.tagIndex.set(tag, new Set());
      }
      this.tagIndex.get(tag)!.add(key);
    }
  }
  
  async invalidateByTag(tag: string): Promise<void> {
    const keys = this.tagIndex.get(tag) || new Set();
    
    // Delete all keys with this tag
    await Promise.all(Array.from(keys).map(key => redis.del(key)));
    
    // Clear tag index
    this.tagIndex.delete(tag);
  }
}
```

### 4. Cache Warming and Preloading

#### Cache Warming Strategy
```typescript
class CacheWarmer {
  async warmUserCache(userId: string): Promise<void> {
    // Preload user data
    const userProfile = await userService.getProfile(userId);
    const userProgress = await analyticsService.getUserProgress(userId);
    const userPermissions = await authService.getUserPermissions(userId);
    
    // Cache with appropriate TTL
    await Promise.all([
      redis.setex(`user:${userId}:profile`, 3600, JSON.stringify(userProfile)),
      redis.setex(`user:${userId}:progress`, 900, JSON.stringify(userProgress)),
      redis.setex(`user:${userId}:permissions`, 1800, JSON.stringify(userPermissions))
    ]);
  }
  
  async warmCourseCache(courseId: string): Promise<void> {
    // Preload course data
    const course = await courseService.getCourse(courseId);
    const modules = await courseService.getModules(courseId);
    const analytics = await analyticsService.getCourseAnalytics(courseId);
    
    // Cache with appropriate TTL
    await Promise.all([
      redis.setex(`course:${courseId}`, 86400, JSON.stringify(course)),
      redis.setex(`course:${courseId}:modules`, 43200, JSON.stringify(modules)),
      redis.setex(`course:${courseId}:analytics`, 3600, JSON.stringify(analytics))
    ]);
  }
}
```

#### Predictive Preloading
```typescript
class PredictiveCache {
  async preloadBasedOnBehavior(userId: string): Promise<void> {
    // Analyze user behavior patterns
    const behavior = await this.analyzeUserBehavior(userId);
    
    // Preload likely next actions
    if (behavior.likelyNextCourse) {
      await this.warmCourseCache(behavior.likelyNextCourse);
    }
    
    if (behavior.likelyNextModule) {
      await this.warmModuleCache(behavior.likelyNextModule);
    }
  }
  
  private async analyzeUserBehavior(userId: string): Promise<UserBehavior> {
    // Analyze learning patterns
    const recentSessions = await this.getRecentSessions(userId);
    const progressHistory = await this.getProgressHistory(userId);
    
    // Predict next actions based on patterns
    return {
      likelyNextCourse: this.predictNextCourse(recentSessions),
      likelyNextModule: this.predictNextModule(progressHistory)
    };
  }
}
```

### 5. Cache Monitoring and Analytics

#### Cache Performance Metrics
```typescript
interface CacheMetrics {
  hitRate: number;              // Cache hit ratio
  missRate: number;            // Cache miss ratio
  evictionRate: number;        // Cache eviction rate
  memoryUsage: number;         // Memory usage in bytes
  keyCount: number;            // Total number of keys
  avgResponseTime: number;     // Average cache response time
  errorRate: number;           // Cache error rate
}

class CacheMonitor {
  async collectMetrics(): Promise<CacheMetrics> {
    const info = await redis.info('memory');
    const stats = await redis.info('stats');
    
    return {
      hitRate: this.extractHitRate(stats),
      missRate: this.extractMissRate(stats),
      evictionRate: this.extractEvictionRate(stats),
      memoryUsage: this.extractMemoryUsage(info),
      keyCount: await redis.dbsize(),
      avgResponseTime: this.measureResponseTime(),
      errorRate: this.measureErrorRate()
    };
  }
  
  async generateReport(): Promise<CacheReport> {
    const metrics = await this.collectMetrics();
    const recommendations = this.generateRecommendations(metrics);
    
    return {
      timestamp: new Date(),
      metrics,
      recommendations,
      alerts: this.checkAlerts(metrics)
    };
  }
}
```

## Consequences

### Benefits
1. **Performance**: Significant reduction in database and blockchain query times
2. **Scalability**: Reduced load on backend services and database
3. **Cost Efficiency**: Fewer blockchain transactions and database queries
4. **User Experience**: Faster response times for frequently accessed data
5. **Reliability**: Cache serves as fallback during service outages

### Drawbacks
1. **Complexity**: Increased system complexity with multiple cache layers
2. **Data Consistency**: Risk of serving stale data during cache invalidation delays
3. **Memory Usage**: Additional memory requirements for caching infrastructure
4. **Maintenance**: Cache configuration and monitoring overhead

### Trade-offs
- **Performance vs Consistency**: Chose eventual consistency for better performance
- **Memory vs Computation**: Increased memory usage for reduced computation
- **Complexity vs Benefits**: Added complexity justified by performance gains

## Implementation

### Cache Configuration
```typescript
// Redis configuration
const redisConfig = {
  host: process.env.REDIS_HOST || 'localhost',
  port: parseInt(process.env.REDIS_PORT || '6379'),
  password: process.env.REDIS_PASSWORD,
  db: parseInt(process.env.REDIS_DB || '0'),
  retryDelayOnFailover: 100,
  maxRetriesPerRequest: 3,
  lazyConnect: true,
  keepAlive: 30000,
  family: 4,
  keyPrefix: 'strellerminds:',
  
  // Cluster configuration for scalability
  cluster: {
    nodes: [
      { host: 'redis-node-1', port: 6379 },
      { host: 'redis-node-2', port: 6379 },
      { host: 'redis-node-3', port: 6379 }
    ],
    options: {
      maxRedirections: 16,
      retryDelayOnFailover: 100,
      enableOfflineQueue: false
    }
  }
};
```

### Cache Service Implementation
```typescript
class CacheService {
  private redis: Redis;
  private localCache: LRUCache<string, any>;
  private metrics: CacheMetrics;
  
  constructor() {
    this.redis = new Redis(redisConfig);
    this.localCache = new LRUCache({ max: 1000, ttl: 300000 }); // 5 minutes
    this.metrics = new CacheMetrics();
  }
  
  async get<T>(key: string, fallback?: () => Promise<T>): Promise<T | null> {
    const start = Date.now();
    
    try {
      // Try local cache first
      let value = this.localCache.get(key);
      if (value) {
        this.metrics.recordHit('local');
        return value;
      }
      
      // Try Redis cache
      const redisValue = await this.redis.get(key);
      if (redisValue) {
        value = JSON.parse(redisValue);
        this.localCache.set(key, value);
        this.metrics.recordHit('redis');
        return value;
      }
      
      // Use fallback if provided
      if (fallback) {
        value = await fallback();
        await this.set(key, value);
        this.metrics.recordMiss();
        return value;
      }
      
      this.metrics.recordMiss();
      return null;
    } finally {
      this.metrics.recordLatency(Date.now() - start);
    }
  }
  
  async set(key: string, value: any, ttl?: number): Promise<void> {
    const serialized = JSON.stringify(value);
    
    // Set in both caches
    this.localCache.set(key, value);
    
    if (ttl) {
      await this.redis.setex(key, ttl, serialized);
    } else {
      await this.redis.set(key, serialized);
    }
  }
  
  async invalidate(pattern: string): Promise<void> {
    // Invalidate in both caches
    const keys = this.localCache.keys();
    for (const key of keys) {
      if (key.includes(pattern)) {
        this.localCache.delete(key);
      }
    }
    
    const redisKeys = await this.redis.keys(`*${pattern}*`);
    if (redisKeys.length > 0) {
      await this.redis.del(...redisKeys);
    }
  }
}
```

## Alternatives Considered

### 1. No Caching
**Pros**: Simpler architecture, no data consistency issues
**Cons**: Poor performance, high database load, expensive blockchain calls
**Rejected**: Performance requirements make caching essential

### 2. Single Cache Layer
**Pros**: Simpler implementation, easier to manage
**Cons**: Limited performance benefits, single point of failure
**Rejected**: Multi-layer approach provides better performance and reliability

### 3. Client-Side Caching Only
**Pros**: Reduced server load
**Cons**: Limited control, security concerns, cache invalidation complexity
**Rejected**: Server-side caching provides better control and security

### 4. Database-Level Caching Only
**Pros**: Transparent to application code
**Cons**: Limited flexibility, application-specific caching needs
**Rejected**: Application-level caching provides better control and optimization

## References

- [Redis Documentation](https://redis.io/documentation)
- [Cache-Aside Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/cache-aside)
- [Write-Through Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/write-through)
- [Cache Implementation](../api/src/cache.ts)
- [Performance Metrics](../api/src/metrics.ts)
- [Monitoring Documentation](../docs/MONITORING.md)
