# API Reference

The StarkMinds Smart Contracts expose robust APIs for integration via the Stellar network using Soroban. Below is an overview of the core and supporting contract interfaces.

## 🌟 Core Contracts

### 📊 [Analytics Contract](../contracts/analytics/README.md)
Provides comprehensive learning analytics, detailed progress tracking, and performance metrics.
- **Key Functions**:
  - `initialize(env, admin, config)`
  - `record_session(env, session)`
  - `complete_session(env, session_id, end_time, final_score, completion_percentage)`
  - `get_progress_analytics(env, student, course_id)`

### 🪙 [Token Contract](../contracts/token/README.md)
Manages token incentives, staking capabilities, and reward mechanisms across the StarkMinds ecosystem.
- **Key Functions**:
  - `mint(env, to, amount)`
  - `transfer(env, from, to, amount)`
  - `stake(env, user, amount)`

### 🔒 [Shared Utilities](../contracts/shared/README.md)
Implements common infrastructure like Role-Based Access Control (RBAC) and reentrancy protections.
- **Key Functions**:
  - `require_role(env, address, role)`
  - `grant_role(env, admin, address, role)`

## 🛠️ Supporting Contracts

### 📱 [Mobile Optimizer](../contracts/mobile-optimizer/README.md)
Handles data syncing and optimized gas usage for mobile clients acting in offline environments.

### 📈 [Progress Tracking](../contracts/progress/README.md)
Basic progress recording and validation for specific courses and assignments.

### 🔄 [Proxy System](../contracts/proxy/README.md)
Facilitates smart contract upgrades using the proxy pattern to preserve state across new logic deployments.

### 🔍 [Search System](../contracts/search/README.md)
Provides query tools for locating public credentials, courses, and student performance statistics on-chain.

### 🎓 [Student Progress Tracker](../contracts/student-progress-tracker/README.md)
Offers granular module-level tracking to measure progression within specific curriculum modules.

---

*For detailed SDK integrations, please reference the [Development Guide](development.md).*
