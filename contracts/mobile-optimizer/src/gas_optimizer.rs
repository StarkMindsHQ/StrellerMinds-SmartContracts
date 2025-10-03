use soroban_sdk::{Address, Env, String, Vec, Map};
use crate::types::*;

/// Gas optimization strategies for mobile devices
pub struct GasOptimizer;

impl GasOptimizer {
    /// Estimate gas for a single operation with mobile optimizations
    pub fn estimate_operation_gas(
        env: &Env,
        operation: &BatchOperation,
        network_quality: &NetworkQuality,
    ) -> Result<GasEstimate, GasError> {
        let base_gas = Self::calculate_base_gas(operation);
        let network_multiplier = Self::get_network_multiplier(network_quality);
        let complexity_factor = Self::analyze_operation_complexity(operation);
        
        let estimated_gas = ((base_gas as f64) * network_multiplier * complexity_factor) as u64;
        
        let confidence_level = Self::determine_confidence_level(operation, network_quality);
        let factors = Self::identify_gas_factors(operation);
        let suggestions = Self::generate_optimization_suggestions(env, operation)?;
        
        let estimated_cost = Self::calculate_cost_in_stroops(estimated_gas);
        let estimated_time = Self::estimate_execution_time(estimated_gas, network_quality);

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

    /// Calculate base gas consumption for operation type
    fn calculate_base_gas(operation: &BatchOperation) -> u64 {
        match operation.operation_type {
            OperationType::CourseEnrollment => 50000,      // Base gas for enrollment
            OperationType::ProgressUpdate => 30000,        // Base gas for progress update
            OperationType::CertificateRequest => 80000,    // Base gas for certificate request
            OperationType::CertificateRenewal => 60000,    // Base gas for renewal
            OperationType::SearchQuery => 20000,           // Base gas for search
            OperationType::PreferenceUpdate => 25000,      // Base gas for preferences
            OperationType::TokenTransfer => 40000,         // Base gas for token transfer
            OperationType::TokenStaking => 70000,          // Base gas for staking
            OperationType::TokenBurning => 45000,          // Base gas for burning
            OperationType::Custom(_) => 50000,             // Default for custom operations
        }
    }

    /// Get network quality multiplier for gas estimation
    fn get_network_multiplier(network_quality: &NetworkQuality) -> f64 {
        match network_quality {
            NetworkQuality::Excellent => 1.0,
            NetworkQuality::Good => 1.1,
            NetworkQuality::Fair => 1.3,
            NetworkQuality::Poor => 1.6,
            NetworkQuality::Offline => 2.0, // High estimate for when connection returns
        }
    }

    /// Analyze operation complexity for gas adjustment
    fn analyze_operation_complexity(operation: &BatchOperation) -> f64 {
        let mut complexity_factor = 1.0;
        
        // Parameter count affects complexity
        let param_count = operation.parameters.len() as f64;
        complexity_factor += param_count * 0.05;
        
        // Dependency count affects complexity
        let dependency_count = operation.dependencies.len() as f64;
        complexity_factor += dependency_count * 0.1;
        
        // Check for complex parameter types
        for param in &operation.parameters {
            match param.param_type {
                ParameterType::Vector => complexity_factor += 0.2,
                ParameterType::Map => complexity_factor += 0.3,
                _ => {}
            }
        }

        complexity_factor
    }

    /// Determine confidence level for gas estimate
    fn determine_confidence_level(
        operation: &BatchOperation,
        network_quality: &NetworkQuality,
    ) -> ConfidenceLevel {
        let mut confidence_score = 100;

        // Reduce confidence for complex operations
        if operation.parameters.len() > 5 {
            confidence_score -= 10;
        }
        
        if !operation.dependencies.is_empty() {
            confidence_score -= 15;
        }

        // Reduce confidence for poor network quality
        match network_quality {
            NetworkQuality::Excellent => {},
            NetworkQuality::Good => confidence_score -= 5,
            NetworkQuality::Fair => confidence_score -= 15,
            NetworkQuality::Poor => confidence_score -= 25,
            NetworkQuality::Offline => confidence_score -= 40,
        }

        // Custom operations have lower confidence
        if matches!(operation.operation_type, OperationType::Custom(_)) {
            confidence_score -= 20;
        }

        match confidence_score {
            95..=100 => ConfidenceLevel::High,
            80..=94 => ConfidenceLevel::Medium,
            60..=79 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::Unknown,
        }
    }

    /// Identify factors affecting gas consumption
    fn identify_gas_factors(operation: &BatchOperation) -> Vec<GasFactor> {
        let mut factors = Vec::new(&operation.operation_id.env());

        // Check for storage operations
        match operation.operation_type {
            OperationType::CourseEnrollment | 
            OperationType::ProgressUpdate |
            OperationType::PreferenceUpdate => {
                factors.push_back(GasFactor::StorageOperations);
            },
            _ => {}
        }

        // Check for computational complexity
        if operation.parameters.len() > 3 {
            factors.push_back(GasFactor::ComputationalLoad);
        }

        // Check for data size
        for param in &operation.parameters {
            match param.param_type {
                ParameterType::Vector | ParameterType::Map => {
                    factors.push_back(GasFactor::DataSize);
                    break;
                }
                _ => {}
            }
        }

        // Check for contract interactions
        if !operation.dependencies.is_empty() {
            factors.push_back(GasFactor::ContractInteractions);
        }

        // Always consider operation complexity
        factors.push_back(GasFactor::OperationComplexity);

        factors
    }

    /// Generate optimization suggestions for mobile users
    fn generate_optimization_suggestions(
        env: &Env,
        operation: &BatchOperation,
    ) -> Result<Vec<OptimizationSuggestion>, GasError> {
        let mut suggestions = Vec::new(env);

        // Suggest batching if operation can be batched
        if Self::can_be_batched(&operation.operation_type) {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::BatchOperations,
                description: String::from_str(env, "Combine with similar operations to reduce gas costs"),
                potential_savings: operation.estimated_gas / 4, // 25% savings
                implementation_effort: EffortLevel::Low,
                applicable: true,
            });
        }

        // Suggest parameter optimization
        if operation.parameters.len() > 5 {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::OptimizeParameters,
                description: String::from_str(env, "Reduce parameter count or use more efficient data types"),
                potential_savings: operation.estimated_gas / 10, // 10% savings
                implementation_effort: EffortLevel::Medium,
                applicable: true,
            });
        }

        // Suggest caching for repeated operations
        if Self::is_cacheable(&operation.operation_type) {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::UseCache,
                description: String::from_str(env, "Cache results to avoid repeated expensive operations"),
                potential_savings: operation.estimated_gas / 2, // 50% savings for cached operations
                implementation_effort: EffortLevel::Low,
                applicable: true,
            });
        }

        // Suggest delayed execution for non-critical operations
        if Self::can_be_delayed(&operation.operation_type) {
            suggestions.push_back(OptimizationSuggestion {
                suggestion_type: SuggestionType::DelayExecution,
                description: String::from_str(env, "Execute during low network congestion for lower costs"),
                potential_savings: operation.estimated_gas / 5, // 20% savings
                implementation_effort: EffortLevel::None,
                applicable: true,
            });
        }

        Ok(suggestions)
    }

    /// Check if operation type can be batched
    fn can_be_batched(operation_type: &OperationType) -> bool {
        match operation_type {
            OperationType::ProgressUpdate |
            OperationType::PreferenceUpdate |
            OperationType::TokenTransfer => true,
            _ => false,
        }
    }

    /// Check if operation results can be cached
    fn is_cacheable(operation_type: &OperationType) -> bool {
        match operation_type {
            OperationType::SearchQuery |
            OperationType::PreferenceUpdate => true,
            _ => false,
        }
    }

    /// Check if operation can be delayed
    fn can_be_delayed(operation_type: &OperationType) -> bool {
        match operation_type {
            OperationType::ProgressUpdate |
            OperationType::PreferenceUpdate |
            OperationType::TokenStaking => true,
            _ => false,
        }
    }

    /// Calculate cost in stroops
    fn calculate_cost_in_stroops(gas_amount: u64) -> i64 {
        // Simplified cost calculation (actual implementation would use current gas prices)
        (gas_amount as i64) * 100 // 100 stroops per gas unit
    }

    /// Estimate execution time based on gas and network quality
    fn estimate_execution_time(gas_amount: u64, network_quality: &NetworkQuality) -> u32 {
        let base_time_ms = (gas_amount / 1000) as u32; // Base time calculation
        
        let network_multiplier = match network_quality {
            NetworkQuality::Excellent => 1.0,
            NetworkQuality::Good => 1.2,
            NetworkQuality::Fair => 1.8,
            NetworkQuality::Poor => 3.0,
            NetworkQuality::Offline => 10.0,
        };

        ((base_time_ms as f64) * network_multiplier) as u32
    }

    /// Optimize batch gas usage
    pub fn optimize_batch_gas(
        env: &Env,
        operations: &Vec<BatchOperation>,
        network_quality: &NetworkQuality,
    ) -> Result<BatchGasOptimization, GasError> {
        let mut total_original_gas = 0u64;
        let mut total_optimized_gas = 0u64;
        let mut optimization_suggestions = Vec::new(env);

        for operation in operations {
            let estimate = Self::estimate_operation_gas(env, operation, network_quality)?;
            total_original_gas += estimate.estimated_gas;
            
            // Apply automatic optimizations
            let optimized_gas = Self::apply_automatic_optimizations(&estimate);
            total_optimized_gas += optimized_gas;
            
            optimization_suggestions.extend(estimate.optimization_suggestions);
        }

        // Batch-level optimizations
        let batch_savings = Self::calculate_batch_savings(operations);
        total_optimized_gas = total_optimized_gas.saturating_sub(batch_savings);

        Ok(BatchGasOptimization {
            original_gas_estimate: total_original_gas,
            optimized_gas_estimate: total_optimized_gas,
            potential_savings: total_original_gas.saturating_sub(total_optimized_gas),
            savings_percentage: if total_original_gas > 0 {
                ((total_original_gas.saturating_sub(total_optimized_gas)) * 100) / total_original_gas
            } else {
                0
            } as u32,
            optimization_suggestions,
            recommended_execution_strategy: Self::recommend_execution_strategy(operations, network_quality),
        })
    }

    /// Apply automatic gas optimizations
    fn apply_automatic_optimizations(estimate: &GasEstimate) -> u64 {
        let mut optimized_gas = estimate.estimated_gas;

        // Apply automatic optimizations based on suggestions
        for suggestion in &estimate.optimization_suggestions {
            if suggestion.implementation_effort == EffortLevel::None {
                optimized_gas = optimized_gas.saturating_sub(suggestion.potential_savings);
            }
        }

        optimized_gas
    }

    /// Calculate batch-level gas savings
    fn calculate_batch_savings(operations: &Vec<BatchOperation>) -> u64 {
        if operations.len() <= 1 {
            return 0;
        }

        // Calculate savings from batching similar operations
        let batch_overhead_reduction = (operations.len() as u64) * 5000; // 5000 gas saved per additional operation
        
        // Calculate savings from shared context
        let context_sharing_savings = (operations.len() as u64) * 2000; // 2000 gas saved per operation

        batch_overhead_reduction + context_sharing_savings
    }

    /// Recommend execution strategy based on operations and network
    fn recommend_execution_strategy(
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
            },
            NetworkQuality::Fair => ExecutionStrategy::Sequential,
            NetworkQuality::Poor | NetworkQuality::Offline => ExecutionStrategy::Conservative,
        }
    }

    /// Get gas optimization tips for mobile users
    pub fn get_mobile_gas_tips(env: &Env) -> Vec<String> {
        let mut tips = Vec::new(env);
        
        tips.push_back(String::from_str(env, "Batch similar operations together to reduce gas costs"));
        tips.push_back(String::from_str(env, "Execute non-urgent operations during off-peak hours"));
        tips.push_back(String::from_str(env, "Use WiFi connections for better network stability"));
        tips.push_back(String::from_str(env, "Cache frequently accessed data to avoid repeated queries"));
        tips.push_back(String::from_str(env, "Enable automatic retry with exponential backoff"));
        tips.push_back(String::from_str(env, "Consider using background execution for non-critical operations"));

        tips
    }

    /// Monitor gas usage patterns for optimization insights
    pub fn analyze_gas_patterns(
        env: &Env,
        user: &Address,
        period_days: u32,
    ) -> Result<GasUsageAnalysis, GasError> {
        // This would analyze historical gas usage patterns
        // For now, return a sample analysis
        Ok(GasUsageAnalysis {
            user: user.clone(),
            period_days,
            total_gas_used: 500000,
            average_gas_per_operation: 45000,
            most_expensive_operation_type: OperationType::CertificateRequest,
            optimization_opportunities: Self::get_mobile_gas_tips(env),
            potential_monthly_savings: 50000,
            efficiency_score: 75, // Out of 100
        })
    }
}

/// Batch gas optimization result
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchGasOptimization {
    pub original_gas_estimate: u64,
    pub optimized_gas_estimate: u64,
    pub potential_savings: u64,
    pub savings_percentage: u32,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub recommended_execution_strategy: ExecutionStrategy,
}

/// Gas usage analysis for optimization insights
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasUsageAnalysis {
    pub user: Address,
    pub period_days: u32,
    pub total_gas_used: u64,
    pub average_gas_per_operation: u64,
    pub most_expensive_operation_type: OperationType,
    pub optimization_opportunities: Vec<String>,
    pub potential_monthly_savings: u64,
    pub efficiency_score: u32, // 0-100
}

/// Gas optimization errors
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GasError {
    EstimationFailed,
    InvalidOperation,
    NetworkError,
    InsufficientData,
    OptimizationFailed,
}
