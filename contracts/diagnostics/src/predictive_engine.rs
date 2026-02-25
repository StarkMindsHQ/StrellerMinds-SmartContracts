use crate::{
    errors::DiagnosticsError, events::DiagnosticsEvents, storage::DiagnosticsStorage, types::*,
};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

/// Predictive analytics engine for system capacity planning
pub struct PredictiveEngine;

impl PredictiveEngine {
    /// Predict system capacity requirements
    pub fn predict_capacity(
        env: &Env,
        contract_address: &Address,
        prediction_horizon: u64,
    ) -> Result<CapacityPrediction, DiagnosticsError> {
        // Validate prediction horizon (must be between 1 hour and 1 year)
        if !(3600..=31_536_000).contains(&prediction_horizon) {
            return Err(DiagnosticsError::InvalidPredictionHorizon);
        }

        // Gather historical performance data
        let historical_data = Self::gather_historical_data(env, contract_address)?;

        if historical_data.len() < 10 {
            return Err(DiagnosticsError::InsufficientDataForPrediction);
        }

        // Analyze trends
        let _load_trend = Self::analyze_load_trend(&historical_data);
        let _resource_trend = Self::analyze_resource_trend(&historical_data);

        // Generate predictions
        let predicted_load = Self::predict_load(env, &historical_data, prediction_horizon);
        let bottleneck_predictions =
            Self::predict_bottlenecks(env, &historical_data, prediction_horizon);
        let cost_projections = Self::predict_costs(&historical_data, prediction_horizon);

        // Calculate confidence score
        let confidence_score = Self::calculate_prediction_confidence(&historical_data);

        // Generate capacity recommendations
        let capacity_recommendations =
            Self::generate_capacity_recommendations(env, &predicted_load, &bottleneck_predictions);

        let prediction_id = Self::generate_prediction_id(env);
        let generated_at = env.ledger().timestamp();

        let prediction = CapacityPrediction {
            prediction_id: prediction_id.clone(),
            contract_address: contract_address.clone(),
            prediction_horizon,
            predicted_load,
            capacity_recommendations,
            bottleneck_predictions,
            cost_projections,
            confidence_score,
            generated_at,
        };

        // Store prediction
        DiagnosticsStorage::store_capacity_prediction(env, contract_address, &prediction);

        // Emit event
        DiagnosticsEvents::emit_prediction_generated(
            env,
            contract_address,
            &prediction_id,
            confidence_score,
        );

        Ok(prediction)
    }

    /// Predict performance degradation
    pub fn predict_performance_degradation(
        env: &Env,
        contract_address: &Address,
    ) -> Result<Vec<DegradationPrediction>, DiagnosticsError> {
        let historical_data = Self::gather_historical_data(env, contract_address)?;

        if historical_data.len() < 5 {
            return Err(DiagnosticsError::InsufficientDataForPrediction);
        }

        let mut predictions = Vec::new(env);

        // Analyze execution time trends
        if let Some(execution_time_prediction) =
            Self::predict_execution_time_degradation(&historical_data)
        {
            predictions.push_back(execution_time_prediction);
        }

        // Analyze memory usage trends
        if let Some(memory_prediction) = Self::predict_memory_degradation(&historical_data) {
            predictions.push_back(memory_prediction);
        }

        // Analyze error rate trends
        if let Some(error_rate_prediction) = Self::predict_error_rate_increase(&historical_data) {
            predictions.push_back(error_rate_prediction);
        }

        Ok(predictions)
    }

    /// Predict optimal scaling points
    pub fn predict_scaling_requirements(
        env: &Env,
        contract_address: &Address,
        target_load_increase: u32, // percentage increase
    ) -> Result<ScalingPrediction, DiagnosticsError> {
        let current_metrics =
            DiagnosticsStorage::get_latest_performance_metrics(env, contract_address)
                .ok_or(DiagnosticsError::MetricsNotFound)?;

        // Calculate required resources for target load
        let predicted_gas_usage =
            (current_metrics.gas_used * (100 + target_load_increase as u64)) / 100;
        let predicted_memory_usage =
            (current_metrics.memory_usage * (100 + target_load_increase)) / 100;
        let predicted_execution_time =
            (current_metrics.execution_time * (100 + target_load_increase as u64)) / 100;

        // Identify scaling bottlenecks
        let scaling_bottlenecks =
            Self::identify_scaling_bottlenecks(env, &current_metrics, target_load_increase);

        // Generate scaling recommendations
        let scaling_recommendations =
            Self::generate_scaling_recommendations(env, &scaling_bottlenecks);

        Ok(ScalingPrediction {
            contract_address: contract_address.clone(),
            target_load_increase,
            predicted_gas_usage,
            predicted_memory_usage,
            predicted_execution_time,
            scaling_bottlenecks,
            scaling_recommendations,
            estimated_cost_increase: Self::calculate_cost_increase(
                current_metrics.gas_used,
                predicted_gas_usage,
            ),
        })
    }

    /// Gather historical performance data
    fn gather_historical_data(
        env: &Env,
        contract_address: &Address,
    ) -> Result<Vec<PerformanceMetrics>, DiagnosticsError> {
        let mut data = Vec::new(env);
        let current_time = env.ledger().timestamp();

        // Gather last 24 hours of data (sample every hour)
        for i in 0..24 {
            let timestamp = current_time - (i * 3600); // 1 hour intervals
            if let Some(metrics) =
                DiagnosticsStorage::get_performance_metrics(env, contract_address, timestamp)
            {
                data.push_back(metrics);
            }
        }

        if data.is_empty() {
            return Err(DiagnosticsError::InsufficientDataForPrediction);
        }

        Ok(data)
    }

    /// Analyze load trends from historical data
    fn analyze_load_trend(data: &Vec<PerformanceMetrics>) -> LoadTrend {
        if data.len() < 3 {
            return LoadTrend::Stable;
        }

        let first_third = data.len() / 3;
        let second_third = (data.len() * 2) / 3;

        let mut early_vec = Vec::new(data.env());
        let mut middle_vec = Vec::new(data.env());
        let mut recent_vec = Vec::new(data.env());

        for i in 0..first_third {
            early_vec.push_back(data.get(i).unwrap());
        }
        for i in first_third..second_third {
            middle_vec.push_back(data.get(i).unwrap());
        }
        for i in second_third..data.len() {
            recent_vec.push_back(data.get(i).unwrap());
        }

        let early_avg = Self::calculate_average_load(&early_vec);
        let middle_avg = Self::calculate_average_load(&middle_vec);
        let recent_avg = Self::calculate_average_load(&recent_vec);

        if recent_avg > middle_avg && middle_avg > early_avg {
            LoadTrend::Increasing
        } else if recent_avg < middle_avg && middle_avg < early_avg {
            LoadTrend::Decreasing
        } else if (recent_avg as i64 - early_avg as i64).abs() > (early_avg / 10) as i64 {
            LoadTrend::Volatile
        } else {
            LoadTrend::Stable
        }
    }

    /// Analyze resource utilization trends
    fn analyze_resource_trend(data: &Vec<PerformanceMetrics>) -> ResourceTrend {
        if data.len() < 2 {
            return ResourceTrend::Stable;
        }

        let first_half = data.len() / 2;
        let early_memory = Self::calculate_average_memory(&data.slice(0..first_half));
        let recent_memory = Self::calculate_average_memory(&data.slice(first_half..data.len()));

        if recent_memory > early_memory * 11 / 10 {
            // 10% increase
            ResourceTrend::Increasing
        } else if recent_memory < early_memory * 9 / 10 {
            // 10% decrease
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        }
    }

    /// Predict future load based on historical trends
    fn predict_load(env: &Env, data: &Vec<PerformanceMetrics>, horizon: u64) -> LoadPrediction {
        let current_avg_transactions = Self::calculate_average_transactions_per_hour(data);
        let current_avg_gas = Self::calculate_average_gas_usage(data);
        let current_avg_storage = Self::calculate_average_storage_usage(data);

        // Simple linear projection (in production, would use more sophisticated ML models)
        let growth_rate = Self::calculate_growth_rate(data);
        let time_multiplier = (horizon / 3600) as u32; // Convert to hours

        LoadPrediction {
            predicted_tx_per_hour: (current_avg_transactions as f64
                * (1.0 + growth_rate * time_multiplier as f64))
                as u32,
            predicted_tx_hourly: (current_avg_transactions as f64
                * (1.0 + growth_rate * time_multiplier as f64))
                as u32,
            predicted_gas_usage: (current_avg_gas as f64
                * (1.0 + growth_rate * time_multiplier as f64))
                as u64,
            predicted_storage_growth: (current_avg_storage as f64
                * (1.0 + growth_rate * time_multiplier as f64))
                as u32,
            peak_load_times: Self::identify_peak_load_times(env, data),
            resource_saturation_risk: Self::assess_saturation_risk(growth_rate, time_multiplier),
        }
    }

    /// Predict potential bottlenecks
    fn predict_bottlenecks(
        env: &Env,
        data: &Vec<PerformanceMetrics>,
        horizon: u64,
    ) -> Vec<BottleneckPrediction> {
        let mut bottlenecks = Vec::new(env);

        // CPU bottleneck prediction
        let cpu_trend = Self::analyze_cpu_trend(data);
        if cpu_trend > 5.0 {
            // 5% increase per hour
            let mut actions = Vec::new(env);
            actions.push_back(String::from_str(env, "Optimize CPU-intensive operations"));
            actions.push_back(String::from_str(env, "Consider load balancing"));

            bottlenecks.push_back(BottleneckPrediction {
                bottleneck_type: BottleneckType::CPU,
                severity: if cpu_trend > 10.0 {
                    RiskLevel::High
                } else {
                    RiskLevel::Medium
                },
                estimated_impact: String::from_str(
                    env,
                    "High CPU usage may slow down transaction processing",
                ),
                recommended_actions: actions,
                estimated_occurrence_time: horizon / 2, // Estimate midpoint
            });
        }

        // Memory bottleneck prediction
        let memory_trend = Self::analyze_memory_trend(data);
        if memory_trend > 3.0 {
            // 3% increase per hour
            let mut actions = Vec::new(env);
            actions.push_back(String::from_str(env, "Optimize memory usage patterns"));
            actions.push_back(String::from_str(env, "Implement garbage collection"));

            bottlenecks.push_back(BottleneckPrediction {
                bottleneck_type: BottleneckType::Memory,
                severity: if memory_trend > 7.0 {
                    RiskLevel::Critical
                } else {
                    RiskLevel::Medium
                },
                estimated_impact: String::from_str(
                    env,
                    "Memory exhaustion may cause contract failures",
                ),
                recommended_actions: actions,
                estimated_occurrence_time: horizon / 3,
            });
        }

        // Gas bottleneck prediction
        let gas_trend = Self::analyze_gas_trend(data);
        if gas_trend > 8.0 {
            // 8% increase per hour
            let mut actions = Vec::new(env);
            actions.push_back(String::from_str(env, "Optimize gas usage in functions"));
            actions.push_back(String::from_str(env, "Implement gas-efficient algorithms"));

            bottlenecks.push_back(BottleneckPrediction {
                bottleneck_type: BottleneckType::Gas,
                severity: if gas_trend > 15.0 {
                    RiskLevel::High
                } else {
                    RiskLevel::Medium
                },
                estimated_impact: String::from_str(env, "High gas costs may deter users"),
                recommended_actions: actions,
                estimated_occurrence_time: horizon / 4,
            });
        }

        bottlenecks
    }

    /// Predict future costs
    fn predict_costs(data: &Vec<PerformanceMetrics>, horizon: u64) -> CostProjection {
        let current_daily_gas = Self::calculate_daily_gas_usage(data);
        let gas_growth_rate = Self::calculate_gas_growth_rate(data);

        let days_in_horizon = horizon / 86400; // Convert to days
        let projected_daily_gas =
            (current_daily_gas as f64 * (1.0 + gas_growth_rate * days_in_horizon as f64)) as u64;

        // Assume 1 gas = 0.0001 cost units (configurable in production)
        let current_daily_cost = current_daily_gas / 10000;
        let projected_daily_cost = projected_daily_gas / 10000;

        CostProjection {
            current_daily_cost,
            projected_daily_cost,
            cost_optimization_potential: (projected_daily_cost - current_daily_cost) / 4, // 25% optimization potential
            cost_trend: if projected_daily_cost > current_daily_cost * 12 / 10 {
                CostTrend::Increasing
            } else if projected_daily_cost < current_daily_cost * 9 / 10 {
                CostTrend::Decreasing
            } else {
                CostTrend::Stable
            },
        }
    }

    /// Calculate prediction confidence based on data quality
    fn calculate_prediction_confidence(data: &Vec<PerformanceMetrics>) -> u32 {
        let mut confidence = 50u32; // Base confidence

        // More data points increase confidence
        confidence += data.len().min(30);

        // Consistent data increases confidence
        let variance = Self::calculate_variance(data);
        if variance < 0.1 {
            confidence += 20;
        } else if variance < 0.3 {
            confidence += 10;
        }

        confidence.min(100)
    }

    /// Generate capacity recommendations
    fn generate_capacity_recommendations(
        env: &Env,
        predicted_load: &LoadPrediction,
        bottlenecks: &Vec<BottleneckPrediction>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        // Load-based recommendations
        match predicted_load.resource_saturation_risk {
            RiskLevel::High | RiskLevel::Critical => {
                recommendations.push_back(String::from_str(
                    env,
                    "Consider horizontal scaling to handle increased load",
                ));
                recommendations
                    .push_back(String::from_str(env, "Implement load balancing strategies"));
            }
            RiskLevel::Medium => {
                recommendations.push_back(String::from_str(env, "Monitor resource usage closely"));
            }
            _ => {}
        }

        // Bottleneck-specific recommendations
        for bottleneck in bottlenecks.iter() {
            match bottleneck.bottleneck_type {
                BottleneckType::Memory => {
                    recommendations
                        .push_back(String::from_str(env, "Optimize memory allocation patterns"));
                }
                BottleneckType::CPU => {
                    recommendations
                        .push_back(String::from_str(env, "Optimize computational algorithms"));
                }
                BottleneckType::Gas => {
                    recommendations.push_back(String::from_str(
                        env,
                        "Implement gas optimization techniques",
                    ));
                }
                _ => {}
            }
        }

        recommendations
    }

    /// Generate unique prediction ID
    fn generate_prediction_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let mut data = [0u8; 32];
        let ts_bytes = timestamp.to_be_bytes();
        let seq_bytes = sequence.to_be_bytes();
        for i in 0..8 {
            data[i] = ts_bytes[i];
            data[i + 8] = seq_bytes[i];
        }
        BytesN::from_array(env, &data)
    }

    // Helper methods for calculations
    fn calculate_average_load(data: &Vec<PerformanceMetrics>) -> u32 {
        if data.is_empty() {
            return 0;
        }
        let mut sum = 0u32;
        for i in 0..data.len() {
            sum += data.get(i).unwrap().transaction_count;
        }
        sum / data.len()
    }

    fn calculate_average_memory(data: &Vec<PerformanceMetrics>) -> u32 {
        if data.is_empty() {
            return 0;
        }
        let mut sum = 0u32;
        for i in 0..data.len() {
            sum += data.get(i).unwrap().memory_usage;
        }
        sum / data.len()
    }

    fn calculate_average_transactions_per_hour(data: &Vec<PerformanceMetrics>) -> u32 {
        if data.is_empty() {
            return 0;
        }
        data.iter().map(|m| m.transaction_count).sum::<u32>() / data.len()
    }

    fn calculate_average_gas_usage(data: &Vec<PerformanceMetrics>) -> u64 {
        if data.is_empty() {
            return 0;
        }
        data.iter().map(|m| m.gas_used).sum::<u64>() / data.len() as u64
    }

    fn calculate_average_storage_usage(data: &Vec<PerformanceMetrics>) -> u32 {
        if data.is_empty() {
            return 0;
        }
        data.iter()
            .map(|m| m.storage_reads + m.storage_writes)
            .sum::<u32>()
            / data.len()
    }

    fn calculate_growth_rate(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first_half_avg = Self::calculate_average_load(&data.slice(0..data.len() / 2));
        let second_half_avg = Self::calculate_average_load(&data.slice(data.len() / 2..data.len()));
        if first_half_avg == 0 {
            return 0.0;
        }
        (second_half_avg as f64 - first_half_avg as f64) / first_half_avg as f64
    }

    fn identify_peak_load_times(env: &Env, data: &Vec<PerformanceMetrics>) -> Vec<u64> {
        let mut peaks = Vec::new(env);
        let avg_load = Self::calculate_average_load(data);

        for metrics in data.iter() {
            if metrics.transaction_count > avg_load * 15 / 10 {
                // 50% above average
                peaks.push_back(metrics.timestamp);
            }
        }
        peaks
    }

    fn assess_saturation_risk(growth_rate: f64, time_multiplier: u32) -> RiskLevel {
        let projected_growth = growth_rate * time_multiplier as f64;
        if projected_growth > 2.0 {
            // 200% growth
            RiskLevel::Critical
        } else if projected_growth > 1.0 {
            // 100% growth
            RiskLevel::High
        } else if projected_growth > 0.5 {
            // 50% growth
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn analyze_cpu_trend(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data.first().unwrap().cpu_utilization as f64;
        let last = data.last().unwrap().cpu_utilization as f64;
        if first == 0.0 {
            return 0.0;
        }
        (last - first) / first * 100.0 / data.len() as f64
    }

    fn analyze_memory_trend(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data.first().unwrap().memory_usage as f64;
        let last = data.last().unwrap().memory_usage as f64;
        if first == 0.0 {
            return 0.0;
        }
        (last - first) / first * 100.0 / data.len() as f64
    }

    fn analyze_gas_trend(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data.first().unwrap().gas_used as f64;
        let last = data.last().unwrap().gas_used as f64;
        if first == 0.0 {
            return 0.0;
        }
        (last - first) / first * 100.0 / data.len() as f64
    }

    fn calculate_daily_gas_usage(data: &Vec<PerformanceMetrics>) -> u64 {
        if data.is_empty() {
            return 0;
        }
        let avg_hourly = data.iter().map(|m| m.gas_used).sum::<u64>() / data.len() as u64;
        avg_hourly * 24
    }

    fn calculate_gas_growth_rate(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mid = data.len() / 2;
        let mut first_half = Vec::new(data.env());
        let mut second_half = Vec::new(data.env());

        for i in 0..mid {
            first_half.push_back(data.get(i).unwrap());
        }
        for i in mid..data.len() {
            second_half.push_back(data.get(i).unwrap());
        }

        let first_half_avg = Self::calculate_average_gas_usage(&first_half);
        let second_half_avg = Self::calculate_average_gas_usage(&second_half);
        if first_half_avg == 0 {
            return 0.0;
        }
        (second_half_avg as f64 - first_half_avg as f64) / first_half_avg as f64
    }

    fn calculate_variance(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let mean = Self::calculate_average_load(data) as f64;
        let variance = data
            .iter()
            .map(|m| (m.transaction_count as f64 - mean).powi(2))
            .sum::<f64>()
            / data.len() as f64;
        variance.sqrt() / mean
    }

    // Additional prediction functions
    fn predict_execution_time_degradation(
        data: &Vec<PerformanceMetrics>,
    ) -> Option<DegradationPrediction> {
        let trend = Self::analyze_execution_time_trend(data);
        if trend > 5.0 {
            // 5% increase trend
            Some(DegradationPrediction {
                degradation_type: DegradationType::ExecutionTime,
                current_value: data.last().unwrap().average_execution_time,
                predicted_value: (data.last().unwrap().average_execution_time as f64
                    * (1.0 + trend / 100.0)) as u64,
                confidence: 75,
                time_to_degradation: 3600, // 1 hour estimate
            })
        } else {
            None
        }
    }

    fn predict_memory_degradation(data: &Vec<PerformanceMetrics>) -> Option<DegradationPrediction> {
        let trend = Self::analyze_memory_trend(data);
        if trend > 3.0 {
            // 3% increase trend
            Some(DegradationPrediction {
                degradation_type: DegradationType::Memory,
                current_value: data.last().unwrap().memory_usage as u64,
                predicted_value: (data.last().unwrap().memory_usage as f64 * (1.0 + trend / 100.0))
                    as u64,
                confidence: 80,
                time_to_degradation: 7200, // 2 hours estimate
            })
        } else {
            None
        }
    }

    fn predict_error_rate_increase(
        data: &Vec<PerformanceMetrics>,
    ) -> Option<DegradationPrediction> {
        let trend = Self::analyze_error_rate_trend(data);
        if trend > 1.0 {
            // 1% increase trend
            Some(DegradationPrediction {
                degradation_type: DegradationType::ErrorRate,
                current_value: data.last().unwrap().error_rate as u64,
                predicted_value: (data.last().unwrap().error_rate as f64 * (1.0 + trend / 100.0))
                    as u64,
                confidence: 70,
                time_to_degradation: 1800, // 30 minutes estimate
            })
        } else {
            None
        }
    }

    fn analyze_execution_time_trend(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data.first().unwrap().average_execution_time as f64;
        let last = data.last().unwrap().average_execution_time as f64;
        if first == 0.0 {
            return 0.0;
        }
        (last - first) / first * 100.0 / data.len() as f64
    }

    fn analyze_error_rate_trend(data: &Vec<PerformanceMetrics>) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data.first().unwrap().error_rate as f64;
        let last = data.last().unwrap().error_rate as f64;
        (last - first) / data.len() as f64
    }

    fn identify_scaling_bottlenecks(
        env: &Env,
        current_metrics: &PerformanceMetrics,
        target_increase: u32,
    ) -> Vec<ScalingBottleneck> {
        let mut bottlenecks = Vec::new(env);

        // CPU scaling bottleneck
        if current_metrics.cpu_utilization > 70 {
            bottlenecks.push_back(ScalingBottleneck {
                resource_type: BottleneckType::CPU,
                current_utilization: current_metrics.cpu_utilization,
                projected_utilization: current_metrics.cpu_utilization + target_increase,
                severity: if current_metrics.cpu_utilization > 85 {
                    RiskLevel::Critical
                } else {
                    RiskLevel::High
                },
            });
        }

        // Memory scaling bottleneck
        if current_metrics.memory_usage > 800_000_000 {
            // 800MB threshold
            bottlenecks.push_back(ScalingBottleneck {
                resource_type: BottleneckType::Memory,
                current_utilization: (current_metrics.memory_usage / 10_000_000), // Convert to percentage-like value
                projected_utilization: (current_metrics.memory_usage * (100 + target_increase)
                    / 100)
                    / 10_000_000,
                severity: RiskLevel::High,
            });
        }

        bottlenecks
    }

    fn generate_scaling_recommendations(
        env: &Env,
        bottlenecks: &Vec<ScalingBottleneck>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        for bottleneck in bottlenecks.iter() {
            match bottleneck.resource_type {
                BottleneckType::CPU => {
                    recommendations.push_back(String::from_str(
                        env,
                        "Consider implementing CPU load balancing",
                    ));
                    recommendations
                        .push_back(String::from_str(env, "Optimize CPU-intensive algorithms"));
                }
                BottleneckType::Memory => {
                    recommendations
                        .push_back(String::from_str(env, "Implement memory pooling strategies"));
                    recommendations.push_back(String::from_str(
                        env,
                        "Optimize data structures for memory efficiency",
                    ));
                }
                BottleneckType::Storage => {
                    recommendations.push_back(String::from_str(env, "Implement storage tiering"));
                }
                _ => {}
            }
        }

        recommendations
    }

    fn calculate_cost_increase(current_gas: u64, predicted_gas: u64) -> u32 {
        if current_gas == 0 {
            return 0;
        }
        ((predicted_gas - current_gas) * 100 / current_gas) as u32
    }
}

// Additional types for predictive analytics
use soroban_sdk::contracttype;

#[derive(Clone, Debug)]
#[contracttype]
pub enum LoadTrend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Clone, Debug)]
#[contracttype]
pub enum ResourceTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct DegradationPrediction {
    pub degradation_type: DegradationType,
    pub current_value: u64,
    pub predicted_value: u64,
    pub confidence: u32,
    pub time_to_degradation: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub enum DegradationType {
    ExecutionTime,
    Memory,
    ErrorRate,
    Throughput,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ScalingPrediction {
    pub contract_address: Address,
    pub target_load_increase: u32,
    pub predicted_gas_usage: u64,
    pub predicted_memory_usage: u32,
    pub predicted_execution_time: u64,
    pub scaling_bottlenecks: Vec<ScalingBottleneck>,
    pub scaling_recommendations: Vec<String>,
    pub estimated_cost_increase: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ScalingBottleneck {
    pub resource_type: BottleneckType,
    pub current_utilization: u32,
    pub projected_utilization: u32,
    pub severity: RiskLevel,
}
