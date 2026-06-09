#![no_std]
extern crate alloc;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, env, Address, Bytes, BytesN, Map, Vec,
    Symbol,
};
use shared::access_control::AccessControl;

#[cfg(test)]
mod tests;

// Contract type for storage
#[contracttype]
pub struct FraudDetectionConfig {
    detection_threshold: u32,
    alert_cooldown: u64,
    pattern_analysis_enabled: bool,
    signature_verification_enabled: bool,
    data_validation_enabled: bool,
    timestamp_analysis_enabled: bool,
}

#[contracttype]
pub struct Credential {
    id: BytesN<32>,
    student_id: BytesN<32>,
    issuer: Address,
    issue_date: u64,
    signature: Bytes,
    student_data: StudentData,
}

#[contracttype]
pub struct StudentData {
    name: Bytes,
    email: Bytes,
    institution: Bytes,
    course: Bytes,
    grade: Bytes,
    completion_date: u64,
}

#[contracttype]
pub struct DetectionEvent {
    credential_id: BytesN<32>,
    fraud_type: FraudType,
    confidence_score: u32,
    timestamp: u64,
    detected_by: Address,
    details: Bytes,
}

#[contracttype]
pub enum FraudType {
    UnusualIssuancePattern,
    ForgedSignature,
    InvalidStudentData,
    TimestampAnomaly,
}

#[contracttype]
pub enum DetectionResult {
    Clean,
    Suspicious { alert_id: u64, confidence: u32 },
    Confirmed { event_id: u64, fraud_type: FraudType },
}

#[contracttype]
pub struct Alert {
    id: u64,
    credential_id: BytesN<32>,
    fraud_type: FraudType,
    severity: AlertSeverity,
    confidence_score: u32,
    timestamp: u64,
    acknowledged: bool,
    details: Bytes,
}

#[contracttype]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[contracttype]
pub struct FraudStatistics {
    total_credentials_checked: u64,
    fraud_detected: u64,
    false_positives: u64,
    accuracy_rate: u32,
    false_positive_rate: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FraudDetectionError {
    InvalidCredential = 1,
    InvalidSignature = 2,
    InvalidTimestamp = 3,
    InvalidStudentData = 4,
    DetectionFailed = 5,
    UnauthorizedAccess = 6,
    AlertNotFound = 7,
    ModelNotInitialized = 8,
}

const CONFIG_KEY: Symbol = Symbol::short("CONFIG");
const DETECTION_EVENTS_KEY: Symbol = Symbol::short("EVENTS");
const ALERTS_KEY: Symbol = Symbol::short("ALERTS");
const STATISTICS_KEY: Symbol = Symbol::short("STATS");
const MODEL_KEY: Symbol = Symbol::short("MODEL");

pub struct FraudDetectionContract;

#[contractimpl]
impl FraudDetectionContract {
    /// Initialize the fraud detection contract
    pub fn __init__(
        env: &Env,
        admin: Address,
        detection_threshold: u32,
        alert_cooldown: u64,
    ) -> Result<(), FraudDetectionError> {
        // Initialize access control
        AccessControl::init(env, admin);
        
        // Initialize configuration
        let config = FraudDetectionConfig {
            detection_threshold,
            alert_cooldown,
            pattern_analysis_enabled: true,
            signature_verification_enabled: true,
            data_validation_enabled: true,
            timestamp_analysis_enabled: true,
        };
        env.storage().instance().set(&CONFIG_KEY, &config);
        
        // Initialize statistics
        let stats = FraudStatistics {
            total_credentials_checked: 0,
            fraud_detected: 0,
            false_positives: 0,
            accuracy_rate: 0,
            false_positive_rate: 0,
        };
        env.storage().instance().set(&STATISTICS_KEY, &stats);
        
        // Initialize storage vectors
        env.storage().instance().set(&DETECTION_EVENTS_KEY, &Vec::<DetectionEvent>::new(env));
        env.storage().instance().set(&ALERTS_KEY, &Vec::<Alert>::new(env));
        
        // Initialize ML model (simplified for demonstration)
        let model_data = Bytes::from_slice(env, &[1, 2, 3, 4, 5]); // Placeholder model weights
        env.storage().instance().set(&MODEL_KEY, &model_data);
        
        Ok(())
    }
    
    /// Main fraud detection function
    pub fn detect_fraud(env: &Env, credential: Credential) -> Result<DetectionResult, FraudDetectionError> {
        // Validate input
        Self::validate_credential(env, &credential)?;
        
        // Get configuration
        let config: FraudDetectionConfig = env.storage().instance().get(&CONFIG_KEY)
            .ok_or(FraudDetectionError::ModelNotInitialized)?;
        
        // Initialize detection results
        let mut max_confidence = 0u32;
        let mut detected_fraud_type = None;
        
        // Run detection modules
        if config.pattern_analysis_enabled {
            if let Ok((confidence, _)) = Self::analyze_issuance_pattern(env, &credential) {
                if confidence > max_confidence && confidence >= config.detection_threshold {
                    max_confidence = confidence;
                    detected_fraud_type = Some(FraudType::UnusualIssuancePattern);
                }
            }
        }
        
        if config.signature_verification_enabled {
            if let Ok((confidence, _)) = Self::verify_signature(env, &credential) {
                if confidence > max_confidence && confidence >= config.detection_threshold {
                    max_confidence = confidence;
                    detected_fraud_type = Some(FraudType::ForgedSignature);
                }
            }
        }
        
        if config.data_validation_enabled {
            if let Ok((confidence, _)) = Self::validate_student_data(env, &credential) {
                if confidence > max_confidence && confidence >= config.detection_threshold {
                    max_confidence = confidence;
                    detected_fraud_type = Some(FraudType::InvalidStudentData);
                }
            }
        }
        
        if config.timestamp_analysis_enabled {
            if let Ok((confidence, _)) = Self::analyze_timestamp(env, &credential) {
                if confidence > max_confidence && confidence >= config.detection_threshold {
                    max_confidence = confidence;
                    detected_fraud_type = Some(FraudType::TimestampAnomaly);
                }
            }
        }
        
        // Update statistics
        Self::update_statistics(env, max_confidence >= config.detection_threshold);
        
        // Return detection result
        if max_confidence >= config.detection_threshold {
            let fraud_type = detected_fraud_type.ok_or(FraudDetectionError::DetectionFailed)?;
            
            if max_confidence >= 90 {
                // High confidence - confirmed fraud
                let event_id = Self::create_detection_event(env, &credential, fraud_type, max_confidence);
                Ok(DetectionResult::Confirmed { event_id, fraud_type })
            } else {
                // Medium confidence - suspicious
                let alert_id = Self::create_alert(env, &credential, fraud_type, max_confidence);
                Ok(DetectionResult::Suspicious { alert_id, confidence: max_confidence })
            }
        } else {
            Ok(DetectionResult::Clean)
        }
    }
    
    /// Analyze unusual issuance patterns
    pub fn analyze_issuance_pattern(env: &Env, credential: &Credential) -> Result<(u32, Bytes), FraudDetectionError> {
        let current_time = env.ledger().timestamp();
        let issue_time = credential.issue_date;
        
        // Check for burst issuance (multiple credentials in short time)
        let time_window = 3600; // 1 hour
        let mut recent_credentials = 0u32;
        
        // This is a simplified version - in practice, you'd query historical data
        if current_time - issue_time < time_window {
            recent_credentials += 1;
        }
        
        // Calculate confidence based on pattern analysis
        let confidence = if recent_credentials > 10 {
            95 // High confidence for burst detection
        } else if recent_credentials > 5 {
            75 // Medium confidence
        } else {
            25 // Low confidence
        };
        
        let details_str = alloc::format!("Recent credentials: {}", recent_credentials);
        let details = Bytes::from_slice(env, details_str.as_bytes());
        Ok((confidence, details))
    }
    
    /// Verify signature authenticity
    pub fn verify_signature(env: &Env, credential: &Credential) -> Result<(u32, Bytes), FraudDetectionError> {
        // Simplified signature verification
        // In practice, this would use proper cryptographic verification
        
        let signature_valid = credential.signature.len() > 0; // Simplified check
        
        let confidence = if signature_valid {
            20 // Low confidence (clean signature)
        } else {
            95 // High confidence (invalid signature)
        };
        
        let details = Bytes::from_slice(env, if signature_valid {
            b"Signature appears valid"
        } else {
            b"Signature verification failed"
        });
        
        Ok((confidence, details))
    }
    
    /// Validate student data integrity
    pub fn validate_student_data(env: &Env, credential: &Credential) -> Result<(u32, Bytes), FraudDetectionError> {
        let data = &credential.student_data;
        let mut issues = 0u32;
        
        // Check email format (simplified)
        let email_str = std::str::from_utf8(&data.email.to_array()).unwrap_or("");
        if !email_str.contains("@") {
            issues += 1;
        }
        
        // Check completion date is reasonable
        let current_time = env.ledger().timestamp();
        if data.completion_date > current_time {
            issues += 1;
        }
        
        // Check name is not empty
        if data.name.is_empty() {
            issues += 1;
        }
        
        let confidence = if issues >= 2 {
            85 // High confidence for invalid data
        } else if issues == 1 {
            60 // Medium confidence
        } else {
            15 // Low confidence (clean data)
        };
        
        let details_str = alloc::format!("Data issues found: {}", issues);
        let details = Bytes::from_slice(env, details_str.as_bytes());
        Ok((confidence, details))
    }
    
    /// Analyze timestamp anomalies
    pub fn analyze_timestamp(env: &Env, credential: &Credential) -> Result<(u32, Bytes), FraudDetectionError> {
        let current_time = env.ledger().timestamp();
        let issue_time = credential.issue_date;
        let completion_time = credential.student_data.completion_date;
        
        let mut anomalies = 0u32;
        
        // Check if issue date is in future
        if issue_time > current_time {
            anomalies += 1;
        }
        
        // Check if completion date is after issue date
        if completion_time > issue_time {
            anomalies += 1;
        }
        
        // Check if dates are too old (before 2000)
        let year_2000 = 946684800; // Unix timestamp for year 2000
        if issue_time < year_2000 || completion_time < year_2000 {
            anomalies += 1;
        }
        
        let confidence = if anomalies >= 2 {
            90 // High confidence for timestamp anomalies
        } else if anomalies == 1 {
            70 // Medium confidence
        } else {
            10 // Low confidence (clean timestamps)
        };
        
        let details_str = alloc::format!("Timestamp anomalies: {}", anomalies);
        let details = Bytes::from_slice(env, details_str.as_bytes());
        Ok((confidence, details))
    }
    
    /// Create a detection event
    fn create_detection_event(env: &Env, credential: &Credential, fraud_type: FraudType, confidence: u32) -> u64 {
        let mut events: Vec<DetectionEvent> = env.storage().instance().get(&DETECTION_EVENTS_KEY)
            .unwrap_or(Vec::new(env));
        
        let event_id = events.len();
        let event = DetectionEvent {
            credential_id: credential.id,
            fraud_type,
            confidence_score: confidence,
            timestamp: env.ledger().timestamp(),
            detected_by: env.current_contract_address(),
            details: Bytes::from_slice(env, b"Fraud detected"),
        };
        
        events.push_back(event);
        env.storage().instance().set(&DETECTION_EVENTS_KEY, &events);
        
        event_id as u64
    }
    
    /// Create an alert
    fn create_alert(env: &Env, credential: &Credential, fraud_type: FraudType, confidence: u32) -> u64 {
        let mut alerts: Vec<Alert> = env.storage().instance().get(&ALERTS_KEY)
            .unwrap_or(Vec::new(env));
        
        let alert_id = alerts.len();
        let severity = if confidence >= 80 {
            AlertSeverity::High
        } else if confidence >= 60 {
            AlertSeverity::Medium
        } else {
            AlertSeverity::Low
        };
        
        let alert = Alert {
            id: alert_id as u64,
            credential_id: credential.id,
            fraud_type,
            severity,
            confidence_score: confidence,
            timestamp: env.ledger().timestamp(),
            acknowledged: false,
            details: Bytes::from_slice(env, b"Suspicious activity detected"),
        };
        
        alerts.push_back(alert);
        env.storage().instance().set(&ALERTS_KEY, &alerts);
        
        alert_id as u64
    }
    
    /// Update fraud detection statistics
    fn update_statistics(env: &Env, fraud_detected: bool) {
        let mut stats: FraudStatistics = env.storage().instance().get(&STATISTICS_KEY)
            .unwrap_or(FraudStatistics {
                total_credentials_checked: 0,
                fraud_detected: 0,
                false_positives: 0,
                accuracy_rate: 0,
                false_positive_rate: 0,
            });
        
        stats.total_credentials_checked += 1;
        
        if fraud_detected {
            stats.fraud_detected += 1;
        }
        
        // Update rates (simplified calculation)
        if stats.total_credentials_checked > 0 {
            stats.accuracy_rate = ((stats.total_credentials_checked - stats.false_positives) * 100 / stats.total_credentials_checked) as u32;
            stats.false_positive_rate = (stats.false_positives * 100 / stats.total_credentials_checked) as u32;
        }
        
        env.storage().instance().set(&STATISTICS_KEY, &stats);
    }
    
    /// Validate credential structure
    fn validate_credential(env: &Env, credential: &Credential) -> Result<(), FraudDetectionError> {
        if credential.id.is_empty() {
            return Err(FraudDetectionError::InvalidCredential);
        }
        
        if credential.student_id.is_empty() {
            return Err(FraudDetectionError::InvalidCredential);
        }
        
        if credential.signature.is_empty() {
            return Err(FraudDetectionError::InvalidSignature);
        }
        
        if credential.issue_date == 0 {
            return Err(FraudDetectionError::InvalidTimestamp);
        }
        
        Ok(())
    }
    
    /// Get recent alerts
    pub fn get_recent_alerts(env: &Env, limit: u32) -> Result<Vec<Alert>, FraudDetectionError> {
        let alerts: Vec<Alert> = env.storage().instance().get(&ALERTS_KEY)
            .unwrap_or(Vec::new(env));
        
        let start = if alerts.len() > limit {
            alerts.len() - limit
        } else {
            0
        };
        
        let mut recent_alerts = Vec::new(env);
        for i in start..alerts.len() {
            recent_alerts.push_back(alerts.get(i).unwrap());
        }
        
        Ok(recent_alerts)
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(env: &Env, alert_id: u64) -> Result<(), FraudDetectionError> {
        let mut alerts: Vec<Alert> = env.storage().instance().get(&ALERTS_KEY)
            .unwrap_or(Vec::new(env));
        
        if alert_id >= alerts.len() as u64 {
            return Err(FraudDetectionError::AlertNotFound);
        }
        
        let alert = alerts.get(alert_id as usize).unwrap();
        let mut updated_alert = alert.clone();
        updated_alert.acknowledged = true;
        
        alerts.set(alert_id as usize, updated_alert);
        env.storage().instance().set(&ALERTS_KEY, &alerts);
        
        Ok(())
    }
    
    /// Get fraud detection statistics
    pub fn get_fraud_statistics(env: &Env) -> Result<FraudStatistics, FraudDetectionError> {
        let stats: FraudStatistics = env.storage().instance().get(&STATISTICS_KEY)
            .ok_or(FraudDetectionError::ModelNotInitialized)?;
        
        Ok(stats)
    }
    
    /// Update configuration (admin only)
    pub fn update_config(env: &Env, admin: Address, config: FraudDetectionConfig) -> Result<(), FraudDetectionError> {
        // Check authorization
        if !AccessControl::has_role(env, &admin, shared::roles::Role::Admin) {
            return Err(FraudDetectionError::UnauthorizedAccess);
        }
        
        env.storage().instance().set(&CONFIG_KEY, &config);
        Ok(())
    }
    
    /// Get current configuration
    pub fn get_config(env: &Env) -> Result<FraudDetectionConfig, FraudDetectionError> {
        let config: FraudDetectionConfig = env.storage().instance().get(&CONFIG_KEY)
            .ok_or(FraudDetectionError::ModelNotInitialized)?;
        
        Ok(config)
    }
}
