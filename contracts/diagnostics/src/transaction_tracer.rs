use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Symbol, Vec};

/// Transaction flow tracer and analyzer
pub struct TransactionTracer;

impl TransactionTracer {
    /// Start tracing a transaction
    pub fn start_trace(
        env: &Env,
        contract_id: &Address,
        function_name: Symbol,
        caller: &Address,
    ) -> Symbol {
        let trace_id = Self::generate_trace_id(env, contract_id, &function_name);
        let timestamp = env.ledger().timestamp();

        // Store trace start time for duration calculation
        env.storage().persistent().set(
            &(Symbol::new(env, "trace_start"), trace_id.clone()),
            &timestamp,
        );

        trace_id
    }

    /// Complete a trace and record metrics
    pub fn complete_trace(
        env: &Env,
        trace_id: Symbol,
        contract_id: &Address,
        function_name: Symbol,
        caller: &Address,
        success: bool,
        error_message: Option<String>,
        child_calls: Vec<Symbol>,
        gas_used: u64,
    ) -> TransactionTrace {
        let end_timestamp = env.ledger().timestamp();
        
        // Get start time
        let start_timestamp: u64 = env
            .storage()
            .persistent()
            .get(&(Symbol::new(env, "trace_start"), trace_id.clone()))
            .unwrap_or(end_timestamp);

        let execution_time_ms = ((end_timestamp - start_timestamp) * 1000) as u32;

        // Count events emitted (simplified)
        let events_emitted = Self::count_events_in_trace(env, &trace_id);

        let trace = TransactionTrace {
            trace_id: trace_id.clone(),
            contract_id: contract_id.clone(),
            function_name,
            caller: caller.clone(),
            timestamp: end_timestamp,
            execution_time_ms,
            gas_used,
            success,
            error_message,
            child_calls,
            events_emitted,
        };

        // Store the trace
        Self::store_trace(env, &trace);

        trace
    }

    /// Analyze transaction flow patterns
    pub fn analyze_flow_patterns(
        env: &Env,
        traces: &Vec<TransactionTrace>,
    ) -> Map<Symbol, u32> {
        let mut patterns = Map::new(env);

        for trace in traces {
            // Count function calls
            let func_key = Symbol::new(env, "func_calls");
            let current_count = patterns.get(func_key.clone()).unwrap_or(0);
            patterns.set(func_key, current_count + 1);

            // Count successful vs failed
            if trace.success {
                let success_key = Symbol::new(env, "successful");
                let count = patterns.get(success_key.clone()).unwrap_or(0);
                patterns.set(success_key, count + 1);
            } else {
                let fail_key = Symbol::new(env, "failed");
                let count = patterns.get(fail_key.clone()).unwrap_or(0);
                patterns.set(fail_key, count + 1);
            }

            // Track child call depth
            if !trace.child_calls.is_empty() {
                let depth_key = Symbol::new(env, "with_subcalls");
                let count = patterns.get(depth_key.clone()).unwrap_or(0);
                patterns.set(depth_key, count + 1);
            }
        }

        patterns
    }

    /// Build transaction call tree visualization
    pub fn build_call_tree(
        env: &Env,
        trace: &TransactionTrace,
    ) -> String {
        // Simple text representation of call tree
        let mut tree = String::from_str(env, "");
        
        tree = String::from_str(
            env,
            &format!(
                "{} -> {} ({}ms, {} gas)",
                trace.function_name.to_string(),
                if trace.success { "SUCCESS" } else { "FAILED" },
                trace.execution_time_ms,
                trace.gas_used
            ),
        );

        // Add child calls
        if !trace.child_calls.is_empty() {
            for child in &trace.child_calls {
                tree = String::from_str(
                    env,
                    &format!("{}
  └─ {}", tree.to_string(), child.to_string()),
                );
            }
        }

        tree
    }

    /// Detect unusual transaction patterns
    pub fn detect_unusual_patterns(
        env: &Env,
        recent_traces: &Vec<TransactionTrace>,
        baseline_avg_time: u32,
        baseline_avg_gas: u64,
    ) -> Vec<String> {
        let mut anomalies = Vec::new(env);

        for trace in recent_traces {
            // Check for unusually long execution
            if trace.execution_time_ms > baseline_avg_time * 3 {
                anomalies.push_back(String::from_str(
                    env,
                    &format!(
                        "Slow execution: {}ms (baseline: {}ms)",
                        trace.execution_time_ms, baseline_avg_time
                    ),
                ));
            }

            // Check for unusually high gas
            if trace.gas_used > baseline_avg_gas * 3 {
                anomalies.push_back(String::from_str(
                    env,
                    &format!(
                        "High gas usage: {} (baseline: {})",
                        trace.gas_used, baseline_avg_gas
                    ),
                ));
            }

            // Check for excessive child calls (potential recursion issue)
            if trace.child_calls.len() > 10 {
                anomalies.push_back(String::from_str(
                    env,
                    &format!(
                        "Excessive child calls: {} (potential recursion)",
                        trace.child_calls.len()
                    ),
                ));
            }
        }

        anomalies
    }

    /// Get transaction statistics
    pub fn get_trace_statistics(
        env: &Env,
        traces: &Vec<TransactionTrace>,
    ) -> Map<Symbol, u64> {
        let mut stats = Map::new(env);

        if traces.is_empty() {
            return stats;
        }

        let mut total_time = 0u64;
        let mut total_gas = 0u64;
        let mut success_count = 0u32;

        for trace in traces {
            total_time += trace.execution_time_ms as u64;
            total_gas += trace.gas_used;
            if trace.success {
                success_count += 1;
            }
        }

        let count = traces.len() as u64;
        stats.set(Symbol::new(env, "total_traces"), count);
        stats.set(Symbol::new(env, "avg_time_ms"), total_time / count);
        stats.set(Symbol::new(env, "avg_gas"), total_gas / count);
        stats.set(
            Symbol::new(env, "success_rate"),
            (success_count as u64 * 100) / count,
        );

        stats
    }

    // Helper functions

    fn generate_trace_id(
        env: &Env,
        contract_id: &Address,
        function_name: &Symbol,
    ) -> Symbol {
        // Generate unique trace ID
        let timestamp = env.ledger().timestamp();
        Symbol::new(
            env,
            &format!(
                "trace_{}_{}_{}",
                contract_id.to_string().chars().take(8).collect::<String>(),
                function_name.to_string(),
                timestamp
            ),
        )
    }

    fn count_events_in_trace(_env: &Env, _trace_id: &Symbol) -> u32 {
        // In production, count actual events
        // For now, return estimated value
        2
    }

    fn store_trace(env: &Env, trace: &TransactionTrace) {
        // Store trace for later analysis
        env.storage().persistent().set(
            &(Symbol::new(env, "trace"), trace.trace_id.clone()),
            trace,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_start_and_complete_trace() {
        let env = Env::default();
        let contract_id = Address::generate(&env);
        let caller = Address::generate(&env);
        let function = Symbol::new(&env, "test_function");

        let trace_id = TransactionTracer::start_trace(&env, &contract_id, function.clone(), &caller);

        // Simulate some work
        env.ledger().with_mut(|li| {
            li.timestamp += 5; // 5 seconds
        });

        let child_calls = Vec::new(&env);
        let trace = TransactionTracer::complete_trace(
            &env,
            trace_id,
            &contract_id,
            function,
            &caller,
            true,
            None,
            child_calls,
            50000,
        );

        assert!(trace.success);
        assert_eq!(trace.gas_used, 50000);
        assert!(trace.execution_time_ms > 0);
    }

    #[test]
    fn test_analyze_flow_patterns() {
        let env = Env::default();
        let contract_id = Address::generate(&env);
        let caller = Address::generate(&env);

        let mut traces = Vec::new(&env);
        
        // Create some sample traces
        for i in 0..5 {
            let trace = TransactionTrace {
                trace_id: Symbol::new(&env, &format!("trace_{}", i)),
                contract_id: contract_id.clone(),
                function_name: Symbol::new(&env, "test_func"),
                caller: caller.clone(),
                timestamp: env.ledger().timestamp(),
                execution_time_ms: 100 + i,
                gas_used: 50000 + (i as u64 * 1000),
                success: i % 2 == 0, // Alternate success/failure
                error_message: None,
                child_calls: Vec::new(&env),
                events_emitted: 2,
            };
            traces.push_back(trace);
        }

        let patterns = TransactionTracer::analyze_flow_patterns(&env, &traces);
        
        assert!(patterns.contains_key(Symbol::new(&env, "func_calls")));
        assert!(patterns.contains_key(Symbol::new(&env, "successful")));
        assert!(patterns.contains_key(Symbol::new(&env, "failed")));
    }

    #[test]
    fn test_detect_unusual_patterns() {
        let env = Env::default();
        let contract_id = Address::generate(&env);
        let caller = Address::generate(&env);

        let mut traces = Vec::new(&env);
        
        // Add a trace with unusually high execution time
        let slow_trace = TransactionTrace {
            trace_id: Symbol::new(&env, "slow_trace"),
            contract_id: contract_id.clone(),
            function_name: Symbol::new(&env, "slow_func"),
            caller: caller.clone(),
            timestamp: env.ledger().timestamp(),
            execution_time_ms: 1000, // 1 second
            gas_used: 50000,
            success: true,
            error_message: None,
            child_calls: Vec::new(&env),
            events_emitted: 2,
        };
        traces.push_back(slow_trace);

        let anomalies = TransactionTracer::detect_unusual_patterns(
            &env,
            &traces,
            100,  // baseline: 100ms
            30000, // baseline: 30000 gas
        );

        assert!(!anomalies.is_empty());
    }
}