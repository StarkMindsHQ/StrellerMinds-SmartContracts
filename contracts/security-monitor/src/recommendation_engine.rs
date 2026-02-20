use crate::errors::SecurityError;
use crate::storage::SecurityStorage;
use crate::types::{RecommendationCategory, SecurityRecommendation, SecurityThreat, ThreatType};
use soroban_sdk::{Env, String, Vec};

/// Engine for generating automated security fix recommendations
pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Generate recommendations for a given threat
    pub fn generate_recommendations(
        env: &Env,
        threat: &SecurityThreat,
    ) -> Result<Vec<SecurityRecommendation>, SecurityError> {
        let mut recommendations = Vec::new(env);

        match threat.threat_type {
            ThreatType::BurstActivity => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::RateLimiting,
                    title: String::from_str(env, "Implement Rate Limiting"),
                    description: String::from_str(
                        env,
                        "High burst activity detected. Consider implementing rate limiting to prevent abuse.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "Use EventUtils::check_rate_limit() before processing events. Example: EventUtils::check_rate_limit(&env, &actor, 100, 3600)?;",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::AccessViolation => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::AccessControl,
                    title: String::from_str(env, "Review RBAC Configuration"),
                    description: String::from_str(
                        env,
                        "Access control violations detected. Review role assignments and permissions.",
                    ),
                    code_location: None,
                    fix_suggestion: String::from_str(
                        env,
                        "1. Audit current role grants. 2. Implement stricter permission checks. 3. Use AccessControl::require_permission() for sensitive operations.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::ReentrancyAttempt => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::ReentrancyPrevention,
                    title: String::from_str(env, "Add Reentrancy Guard"),
                    description: String::from_str(
                        env,
                        "Potential reentrancy detected. Ensure all sensitive functions use reentrancy guards.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "Add 'let _guard = ReentrancyLock::new(&env);' at the start of sensitive functions.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::ValidationFailure => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::InputValidation,
                    title: String::from_str(env, "Strengthen Input Validation"),
                    description: String::from_str(
                        env,
                        "Input validation failures detected. Add comprehensive input validation.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "Use CoreValidator from shared library. Example: CoreValidator::validate_string_length(&field, 3, 200)?;",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::ErrorRateSpike => {
                let rec1 = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::InputValidation,
                    title: String::from_str(env, "Review Input Validation"),
                    description: String::from_str(
                        env,
                        "High error rate indicates potential input validation issues.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "Review and strengthen input validation using CoreValidator patterns.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec1);
                recommendations.push_back(rec1);

                let rec2 = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 1),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::Configuration,
                    title: String::from_str(env, "Implement Circuit Breaker"),
                    description: String::from_str(
                        env,
                        "Consider circuit breaker pattern to prevent cascading failures.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "Use CircuitBreaker::check_and_record() to track failures and prevent cascading issues.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec2);
                recommendations.push_back(rec2);
            }

            ThreatType::AnomalousActor => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::RateLimiting,
                    title: String::from_str(env, "Implement Per-Actor Rate Limiting"),
                    description: String::from_str(
                        env,
                        "Anomalous actor behavior detected. Implement per-actor rate limits.",
                    ),
                    code_location: None,
                    fix_suggestion: String::from_str(
                        env,
                        "Track actor event counts and enforce per-actor limits. Review SecurityMonitor::check_rate_limit() implementation.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::RateLimitExceeded => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::RateLimiting,
                    title: String::from_str(env, "Adjust Rate Limit Thresholds"),
                    description: String::from_str(
                        env,
                        "Rate limits are being exceeded. Review and adjust thresholds or investigate actor behavior.",
                    ),
                    code_location: None,
                    fix_suggestion: String::from_str(
                        env,
                        "1. Investigate if this is legitimate traffic. 2. Adjust rate_limit_per_window in config if needed. 3. Consider temporary actor restrictions.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::SequenceIntegrityIssue => {
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::EventIntegrity,
                    title: String::from_str(env, "Investigate Event Integrity"),
                    description: String::from_str(
                        env,
                        "Event sequence integrity issue detected. Review event emission and ordering.",
                    ),
                    code_location: None, // TODO: Convert Symbol to String when supported
                    fix_suggestion: String::from_str(
                        env,
                        "1. Ensure all events use EventManager::publish_event(). 2. Review event emission patterns. 3. Check for missing event emissions.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }

            ThreatType::BehavioralAnomaly 
            | ThreatType::CredentialFraud 
            | ThreatType::BiometricFailure 
            | ThreatType::KnownMaliciousActor => {
                // For AI/Oracle based threats, recommendations are integrated in the AI response or handled dynamically.
                // Or we generate a generic placeholder:
                let rec = SecurityRecommendation {
                    recommendation_id: Self::generate_recommendation_id(env, &threat.threat_id, 0),
                    threat_id: threat.threat_id.clone(),
                    severity: threat.threat_level.clone(),
                    category: RecommendationCategory::Configuration,
                    title: String::from_str(env, "Review AI Security Insight"),
                    description: String::from_str(
                        env,
                        "Please verify the flagged behavior or credential with external evidence.",
                    ),
                    code_location: None, 
                    fix_suggestion: String::from_str(
                        env,
                        "Evaluate the actor's recent activity logs and adjust threshold if necessary.",
                    ),
                    created_at: env.ledger().timestamp(),
                    acknowledged: false,
                };
                SecurityStorage::set_recommendation(env, &rec);
                recommendations.push_back(rec);
            }
        }

        Ok(recommendations)
    }

    /// Generate a unique recommendation ID
    fn generate_recommendation_id(
        env: &Env,
        threat_id: &soroban_sdk::BytesN<32>,
        index: u32,
    ) -> soroban_sdk::BytesN<32> {
        let mut data = [0u8; 32];

        // Copy threat ID
        data.copy_from_slice(&threat_id.to_array());

        // XOR with index to make unique
        let index_bytes = index.to_be_bytes();
        for i in 0..4 {
            data[i] ^= index_bytes[i];
        }

        soroban_sdk::BytesN::from_array(env, &data)
    }
}
