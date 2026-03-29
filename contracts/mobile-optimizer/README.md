# Mobile Optimizer Contract

## Purpose

The Mobile Optimizer contract provides a comprehensive on-chain infrastructure for mobile-first interactions with the StrellerMinds platform. It tackles the unique challenges of mobile blockchain usage: unreliable networks, battery constraints, and the need for seamless offline-to-online transitions. The contract manages mobile session lifecycles (create, suspend, resume, end), batches multiple operations into single transactions to reduce gas costs, estimates gas for operations given current network quality, exposes quick interaction flows for common learning actions, manages offline operation queues with conflict resolution and sync, caches content for offline access, handles multi-device registration and state synchronization, orchestrates Progressive Web App (PWA) capabilities, and provides comprehensive analytics and network adaptation.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — 50+ public functions organized across 15 domain sections |
| `session_manager.rs` | `SessionManager` — session CRUD, suspend/resume, cross-device state sync, optimization analysis |
| `batch_manager.rs` | `BatchManager` — batch creation, execution, and cancellation with configurable priority and strategy |
| `gas_optimizer.rs` | `GasOptimizer` — per-operation gas estimation, mobile gas optimization tips |
| `interaction_flows.rs` | `InteractionFlows` — quick mobile-optimized flows for enrollment, completion, and certificate viewing |
| `network_manager.rs` | `NetworkManager` — network quality adaptation, bandwidth optimization, connection settings |
| `offline_manager.rs` | `OfflineManager` — offline operation queuing, sync, conflict resolution, capability reporting |
| `content_cache.rs` | `ContentCacheManager` — content caching with TTL management |
| `content_manager.rs` | `ContentManager` — mobile content delivery management |
| `notification_manager.rs` | `NotificationManager` — push notification and reminder scheduling |
| `pwa_manager.rs` | `PwaManager` — PWA capability management and offline capability reporting |
| `security_manager.rs` | `SecurityManager` — mobile security policies, biometric auth, device-level access control |
| `battery_optimizer.rs` | `BatteryOptimizer` — battery-aware settings that reduce operation frequency on low charge |
| `collaboration_manager.rs` | `CollaborationManager` — real-time collaborative learning session management |
| `user_experience_manager.rs` | `UserExperienceManager` — UI preference and accessibility settings per user |
| `analytics_monitor.rs` | `AnalyticsMonitor` — mobile-specific usage analytics collection |
| `types.rs` | All `contracttype`-derived structs for mobile sessions, batches, preferences, network state, offline ops, device info, etc. |
| `errors.rs` | `MobileOptimizerError` — 37 typed error variants |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; stores default config and counters | Admin |
| `get_config()` | Returns the current optimizer configuration | None |
| `update_config(admin, config)` | Replaces the optimizer configuration | Admin |
| **Sessions** | | |
| `create_session(user, device_id, preferences)` | Creates a new mobile session; returns session ID | User |
| `get_session(user, session_id)` | Retrieves an existing session by ID | User |
| `update_session(user, session_id, network_quality)` | Updates network quality on an active session | User |
| `update_mobile_preferences(user, session_id, preferences)` | Updates mobile preferences on a session | User |
| `suspend_session(user, session_id)` | Suspends an active session | User |
| `resume_session(user, session_id, network_quality)` | Resumes a suspended session | User |
| `end_session(user, session_id)` | Terminates an active session | User |
| `get_session_stats(user)` | Returns aggregate session statistics for a user | User |
| `optimize_session(user, session_id)` | Returns performance optimization recommendations for a session | User |
| `sync_session_state(user, source_session_id, target_device_id)` | Clones a session's state to a new device | User |
| **Batch Execution** | | |
| `create_batch(user, operations, priority, strategy)` | Creates a batch of operations; returns batch ID | User |
| `execute_batch(user, batch_id)` | Executes a pending batch; returns execution result | User |
| `cancel_batch(user, batch_id)` | Cancels a pending batch before execution | User |
| **Gas Optimization** | | |
| `estimate_gas(operations, network_quality)` | Returns gas estimates for a list of operations | None |
| `get_gas_tips()` | Returns mobile-specific gas optimization tips | None |
| **Quick Interaction Flows** | | |
| `quick_enroll(user, course_id, session_id)` | Mobile-optimized course enrollment flow | User |
| `quick_complete_module(user, course_id, module_id, session_id)` | Mobile-optimized module completion flow | User |
| `quick_view_certificate(user, certificate_id, session_id)` | Mobile-optimized certificate viewing flow | User |
| **Network Management** | | |
| `adapt_to_network(user, session_id, network_quality)` | Adapts session settings to current network conditions | User |
| `get_network_stats(user, session_id)` | Returns network statistics for a session | User |
| `optimize_bandwidth(user, session_id)` | Returns bandwidth optimization recommendations | User |
| **Offline Operations** | | |
| `queue_offline_operation(user, operation_type, data, priority)` | Queues an operation for later sync | User |
| `sync_offline_operations(user)` | Syncs all queued offline operations to the network | User |
| `get_offline_queue_status(user)` | Returns the current offline operation queue status | User |
| `resolve_offline_conflicts(user, resolutions)` | Applies conflict resolutions for synced operations | User |
| `get_offline_capabilities(user)` | Returns supported offline capabilities for the user | User |

## Usage Example

```
# 1. Admin initializes the contract
mobile_optimizer.initialize(admin)

# 2. User creates a session on their mobile device
session_id = mobile_optimizer.create_session(student, "iPhone-14-ABC", {
    data_saver_mode: true,
    offline_mode_enabled: true,
    ...
})

# 3. Student goes offline — queue an operation for later sync
mobile_optimizer.queue_offline_operation(student, "ModuleComplete", {course: "RUST101", module: "M1"}, Priority::Normal)

# 4. When back online, sync queued operations
sync_result = mobile_optimizer.sync_offline_operations(student)

# 5. Student uses quick interaction flows for common actions
enroll_result = mobile_optimizer.quick_enroll(student, "RUST101", session_id)

# 6. Batch multiple operations to reduce gas
batch_id = mobile_optimizer.create_batch(student, [op1, op2, op3], BatchPriority::Normal, ExecutionStrategy::Sequential)
result = mobile_optimizer.execute_batch(student, batch_id)

# 7. End session when done
mobile_optimizer.end_session(student, session_id)
```

## Errors

| Error | Code | Description |
|---|---|---|
| `NotInitialized` | 1 | Contract has not been initialized |
| `AlreadyInitialized` | 2 | Contract has already been initialized |
| `SessionCreationFailed` | 3 | Creating a new session failed |
| `SessionUpdateFailed` | 4 | Updating an existing session failed |
| `SessionNotFound` | 5 | No session found for the specified ID |
| `SessionExpired` | 6 | Session has expired |
| `BatchExecutionFailed` | 7 | Batch execution failed |
| `BatchNotFound` | 8 | No batch found for the specified ID |
| `BatchExpired` | 9 | Batch expired before execution completed |
| `GasEstimationFailed` | 10 | Gas cost estimation failed |
| `OptimizationFailed` | 11 | Optimization process failed to produce a result |
| `InteractionFailed` | 12 | Quick interaction flow failed |
| `OfflineOperationFailed` | 13 | Queuing or executing an offline operation failed |
| `OfflineSyncFailed` | 14 | Syncing offline operations failed |
| `OfflineQueueFull` | 15 | Offline operation queue is at capacity |
| `ConflictResolutionFailed` | 16 | Resolving offline operation conflicts failed |
| `PreferenceUpdateFailed` | 17 | Updating mobile preferences failed |
| `AnalyticsNotAvailable` | 18 | Analytics data is not available |
| `ConfigNotFound` | 19 | Optimizer configuration not found in storage |
| `AdminNotSet` | 20 | Admin address not set in storage |
| `UnauthorizedAdmin` | 21 | Caller is not the authorized admin |
| `Unauthorized` | 22 | Caller is not authorized |
| `CacheError` | 23 | Content cache read/write operation failed |
| `CacheFull` | 24 | Content cache has reached its size limit |
| `DeviceNotRegistered` | 25 | Device has not been registered for this user |
| `MaxDevicesReached` | 26 | User has reached the maximum number of registered devices |
| `SyncFailed` | 27 | Data synchronization operation failed |
| `SecurityViolation` | 28 | A security policy violation was detected |
| `BiometricAuthFailed` | 29 | Biometric authentication verification failed |
| `AccountLocked` | 30 | Account is locked due to failed authentication attempts |
| `NotificationError` | 31 | Push notification operation failed |
| `PwaError` | 32 | Progressive Web App operation failed |
| `InvalidInput` | 33 | Input value is invalid or out of range |
| `InternalError` | 34 | Unexpected internal error |
| `ContentError` | 35 | Content management operation failed |
| `CollaborationError` | 36 | Collaboration feature operation failed |
| `UserExperienceError` | 37 | User experience operation failed |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `analytics` | Mobile session and interaction analytics feed the analytics contract |
| `progress` | Quick completion flows trigger progress updates via the progress contract |
| `certificate` | Quick certificate viewing flows query the certificate contract |
| `gamification` | Mobile activity flows can trigger gamification activity records |
| `security-monitor` | Biometric authentication and security policy checks interact with the security monitor |
