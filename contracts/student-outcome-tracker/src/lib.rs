#![no_std]

pub mod errors;

use crate::errors::OutcomeError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::emit_access_control_event;
use shared::gas_optimizer::{TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Employment status of a graduate.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EmploymentStatus {
    Employed,
    Unemployed,
    SelfEmployed,
    FurtherStudy,
    Unknown,
}

/// Privacy-preserving outcome record for a graduate.
/// Salary is stored as a range bucket (e.g. 0=<30k, 1=30-60k, 2=60-100k, 3=100k+)
/// to avoid exposing exact figures on-chain.
#[derive(Clone)]
#[contracttype]
pub struct StudentOutcome {
    /// Hashed/anonymised student identifier (caller provides their own hash)
    pub student_id: Symbol,
    /// Employment status post-graduation
    pub employment_status: EmploymentStatus,
    /// Salary range bucket (0–3); u32::MAX means not disclosed
    pub salary_range: u32,
    /// Job satisfaction score 1–10
    pub satisfaction_score: u32,
    /// Free-form impact metric tag (e.g. "promoted", "startup_founded")
    pub impact_tag: Symbol,
    /// Ledger timestamp of last update
    pub updated_at: u64,
}

#[contracttype]
enum DataKey {
    Admin,
    Outcome(Symbol), // keyed by student_id symbol
}

#[contract]
pub struct OutcomeTracker;

#[contractimpl]
impl OutcomeTracker {
    /// Initialise the contract and set the admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), OutcomeError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(OutcomeError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .extend_ttl(TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
        emit_access_control_event!(
            &env,
            symbol_short!("outcome"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Record or update a student's post-graduation outcome.
    ///
    /// Only the admin may submit outcome data to protect student privacy.
    ///
    /// # Arguments
    /// * `student_id`        – Anonymised identifier (e.g. hash of student address).
    /// * `employment_status` – Current employment status.
    /// * `salary_range`      – Salary bucket 0–3, or u32::MAX for undisclosed.
    /// * `satisfaction_score`– Job satisfaction 1–10.
    /// * `impact_tag`        – Short impact descriptor symbol.
    pub fn record_outcome(
        env: Env,
        student_id: Symbol,
        employment_status: EmploymentStatus,
        salary_range: u32,
        satisfaction_score: u32,
        impact_tag: Symbol,
    ) -> Result<(), OutcomeError> {
        // salary_range: 0-3 or u32::MAX (undisclosed)
        if salary_range > 3 && salary_range != u32::MAX {
            return Err(OutcomeError::InvalidSalary);
        }
        if satisfaction_score < 1 || satisfaction_score > 10 {
            return Err(OutcomeError::InvalidSatisfactionScore);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OutcomeError::AdminNotSet)?;
        admin.require_auth();

        let outcome = StudentOutcome {
            student_id: student_id.clone(),
            employment_status,
            salary_range,
            satisfaction_score,
            impact_tag,
            updated_at: env.ledger().timestamp(),
        };

        let key = DataKey::Outcome(student_id);
        env.storage().persistent().set(&key, &outcome);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
        env.storage()
            .instance()
            .extend_ttl(TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);

        env.events().publish(
            (symbol_short!("outcome"), symbol_short!("recorded")),
            outcome.student_id,
        );

        Ok(())
    }

    /// Retrieve a student's outcome record.
    pub fn get_outcome(
        env: Env,
        student_id: Symbol,
    ) -> Result<StudentOutcome, OutcomeError> {
        let key = DataKey::Outcome(student_id);
        let outcome: StudentOutcome = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(OutcomeError::OutcomeNotFound)?;
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
        env.storage()
            .instance()
            .extend_ttl(TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
        Ok(outcome)
    }

    /// Return the admin address.
    pub fn get_admin(env: Env) -> Result<Address, OutcomeError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(OutcomeError::AdminNotSet)
    }

    /// Health check for monitoring.
    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let report = Monitor::build_health_report(&env, symbol_short!("outcome"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}

#[cfg(test)]
mod test;
