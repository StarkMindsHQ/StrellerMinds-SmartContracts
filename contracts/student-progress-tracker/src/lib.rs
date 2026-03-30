#![no_std]

pub mod errors;

use crate::errors::StudentProgressError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, ProgressEventData, ProgressUpdatedEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::{emit_access_control_event, emit_progress_event};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol};

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
    ) -> Result<(), StudentProgressError> {
        if percent > 100 {
            return Err(StudentProgressError::InvalidPercent);
        }
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(StudentProgressError::AdminNotSet)?;
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
        Ok(())
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

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let report = Monitor::build_health_report(&env, symbol_short!("stracker"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}

pub mod gas_optimized;
#[cfg(test)]
mod test;
