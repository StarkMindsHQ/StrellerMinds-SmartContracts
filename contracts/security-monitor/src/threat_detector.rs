use crate::errors::SecurityError;
use crate::storage::SecurityStorage;
use crate::types::{SecurityMetrics, SecurityThreat, ThreatLevel, ThreatType};
use soroban_sdk::{Env, String, Symbol};

/// Core threat detection engine
pub struct ThreatDetector;

impl ThreatDetector {
    /// Detect burst activity (spike in events)
    pub fn detect_burst_activity(
        env: &Env,
        contract: &Symbol,
        window_seconds: u64,
    ) -> Result<Option<SecurityThreat>, SecurityError> {
        let config = SecurityStorage::get_config(env).ok_or(SecurityError::NotInitialized)?;
        let current_time = env.ledger().timestamp();
        let window_start = current_time.saturating_sub(window_seconds);

        // In a real implementation, we would use EventReplay to get actual events
        // For now, we'll use a simplified detection based on stored metrics
        let window_id = current_time / 3600;
        let event_count = if let Some(metrics) = SecurityStorage::get_security_metrics(env, contract, window_id) {
            metrics.total_events
        } else {
            // No metrics available yet
            return Ok(None);
        };

        if event_count > config.burst_detection_threshold {
            let threat_level = Self::classify_burst_threat_level(event_count, config.burst_detection_threshold);

            let threat = SecurityThreat {
                threat_id: Self::generate_threat_id(env, contract),
                threat_type: ThreatType::BurstActivity,
                threat_level,
                detected_at: current_time,
                contract: contract.clone(),
                actor: None,
                description: String::from_str(env, "Burst activity detected"),
                metric_value: event_count,
                threshold_value: config.burst_detection_threshold,
                auto_mitigated: false,
                mitigation_action: None,
            };

            Ok(Some(threat))
        } else {
            Ok(None)
        }
    }

    /// Detect error rate spikes
    pub fn detect_error_rate_spike(
        env: &Env,
        contract: &Symbol,
        window_id: u64,
    ) -> Result<Option<SecurityThreat>, SecurityError> {
        let config = SecurityStorage::get_config(env).ok_or(SecurityError::NotInitialized)?;

        let metrics = match SecurityStorage::get_security_metrics(env, contract, window_id) {
            Some(m) => m,
            None => return Ok(None), // No metrics available yet
        };

        if metrics.error_rate > config.error_rate_threshold {
            let threat_level = if metrics.error_rate > 50 {
                ThreatLevel::Critical
            } else if metrics.error_rate > 30 {
                ThreatLevel::High
            } else if metrics.error_rate > 20 {
                ThreatLevel::Medium
            } else {
                ThreatLevel::Low
            };

            let threat = SecurityThreat {
                threat_id: Self::generate_threat_id(env, contract),
                threat_type: ThreatType::ErrorRateSpike,
                threat_level,
                detected_at: env.ledger().timestamp(),
                contract: contract.clone(),
                actor: None,
                description: String::from_str(env, "Error rate spike detected"),
                metric_value: metrics.error_rate,
                threshold_value: config.error_rate_threshold,
                auto_mitigated: false,
                mitigation_action: None,
            };

            Ok(Some(threat))
        } else {
            Ok(None)
        }
    }

    /// Calculate security metrics for a time window
    pub fn calculate_metrics(
        env: &Env,
        contract: &Symbol,
        start_time: u64,
        end_time: u64,
        window_id: u64,
    ) -> Result<SecurityMetrics, SecurityError> {
        // In a real implementation, this would use EventReplay to analyze actual events
        // For this implementation, we'll create basic metrics

        // Get threat count for this contract
        let threats = SecurityStorage::get_contract_threats(env, contract);
        let mut threat_count = 0u32;
        let mut highest_threat_level = ThreatLevel::Low;

        for i in 0..threats.len() {
            if let Some(threat) = SecurityStorage::get_threat(env, &threats.get(i).unwrap()) {
                if threat.detected_at >= start_time && threat.detected_at <= end_time {
                    threat_count += 1;
                    if threat.threat_level > highest_threat_level {
                        highest_threat_level = threat.threat_level.clone();
                    }
                }
            }
        }

        // Simplified metrics calculation
        // In a real implementation, these would be calculated from actual events
        let total_events = 0u32; // Would be counted from event replay
        let error_events = 0u32; // Would be filtered from events
        let error_rate = if total_events > 0 {
            (error_events * 100) / total_events
        } else {
            0
        };

        let metrics = SecurityMetrics {
            window_id,
            contract: contract.clone(),
            start_time,
            end_time,
            total_events,
            error_events,
            error_rate,
            unique_actors: 0, // Would be calculated from events
            access_violations: 0, // Would be filtered from access control events
            threat_count,
            highest_threat_level,
            security_score: Self::calculate_basic_security_score(error_rate, threat_count),
        };

        Ok(metrics)
    }

    /// Calculate basic security score
    fn calculate_basic_security_score(error_rate: u32, threat_count: u32) -> u32 {
        let mut score = 100u32;

        // Deduct for error rate
        if error_rate > 0 {
            score = score.saturating_sub(error_rate.min(50));
        }

        // Deduct for threats (5 points per threat, max 50 points)
        let threat_penalty = (threat_count * 5).min(50);
        score = score.saturating_sub(threat_penalty);

        score
    }

    /// Classify burst threat level based on how much the threshold was exceeded
    fn classify_burst_threat_level(event_count: u32, threshold: u32) -> ThreatLevel {
        let ratio = event_count / threshold;

        if ratio >= 10 {
            ThreatLevel::Critical
        } else if ratio >= 5 {
            ThreatLevel::High
        } else if ratio >= 2 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }

    /// Generate a unique threat ID
    fn generate_threat_id(env: &Env, contract: &Symbol) -> soroban_sdk::BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();

        // Create deterministic but unique ID from timestamp, sequence, and contract
        let mut data = [0u8; 32];
        let ts_bytes = timestamp.to_be_bytes();
        let seq_bytes = sequence.to_be_bytes();

        // Mix timestamp and sequence
        for i in 0..8 {
            data[i] = ts_bytes[i];
            data[i + 8] = seq_bytes[i];
        }

        // Mix in contract symbol hash (simplified without to_string)
        // Just use the timestamp/sequence for unique IDs since Symbol doesn't support to_string

        soroban_sdk::BytesN::from_array(env, &data)
    }
}
