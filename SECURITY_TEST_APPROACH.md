# Security Test Suite Implementation Approach — Issue #425

## Executive Summary

This document outlines the comprehensive security test suite for StrellerMinds, covering all applicable attack vectors across the identified layers. The reconnaissance has identified a **three-layer architecture**: smart contracts (Soroban/Rust), a backend REST API (Node.js/Express/TypeScript), and no frontend rendering layer. Security tests will target both layers with full coverage of authentication, authorization, input validation, and output encoding mechanisms.

---

## Architecture & Applicable Layers

### Layers Present
1. **Smart Contracts Layer** (Soroban/Rust): Certificate contract with multi-sig, templates, compliance, and recovery features
2. **Backend API Layer** (Node.js/Express/TypeScript): REST API for certificate verification, student management, analytics, and admin operations
3. **Frontend Layer**: None identified — no HTML rendering or DOM manipulation in the codebase

### Attack Vectors by Layer

| Attack Vector | Contracts | Backend API | Frontend | Applicable |
|---|---|---|---|---|
| SQL Injection | ❌ | ✅ | ❌ | Backend only |
| XSS (Reflected/Stored/DOM) | ❌ | ⚠️ Limited | ❌ | Backend JSON responses only (no HTML rendering) |
| CSRF | ❌ | ✅ | ❌ | Backend state-changing endpoints |
| Authentication Bypass | ✅ | ✅ | ❌ | Both layers |
| Authorization Bypass | ✅ | ✅ | ❌ | Both layers |
| Reentrancy | ✅ | ❌ | ❌ | Contracts only |
| Integer Overflow | ✅ | ❌ | ❌ | Contracts only |

---

## Complete Attack Surface Map

### Backend API Layer

#### Authentication Mechanisms Found
- **JWT Bearer Token Authentication** (`api/src/middleware/auth.ts`)
  - Token validation: `jwt.verify(token, config.jwt.secret)`
  - Expiry check: `TokenExpiredError` handling
  - Scope-based authorization: `requireScope(scope)` middleware
  - **Bypass vectors to test**:
    - No token provided
    - Malformed token (truncated, invalid encoding)
    - Expired token (advance time in test)
    - Token signed with `none` algorithm
    - Token with tampered payload (role/scope modification)
    - Token from different environment/contract

#### Authorization Mechanisms Found
- **Scope-based RBAC** (`api/src/middleware/auth.ts`)
  - Scopes: `["verify", "read"]` (from `api/src/routes/auth.ts`)
  - Check: `req.auth?.scope?.includes(scope)`
  - **Bypass vectors to test**:
    - Horizontal escalation: access other users' resources
    - Vertical escalation: attempt admin operations as non-admin
    - Role claim manipulation in token
    - Missing authorization checks on endpoints

#### State-Changing Endpoints (CSRF-Protected)
- `POST /api/v1/auth/token` — issues JWT
- `POST /api/v1/certificates/:id/verify` — public, rate-limited
- `GET /api/v1/certificates/:id` — authenticated read
- `GET /api/v1/certificates/:id/revocation` — authenticated read
- `GET /api/v1/students/:address/certificates` — authenticated read
- Additional routes: `/analytics`, `/consent`, `/export`, `/cohorts`, `/slack`, `/social-sharing`, `/performance`, `/rate-limit`, `/cdn`, `/certificate-templates`

**CSRF Protection Status**: 
- ❌ **NO CSRF TOKENS FOUND** — No CSRF token generation, validation, or middleware
- ❌ **NO ORIGIN/REFERER VALIDATION** — No Origin or Referer header checks
- ❌ **NO SAMESITE COOKIES** — JWT is Bearer token, not cookie-based
- **Vulnerability**: All state-changing endpoints are vulnerable to CSRF attacks

#### Database Queries & SQL Injection
- **Database Layer**: PostgreSQL via `pg` library (`api/src/utils/dbPool.ts`)
- **Query Pattern**: Parameterized queries via `client.query(text, params)`
- **Queries Found**:
  - Health check: `SELECT NOW()` (no user input)
  - All queries use parameterized approach via `dbPool.query(text, params)`
- **Vulnerability Status**: ✅ **NO SQL INJECTION FOUND** — All queries use parameterized statements
- **Vacuousness Check**: Will verify that removing parameterization causes injection to succeed

#### Input Validation
- **Validation Library**: Zod (`api/src/utils/validate.ts`)
- **Validated Inputs**:
  - Certificate ID: `certificateIdSchema` — 64-char hex string (with optional 0x prefix)
  - Stellar Address: `stellarAddressSchema` — G-prefixed 56-char address
  - API Key: minimum 16 characters
- **Encoding**: JSON responses via Express (automatic JSON encoding, no HTML rendering)
- **Vulnerability Status**: ✅ **NO XSS FOUND** — No HTML rendering; JSON responses are automatically encoded

#### Rate Limiting
- **IP-based rate limiting**: `express-rate-limit` middleware
- **Per-user rate limiting**: `userRateLimiter` middleware (tier-based: free, pro, enterprise, internal)
- **Endpoints**:
  - General: 60 requests per minute
  - Verify: 100 requests per minute (public endpoint)
- **Bypass vectors to test**:
  - Rate limit bypass via header manipulation
  - Rate limit bypass via IP spoofing (X-Forwarded-For)

#### Security Headers
- **Helmet.js Configuration** (`api/src/app.ts`):
  - HSTS: `maxAge: 31536000`, `includeSubDomains: true`, `preload: true`
  - CSP: `defaultSrc: ["'self'"]`, `scriptSrc: ["'self'", "'unsafe-inline'"]` (for Swagger UI)
  - X-Frame-Options: `sameorigin`
  - Content-Type: `nosniff`
- **Validation Middleware** (`api/src/middleware/securityHeaders.ts`):
  - Checks for presence of required headers on response finish
  - Logs warnings if headers are missing
- **Vulnerability Status**: ⚠️ **CSP ALLOWS UNSAFE-INLINE** — Necessary for Swagger UI but reduces XSS protection

### Smart Contract Layer (Soroban/Rust)

#### Authentication Mechanisms Found
- **Soroban Native Authentication**: `caller.require_auth()` in `require_admin()` function
- **Admin Check**: `require_admin(env, caller)` verifies caller is the stored admin address
- **Bypass vectors to test**:
  - Call admin-only functions without authentication
  - Call admin-only functions as non-admin address
  - Signature replay attacks (if applicable)

#### Authorization Mechanisms Found
- **Admin-Only Functions** (`contracts/certificate/src/lib.rs`):
  - `initialize(env, admin)` — sets admin, checks `is_initialized()`
  - `cleanup_expired_certificates(env, caller)` — requires admin via `require_admin()`
- **Multi-Sig Approvers** (`contracts/certificate/src/types.rs`):
  - `authorized_approvers: Vec<Address>` — list of approved signers
  - Approval check: verify caller is in `authorized_approvers`
- **Bypass vectors to test**:
  - Call `initialize()` twice (should fail with `AlreadyInitialized`)
  - Call `cleanup_expired_certificates()` as non-admin
  - Approve multi-sig request as unauthorized address
  - Horizontal escalation: access other students' certificates

#### Storage Access Patterns
- **Storage Keys**: Enum-based (`CertDataKey`) with namespacing
- **User-Supplied Input in Keys**:
  - `StudentCertificates(Address)` — keyed by student address
  - `CourseStudentCertificate(String, Address)` — keyed by course ID and student
  - `MultiSigConfig(String)` — keyed by course ID
- **Vulnerability Status**: ✅ **NO STORAGE INJECTION** — Keys are strongly typed; no string concatenation

#### Cross-Contract Calls
- **Soroban SDK**: Uses `soroban_sdk::Address` for contract calls
- **Calls Found**: None identified in certificate contract (contract is self-contained)

#### Event Emissions
- **Events Module**: `contracts/certificate/src/events.rs` (not fully read, but referenced)
- **Vulnerability Status**: Events are typically safe; will verify no sensitive data is logged

### Test Framework & Utilities

#### Backend API Tests
- **Framework**: Jest (`jest` v29.7.0, `ts-jest` preset)
- **Test Location**: `api/src/**/__tests__/**/*.test.ts`
- **Existing Test Pattern** (`api/src/middleware/__tests__/analyticsConsent.test.ts`):
  - Helper functions: `makeReq()`, `makeRes()`, `makeNext()`
  - Mocking: Jest mocks for middleware
  - Assertions: Jest matchers (`expect()`)
- **Test Utilities to Reuse**:
  - Request/Response mocking pattern
  - Middleware testing pattern
  - Jest setup and teardown

#### Smart Contract Tests
- **Framework**: Soroban SDK test utilities (Rust)
- **Test Location**: `contracts/certificate/src/test.rs`
- **Existing Test Pattern**:
  - `setup_env()` helper: creates `Env`, contract client, and admin address
  - `make_cert_params()` helper: creates certificate parameters
  - Test structure: `#[test]` attribute, assertions via `assert_eq!()` and `assert!()`
- **Test Utilities to Reuse**:
  - `setup_env()` function
  - Certificate parameter builders
  - Soroban contract invocation patterns

---

## Dependency Analysis

### Backend Dependencies (from `api/package.json`)
| Package | Version | Security Role | Known CVEs |
|---|---|---|---|
| `jsonwebtoken` | ^9.0.2 | JWT signing/verification | ✅ No known critical CVEs |
| `pg` | ^8.11.3 | PostgreSQL driver | ✅ No known critical CVEs |
| `express` | ^4.18.2 | Web framework | ✅ No known critical CVEs |
| `helmet` | ^7.1.0 | Security headers | ✅ No known critical CVEs |
| `cors` | ^2.8.5 | CORS middleware | ✅ No known critical CVEs |
| `express-rate-limit` | ^7.1.5 | Rate limiting | ✅ No known critical CVEs |
| `zod` | ^3.22.4 | Input validation | ✅ No known critical CVEs |

### Smart Contract Dependencies (from `Cargo.toml`)
| Package | Version | Security Role | Known CVEs |
|---|---|---|---|
| `soroban-sdk` | 22.0.0 | Contract framework | ✅ No known critical CVEs |
| `ed25519-dalek` | 2.0.0 | Signature verification | ✅ No known critical CVEs |

---

## Vulnerabilities Found During Reconnaissance

### Critical Vulnerabilities

#### SEC-001: CSRF Vulnerability on State-Changing Endpoints
- **Severity**: High
- **Layer**: Backend API
- **Location**: All state-changing endpoints (POST, PUT, PATCH, DELETE)
- **Description**: No CSRF protection mechanism found. Endpoints accept state-changing requests without CSRF tokens, Origin validation, or SameSite cookie protection.
- **Affected Endpoints**:
  - `POST /api/v1/auth/token`
  - All POST/PUT/PATCH/DELETE endpoints in routes
- **Proof of Concept**: Submit a state-changing request from a cross-origin context without any CSRF token
- **Fix Required**: Implement CSRF token validation or Origin/Referer header validation
- **Status**: To be fixed in this PR

#### SEC-002: Missing Authorization Check on Certain Endpoints
- **Severity**: High (if confirmed)
- **Layer**: Backend API
- **Location**: To be determined during testing
- **Description**: Some endpoints may lack authorization checks, allowing unauthorized access
- **Status**: To be discovered during testing

### Medium Vulnerabilities

#### SEC-003: CSP Allows Unsafe-Inline Scripts
- **Severity**: Medium
- **Layer**: Backend API
- **Location**: `api/src/app.ts` (Helmet CSP configuration)
- **Description**: CSP directive `scriptSrc: ["'self'", "'unsafe-inline'"]` allows inline scripts, reducing XSS protection
- **Justification**: Required for Swagger UI functionality
- **Fix**: Consider using nonce-based CSP or moving Swagger UI to separate domain
- **Status**: Deferred (acceptable for development; should be addressed in production)

---

## Test Coverage Goals

### Security-Critical Paths (Target: ≥95% coverage)
1. **Authentication checks**: Every authenticated endpoint
2. **Authorization checks**: Every role/permission decision
3. **Input validation**: Every user-supplied input field
4. **Output encoding**: Every response containing user data
5. **CSRF protection**: Every state-changing endpoint
6. **Rate limiting**: Bypass attempts

### Test Organization

#### Backend API Security Tests
- File: `api/src/__tests__/security.test.ts`
- Groups:
  - Authentication bypass tests
  - Authorization bypass tests
  - SQL injection tests (if applicable)
  - CSRF vulnerability tests
  - Rate limiting bypass tests
  - Input validation tests

#### Smart Contract Security Tests
- File: `contracts/certificate/src/security_tests.rs`
- Groups:
  - Authentication bypass tests
  - Authorization bypass tests
  - Storage injection tests
  - Reentrancy tests (if applicable)

---

## CI/CD Configuration

### Build & Test Commands
- **Contracts**: `cargo build --workspace`
- **API**: `npm run build` (TypeScript compilation)
- **Linting**: `npm run lint` (ESLint)
- **Tests**: `npm test` (Jest)
- **Coverage**: `npm run test -- --coverage` (Jest coverage)
- **Audit**: `cargo audit` (Rust dependencies), `npm audit` (Node dependencies)

### Security Checks
- ✅ Compilation without errors
- ✅ Linting without errors
- ✅ All tests passing
- ✅ Coverage ≥95% on security-critical paths
- ✅ No High/Critical dependency vulnerabilities

---

## Implementation Plan

### Phase 1: Backend API Security Tests
1. Authentication bypass tests (JWT validation)
2. Authorization bypass tests (scope/role checks)
3. CSRF vulnerability tests (state-changing endpoints)
4. Rate limiting bypass tests
5. Input validation tests

### Phase 2: Smart Contract Security Tests
1. Authentication bypass tests (admin checks)
2. Authorization bypass tests (multi-sig approvers)
3. Storage injection tests
4. Reentrancy tests (if applicable)

### Phase 3: Vulnerability Fixes
1. Implement CSRF protection (token-based or Origin validation)
2. Fix any discovered authorization gaps
3. Verify all fixes with corresponding tests

### Phase 4: Documentation
1. Create `SECURITY_FINDINGS.md` with all vulnerabilities and fixes
2. Update existing security documentation (if any)
3. Add instructions for running security test suite

---

## Vacuousness Check Strategy

Every negative security test will include a vacuousness check that:
1. Temporarily disables or removes the security control
2. Verifies the attack succeeds without the control
3. Re-enables the control and verifies the attack is blocked
4. Documents the check in a code comment

This ensures tests are not passing vacuously due to missing guards.

---

## Scope Discipline

### Files to Modify
- **New**: `api/src/__tests__/security.test.ts` (backend security tests)
- **New**: `contracts/certificate/src/security_tests.rs` (contract security tests)
- **New**: `SECURITY_FINDINGS.md` (vulnerability documentation)
- **Modified**: Source files requiring security fixes (minimum changes only)

### Files NOT to Modify
- Unrelated source files
- Formatting-only changes
- Dependency upgrades not required for security fixes
- Refactoring outside the scope of security hardening

---

## Next Steps

1. ✅ Reconnaissance complete
2. ⏳ Implement backend API security tests
3. ⏳ Implement smart contract security tests
4. ⏳ Fix discovered vulnerabilities
5. ⏳ Create vulnerability documentation
6. ⏳ Run all CI checks locally
7. ⏳ Open PR with comprehensive security test suite
