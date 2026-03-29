use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, ProgressEventData, ProgressUpdatedEvent,
};
use shared::{emit_access_control_event, emit_progress_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Error, Symbol, Vec};

#[contract]
pub struct Progress;

#[contractimpl]
impl Progress {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
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

    pub fn record_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
        progress: u32,
    ) -> Result<(), Error> {
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

    pub fn get_progress(_env: Env, _student: Address, _course_id: Symbol) -> Result<u32, Error> {
        Ok(0)
    }

    pub fn get_student_courses(_env: Env, _student: Address) -> Vec<Symbol> {
        Vec::new(&_env)
    }
}
pub mod gas_optimized;

#[cfg(test)]
pub mod tests;
