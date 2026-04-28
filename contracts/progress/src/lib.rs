#![no_std]
extern crate alloc;

pub mod errors;

use crate::errors::ProgressError;
use alloc::string::ToString;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, ProgressEventData, ProgressUpdatedEvent,
};
use shared::gdpr_types::GdprProgressExport;
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use shared::{emit_access_control_event, emit_progress_event};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec};

/// Storage key for progress records.
#[contracttype]
#[derive(Clone)]
pub enum ProgressKey {
    /// Stores the progress percentage for (student, course_id).
    Progress(Address, Symbol),
    /// Stores the list of course IDs a student has recorded progress in.
    StudentCourses(Address),
    /// Rate limit state for a student's record_progress calls.
    RateLimit(Address),
}

/// Rate limit: max 100 progress updates per day per student.
const RATE_LIMIT_CFG: RateLimitConfig = RateLimitConfig { max_calls: 100, window_seconds: 86_400 };

#[contract]
pub struct Progress;

#[contractimpl]
impl Progress {
    /// Initializes the progress contract and records the admin address.
    ///
    /// # Arguments
    /// * `admin` - Address that will have administrative control over the contract.
    ///
    /// # Errors
    /// Returns [`ProgressError::AlreadyInitialized`] if the contract has already been initialized.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ProgressError> {
        admin.require_auth();
        if env.storage().instance().has(&soroban_sdk::symbol_short!("admin")) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&soroban_sdk::symbol_short!("admin"), &admin);

        emit_access_control_event!(
            &env,
            symbol_short!("progress"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Records a student's progress percentage for a given course.
    ///
    /// Enforces a per-student rate limit (100 calls/day) to prevent abuse.
    /// Stores the progress value on-chain and tracks the course in the student's course list.
    /// Emits a `ProgressUpdated` event on success.
    ///
    /// # Arguments
    /// * `student` - Address of the student whose progress is being recorded.
    /// * `course_id` - Symbol identifier for the course.
    /// * `progress` - Progress percentage (0–100).
    ///
    /// # Errors
    /// Returns [`ProgressError::Unauthorized`] if the caller is not authorized.
    /// Returns [`ProgressError::InvalidProgress`] if `progress` exceeds 100.
    pub fn record_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
        progress: u32,
    ) -> Result<(), ProgressError> {
        student.require_auth();

        if progress > 100 {
            return Err(ProgressError::InvalidProgress);
        }

        // Enforce per-student rate limit (#363)
        let rl_key = ProgressKey::RateLimit(student.clone());
        enforce_rate_limit(&env, &rl_key, &RATE_LIMIT_CFG)
            .map_err(|_| ProgressError::Unauthorized)?;

        // Store progress (#365)
        let progress_key = ProgressKey::Progress(student.clone(), course_id.clone());
        env.storage().persistent().set(&progress_key, &progress);

        // Track course in student's course list if not already present (#365)
        let courses_key = ProgressKey::StudentCourses(student.clone());
        let mut courses: Vec<Symbol> =
            env.storage().persistent().get(&courses_key).unwrap_or_else(|| Vec::new(&env));
        let already_tracked = courses.iter().any(|c| c == course_id);
        if !already_tracked {
            courses.push_back(course_id.clone());
            env.storage().persistent().set(&courses_key, &courses);
        }

        emit_progress_event!(
            &env,
            symbol_short!("progress"),
            student.clone(),
            ProgressEventData::ProgressUpdated(ProgressUpdatedEvent {
                student,
                course_id,
                module_id: symbol_short!("record"),
                progress_percentage: progress,
            })
        );
        Ok(())
    }

    /// Returns the recorded progress percentage for a student in a given course.
    ///
    /// # Arguments
    /// * `student` - Address of the student to query.
    /// * `course_id` - Symbol identifier for the course.
    ///
    /// # Errors
    /// Returns [`ProgressError::ProgressNotFound`] if no progress has been recorded.
    pub fn get_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<u32, ProgressError> {
        let key = ProgressKey::Progress(student, course_id);
        env.storage().persistent().get(&key).ok_or(ProgressError::ProgressNotFound)
    }

    /// Returns all course IDs in which the student has recorded progress.
    ///
    /// # Arguments
    /// * `student` - Address of the student to query.
    pub fn get_student_courses(env: Env, student: Address) -> Vec<Symbol> {
        let key = ProgressKey::StudentCourses(student);
        env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(&env))
    }

    pub fn export_user_data(env: Env, user: Address) -> Vec<GdprProgressExport> {
        let courses = Self::get_student_courses(env.clone(), user.clone());
        let mut exports: Vec<GdprProgressExport> = Vec::new(&env);

        for i in 0..courses.len() {
            if let Some(course_id) = courses.get(i) {
                if let Ok(progress) = Self::get_progress(env.clone(), user.clone(), course_id.clone()) {
                    let mut bytes = [0u8; 32];
                    bytes[31] = (i as u8).wrapping_mul(3);
                    let mut j: usize = 0;
                    let course_str = course_id.to_string();
                    for (idx, b) in course_str.as_bytes().iter().enumerate() {
                        if idx < 31 {
                            bytes[idx] = *b;
                            j = idx + 1;
                        }
                    }
                    bytes[31] = bytes[31].wrapping_add(j as u8);
                    exports.push_back(GdprProgressExport {
                        course_id: BytesN::from_array(&env, &bytes),
                        progress_percentage: progress,
                        last_updated: env.ledger().timestamp(),
                    });
                }
            }
        }

        exports
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&symbol_short!("admin"));
        let report = Monitor::build_health_report(&env, symbol_short!("progress"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}
pub mod gas_optimized;

#[cfg(test)]
pub mod tests;
