use soroban_sdk::{contracttype, Address, Env, String, Vec};

use crate::types::*;

pub struct BatchManager;

impl BatchManager {
    pub fn create_batch(
        env: &Env,
        user: Address,
        operations: Vec<BatchOperation>,
        priority: BatchPriority,
        execution_strategy: ExecutionStrategy,
    ) -> Result<String, MobileOptimizerError> {
        let batch_id = Self::generate_batch_id(env);

        let estimated_gas = Self::estimate_batch_gas(&operations);
        let retry_config = Self::create_retry_config(&priority);

        let batch = TransactionBatch {
            batch_id: batch_id.clone(),
            user: user.clone(),
            operations,
            estimated_gas,
            priority: priority.clone(),
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + Self::get_expiry_duration(&priority),
            status: BatchStatus::Pending,
            execution_strategy,
            retry_config,
            network_quality: NetworkQuality::Good,
        };

        env.storage()
            .persistent()
            .set(&DataKey::TransactionBatch(batch_id.clone()), &batch);
        Self::add_to_user_batches(env, &user, &batch_id);

        Ok(batch_id)
    }

    pub fn execute_batch(
        env: &Env,
        batch_id: String,
        user: Address,
    ) -> Result<BatchExecutionResult, MobileOptimizerError> {
        let mut batch: TransactionBatch = env
            .storage()
            .persistent()
            .get(&DataKey::TransactionBatch(batch_id.clone()))
            .ok_or(MobileOptimizerError::BatchNotFound)?;

        if batch.user != user {
            return Err(MobileOptimizerError::Unauthorized);
        }

        if env.ledger().timestamp() > batch.expires_at {
            batch.status = BatchStatus::Expired;
            env.storage()
                .persistent()
                .set(&DataKey::TransactionBatch(batch_id), &batch);
            return Err(MobileOptimizerError::BatchExpired);
        }

        batch.status = BatchStatus::Executing;
        env.storage()
            .persistent()
            .set(&DataKey::TransactionBatch(batch_id.clone()), &batch);

        let result = Self::execute_sequential(env, &mut batch);

        batch.status = if result.failed_count == 0 {
            BatchStatus::Completed
        } else if result.successful_count > 0 {
            BatchStatus::PartialSuccess
        } else {
            BatchStatus::Failed
        };

        env.storage()
            .persistent()
            .set(&DataKey::TransactionBatch(batch_id), &batch);

        Ok(result)
    }

    fn execute_sequential(env: &Env, batch: &mut TransactionBatch) -> BatchExecutionResult {
        let mut successful_ids = Vec::new(env);
        let mut failed_ids = Vec::new(env);
        let mut total_gas = 0u64;

        for i in 0..batch.operations.len() {
            if let Some(operation) = batch.operations.get(i) {
                match Self::execute_operation(env, &operation) {
                    Ok(gas_used) => {
                        successful_ids.push_back(operation.operation_id.clone());
                        total_gas += gas_used;
                    }
                    Err(_) => {
                        failed_ids.push_back(operation.operation_id.clone());
                    }
                }
            }
        }

        let sc = successful_ids.len() as u32;
        let fc = failed_ids.len() as u32;

        BatchExecutionResult {
            batch_id: batch.batch_id.clone(),
            successful_operations: successful_ids,
            failed_operations: failed_ids,
            total_gas_used: total_gas,
            execution_time_ms: 0,
            successful_count: sc,
            failed_count: fc,
        }
    }

    fn execute_operation(
        _env: &Env,
        operation: &BatchOperation,
    ) -> Result<u64, MobileOptimizerError> {
        Ok(operation.estimated_gas)
    }

    pub fn cancel_batch(
        env: &Env,
        batch_id: String,
        user: Address,
    ) -> Result<(), MobileOptimizerError> {
        let mut batch: TransactionBatch = env
            .storage()
            .persistent()
            .get(&DataKey::TransactionBatch(batch_id.clone()))
            .ok_or(MobileOptimizerError::BatchNotFound)?;

        if batch.user != user {
            return Err(MobileOptimizerError::Unauthorized);
        }
        if batch.status == BatchStatus::Executing {
            return Err(MobileOptimizerError::BatchExecutionFailed);
        }

        batch.status = BatchStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::TransactionBatch(batch_id), &batch);
        Ok(())
    }

    pub fn get_user_batches(env: &Env, user: &Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::UserBatches(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn estimate_batch_gas(operations: &Vec<BatchOperation>) -> u64 {
        let mut total = 0u64;
        for op in operations.iter() {
            total += op.estimated_gas;
        }
        total + total / 10 // 10% overhead
    }

    fn create_retry_config(priority: &BatchPriority) -> RetryConfig {
        match priority {
            BatchPriority::Critical => RetryConfig {
                max_retries: 5,
                retry_delay_ms: 100,
                backoff_multiplier: 2,
                max_delay_ms: 5000,
                retry_on_network_error: true,
                retry_on_gas_error: true,
                retry_on_timeout: true,
            },
            BatchPriority::High => RetryConfig {
                max_retries: 3,
                retry_delay_ms: 200,
                backoff_multiplier: 2,
                max_delay_ms: 10000,
                retry_on_network_error: true,
                retry_on_gas_error: true,
                retry_on_timeout: true,
            },
            BatchPriority::Normal => RetryConfig {
                max_retries: 2,
                retry_delay_ms: 500,
                backoff_multiplier: 2,
                max_delay_ms: 15000,
                retry_on_network_error: true,
                retry_on_gas_error: false,
                retry_on_timeout: true,
            },
            BatchPriority::Low => RetryConfig {
                max_retries: 1,
                retry_delay_ms: 1000,
                backoff_multiplier: 1,
                max_delay_ms: 30000,
                retry_on_network_error: true,
                retry_on_gas_error: false,
                retry_on_timeout: false,
            },
            BatchPriority::Background => RetryConfig {
                max_retries: 1,
                retry_delay_ms: 5000,
                backoff_multiplier: 1,
                max_delay_ms: 60000,
                retry_on_network_error: false,
                retry_on_gas_error: false,
                retry_on_timeout: false,
            },
        }
    }

    fn get_expiry_duration(priority: &BatchPriority) -> u64 {
        match priority {
            BatchPriority::Critical => 300,
            BatchPriority::High => 900,
            BatchPriority::Normal => 3600,
            BatchPriority::Low => 86400,
            BatchPriority::Background => 604800,
        }
    }

    fn generate_batch_id(env: &Env) -> String {
        String::from_str(env, "batch")
    }

    fn add_to_user_batches(env: &Env, user: &Address, batch_id: &String) {
        let mut batches: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::UserBatches(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        batches.push_back(batch_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserBatches(user.clone()), &batches);
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchExecutionResult {
    pub batch_id: String,
    pub successful_operations: Vec<String>,
    pub failed_operations: Vec<String>,
    pub total_gas_used: u64,
    pub execution_time_ms: u32,
    pub successful_count: u32,
    pub failed_count: u32,
}
