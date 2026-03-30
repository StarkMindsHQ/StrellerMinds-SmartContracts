#![no_std]

pub mod errors;

use crate::errors::StudentProgressError;
use shared::circuit_breaker::{
    ensure_call_allowed, force_reset_closed, record_failure, record_success, CircuitBreakerConfig,
    CircuitBreakerStatus, CircuitCheckError, CircuitTransition,
};
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, ProgressEventData, ProgressUpdatedEvent,
};
use shared::{emit_access_control_event, emit_progress_event};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Map, Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct Progress {
    module_id: Symbol,
    percent: u32,
}

#[contracttype]
enum DataKey {
    Progress(Address, Symbol), // (student, course_id)
    Admin,
    CircuitConfig,
    CircuitStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProgressError {
    NotInitialized = 1,
    InvalidPercentage = 2,
    CircuitBreakerOpen = 3,
    CircuitBreakerHalfOpenLimit = 4,
    InvalidCircuitConfiguration = 5,
    Unauthorized = 6,
}

#[contract]
pub struct ProgressTracker;

#[contractimpl]
impl ProgressTracker {
    /// Initializes the progress tracker and sets the admin address.
    ///
    /// Requires authorization from `admin`. Must be called once before any other function.
    ///
    /// # Arguments
    /// * `admin` - Address that will have administrative control.
    ///
    /// # Errors
    /// Returns [`StudentProgressError::AlreadyInitialized`] if called more than once.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), StudentProgressError> {
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CircuitConfig, &CircuitBreakerConfig::default());
        env.storage().instance().set(&DataKey::CircuitStatus, &CircuitBreakerStatus::new());
        emit_access_control_event!(
            &env,
            symbol_short!("progress"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Records or updates a student's module completion percentage for a course.
    ///
    /// Requires authorization from `student` (or admin if student is the admin).
    ///
    /// # Arguments
    /// * `student` - Address of the student whose progress is being updated.
    /// * `course_id` - Symbol identifier for the course.
    /// * `module_id` - Symbol identifier for the module within the course.
    /// * `percent` - Completion percentage, must be in the range 0–100.
    ///
    /// # Errors
    /// Returns [`StudentProgressError::InvalidPercent`] if `percent` is greater than 100.
    /// Returns [`StudentProgressError::AdminNotSet`] if the contract has not been initialized.
    ///
    /// # Example
    /// ```ignore
    /// client.update_progress(&student, &course_id, &module_id, &80u32);
    /// ```
    pub fn update_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
        module_id: Symbol,
        percent: u32,
    ) {
        let result =
            Self::update_progress_safe(env.clone(), student, course_id, module_id, percent);

        match result {
            Ok(()) => {}
            Err(ProgressError::InvalidPercentage) => {
                panic!("percentage cannot be more than 100");
            }
            Err(err) => {
                panic_with_error!(&env, err);
            }
        }
    }

    pub fn update_progress_safe(
        env: Env,
        student: Address,
        course_id: Symbol,
        module_id: Symbol,
        percent: u32,
    ) -> Result<(), ProgressError> {
        Self::require_initialized(&env)?;
        Self::ensure_operational(&env)?;

        if percent > 100 {
            return Err(ProgressError::InvalidPercentage);
        }

        let admin: Address =
            env.storage().instance().get(&DataKey::Admin).ok_or(ProgressError::NotInitialized)?;
        if student != admin {
            student.require_auth();
        } else {
            admin.require_auth();
        }

        let key = DataKey::Progress(student.clone(), course_id.clone());

        let mut progress_map: Map<Symbol, u32> =
            env.storage().persistent().get(&key).unwrap_or(Map::new(&env));

        progress_map.set(module_id.clone(), percent);
        env.storage().persistent().set(&key, &progress_map);

        emit_progress_event!(
            &env,
            symbol_short!("progress"),
            student.clone(),
            ProgressEventData::ProgressUpdated(ProgressUpdatedEvent {
                student,
                course_id,
                module_id,
                progress_percentage: percent,
            })
        );

        Self::record_operation_result(&env, true);
        Ok(())
    }

    pub fn configure_circuit_breaker(
        env: Env,
        admin: Address,
        failure_threshold: u32,
        recovery_timeout_seconds: u64,
        half_open_max_calls: u32,
        half_open_success_threshold: u32,
    ) -> Result<(), ProgressError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if failure_threshold == 0
            || recovery_timeout_seconds == 0
            || half_open_max_calls == 0
            || half_open_success_threshold == 0
        {
            return Err(ProgressError::InvalidCircuitConfiguration);
        }

        let config = CircuitBreakerConfig {
            failure_threshold,
            recovery_timeout_seconds,
            half_open_max_calls,
            half_open_success_threshold,
        };

        env.storage().instance().set(&DataKey::CircuitConfig, &config);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("config")),
            (symbol_short!("progress"), failure_threshold, recovery_timeout_seconds),
        );

        Ok(())
    }

    pub fn reset_circuit_breaker(env: Env, admin: Address) -> Result<(), ProgressError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut status = Self::get_circuit_status_internal(&env);
        force_reset_closed(&mut status, env.ledger().timestamp());
        env.storage().instance().set(&DataKey::CircuitStatus, &status);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("reset")),
            (symbol_short!("progress"), admin),
        );

        Ok(())
    }

    pub fn report_operation_failure(env: Env, admin: Address) -> Result<(), ProgressError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::record_operation_result(&env, false);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("failure")),
            (Symbol::new(&env, "progress"), admin),
        );
        Ok(())
    }

    pub fn get_circuit_breaker_status(env: Env) -> CircuitBreakerStatus {
        Self::get_circuit_status_internal(&env)
    }

    /// Returns a map of module IDs to completion percentages for a student in a course.
    ///
    /// Returns an empty map if no progress has been recorded yet.
    ///
    /// # Arguments
    /// * `student` - Address of the student to query.
    /// * `course_id` - Symbol identifier for the course.
    ///
    /// # Example
    /// ```ignore
    /// let progress_map = client.get_progress(&student, &course_id);
    /// ```
    pub fn get_progress(env: Env, student: Address, course_id: Symbol) -> Map<Symbol, u32> {
        let key = DataKey::Progress(student, course_id);
        env.storage().persistent().get(&key).unwrap_or(Map::new(&env))
    }

    /// Returns the admin address stored during initialization.
    ///
    /// # Errors
    /// Returns [`StudentProgressError::AdminNotSet`] if the contract has not been initialized.
    ///
    /// # Example
    /// ```ignore
    /// let admin = client.get_admin();
    /// ```
    pub fn get_admin(env: Env) -> Result<Address, StudentProgressError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(StudentProgressError::AdminNotSet)
    }

    fn require_initialized(env: &Env) -> Result<(), ProgressError> {
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(ProgressError::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), ProgressError> {
        let admin: Address =
            env.storage().instance().get(&DataKey::Admin).ok_or(ProgressError::NotInitialized)?;

        if admin != *caller {
            return Err(ProgressError::Unauthorized);
        }

        Ok(())
    }

    fn get_circuit_config_internal(env: &Env) -> CircuitBreakerConfig {
        env.storage()
            .instance()
            .get(&DataKey::CircuitConfig)
            .unwrap_or(CircuitBreakerConfig::default())
    }

    fn get_circuit_status_internal(env: &Env) -> CircuitBreakerStatus {
        env.storage().instance().get(&DataKey::CircuitStatus).unwrap_or(CircuitBreakerStatus::new())
    }

    fn emit_circuit_transition(
        env: &Env,
        transition: CircuitTransition,
        status: &CircuitBreakerStatus,
    ) {
        let action = match transition {
            CircuitTransition::Opened => symbol_short!("open"),
            CircuitTransition::HalfOpened => symbol_short!("halfopen"),
            CircuitTransition::Closed => symbol_short!("closed"),
        };

        env.events().publish(
            (symbol_short!("circuit"), action),
            (
                Symbol::new(env, "progress"),
                status.total_failures,
                status.total_successes,
                status.alert_count,
            ),
        );
    }

    fn ensure_operational(env: &Env) -> Result<(), ProgressError> {
        let mut status = Self::get_circuit_status_internal(env);
        let config = Self::get_circuit_config_internal(env);
        let now = env.ledger().timestamp();

        match ensure_call_allowed(&mut status, &config, now) {
            Ok(transition) => {
                env.storage().instance().set(&DataKey::CircuitStatus, &status);
                if let Some(state_change) = transition {
                    Self::emit_circuit_transition(env, state_change, &status);
                }
                Ok(())
            }
            Err(CircuitCheckError::Open) => {
                env.events().publish(
                    (symbol_short!("circuit"), symbol_short!("blocked")),
                    (Symbol::new(env, "progress"), Symbol::new(env, "open")),
                );
                Err(ProgressError::CircuitBreakerOpen)
            }
            Err(CircuitCheckError::HalfOpenCapacityReached) => {
                env.events().publish(
                    (symbol_short!("circuit"), symbol_short!("blocked")),
                    (Symbol::new(env, "progress"), Symbol::new(env, "halfopen")),
                );
                Err(ProgressError::CircuitBreakerHalfOpenLimit)
            }
        }
    }

    fn record_operation_result(env: &Env, success: bool) {
        let mut status = Self::get_circuit_status_internal(env);
        let config = Self::get_circuit_config_internal(env);
        let now = env.ledger().timestamp();

        let transition = if success {
            record_success(&mut status, &config, now)
        } else {
            record_failure(&mut status, &config, now)
        };

        env.storage().instance().set(&DataKey::CircuitStatus, &status);
        if let Some(state_change) = transition {
            Self::emit_circuit_transition(env, state_change, &status);
        }
    }
}

pub mod gas_optimized;
#[cfg(test)]
mod test;
