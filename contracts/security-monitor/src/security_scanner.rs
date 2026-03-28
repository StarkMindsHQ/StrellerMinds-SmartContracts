//! Security scanner utilities for static and runtime analysis.
//!
//! This module provides:
//! - `SecurityScanner` – a runtime scan engine that evaluates stored metrics
//!   and threat data to produce a holistic security report.
//! - `ScanReport` – a structured scan result with severity breakdown.
//! - `HealthStatus` – a quick contract-health summary.
//!
//! These utilities complement the threat-detection engine in `threat_detector.rs`
//! and are designed to be called from off-chain tooling (test harnesses, dashboards)
//! as well as on-chain administrative flows.

use crate::{
    errors::SecurityError,
    storage::SecurityStorage,
    types::{SecurityThreat, ThreatLevel},
};
use soroban_sdk::{Env, Symbol, Vec};

// ─────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────

/// A summary produced by the security scanner for a single contract.
#[derive(Clone, Debug)]
pub struct ScanReport {
    /// Contract that was scanned.
    pub contract: Symbol,
    /// All active threats for the contract at the time of the scan.
    pub threats: Vec<SecurityThreat>,
    /// Count of Low-severity threats.
    pub low_count: u32,
    /// Count of Medium-severity threats.
    pub medium_count: u32,
    /// Count of High-severity threats.
    pub high_count: u32,
    /// Count of Critical-severity threats.
    pub critical_count: u32,
    /// Aggregate security score (0 = worst, 100 = best).
    pub aggregate_score: u32,
    /// Recommended action derived from the scan.
    pub recommendation: SecurityScanAction,
}

/// High-level action the scanner recommends after a scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecurityScanAction {
    /// No issues detected; normal operation.
    NoAction,
    /// Minor issues; increase monitoring frequency.
    IncreasedMonitoring,
    /// Elevated issues; alert operators.
    AlertOperators,
    /// Serious issues; trigger automated mitigations.
    TriggerMitigation,
    /// Critical issues; open a circuit-breaker and halt operations.
    HaltOperations,
}

/// A compact contract health indicator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    /// Security score ≥ 80; no high/critical threats.
    Healthy,
    /// Security score 50–79 or at least one high threat.
    Degraded,
    /// Security score < 50 or at least one critical threat.
    Critical,
}

// ─────────────────────────────────────────────────────────────
// SecurityScanner implementation
// ─────────────────────────────────────────────────────────────

/// Stateless security scanner that operates on the contract's persistent state.
pub struct SecurityScanner;

impl SecurityScanner {
    /// Perform a full scan of `contract` and return a [`ScanReport`].
    ///
    /// # Errors
    /// Returns [`SecurityError::NotInitialized`] when called on a contract
    /// whose SecurityMonitor has not been initialised.
    pub fn scan(
        env: &Env,
        contract: &Symbol,
    ) -> Result<ScanReport, SecurityError> {
        // Require initialisation.
        let _ = SecurityStorage::get_config(env).ok_or(SecurityError::NotInitialized)?;

        // Collect all known threat IDs for this contract.
        let threat_ids = SecurityStorage::get_contract_threats(env, contract);

        // Resolve full threat records.
        let mut threats: Vec<SecurityThreat> = Vec::new(env);
        let (mut low, mut medium, mut high, mut critical) = (0u32, 0u32, 0u32, 0u32);

        for id in threat_ids.iter() {
            if let Some(threat) = SecurityStorage::get_threat(env, &id) {
                match threat.threat_level {
                    ThreatLevel::Low => low += 1,
                    ThreatLevel::Medium => medium += 1,
                    ThreatLevel::High => high += 1,
                    ThreatLevel::Critical => critical += 1,
                }
                threats.push_back(threat);
            }
        }

        let aggregate_score = Self::compute_score(low, medium, high, critical);
        let recommendation = Self::derive_action(high, critical, aggregate_score);

        Ok(ScanReport {
            contract: contract.clone(),
            threats,
            low_count: low,
            medium_count: medium,
            high_count: high,
            critical_count: critical,
            aggregate_score,
            recommendation,
        })
    }

    /// Quick health-check for `contract`.
    pub fn health_check(env: &Env, contract: &Symbol) -> Result<HealthStatus, SecurityError> {
        let report = Self::scan(env, contract)?;
        let status = if report.critical_count > 0 || report.aggregate_score < 50 {
            HealthStatus::Critical
        } else if report.high_count > 0 || report.aggregate_score < 80 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        Ok(status)
    }

    /// Check whether all known threats for `contract` have been auto-mitigated.
    pub fn all_mitigated(env: &Env, contract: &Symbol) -> bool {
        let threat_ids = SecurityStorage::get_contract_threats(env, contract);
        for id in threat_ids.iter() {
            if let Some(threat) = SecurityStorage::get_threat(env, &id) {
                if !threat.auto_mitigated {
                    return false;
                }
            }
        }
        true
    }

    // ── Private helpers ──────────────────────────────────────

    /// Compute an aggregate security score (0–100) from severity counts.
    /// Weights: Critical = −25, High = −10, Medium = −5, Low = −1.
    fn compute_score(low: u32, medium: u32, high: u32, critical: u32) -> u32 {
        let penalty =
            critical.saturating_mul(25)
                + high.saturating_mul(10)
                + medium.saturating_mul(5)
                + low;
        100u32.saturating_sub(penalty)
    }

    /// Derive the recommended action given the current threat landscape.
    fn derive_action(high: u32, critical: u32, score: u32) -> SecurityScanAction {
        if critical > 0 || score == 0 {
            SecurityScanAction::HaltOperations
        } else if high > 2 || score < 30 {
            SecurityScanAction::TriggerMitigation
        } else if high > 0 || score < 60 {
            SecurityScanAction::AlertOperators
        } else if score < 80 {
            SecurityScanAction::IncreasedMonitoring
        } else {
            SecurityScanAction::NoAction
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{MitigationAction, SecurityConfig, ThreatType},
        SecurityMonitor, SecurityMonitorClient,
    };
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, String,
    };

    fn setup_scanner() -> (Env, Address /* contract_id */) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(SecurityMonitor, ());
        let client = SecurityMonitorClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin, &SecurityConfig::default_config());
        (env, contract_id)
    }

    fn dummy_threat(
        env: &Env,
        contract: &Symbol,
        seed: u8,
        level: ThreatLevel,
    ) -> SecurityThreat {
        SecurityThreat {
            threat_id: BytesN::from_array(env, &[seed; 32]),
            threat_type: ThreatType::BurstActivity,
            threat_level: level,
            detected_at: 0,
            contract: contract.clone(),
            actor: None,
            description: String::from_str(env, "scanner-test"),
            metric_value: 1,
            threshold_value: 1,
            auto_mitigated: false,
            mitigation_action: MitigationAction::NoAction,
        }
    }

    #[test]
    fn test_scan_clean_contract_gives_full_score() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "cleanapp");
        let report = env.as_contract(&contract_id, || {
            SecurityScanner::scan(&env, &contract_sym).unwrap()
        });
        assert_eq!(report.aggregate_score, 100);
        assert_eq!(report.recommendation, SecurityScanAction::NoAction);
    }

    #[test]
    fn test_scan_detects_stored_threats() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "threatenedapp");

        let t = dummy_threat(&env, &contract_sym, 0x01, ThreatLevel::High);
        env.as_contract(&contract_id, || {
            SecurityStorage::set_threat(&env, &t);
        });

        let report = env.as_contract(&contract_id, || {
            SecurityScanner::scan(&env, &contract_sym).unwrap()
        });
        assert_eq!(report.high_count, 1);
        assert_eq!(report.recommendation, SecurityScanAction::AlertOperators);
    }

    #[test]
    fn test_health_check_healthy_when_no_threats() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "okapp");
        let status = env.as_contract(&contract_id, || {
            SecurityScanner::health_check(&env, &contract_sym).unwrap()
        });
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_critical_on_critical_threat() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "critapp");

        let t = dummy_threat(&env, &contract_sym, 0xCC, ThreatLevel::Critical);
        env.as_contract(&contract_id, || {
            SecurityStorage::set_threat(&env, &t);
        });

        let status = env.as_contract(&contract_id, || {
            SecurityScanner::health_check(&env, &contract_sym).unwrap()
        });
        assert_eq!(status, HealthStatus::Critical);
    }

    #[test]
    fn test_all_mitigated_true_when_no_threats() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "safeapp");
        let result = env.as_contract(&contract_id, || {
            SecurityScanner::all_mitigated(&env, &contract_sym)
        });
        assert!(result);
    }

    #[test]
    fn test_all_mitigated_false_when_unmitigated_threat_present() {
        let (env, contract_id) = setup_scanner();
        let contract_sym = Symbol::new(&env, "riskyapp");

        let t = dummy_threat(&env, &contract_sym, 0xAB, ThreatLevel::Medium);
        env.as_contract(&contract_id, || {
            SecurityStorage::set_threat(&env, &t);
        });

        let result = env.as_contract(&contract_id, || {
            SecurityScanner::all_mitigated(&env, &contract_sym)
        });
        assert!(!result);
    }

    #[test]
    fn test_score_computation_with_multiple_levels() {
        // 0 critical, 1 high, 2 medium, 3 low
        // penalty = 0*25 + 1*10 + 2*5 + 3*1 = 10+10+3 = 23
        let score = SecurityScanner::compute_score(3, 2, 1, 0);
        assert_eq!(score, 77);
    }

    #[test]
    fn test_halt_operations_recommended_on_critical_threat() {
        let action = SecurityScanner::derive_action(0, 1, 75);
        assert_eq!(action, SecurityScanAction::HaltOperations);
    }

    #[test]
    fn test_trigger_mitigation_recommended_on_many_high_threats() {
        let action = SecurityScanner::derive_action(3, 0, 55);
        assert_eq!(action, SecurityScanAction::TriggerMitigation);
    }

    #[test]
    fn test_no_action_on_perfect_score() {
        let action = SecurityScanner::derive_action(0, 0, 100);
        assert_eq!(action, SecurityScanAction::NoAction);
    }
}
