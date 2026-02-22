use soroban_sdk::{contracttype, Env, String, Vec};

use crate::types::*;

pub struct GasOptimizer;

impl GasOptimizer {
    pub fn estimate_operation_gas(
        env: &Env,
        operation: &BatchOperation,
        network_quality: &NetworkQuality,
    ) -> Result<GasEstimate, MobileOptimizerError> {
        let base_gas = Self::calculate_base_gas(operation);
        let network_mult_bps = Self::get_network_multiplier_bps(network_quality);
        let complexity_bps = Self::analyze_operation_complexity_bps(operation);

        // (base_gas * network_mult_bps * complexity_bps) / (10000 * 10000)
        let estimated_gas = (base_gas as u128)
            .checked_mul(network_mult_bps as u128)
            .unwrap_or(base_gas as u128)
            .checked_mul(complexity_bps as u128)
            .unwrap_or(base_gas as u128)
            / (10000u128 * 10000u128);
        let estimated_gas = estimated_gas as u64;

        let confidence_level =
            Self::determine_confidence_level(operation, network_quality);
        let factors = Self::identify_gas_factors(env, operation);
        let suggestions =
            Self::generate_optimization_suggestions(env, operation);

        let estimated_cost = Self::calculate_cost_in_stroops(estimated_gas);
        let estimated_time =
            Self::estimate_execution_time(estimated_gas, network_quality);

        Ok(GasEstimate {
            operation_id: operation.operation_id.clone(),
            estimated_gas,
            confidence_level,
            factors,
            optimization_suggestions: suggestions,
            estimated_cost_stroops: estimated_cost,
            estimated_time_ms: estimated_time,
        })
    }

    pub fn optimize_batch_gas(
        env: &Env,
        operations: &Vec<BatchOperation>,
        network_quality: &NetworkQuality,
    ) -> Result<BatchGasOptimization, MobileOptimizerError> {
        let mut total_original = 0u64;
        let mut total_optimized = 0u64;
        let mut all_suggestions = Vec::new(env);

        for operation in operations.iter() {
            let estimate =
                Self::estimate_operation_gas(env, &operation, network_quality)?;
            total_original += estimate.estimated_gas;
            let opt_gas = Self::apply_automatic_optimizations(&estimate);
            total_optimized += opt_gas;

            for s in estimate.optimization_suggestions.iter() {
                all_suggestions.push_back(s.clone());
            }
        }

        let batch_savings = Self::calculate_batch_savings(operations);
        total_optimized = total_optimized.saturating_sub(batch_savings);

        let savings_pct = if total_original > 0 {
            ((total_original.saturating_sub(total_optimized)) * 100
                / total_original) as u32
        } else {
            0
        };

        Ok(BatchGasOptimization {
            original_gas_estimate: total_original,
            optimized_gas_estimate: total_optimized,
            potential_savings: total_original.saturating_sub(total_optimized),
            savings_percentage: savings_pct,
            optimization_suggestions: all_suggestions,
            recommended_execution_strategy: Self::recommend_strategy(
                operations,
                network_quality,
            ),
        })
    }

    pub fn get_mobile_gas_tips(env: &Env) -> Vec<String> {
        let mut tips = Vec::new(env);
        tips.push_back(String::from_str(
            env,
            "Batch similar operations together to reduce gas costs",
        ));
        tips.push_back(String::from_str(
            env,
            "Execute non-urgent operations during off-peak hours",
        ));
        tips.push_back(String::from_str(
            env,
            "Use WiFi connections for better network stability",
        ));
        tips.push_back(String::from_str(
            env,
            "Cache frequently accessed data to avoid repeated queries",
        ));
        tips.push_back(String::from_str(
            env,
            "Enable automatic retry with exponential backoff",
        ));
        tips
    }

    fn calculate_base_gas(operation: &BatchOperation) -> u64 {
        match operation.operation_type {
            OperationType::CourseEnrollment => 50000,
            OperationType::ProgressUpdate => 30000,
            OperationType::CertificateRequest => 80000,
            OperationType::CertificateRenewal => 60000,
            OperationType::CertificateGeneration => 80000,
            OperationType::SearchQuery => 20000,
            OperationType::PreferenceUpdate => 25000,
            OperationType::TokenTransfer => 40000,
            OperationType::TokenStaking => 70000,
            OperationType::TokenBurning => 45000,
            OperationType::TokenReward => 40000,
            OperationType::ContentCache => 15000,
            OperationType::LearningSync => 35000,
            OperationType::NotificationConfig => 10000,
            OperationType::SecurityUpdate => 20000,
            OperationType::AnalyticsEvent => 5000,
            OperationType::Custom => 50000,
        }
    }

    /// Returns basis points (10000 = 1.0x)
    fn get_network_multiplier_bps(network_quality: &NetworkQuality) -> u64 {
        match network_quality {
            NetworkQuality::Excellent => 10000,
            NetworkQuality::Good => 11000,
            NetworkQuality::Fair => 13000,
            NetworkQuality::Poor => 16000,
            NetworkQuality::Offline => 20000,
        }
    }

    /// Returns basis points (10000 = 1.0x)
    fn analyze_operation_complexity_bps(operation: &BatchOperation) -> u64 {
        let mut factor_bps = 10000u64;
        let param_count = operation.parameters.len() as u64;
        factor_bps += param_count * 500; // 0.05 per param
        let dep_count = operation.dependencies.len() as u64;
        factor_bps += dep_count * 1000; // 0.1 per dep

        for param in operation.parameters.iter() {
            match param.param_type {
                ParameterType::Vector => factor_bps += 2000,
                ParameterType::Map => factor_bps += 3000,
                _ => {}
            }
        }
        factor_bps
    }

    fn determine_confidence_level(
        operation: &BatchOperation,
        network_quality: &NetworkQuality,
    ) -> ConfidenceLevel {
        let mut score = 100i32;

        if operation.parameters.len() > 5 {
            score -= 10;
        }
        if !operation.dependencies.is_empty() {
            score -= 15;
        }
        match network_quality {
            NetworkQuality::Excellent => {}
            NetworkQuality::Good => score -= 5,
            NetworkQuality::Fair => score -= 15,
            NetworkQuality::Poor => score -= 25,
            NetworkQuality::Offline => score -= 40,
        }
        if operation.operation_type == OperationType::Custom {
            score -= 20;
        }

        if score >= 95 {
            ConfidenceLevel::High
        } else if score >= 80 {
            ConfidenceLevel::Medium
        } else if score >= 60 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::Unknown
        }
    }

    fn identify_gas_factors(
        env: &Env,
        operation: &BatchOperation,
    ) -> Vec<GasFactor> {
        let mut factors = Vec::new(env);

        match operation.operation_type {
            OperationType::CourseEnrollment
            | OperationType::ProgressUpdate
            | OperationType::PreferenceUpdate => {
                factors.push_back(GasFactor::StorageOperations);
            }
            _ => {}
        }
        if operation.parameters.len() > 3 {
            factors.push_back(GasFactor::ComputationalLoad);
        }
        if !operation.dependencies.is_empty() {
            factors.push_back(GasFactor::ContractInteractions);
        }
        factors.push_back(GasFactor::OperationComplexity);
        factors
    }

    fn generate_optimization_suggestions(
        env: &Env,
        operation: &BatchOperation,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new(env);

        if Self::can_be_batched(&operation.operation_type) {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::BatchOperations,
                description: String::from_str(
                    env,
                    "Combine with similar operations to reduce gas costs",
                ),
                potential_savings: operation.estimated_gas / 4,
                implementation_effort: EffortLevel::Low,
                applicable: true,
            });
        }

        if operation.parameters.len() > 5 {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::OptimizeParameters,
                description: String::from_str(
                    env,
                    "Reduce parameter count for lower gas usage",
                ),
                potential_savings: operation.estimated_gas / 10,
                implementation_effort: EffortLevel::Medium,
                applicable: true,
            });
        }

        if Self::is_cacheable(&operation.operation_type) {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::UseCache,
                description: String::from_str(
                    env,
                    "Cache results to avoid repeated operations",
                ),
                potential_savings: operation.estimated_gas / 2,
                implementation_effort: EffortLevel::Low,
                applicable: true,
            });
        }

        suggestions
    }

    fn can_be_batched(op_type: &OperationType) -> bool {
        matches!(
            op_type,
            OperationType::ProgressUpdate
                | OperationType::PreferenceUpdate
                | OperationType::TokenTransfer
                | OperationType::AnalyticsEvent
        )
    }

    fn is_cacheable(op_type: &OperationType) -> bool {
        matches!(
            op_type,
            OperationType::SearchQuery | OperationType::PreferenceUpdate
        )
    }

    fn calculate_cost_in_stroops(gas_amount: u64) -> i64 {
        (gas_amount as i64) * 100
    }

    fn estimate_execution_time(gas_amount: u64, network_quality: &NetworkQuality) -> u32 {
        let base_ms = (gas_amount / 1000) as u32;
        let mult_bps = match network_quality {
            NetworkQuality::Excellent => 10000u32,
            NetworkQuality::Good => 12000,
            NetworkQuality::Fair => 18000,
            NetworkQuality::Poor => 30000,
            NetworkQuality::Offline => 100000,
        };
        (base_ms as u64 * mult_bps as u64 / 10000) as u32
    }

    fn apply_automatic_optimizations(estimate: &GasEstimate) -> u64 {
        let mut gas = estimate.estimated_gas;
        for suggestion in estimate.optimization_suggestions.iter() {
            if suggestion.implementation_effort == EffortLevel::None {
                gas = gas.saturating_sub(suggestion.potential_savings);
            }
        }
        gas
    }

    fn calculate_batch_savings(operations: &Vec<BatchOperation>) -> u64 {
        if operations.len() <= 1 {
            return 0;
        }
        let count = operations.len() as u64;
        count * 5000 + count * 2000
    }

    fn recommend_strategy(
        operations: &Vec<BatchOperation>,
        network_quality: &NetworkQuality,
    ) -> ExecutionStrategy {
        match network_quality {
            NetworkQuality::Excellent | NetworkQuality::Good => {
                if operations.len() > 5 {
                    ExecutionStrategy::Parallel
                } else {
                    ExecutionStrategy::Optimized
                }
            }
            NetworkQuality::Fair => ExecutionStrategy::Sequential,
            NetworkQuality::Poor | NetworkQuality::Offline => {
                ExecutionStrategy::Conservative
            }
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchGasOptimization {
    pub original_gas_estimate: u64,
    pub optimized_gas_estimate: u64,
    pub potential_savings: u64,
    pub savings_percentage: u32,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub recommended_execution_strategy: ExecutionStrategy,
}
