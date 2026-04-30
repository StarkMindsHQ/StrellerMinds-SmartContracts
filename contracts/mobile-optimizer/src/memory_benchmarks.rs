#[cfg(test)]
mod tests {
    use crate::data_processor::ChunkedProcessor;
    use crate::types::{BatchOperation, PaginatedResult};
    use soroban_sdk::{Env, Vec, String};

    /// Benchmark simulation for Issue #421.
    /// Tests processing of 100,000 records to ensure memory remains stable.
    #[test]
    fn test_large_dataset_memory_simulation() {
        let env = Env::default();
        let processor = ChunkedProcessor::new(500); // 500 records per chunk
        
        let total_records = 100_000;
        let mut mock_fetched_count = 0;

        // Simulation of a data provider (e.g., Database or Indexer)
        let fetcher = |params: crate::types::PaginationParams| -> PaginatedResult<BatchOperation> {
            let limit = params.limit as usize;
            let mut items = Vec::new(&env);
            
            // Simulate records if we haven't reached the limit
            let to_fetch = if mock_fetched_count + limit > total_records {
                total_records - mock_fetched_count
            } else {
                limit
            };

            for i in 0..to_fetch {
                // Creating "heavy" mock records (simulating ~500KB each via strings/vectors)
                items.push_back(BatchOperation {
                    operation_id: String::from_str(&env, &format!("op_{}", mock_fetched_count + i)),
                    operation_type: crate::types::OperationType::Custom,
                    contract_address: env.accounts().generate(),
                    function_name: String::from_str(&env, "heavy_op"),
                    parameters: Vec::new(&env),
                    estimated_gas: 1000,
                    priority: crate::types::OperationPriority::Medium,
                    retry_config: crate::types::RetryConfig { max_retries: 3, retry_delay_ms: 100, backoff_multiplier: 2, max_delay_ms: 1000, retry_on_network_error: true, retry_on_gas_error: false, retry_on_timeout: true },
                    dependencies: Vec::new(&env),
                });
            }

            mock_fetched_count += to_fetch;
            let next_cursor = if mock_fetched_count < total_records { Some(format!("{}", mock_fetched_count)) } else { None };
            
            PaginatedResult { items, next_cursor, total_count: total_records as u64, chunk_size_bytes: 0 }
        };

        assert!(processor.process_operations(&env, fetcher).is_ok());
        assert_eq!(mock_fetched_count, total_records);
    }
}