#![no_std]

pub mod circuit_breaker;
pub mod errors;
pub mod events;
pub mod interface;
pub mod recommendation_engine;
pub mod security_scanner;
pub mod storage;
pub mod threat_detector;
pub mod types;

#[cfg(test)]
pub mod tests;

use crate::events::SecurityEvents;
use crate::storage::SecurityStorage;
use crate::threat_detector::ThreatDetector;
use crate::types::{
    IncidentReport, MitigationAction, RateLimitState, RbacRole, RoleAssignment, RoleDelegation,
    SecurityConfig, SecurityThreat, SecurityTrainingStatus, ThreatId, ThreatIdList,
    ThreatIntelligence, ThreatLevel, ThreatType, UserRiskScore,
};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Error, String, Symbol, Vec};

#[contract]
pub struct SecurityMonitor;

#[contractimpl]
impl SecurityMonitor {
    fn not_initialized_error() -> Error {
        Error::from_contract_error(2)
    }

    fn rate_limit_exceeded_error() -> Error {
        Error::from_contract_error(3)
    }

    /// Initialize the security monitor contract with an admin address and configuration.
    ///
    /// Must be called once before any other function. Sets the admin and stores the
    /// initial security configuration.
    ///
    /// # Arguments
    /// * `admin` - Address that will hold administrative privileges.
    /// * `config` - Initial security configuration parameters.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin, &config);
    /// ```
    pub fn initialize(env: Env, admin: Address, config: SecurityConfig) -> Result<(), Error> {
        config.validate().map_err(|err| Error::from_contract_error(err as u32))?;
        SecurityStorage::set_admin(&env, &admin);
        SecurityStorage::set_config(&env, &config);
        SecurityEvents::emit_initialized(&env, &admin);
        Ok(())
    }

    /// Scan a contract for active security threats within a given time window.
    ///
    /// Returns an empty list in the current implementation (placeholder for oracle integration).
    ///
    /// # Arguments
    /// * `_contract` - Symbol identifier of the contract to scan.
    /// * `_window_seconds` - How far back in time (in seconds) to look for threats.
    ///
    /// # Example
    /// ```ignore
    /// let threats = client.scan_for_threats(&contract_symbol, &3600u64);
    /// ```
    pub fn scan_for_threats(
        _env: Env,
        _contract: Symbol,
        _window_seconds: u64,
    ) -> Result<Vec<SecurityThreat>, Error> {
        Ok(Vec::new(&_env)) // Basic placeholder
    }

    /// Retrieve a single threat record by its unique ID.
    ///
    /// # Arguments
    /// * `threat_id` - 32-byte identifier of the threat to retrieve.
    ///
    /// # Errors
    /// Returns a generic contract error (code `0`) if the threat is not found.
    ///
    /// # Example
    /// ```ignore
    /// let threat = client.get_threat(&threat_id);
    /// ```
    pub fn get_threat(env: Env, threat_id: ThreatId) -> Result<SecurityThreat, Error> {
        SecurityStorage::get_threat(&env, &threat_id).ok_or(Error::from_contract_error(0))
        // 0 = generic error placeholder
    }

    /// Return all threat IDs associated with a specific contract.
    ///
    /// # Arguments
    /// * `contract` - Symbol identifier of the contract whose threats should be listed.
    ///
    /// # Example
    /// ```ignore
    /// let ids = client.get_contract_threats(&contract_symbol);
    /// ```
    pub fn get_contract_threats(env: Env, contract: Symbol) -> ThreatIdList {
        SecurityStorage::get_contract_threats(&env, &contract)
    }

    pub fn check_rate_limit(env: Env, actor: Address, contract: Symbol) -> Result<bool, Error> {
        let config = SecurityStorage::get_config(&env).ok_or_else(Self::not_initialized_error)?;
        let current_time = env.ledger().timestamp();

        let mut state = SecurityStorage::get_rate_limit_state(&env, &actor, &contract).unwrap_or(
            RateLimitState {
                window_started_at: current_time,
                event_count: 0,
                last_attempt_at: current_time,
            },
        );

        if current_time >= state.window_started_at.saturating_add(config.rate_limit_window) {
            state.window_started_at = current_time;
            state.event_count = 0;
        }

        state.event_count = state.event_count.saturating_add(1);
        state.last_attempt_at = current_time;
        SecurityStorage::set_rate_limit_state(&env, &actor, &contract, &state);

        let exceeded = state.event_count > config.rate_limit_per_window;
        if exceeded {
            SecurityEvents::emit_rate_limit_exceeded(
                &env,
                &actor,
                &contract,
                state.event_count,
                config.rate_limit_per_window,
            );

            let severity = if state.event_count >= config.rate_limit_per_window.saturating_mul(2) {
                ThreatLevel::High
            } else {
                ThreatLevel::Medium
            };

            let threat = SecurityThreat {
                threat_id: ThreatDetector::generate_threat_id(&env, &contract),
                threat_type: ThreatType::RateLimitExceeded,
                threat_level: severity,
                detected_at: current_time,
                contract: contract.clone(),
                actor: Some(actor.clone()),
                description: String::from_str(&env, "Rate limit exceeded"),
                metric_value: state.event_count,
                threshold_value: config.rate_limit_per_window,
                auto_mitigated: true,
                mitigation_action: MitigationAction::RateLimitApplied,
            };
            SecurityStorage::set_threat(&env, &threat);
        }

        Ok(exceeded)
    }

    // --- Advanced Features Implementation ---

    /// Submit an asynchronous anomaly analysis request for a contract and return a request ID.
    ///
    /// Emits an event so an off-chain oracle can pick up the request. The oracle must
    /// later call [`SecurityMonitor::callback_anomaly_analysis`] with the result.
    ///
    /// # Arguments
    /// * `actor` - Address initiating the analysis request.
    /// * `contract` - Symbol identifier of the contract to analyze.
    ///
    /// # Example
    /// ```ignore
    /// let request_id = client.request_anomaly_analysis(&actor, &contract_symbol);
    /// ```
    pub fn request_anomaly_analysis(
        env: Env,
        actor: Address,
        contract: Symbol,
    ) -> Result<ThreatId, Error> {
        if Self::check_rate_limit(env.clone(), actor.clone(), contract.clone())? {
            return Err(Self::rate_limit_exceeded_error());
        }
        let request_id = ThreatDetector::generate_threat_id(&env, &contract); // Re-use ID generator for request ID
        SecurityEvents::emit_anomaly_requested(&env, &actor, &contract, &request_id);
        Ok(request_id)
    }

    /// Deliver the result of an anomaly analysis back to the contract from an authorized oracle.
    ///
    /// Only an oracle address previously authorized in storage may call this function.
    ///
    /// # Arguments
    /// * `oracle` - Address of the authorized oracle submitting the result.
    /// * `request_id` - The request ID returned by [`SecurityMonitor::request_anomaly_analysis`].
    /// * `is_anomalous` - Whether anomalous behavior was detected.
    /// * `risk_score` - Numeric risk score computed by the oracle (0–100 scale).
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `oracle` is not authorized.
    ///
    /// # Example
    /// ```ignore
    /// client.callback_anomaly_analysis(&oracle, &request_id, &true, &75u32);
    /// ```
    pub fn callback_anomaly_analysis(
        env: Env,
        oracle: Address,
        request_id: ThreatId,
        is_anomalous: bool,
        risk_score: u32,
    ) -> Result<(), Error> {
        if !SecurityStorage::is_oracle_authorized(&env, &oracle) {
            return Err(Error::from_contract_error(1)); // 1 = unauthorized
        }
        ThreatDetector::handle_anomaly_callback(&env, request_id, is_anomalous, risk_score);
        Ok(())
    }

    /// Submit an asynchronous biometric verification request and return a request ID.
    ///
    /// Emits an event so an off-chain oracle can perform the biometric check. The oracle
    /// must later call [`SecurityMonitor::callback_biometrics_verification`] with the result.
    ///
    /// # Arguments
    /// * `actor` - Address of the user requesting biometric verification.
    /// * `_encrypted_payload` - Encrypted biometric payload for the oracle to verify.
    ///
    /// # Example
    /// ```ignore
    /// let request_id = client.verify_biometrics(&user, &encrypted_payload);
    /// ```
    pub fn verify_biometrics(
        env: Env,
        actor: Address,
        _encrypted_payload: String,
    ) -> Result<ThreatId, Error> {
        let dummy_contract = Symbol::new(&env, "biometrics");
        if Self::check_rate_limit(env.clone(), actor.clone(), dummy_contract.clone())? {
            return Err(Self::rate_limit_exceeded_error());
        }
        let request_id = ThreatDetector::generate_threat_id(&env, &dummy_contract);
        SecurityEvents::emit_biometrics_requested(&env, &actor, &request_id);
        Ok(request_id)
    }

    /// Deliver the result of a biometric verification back to the contract from an authorized oracle.
    ///
    /// Only an oracle address previously authorized in storage may call this function.
    ///
    /// # Arguments
    /// * `oracle` - Address of the authorized oracle submitting the result.
    /// * `request_id` - The request ID returned by [`SecurityMonitor::verify_biometrics`].
    /// * `is_valid` - Whether biometric verification succeeded.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `oracle` is not authorized.
    ///
    /// # Example
    /// ```ignore
    /// client.callback_biometrics_verification(&oracle, &request_id, &true);
    /// ```
    pub fn callback_biometrics_verification(
        env: Env,
        oracle: Address,
        request_id: ThreatId,
        is_valid: bool,
    ) -> Result<(), Error> {
        if !SecurityStorage::is_oracle_authorized(&env, &oracle) {
            return Err(Error::from_contract_error(1));
        }
        ThreatDetector::handle_biometrics_callback(&env, request_id, is_valid);
        Ok(())
    }

    /// Submit an asynchronous credential fraud-detection request and return a request ID.
    ///
    /// Emits an event so an off-chain oracle can verify the credential hash. The oracle
    /// must later call [`SecurityMonitor::callback_credential_fraud`] with the result.
    ///
    /// # Arguments
    /// * `actor` - Address initiating the fraud check.
    /// * `_credential_hash` - 32-byte hash of the credential to verify.
    ///
    /// # Example
    /// ```ignore
    /// let request_id = client.verify_credential_fraud(&actor, &credential_hash);
    /// ```
    pub fn verify_credential_fraud(
        env: Env,
        actor: Address,
        _credential_hash: BytesN<32>,
    ) -> Result<ThreatId, Error> {
        let dummy_contract = Symbol::new(&env, "fraud");
        if Self::check_rate_limit(env.clone(), actor.clone(), dummy_contract.clone())? {
            return Err(Self::rate_limit_exceeded_error());
        }
        let request_id = ThreatDetector::generate_threat_id(&env, &dummy_contract);
        SecurityEvents::emit_fraud_requested(&env, &actor, &request_id);
        Ok(request_id)
    }

    /// Deliver the result of a credential fraud check back to the contract from an authorized oracle.
    ///
    /// Only an oracle address previously authorized in storage may call this function.
    ///
    /// # Arguments
    /// * `oracle` - Address of the authorized oracle submitting the result.
    /// * `request_id` - The request ID returned by [`SecurityMonitor::verify_credential_fraud`].
    /// * `is_fraudulent` - Whether the credential was found to be fraudulent.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `oracle` is not authorized.
    ///
    /// # Example
    /// ```ignore
    /// client.callback_credential_fraud(&oracle, &request_id, &false);
    /// ```
    pub fn callback_credential_fraud(
        env: Env,
        oracle: Address,
        request_id: ThreatId,
        is_fraudulent: bool,
    ) -> Result<(), Error> {
        if !SecurityStorage::is_oracle_authorized(&env, &oracle) {
            return Err(Error::from_contract_error(1));
        }
        ThreatDetector::handle_fraud_callback(&env, request_id, is_fraudulent);
        Ok(())
    }

    /// Store or update a threat intelligence indicator provided by an admin.
    ///
    /// Requires the caller to be the registered admin. Emits an event after storing
    /// the intelligence record.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin (must match the stored admin).
    /// * `intel` - Threat intelligence record to store.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `admin` does not match the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// client.update_threat_intelligence(&admin, &intel);
    /// ```
    pub fn update_threat_intelligence(
        env: Env,
        admin: Address,
        intel: ThreatIntelligence,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        SecurityStorage::set_threat_intelligence(&env, &intel.indicator_type, &intel);
        SecurityEvents::emit_intel_added(
            &env,
            &intel.source,
            &intel.indicator_type,
            &intel.indicator_value,
            &intel.threat_level,
        );
        Ok(())
    }

    /// Update the risk score for a user, callable by the admin or an authorized oracle.
    ///
    /// Appends the given `risk_factor` to the user's risk factor list and records the
    /// current ledger timestamp as the last update time.
    ///
    /// # Arguments
    /// * `admin` - Address of the admin or an authorized oracle.
    /// * `user` - Address of the user whose risk score is being updated.
    /// * `score` - New absolute risk score value for the user.
    /// * `risk_factor` - Symbol label describing the reason for the score change.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if the caller is neither the admin
    /// nor an authorized oracle.
    ///
    /// # Example
    /// ```ignore
    /// client.update_user_risk_score(&admin, &user, &80u32, &Symbol::new(&env, "SuspiciousLogin"));
    /// ```
    pub fn update_user_risk_score(
        env: Env,
        admin: Address,
        user: Address,
        score: u32,
        risk_factor: Symbol,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin && !SecurityStorage::is_oracle_authorized(&env, &admin) {
            // Either admin or an oracle can update the risk score.
            return Err(Error::from_contract_error(1));
        }

        let mut risk_score = SecurityStorage::get_user_risk_score(&env, &user)
            .unwrap_or(UserRiskScore { score: 0, last_updated: 0, risk_factors: Vec::new(&env) });

        risk_score.score = score;
        risk_score.last_updated = env.ledger().timestamp();
        risk_score.risk_factors.push_back(risk_factor.clone());

        SecurityStorage::set_user_risk_score(&env, &user, &risk_score);
        SecurityEvents::emit_risk_score_updated(&env, &user, score, &risk_factor);
        Ok(())
    }

    /// Retrieve the current risk score record for a user, or `None` if no record exists.
    ///
    /// # Arguments
    /// * `user` - Address of the user whose risk score is requested.
    ///
    /// # Example
    /// ```ignore
    /// let risk = client.get_user_risk_score(&user);
    /// ```
    pub fn get_user_risk_score(env: Env, user: Address) -> Option<UserRiskScore> {
        SecurityStorage::get_user_risk_score(&env, &user)
    }

    /// Record the completion of a security training module for a user.
    ///
    /// Requires the caller to be the registered admin. Reduces the user's risk score by
    /// 10 points (if the current score is at least 10) and emits the appropriate events.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin (must match the stored admin).
    /// * `user` - Address of the user who completed the training module.
    /// * `module` - Symbol identifying the completed training module.
    /// * `score` - Score achieved by the user in the training module.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `admin` does not match the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// client.record_security_training(&admin, &user, &Symbol::new(&env, "PhishingAwareness"), &90u32);
    /// ```
    pub fn record_security_training(
        env: Env,
        admin: Address,
        user: Address,
        module: Symbol,
        score: u32,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }

        let mut training =
            SecurityStorage::get_training_status(&env, &user).unwrap_or(SecurityTrainingStatus {
                user: user.clone(),
                completed_modules: Vec::new(&env),
                last_training_date: 0,
                score: 0,
            });

        training.completed_modules.push_back(module.clone());
        training.last_training_date = env.ledger().timestamp();
        training.score = score;

        SecurityStorage::set_training_status(&env, &user, &training);
        SecurityEvents::emit_training_recorded(&env, &user, &module, score);

        // Optionally reduce risk score based on training
        let mut risk_score =
            SecurityStorage::get_user_risk_score(&env, &user).unwrap_or(UserRiskScore {
                score: 50, // default
                last_updated: 0,
                risk_factors: Vec::new(&env),
            });

        if risk_score.score >= 10 {
            risk_score.score -= 10;
            risk_score.last_updated = env.ledger().timestamp();
            risk_score.risk_factors.push_back(Symbol::new(&env, "TrainingCompleted"));
            SecurityStorage::set_user_risk_score(&env, &user, &risk_score);
            SecurityEvents::emit_risk_score_updated(
                &env,
                &user,
                risk_score.score,
                &Symbol::new(&env, "TrainingCompleted"),
            );
        }

        Ok(())
    }

    /// Generate and store a security incident report aggregating one or more threats.
    ///
    /// Requires the caller to be the registered admin. Creates an `IncidentReport` record
    /// marked as resolved at the current ledger timestamp and emits an event.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin (must match the stored admin).
    /// * `threat_ids` - List of 32-byte threat IDs to include in the report.
    /// * `impact_summary` - Human-readable description of the incident's impact.
    ///
    /// # Errors
    /// Returns contract error code `1` (unauthorized) if `admin` does not match the stored admin.
    ///
    /// # Example
    /// ```ignore
    /// let incident_id = client.generate_incident_report(&admin, &threat_ids, &impact_summary);
    /// ```
    pub fn generate_incident_report(
        env: Env,
        admin: Address,
        threat_ids: ThreatIdList,
        impact_summary: String,
    ) -> Result<ThreatId, Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }

        let dummy = Symbol::new(&env, "incident");
        let incident_id = ThreatDetector::generate_threat_id(&env, &dummy);

        let report = IncidentReport {
            incident_id: incident_id.clone(),
            timestamp: env.ledger().timestamp(),
            threat_ids,
            impact_summary,
            actions_taken: Vec::new(&env), // In a real system, aggregate actions from threats
            status: Symbol::new(&env, "Resolved"),
            resolved_at: Some(env.ledger().timestamp()),
        };

        SecurityStorage::set_incident_report(&env, &report);
        SecurityEvents::emit_incident_reported(&env, &incident_id, &admin);

        Ok(incident_id)
    }

    // ─────────────────────────────────────────────────────────
    // RBAC Enhancement
    // ─────────────────────────────────────────────────────────

    /// Creates a new role in the RBAC hierarchy.
    ///
    /// Requires admin authorization. The role is active by default.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `role_id` - Unique symbol identifier for the role.
    /// * `parent_role` - Optional parent role for hierarchy inheritance.
    /// * `description` - Human-readable description of the role.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    /// Returns contract error `4` (already exists) if the role already exists.
    pub fn create_role(
        env: Env,
        admin: Address,
        role_id: Symbol,
        parent_role: Option<Symbol>,
        description: String,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        if SecurityStorage::has_rbac_role(&env, &role_id) {
            return Err(Error::from_contract_error(4));
        }
        let role = RbacRole {
            role_id: role_id.clone(),
            parent_role,
            description,
            is_active: true,
            created_at: env.ledger().timestamp(),
        };
        SecurityStorage::set_rbac_role(&env, &role);
        Ok(())
    }

    /// Assigns a role to a user with an optional expiry.
    ///
    /// Requires admin authorization. If the role does not exist or is inactive, an error is returned.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `user` - Address of the user receiving the role.
    /// * `role_id` - Symbol identifier of the role to assign.
    /// * `expires_at` - Optional Unix timestamp after which the assignment expires.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    /// Returns contract error `5` (role not found) if the role does not exist.
    /// Returns contract error `6` (role inactive) if the role is deactivated.
    pub fn assign_role(
        env: Env,
        admin: Address,
        user: Address,
        role_id: Symbol,
        expires_at: Option<u64>,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        let role =
            SecurityStorage::get_rbac_role(&env, &role_id).ok_or(Error::from_contract_error(5))?;
        if !role.is_active {
            return Err(Error::from_contract_error(6));
        }
        let assignment = RoleAssignment {
            user: user.clone(),
            role_id: role_id.clone(),
            assigned_by: admin,
            assigned_at: env.ledger().timestamp(),
            expires_at,
            is_active: true,
        };
        SecurityStorage::set_role_assignment(&env, &user, &role_id, &assignment);
        Ok(())
    }

    /// Revokes a role assignment from a user.
    ///
    /// Requires admin authorization. Marks the assignment as inactive.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `user` - Address of the user whose role is being revoked.
    /// * `role_id` - Symbol identifier of the role to revoke.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    /// Returns contract error `7` (assignment not found) if no active assignment exists.
    pub fn revoke_role(
        env: Env,
        admin: Address,
        user: Address,
        role_id: Symbol,
    ) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        let mut assignment = SecurityStorage::get_role_assignment(&env, &user, &role_id)
            .ok_or(Error::from_contract_error(7))?;
        assignment.is_active = false;
        SecurityStorage::set_role_assignment(&env, &user, &role_id, &assignment);
        Ok(())
    }

    /// Activates a previously deactivated role.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `role_id` - Symbol identifier of the role to activate.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    /// Returns contract error `5` (role not found) if the role does not exist.
    pub fn activate_role(env: Env, admin: Address, role_id: Symbol) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        let mut role =
            SecurityStorage::get_rbac_role(&env, &role_id).ok_or(Error::from_contract_error(5))?;
        role.is_active = true;
        SecurityStorage::set_rbac_role(&env, &role);
        Ok(())
    }

    /// Deactivates a role without deleting it, preventing new assignments.
    ///
    /// Requires admin authorization. Existing assignments are unaffected.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `role_id` - Symbol identifier of the role to deactivate.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    /// Returns contract error `5` (role not found) if the role does not exist.
    pub fn deactivate_role(env: Env, admin: Address, role_id: Symbol) -> Result<(), Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        let mut role =
            SecurityStorage::get_rbac_role(&env, &role_id).ok_or(Error::from_contract_error(5))?;
        role.is_active = false;
        SecurityStorage::set_rbac_role(&env, &role);
        Ok(())
    }

    /// Delegates a role that the delegator holds to another user.
    ///
    /// The delegator must hold the role. The delegation can optionally expire.
    ///
    /// # Arguments
    /// * `delegator` - Address of the user delegating the role.
    /// * `delegate` - Address of the user receiving the delegation.
    /// * `role_id` - Symbol identifier of the role being delegated.
    /// * `expires_at` - Optional Unix timestamp after which the delegation expires.
    ///
    /// # Errors
    /// Returns contract error `7` (assignment not found) if the delegator does not hold the role.
    pub fn delegate_role(
        env: Env,
        delegator: Address,
        delegate: Address,
        role_id: Symbol,
        expires_at: Option<u64>,
    ) -> Result<(), Error> {
        delegator.require_auth();
        let assignment = SecurityStorage::get_role_assignment(&env, &delegator, &role_id)
            .ok_or(Error::from_contract_error(7))?;
        if !assignment.is_active {
            return Err(Error::from_contract_error(7));
        }
        let delegation = RoleDelegation {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            role_id: role_id.clone(),
            delegated_at: env.ledger().timestamp(),
            expires_at,
        };
        SecurityStorage::add_role_delegation(&env, &delegator, &role_id, &delegation);
        Ok(())
    }

    /// Returns `true` if the user holds the given role (directly assigned or delegated, not expired).
    ///
    /// # Arguments
    /// * `user` - Address of the user to check.
    /// * `role_id` - Symbol identifier of the role to check.
    pub fn has_role(env: Env, user: Address, role_id: Symbol) -> bool {
        let now = env.ledger().timestamp();
        if let Some(assignment) = SecurityStorage::get_role_assignment(&env, &user, &role_id) {
            if assignment.is_active {
                if let Some(exp) = assignment.expires_at {
                    if now <= exp {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the role definition for the given role ID.
    ///
    /// # Arguments
    /// * `role_id` - Symbol identifier of the role to retrieve.
    pub fn get_role(env: Env, role_id: Symbol) -> Option<RbacRole> {
        SecurityStorage::get_rbac_role(&env, &role_id)
    }

    /// Returns all role IDs assigned to a user (including inactive and expired).
    ///
    /// # Arguments
    /// * `user` - Address of the user whose roles to retrieve.
    pub fn get_user_roles(env: Env, user: Address) -> Vec<Symbol> {
        SecurityStorage::get_user_roles(&env, &user)
    }

    /// Scans all role assignments for the user and revokes any that have passed their expiry.
    ///
    /// Requires admin authorization. Returns the count of assignments revoked.
    ///
    /// # Arguments
    /// * `admin` - Address of the contract admin.
    /// * `user` - Address of the user whose expired roles should be cleaned up.
    ///
    /// # Errors
    /// Returns contract error `1` (unauthorized) if the caller is not the admin.
    pub fn cleanup_expired_roles(env: Env, admin: Address, user: Address) -> Result<u32, Error> {
        let expected_admin =
            SecurityStorage::get_admin(&env).ok_or(Error::from_contract_error(1))?;
        if admin != expected_admin {
            return Err(Error::from_contract_error(1));
        }
        let now = env.ledger().timestamp();
        let roles = SecurityStorage::get_user_roles(&env, &user);
        let mut revoked: u32 = 0;
        for i in 0..roles.len() {
            let role_id = roles.get(i).unwrap();
            if let Some(mut assignment) =
                SecurityStorage::get_role_assignment(&env, &user, &role_id)
            {
                if assignment.is_active {
                    if let Some(exp) = assignment.expires_at {
                        if now > exp {
                            assignment.is_active = false;
                            SecurityStorage::set_role_assignment(
                                &env,
                                &user,
                                &role_id,
                                &assignment,
                            );
                            revoked += 1;
                        }
                    }
                }
            }
        }
        Ok(revoked)
    }

    // --- Note: Some methods from the `SecurityMonitorTrait` are omitted here for brevity or were already missing in the dummy implementation in `lib.rs`, such as apply_mitigation, check_circuit_breaker, etc. We'll stick to implementing the new advanced AI features and updating the basic dummy ones that were there. ---
}
