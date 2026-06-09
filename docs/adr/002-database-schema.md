# ADR-002: Database Schema Design

## Status
Accepted

## Context
The StrellerMinds platform requires a comprehensive database schema to manage educational data across multiple domains. The system needs to handle:

1. **Educational Data**: Courses, modules, learning paths, and content
2. **User Management**: Students, instructors, administrators, and institutions
3. **Analytics & Progress**: Learning sessions, achievements, and performance metrics
4. **Token Economy**: Token balances, transactions, and incentive programs
5. **Audit & Compliance**: GDPR compliance, audit trails, and data retention

Key requirements include:
- Support for multi-tenant architecture (multiple institutions)
- Scalable analytics storage for large datasets
- Efficient querying for real-time and batch analytics
- Data integrity and consistency across domains
- Support for both on-chain and off-chain data storage

## Decision
We adopted a **hybrid database schema** combining on-chain storage for critical data with off-chain storage for analytics and large datasets. The schema follows these principles:

### 1. Data Classification by Storage Type

#### On-Chain Storage (Critical & Immutable)
- **User Identities**: Addresses and roles
- **Token Balances**: Token ownership and transfers
- **Achievements**: Earned credentials and certificates
- **Access Control**: Permissions and role assignments
- **Critical Analytics**: Finalized progress and completion data

#### Off-Chain Storage (Analytics & Large Datasets)
- **Learning Sessions**: Detailed session data and interactions
- **Course Content**: Educational materials and metadata
- **Analytics Aggregates**: Computed metrics and reports
- **Audit Logs**: System events and access logs
- **Temporary Data**: Cache and session state

### 2. Schema Organization

#### Core Entities
```rust
// User Management
pub struct User {
    pub address: Address,
    pub roles: Vec<Role>,
    pub institution: Option<Address>,
    pub profile: UserProfile,
    pub created_at: u64,
}

// Educational Structure
pub struct Course {
    pub course_id: Symbol,
    pub institution: Address,
    pub title: String,
    pub modules: Vec<Module>,
    pub settings: CourseSettings,
}

pub struct Module {
    pub module_id: Symbol,
    pub course_id: Symbol,
    pub title: String,
    pub content_type: ContentType,
    pub difficulty: DifficultyRating,
}

// Learning Data
pub struct LearningSession {
    pub session_id: BytesN<32>,
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub progress: u32,
    pub score: Option<u32>,
}
```

#### Analytics Schema
```rust
pub struct ProgressAnalytics {
    pub student: Address,
    pub course_id: Symbol,
    pub total_modules: u32,
    pub completed_modules: u32,
    pub completion_percentage: u32,
    pub total_time_spent: u64,
    pub average_session_time: u64,
    pub streak_days: u32,
    pub performance_trend: PerformanceTrend,
}

pub struct CourseAnalytics {
    pub course_id: Symbol,
    pub total_students: u32,
    pub active_students: u32,
    pub completion_rate: u32,
    pub average_completion_time: u64,
    pub dropout_rate: u32,
}
```

### 3. Storage Key Design

#### Hierarchical Key Structure
```rust
#[contracttype]
pub enum DataKey {
    // User Management
    User(Address),                    // User profile data
    UserRoles(Address),              // User role assignments
    InstitutionUsers(Address),       // Institution member list
    
    // Educational Data
    Course(Symbol),                  // Course metadata
    CourseModules(Symbol),           // Course module list
    Module(Symbol),                  // Module metadata
    
    // Analytics
    ProgressAnalytics(Address, Symbol),  // Student progress per course
    CourseAnalytics(Symbol),        // Course-wide analytics
    ModuleAnalytics(Symbol, Symbol),    // Module analytics
    
    // Sessions
    Session(BytesN<32>),            // Individual session data
    StudentSessions(Address, Symbol),   // Student sessions per course
    
    // Achievements
    UserAchievements(Address),      // User achievement list
    Achievement(Symbol),            // Achievement metadata
}
```

### 4. Data Relationships and Indexing

#### Primary Relationships
- **User ↔ Institution**: Many-to-one relationship
- **User ↔ Course**: Many-to-many through enrollment
- **Course ↔ Module**: One-to-many relationship
- **User ↔ Session**: One-to-many relationship
- **Session ↔ Analytics**: One-to-one derived relationship

#### Indexing Strategy
- **User-centric**: All data indexed by user address for efficient retrieval
- **Course-centric**: Course-level analytics for institutional reporting
- **Time-based**: Timestamp-based indexing for temporal analytics
- **Composite Keys**: Multi-field keys for complex queries

### 5. Data Lifecycle Management

#### Retention Policies
- **Permanent**: User identities, achievements, token balances
- **Long-term (7 years)**: Analytics data, audit logs (GDPR compliance)
- **Medium-term (1 year)**: Detailed session data, intermediate analytics
- **Short-term (30 days)**: Cache data, temporary state

#### Archival Strategy
- **Hot Storage**: Frequently accessed data (current courses, active users)
- **Warm Storage**: Historical analytics and completed courses
- **Cold Storage**: Archived data and compliance records

## Consequences

### Benefits
1. **Performance**: Optimized storage patterns reduce gas costs and improve query performance
2. **Scalability**: Hierarchical design supports growth in users and courses
3. **Data Integrity**: Clear relationships ensure consistency across domains
4. **Flexibility**: Schema supports multiple educational models and institution types
5. **Compliance**: Built-in support for GDPR and data retention requirements

### Drawbacks
1. **Complexity**: More complex schema requires careful data management
2. **Storage Costs**: On-chain storage is expensive, requiring careful optimization
3. **Migration Complexity**: Schema changes require careful migration planning
4. **Query Complexity**: Some queries require multiple storage reads

### Trade-offs
- **Normalization vs Performance**: Chose some denormalization for query performance
- **On-chain vs Off-chain**: Balanced data immutability with cost considerations
- **Flexibility vs Simplicity**: Schema flexibility increases implementation complexity

## Implementation

### Storage Implementation
```rust
// Shared storage utilities
pub struct StorageUtils;

impl StorageUtils {
    pub fn set_user(env: &Env, user: &User) {
        env.storage().persistent().set(&DataKey::User(user.address), user);
    }
    
    pub fn get_user(env: &Env, address: &Address) -> Option<User> {
        env.storage().persistent().get(&DataKey::User(address.clone()))
    }
    
    pub fn set_progress_analytics(env: &Env, student: &Address, course_id: &Symbol, analytics: &ProgressAnalytics) {
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().set(&key, analytics);
    }
}
```

### Data Validation
```rust
pub struct DataValidator;

impl DataValidator {
    pub fn validate_user(user: &User) -> Result<(), ValidationError> {
        if user.address.is_none() {
            return Err(ValidationError::InvalidAddress);
        }
        if user.roles.is_empty() {
            return Err(ValidationError::MissingRoles);
        }
        Ok(())
    }
    
    pub fn validate_session(session: &LearningSession) -> Result<(), ValidationError> {
        if session.start_time >= session.end_time.unwrap_or(u64::MAX) {
            return Err(ValidationError::InvalidTimeRange);
        }
        if session.progress > 100 {
            return Err(ValidationError::InvalidProgress);
        }
        Ok(())
    }
}
```

### Migration Strategy
1. **Versioned Schema**: Include version numbers in data structures
2. **Migration Functions**: Built-in migration capabilities for schema updates
3. **Backward Compatibility**: Support for reading old data formats during transition
4. **Testing**: Comprehensive migration testing in development environment

## Alternatives Considered

### 1. Fully On-Chain Schema
**Pros**: Complete immutability, no external dependencies
**Cons**: Extremely high gas costs, limited storage capacity
**Rejected**: Cost and storage limitations make this impractical

### 2. Fully Off-Chain Schema
**Pros**: Low cost, unlimited storage, flexible querying
**Cons**: Reduced trustlessness, potential data availability issues
**Rejected**: Critical data must be on-chain for trust and security

### 3. Document-Based Schema (NoSQL)
**Pros**: Flexible schema, good for unstructured data
**Cons**: Limited query capabilities, weaker consistency guarantees
**Rejected**: Structured relational approach better fits educational data needs

### 4. Graph-Based Schema
**Pros**: Excellent for complex relationships
**Cons**: Higher complexity, not well-suited for blockchain storage
**Rejected**: Over-engineered for current requirements

## References

- [Soroban Storage Documentation](https://soroban.stellar.org/docs/learn/storage)
- [Stellar Data Best Practices](https://stellar.org/developers/data)
- [GDPR Compliance Guidelines](../docs/GDPR_COMPLIANCE.md)
- [Analytics Contract Schema](../contracts/analytics/src/types.rs)
- [Token Contract Schema](../contracts/token/src/types.rs)
- [Shared Storage Utilities](../contracts/shared/src/storage.rs)
