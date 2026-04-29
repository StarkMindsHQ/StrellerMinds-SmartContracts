# ADR-003: API Design Principles

## Status
Accepted

## Context
The StrellerMinds platform requires a comprehensive API layer to bridge smart contracts with frontend applications and external services. The API needs to handle:

1. **Smart Contract Integration**: Interact with Stellar/Soroban smart contracts
2. **Educational Data Management**: Courses, modules, progress, and analytics
3. **Authentication & Authorization**: Secure access to educational resources
4. **Real-time Features**: Live progress updates and notifications
5. **External Integrations**: Third-party services and institutional systems

Key requirements include:
- RESTful API design for consistency and predictability
- GraphQL support for flexible data queries
- Real-time capabilities for live updates
- Comprehensive error handling and validation
- Rate limiting and security measures
- Multi-language support (i18n)
- Comprehensive documentation and testing

## Decision
We adopted a **layered API architecture** with RESTful endpoints, GraphQL support, and real-time capabilities. The design follows these principles:

### 1. API Architecture Layers

#### Presentation Layer
```typescript
// API Gateway and Routing
├── REST API Endpoints          // Standard REST operations
├── GraphQL Endpoint            // Flexible querying
├── WebSocket Server            // Real-time updates
├── Static Assets               // Documentation and UI
└── API Documentation           // OpenAPI/Swagger specs
```

#### Service Layer
```typescript
// Business Logic and Orchestration
├── Contract Services          // Smart contract interactions
├── Analytics Services         // Data processing and reporting
├── User Services              // User management and authentication
├── Notification Services      // Email and push notifications
└── Integration Services       // Third-party integrations
```

#### Data Access Layer
```typescript
// Data Persistence and Caching
├── Stellar Client            // Blockchain interactions
├── Database Client           // Off-chain data storage
├── Cache Client              // Redis/caching layer
└── External APIs             // Third-party service calls
```

### 2. RESTful API Design

#### Resource-Oriented URLs
```typescript
// Educational Resources
GET    /api/v1/courses                    // List courses
GET    /api/v1/courses/:id                // Get course details
POST   /api/v1/courses                    // Create course
PUT    /api/v1/courses/:id                // Update course
DELETE /api/v1/courses/:id                // Delete course

// User Management
GET    /api/v1/users/profile              // Get user profile
PUT    /api/v1/users/profile              // Update profile
GET    /api/v1/users/:id/progress         // Get user progress
POST   /api/v1/users/:id/sessions        // Record learning session

// Analytics
GET    /api/v1/analytics/courses/:id      // Course analytics
GET    /api/v1/analytics/users/:id        // User analytics
GET    /api/v1/analytics/reports/:type    // Generate reports
```

#### HTTP Status Codes
```typescript
// Success Codes
200 OK                  // Successful GET/PUT/DELETE
201 Created            // Successful POST
202 Accepted           // Accepted for processing
204 No Content         // Successful deletion

// Client Errors
400 Bad Request        // Invalid request data
401 Unauthorized       // Authentication required
403 Forbidden          // Insufficient permissions
404 Not Found          // Resource not found
409 Conflict           // Resource conflict
422 Unprocessable Entity // Validation errors

// Server Errors
500 Internal Server Error // Unexpected error
502 Bad Gateway           // Service unavailable
503 Service Unavailable   // Temporary outage
```

### 3. GraphQL Schema Design

#### Query Schema
```graphql
type Query {
  # User queries
  user(id: ID!): User
  users(filter: UserFilter, pagination: Pagination): UserConnection!
  currentUser: User
  
  # Course queries
  course(id: ID!): Course
  courses(filter: CourseFilter, pagination: Pagination): CourseConnection!
  
  # Analytics queries
  analytics(courseId: ID!, userId: ID): Analytics
  reports(type: ReportType!, filters: ReportFilters): Report
  
  # Token queries
  tokenBalance(address: ID!): TokenBalance
  tokenTransactions(address: ID!, pagination: Pagination): TransactionConnection!
}
```

#### Mutation Schema
```graphql
type Mutation {
  # User mutations
  updateProfile(input: ProfileInput!): User!
  enrollInCourse(courseId: ID!): Enrollment!
  
  # Learning mutations
  startSession(input: StartSessionInput!): LearningSession!
  completeSession(input: CompleteSessionInput!): LearningSession!
  
  # Course mutations
  createCourse(input: CreateCourseInput!): Course!
  updateCourse(id: ID!, input: UpdateCourseInput!): Course!
  
  # Token mutations
  transferToken(input: TransferTokenInput!): Transaction!
  stakeToken(input: StakeTokenInput!): StakingPosition!
}
```

#### Subscription Schema
```graphql
type Subscription {
  # Real-time updates
  userProgressUpdated(userId: ID!): ProgressUpdate!
  courseActivity(courseId: ID!): ActivityEvent!
  tokenBalanceUpdated(address: ID!): BalanceUpdate!
  
  # Notifications
  userNotifications(userId: ID!): Notification!
  systemAnnouncements: Announcement!
}
```

### 4. Request/Response Design

#### Standard Response Format
```typescript
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  meta?: ResponseMeta;
  pagination?: PaginationInfo;
}

interface ApiError {
  code: string;
  message: string;
  details?: any;
  timestamp: string;
  requestId: string;
}

interface ResponseMeta {
  version: string;
  requestId: string;
  processingTime: number;
  rateLimit?: RateLimitInfo;
}
```

#### Validation and Error Handling
```typescript
// Input Validation
interface ValidationRule {
  field: string;
  rules: ValidationRule[];
  messages: Record<string, string>;
}

// Error Types
enum ErrorType {
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  AUTHENTICATION_ERROR = 'AUTHENTICATION_ERROR',
  AUTHORIZATION_ERROR = 'AUTHORIZATION_ERROR',
  CONTRACT_ERROR = 'CONTRACT_ERROR',
  NETWORK_ERROR = 'NETWORK_ERROR',
  INTERNAL_ERROR = 'INTERNAL_ERROR'
}
```

### 5. Authentication & Authorization

#### JWT Token Structure
```typescript
interface JWTPayload {
  sub: string;        // User ID
  iat: number;        // Issued at
  exp: number;        // Expires at
  aud: string;        // Audience (API)
  iss: string;        // Issuer
  roles: string[];    // User roles
  permissions: string[]; // Specific permissions
  institution?: string; // Institution ID
}
```

#### Permission Model
```typescript
interface Permission {
  resource: string;   // Resource type (course, user, analytics)
  action: string;     // Action (read, write, delete, admin)
  scope: string;      // Scope (own, institution, global)
  conditions?: Record<string, any>; // Additional conditions
}

// Role-based permissions
const ROLE_PERMISSIONS = {
  STUDENT: ['read:own:profile', 'read:own:progress', 'write:own:sessions'],
  INSTRUCTOR: ['read:institution:courses', 'write:institution:courses', 'read:institution:analytics'],
  ADMIN: ['*'], // Full access
  INSTITUTION_ADMIN: ['read:institution:*', 'write:institution:*', 'read:global:analytics']
};
```

### 6. Rate Limiting and Security

#### Rate Limiting Strategy
```typescript
interface RateLimitConfig {
  windowMs: number;     // Time window in milliseconds
  maxRequests: number;  // Max requests per window
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
}

// Tiered rate limiting
const RATE_LIMITS = {
  '/api/v1/auth/': { windowMs: 15 * 60 * 1000, maxRequests: 5 },    // Auth endpoints
  '/api/v1/analytics/': { windowMs: 60 * 1000, maxRequests: 100 }, // Analytics
  '/api/v1/courses/': { windowMs: 60 * 1000, maxRequests: 1000 },  // General endpoints
  '/api/v1/graphql': { windowMs: 60 * 1000, maxRequests: 500 },     // GraphQL
  default: { windowMs: 60 * 1000, maxRequests: 1000 }               // Default
};
```

#### Security Headers
```typescript
const SECURITY_HEADERS = {
  'X-Content-Type-Options': 'nosniff',
  'X-Frame-Options': 'DENY',
  'X-XSS-Protection': '1; mode=block',
  'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
  'Content-Security-Policy': "default-src 'self'",
  'Referrer-Policy': 'strict-origin-when-cross-origin'
};
```

### 7. Documentation and Testing

#### OpenAPI Specification
```yaml
openapi: 3.0.3
info:
  title: StrellerMinds API
  version: 1.0.0
  description: Educational platform API for smart contracts integration
servers:
  - url: https://api.strellerminds.com/v1
    description: Production server
  - url: https://staging-api.strellerminds.com/v1
    description: Staging server
```

#### Testing Strategy
```typescript
// Test categories
describe('API Tests', () => {
  describe('Unit Tests', () => {
    // Individual endpoint testing
  });
  
  describe('Integration Tests', () => {
    // Contract integration testing
  });
  
  describe('E2E Tests', () => {
    // Full workflow testing
  });
  
  describe('Performance Tests', () => {
    // Load and stress testing
  });
});
```

## Consequences

### Benefits
1. **Consistency**: Standardized patterns across all endpoints
2. **Flexibility**: GraphQL provides flexible data querying
3. **Real-time**: WebSocket support for live updates
4. **Security**: Comprehensive authentication and authorization
5. **Scalability**: Layered architecture supports growth
6. **Developer Experience**: Comprehensive documentation and testing

### Drawbacks
1. **Complexity**: Multiple API types increase implementation complexity
2. **Maintenance**: More endpoints and schemas to maintain
3. **Performance**: GraphQL query complexity can impact performance
4. **Learning Curve**: Developers need to learn multiple API patterns

### Trade-offs
- **REST vs GraphQL**: Chose both to serve different use cases
- **Security vs Performance**: Added security layers may impact response times
- **Flexibility vs Simplicity**: GraphQL flexibility adds complexity

## Implementation

### Technology Stack
```typescript
// Core dependencies
{
  "express": "^4.18.0",           // Web framework
  "apollo-server-express": "^3.12.0", // GraphQL server
  "socket.io": "^4.7.0",          // WebSocket server
  "jsonwebtoken": "^9.0.0",       // JWT handling
  "joi": "^17.9.0",               // Validation
  "helmet": "^7.0.0",             // Security headers
  "express-rate-limit": "^6.8.0", // Rate limiting
  "swagger-jsdoc": "^6.2.0",      // Documentation
  "winston": "^3.8.0"             // Logging
}
```

### API Structure
```typescript
// src/
├── routes/
│   ├── auth.routes.ts           // Authentication endpoints
│   ├── courses.routes.ts        // Course management
│   ├── users.routes.ts          // User management
│   ├── analytics.routes.ts     // Analytics endpoints
│   └── tokens.routes.ts         // Token operations
├── graphql/
│   ├── schema.ts                // GraphQL schema
│   ├── resolvers.ts             // GraphQL resolvers
│   └── subscriptions.ts         // Real-time subscriptions
├── middleware/
│   ├── auth.middleware.ts        // Authentication
│   ├── validation.middleware.ts  // Input validation
│   ├── rateLimit.middleware.ts   // Rate limiting
│   └── error.middleware.ts       // Error handling
├── services/
│   ├── contract.service.ts       // Smart contract interactions
│   ├── analytics.service.ts      // Analytics processing
│   └── notification.service.ts   // Notifications
└── utils/
    ├── logger.ts                 // Logging utilities
    ├── cache.ts                  // Caching utilities
    └── validation.ts             // Validation helpers
```

## Alternatives Considered

### 1. REST Only
**Pros**: Simpler implementation, widely understood
**Cons**: Limited flexibility, multiple requests for complex data
**Rejected**: GraphQL provides significant benefits for educational data queries

### 2. GraphQL Only
**Pros**: Single endpoint, flexible queries
**Cons**: Complexity for simple operations, steeper learning curve
**Rejected**: REST is better for simple CRUD operations and caching

### 3. gRPC
**Pros**: High performance, strict typing
**Cons**: Limited browser support, more complex setup
**Rejected**: REST/GraphQL better suit web application needs

### 4. Serverless Functions
**Pros**: Scalable, pay-per-use
**Cons**: Cold starts, limited execution time
**Rejected**: Need persistent connections for real-time features

## References

- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [GraphQL Specification](https://graphql.org/)
- [JWT Best Practices](https://jwt.io/)
- [Express.js Documentation](https://expressjs.com/)
- [Apollo Server Documentation](https://www.apollographql.com/docs/)
- [API Implementation](../api/src/)
