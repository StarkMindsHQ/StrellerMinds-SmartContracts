//! Gas profiling utilities for StrellerMinds contracts
//! 
//! This module provides tools to measure and analyze gas consumption
//! across different contract functions and operations.

#![no_std]

use soroban_sdk::{env::Env, Address, BytesN, Symbol, Vec, Map};

/// Gas measurement data for a single function call
#[derive(Clone, Debug)]
#[contracttype]
pub struct GasMeasurement {
    pub function_name: Symbol,
    pub gas_consumed: u64,
    pub storage_reads: u32,
    pub storage_writes: u32,
    pub events_emitted: u32,
    pub timestamp: u64,
}

/// Aggregated gas profile for a contract
#[derive(Clone, Debug)]
#[contracttype]
pub struct GasProfile {
    pub contract_name: Symbol,
    pub total_measurements: u32,
    pub average_gas_per_call: u64,
    pub peak_gas_consumption: u64,
    pub measurements: Vec<GasMeasurement>,
}

/// Gas profiler for tracking function execution costs
pub struct GasProfiler;

impl GasProfiler {
    /// Start measuring gas consumption for a function
    pub fn start_measurement(env: &Env, function_name: Symbol) -> u64 {
        let start_gas = env.contract().get_current_gas();
        env.storage().instance().set(&Symbol::short("prof_start"), &start_gas);
        env.storage().instance().set(&Symbol::short("prof_func"), &function_name);
        start_gas
    }

    /// End measurement and record the results
    pub fn end_measurement(env: &Env, storage_reads: u32, storage_writes: u32, events: u32) -> GasMeasurement {
        let start_gas: u64 = env.storage().instance()
            .get(&Symbol::short("prof_start"))
            .unwrap_or(0);
        let function_name: Symbol = env.storage().instance()
            .get(&Symbol::short("prof_func"))
            .unwrap_or(Symbol::short("unknown"));
        
        let end_gas = env.contract().get_current_gas();
        let gas_consumed = end_gas.saturating_sub(start_gas);
        
        let measurement = GasMeasurement {
            function_name,
            gas_consumed,
            storage_reads,
            storage_writes,
            events_emitted: events,
            timestamp: env.ledger().timestamp(),
        };

        // Clean up temporary storage
        env.storage().instance().remove(&Symbol::short("prof_start"));
        env.storage().instance().remove(&Symbol::short("prof_func"));

        measurement
    }

    /// Record a gas measurement in the contract's profile
    pub fn record_measurement(env: &Env, measurement: GasMeasurement) {
        let profile_key = Symbol::short("gas_profile");
        let mut profile: GasProfile = env.storage().instance()
            .get(&profile_key)
            .unwrap_or(GasProfile {
                contract_name: Symbol::short("contract"),
                total_measurements: 0,
                average_gas_per_call: 0,
                peak_gas_consumption: 0,
                measurements: Vec::new(env),
            });

        // Update statistics
        profile.total_measurements += 1;
        profile.measurements.push_back(measurement.clone());
        
        // Update peak consumption
        if measurement.gas_consumed > profile.peak_gas_consumption {
            profile.peak_gas_consumption = measurement.gas_consumed;
        }

        // Recalculate average
        let total_gas: u64 = profile.measurements.iter()
            .fold(0, |acc, m| acc + m.gas_consumed);
        profile.average_gas_per_call = total_gas / profile.total_measurements as u64;

        env.storage().instance().set(&profile_key, &profile);
    }

    /// Get the current gas profile
    pub fn get_profile(env: &Env) -> Option<GasProfile> {
        env.storage().instance().get(&Symbol::short("gas_profile"))
    }

    /// Get gas measurements for a specific function
    pub fn get_function_measurements(env: &Env, function_name: Symbol) -> Vec<GasMeasurement> {
        if let Some(profile) = Self::get_profile(env) {
            let mut result = Vec::new(env);
            for measurement in profile.measurements.iter() {
                if measurement.function_name == function_name {
                    result.push_back(measurement.clone());
                }
            }
            result
        } else {
            Vec::new(env)
        }
    }

    /// Reset the gas profile (useful for testing)
    pub fn reset_profile(env: &Env) {
        env.storage().instance().remove(&Symbol::short("gas_profile"));
    }

    /// Generate a gas efficiency report
    pub fn generate_efficiency_report(env: &Env) -> Map<Symbol, u64> {
        let mut report = Map::new(env);
        
        if let Some(profile) = Self::get_profile(env) {
            // Calculate average gas per function type
            let mut function_totals: Map<Symbol, (u64, u32)> = Map::new(env);
            
            for measurement in profile.measurements.iter() {
                let (total, count) = function_totals
                    .get(measurement.function_name.clone())
                    .unwrap_or((0, 0));
                function_totals.set(
                    measurement.function_name.clone(),
                    (total + measurement.gas_consumed, count + 1)
                );
            }
            
            // Convert to averages
            for (func_name, (total, count)) in function_totals.iter() {
                if count > 0 {
                    report.set(func_name.clone(), total / count as u64);
                }
            }
        }
        
        report
    }
}

/// Macro for easy gas profiling of functions
#[macro_export]
macro_rules! profile_gas {
    ($env:expr, $func_name:expr, $code:block) => {{
        use crate::shared::gas_profiler::GasProfiler;
        
        // Start measurement
        GasProfiler::start_measurement($env, $func_name);
        
        // Execute the code and track storage operations
        let storage_reads_before = $env.storage().persistent().usage().reads;
        let storage_writes_before = $env.storage().persistent().usage().writes;
        
        let result = $code;
        
        let storage_reads = $env.storage().persistent().usage().reads - storage_reads_before;
        let storage_writes = $env.storage().persistent().usage().writes - storage_writes_before;
        
        // End measurement and record
        let measurement = GasProfiler::end_measurement($env, storage_reads, storage_writes, 0);
        GasProfiler::record_measurement($env, measurement);
        
        result
    }};
}

/// Gas optimization recommendations based on profiling data
pub struct GasOptimizer;

impl GasOptimizer {
    /// Analyze gas profile and provide optimization recommendations
    pub fn analyze_and_recommend(env: &Env) -> Vec<Symbol> {
        let mut recommendations = Vec::new(env);
        
        if let Some(profile) = GasProfiler::get_profile(env) {
            // Check for high gas consumption patterns
            if profile.average_gas_per_call > 100_000 {
                recommendations.push_back(Symbol::short("high_average_gas"));
            }
            
            if profile.peak_gas_consumption > 500_000 {
                recommendations.push_back(Symbol::short("high_peak_gas"));
            }
            
            // Analyze individual functions
            for measurement in profile.measurements.iter() {
                if measurement.gas_consumed > 200_000 {
                    recommendations.push_back(Symbol::short(&format!("{}_high_gas", measurement.function_name)));
                }
                
                if measurement.storage_writes > 10 {
                    recommendations.push_back(Symbol::short(&format!("{}_many_writes", measurement.function_name)));
                }
            }
        }
        
        recommendations
    }
}
