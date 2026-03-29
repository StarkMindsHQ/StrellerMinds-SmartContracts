#![no_std]

pub mod errors;

use crate::errors::StudentProgressError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, ProgressEventData, ProgressUpdatedEvent,
};
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

    pub fn get_progress(env: Env, student: Address, course_id: Symbol) -> Map<Symbol, u32> {
        let key = DataKey::Progress(student, course_id);
        env.storage().persistent().get(&key).unwrap_or(Map::new(&env))
    }

    pub fn get_admin(env: Env) -> Result<Address, StudentProgressError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(StudentProgressError::AdminNotSet)
    }
}

pub mod gas_optimized;
#[cfg(test)]
mod test;
