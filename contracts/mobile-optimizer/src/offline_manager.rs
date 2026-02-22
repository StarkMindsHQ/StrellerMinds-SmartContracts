use soroban_sdk::{contracttype, Address, Env, String, Vec};

use crate::types::*;

pub struct OfflineManager;

impl OfflineManager {
    pub fn queue_operation(
        env: &Env,
        user: Address,
        device_id: String,
        operation: QueuedOperation,
    ) -> Result<(), MobileOptimizerError> {
        let mut queue: OfflineQueue = env
            .storage()
            .persistent()
            .get(&DataKey::OfflineQueue(user.clone()))
            .unwrap_or_else(|| Self::create_empty_queue(env, &user, &device_id));

        if queue.device_id != device_id {
            return Err(MobileOptimizerError::InvalidInput);
        }

        queue.total_estimated_gas += operation.estimated_gas;
        queue.queued_operations.push_back(operation);
        queue.last_sync_attempt = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::OfflineQueue(user), &queue);
        Ok(())
    }

    pub fn sync_offline_operations(
        env: &Env,
        user: Address,
        device_id: String,
        _network_quality: NetworkQuality,
    ) -> Result<OfflineSyncResult, MobileOptimizerError> {
        let mut queue: OfflineQueue = env
            .storage()
            .persistent()
            .get(&DataKey::OfflineQueue(user.clone()))
            .ok_or(MobileOptimizerError::OfflineSyncFailed)?;

        if queue.device_id != device_id {
            return Err(MobileOptimizerError::InvalidInput);
        }

        queue.sync_status = SyncStatus::Syncing;
        queue.last_sync_attempt = env.ledger().timestamp();

        let total_ops = queue.queued_operations.len();
        let mut successful = 0u32;
        let mut failed = 0u32;
        let mut conflicts = 0u32;

        let mut updated_ops = Vec::new(env);
        for op in queue.queued_operations.iter() {
            let mut o = op.clone();
            match Self::sync_single_operation(env, &o) {
                Ok(()) => {
                    o.status = QueuedOperationStatus::Synced;
                    successful += 1;
                }
                Err(MobileOptimizerError::ConflictResolutionFailed) => {
                    o.status = QueuedOperationStatus::Conflict;
                    conflicts += 1;
                }
                Err(_) => {
                    o.status = QueuedOperationStatus::Failed;
                    failed += 1;
                }
            }
            updated_ops.push_back(o);
        }
        queue.queued_operations = updated_ops;

        queue.sync_status = if conflicts > 0 {
            SyncStatus::Conflicts
        } else if failed > 0 {
            SyncStatus::SyncFailed
        } else {
            SyncStatus::InSync
        };

        env.storage()
            .persistent()
            .set(&DataKey::OfflineQueue(user), &queue);

        Ok(OfflineSyncResult {
            total_operations: total_ops,
            successful_syncs: successful,
            failed_syncs: failed,
            conflicts_detected: conflicts,
            sync_status: queue.sync_status,
        })
    }

    pub fn get_queue_status(
        env: &Env,
        user: &Address,
        device_id: &String,
    ) -> Result<OfflineQueueStatus, MobileOptimizerError> {
        let queue: OfflineQueue = env
            .storage()
            .persistent()
            .get(&DataKey::OfflineQueue(user.clone()))
            .ok_or(MobileOptimizerError::OfflineOperationFailed)?;

        if queue.device_id != *device_id {
            return Err(MobileOptimizerError::InvalidInput);
        }

        let mut conflict_count = 0u32;
        let mut pending_count = 0u32;
        for op in queue.queued_operations.iter() {
            if op.status == QueuedOperationStatus::Conflict {
                conflict_count += 1;
            }
            if op.status == QueuedOperationStatus::Queued {
                pending_count += 1;
            }
        }

        Ok(OfflineQueueStatus {
            total_operations: queue.queued_operations.len(),
            pending_operations: pending_count,
            sync_status: queue.sync_status,
            last_sync_attempt: queue.last_sync_attempt,
            conflicts_count: conflict_count,
            estimated_sync_time_ms: pending_count * 500,
        })
    }

    pub fn resolve_conflicts(
        env: &Env,
        user: Address,
        device_id: String,
        resolution_strategy: ConflictResolution,
    ) -> Result<u32, MobileOptimizerError> {
        let mut queue: OfflineQueue = env
            .storage()
            .persistent()
            .get(&DataKey::OfflineQueue(user.clone()))
            .ok_or(MobileOptimizerError::OfflineOperationFailed)?;

        if queue.device_id != device_id {
            return Err(MobileOptimizerError::InvalidInput);
        }

        let mut resolved = 0u32;
        let mut updated_ops = Vec::new(env);

        for op in queue.queued_operations.iter() {
            let mut o = op.clone();
            if o.status == QueuedOperationStatus::Conflict {
                match resolution_strategy {
                    ConflictResolution::ServerWins | ConflictResolution::Abort => {
                        o.status = QueuedOperationStatus::Cancelled;
                        resolved += 1;
                    }
                    ConflictResolution::ClientWins
                    | ConflictResolution::MergeChanges
                    | ConflictResolution::UserDecision => {
                        o.status = QueuedOperationStatus::Queued;
                        resolved += 1;
                    }
                }
            }
            updated_ops.push_back(o);
        }

        queue.queued_operations = updated_ops;
        queue.conflict_resolution = resolution_strategy;
        env.storage()
            .persistent()
            .set(&DataKey::OfflineQueue(user), &queue);

        Ok(resolved)
    }

    pub fn cleanup_completed_operations(
        env: &Env,
        user: Address,
        device_id: String,
    ) -> Result<u32, MobileOptimizerError> {
        let mut queue: OfflineQueue = env
            .storage()
            .persistent()
            .get(&DataKey::OfflineQueue(user.clone()))
            .ok_or(MobileOptimizerError::OfflineOperationFailed)?;

        if queue.device_id != device_id {
            return Err(MobileOptimizerError::InvalidInput);
        }

        let mut kept = Vec::new(env);
        let mut cleaned = 0u32;

        for op in queue.queued_operations.iter() {
            match op.status {
                QueuedOperationStatus::Synced | QueuedOperationStatus::Cancelled => {
                    cleaned += 1;
                }
                _ => {
                    kept.push_back(op.clone());
                }
            }
        }

        queue.queued_operations = kept;
        env.storage()
            .persistent()
            .set(&DataKey::OfflineQueue(user), &queue);
        Ok(cleaned)
    }

    pub fn get_offline_capabilities(env: &Env) -> OfflineCapabilities {
        let mut supported = Vec::new(env);
        supported.push_back(OperationType::ProgressUpdate);
        supported.push_back(OperationType::PreferenceUpdate);
        supported.push_back(OperationType::SearchQuery);
        supported.push_back(OperationType::CourseEnrollment);
        supported.push_back(OperationType::ContentCache);
        supported.push_back(OperationType::AnalyticsEvent);

        OfflineCapabilities {
            supported_operations: supported,
            max_queue_size: 100,
            max_offline_duration_hours: 168,
        }
    }

    fn sync_single_operation(
        _env: &Env,
        _operation: &QueuedOperation,
    ) -> Result<(), MobileOptimizerError> {
        // Simulated sync: all operations succeed
        Ok(())
    }

    fn create_empty_queue(env: &Env, user: &Address, device_id: &String) -> OfflineQueue {
        OfflineQueue {
            user: user.clone(),
            device_id: device_id.clone(),
            queued_operations: Vec::new(env),
            total_estimated_gas: 0,
            created_at: env.ledger().timestamp(),
            last_sync_attempt: 0,
            sync_status: SyncStatus::Offline,
            conflict_resolution: ConflictResolution::UserDecision,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineSyncResult {
    pub total_operations: u32,
    pub successful_syncs: u32,
    pub failed_syncs: u32,
    pub conflicts_detected: u32,
    pub sync_status: SyncStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineQueueStatus {
    pub total_operations: u32,
    pub pending_operations: u32,
    pub sync_status: SyncStatus,
    pub last_sync_attempt: u64,
    pub conflicts_count: u32,
    pub estimated_sync_time_ms: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineCapabilities {
    pub supported_operations: Vec<OperationType>,
    pub max_queue_size: u32,
    pub max_offline_duration_hours: u32,
}
