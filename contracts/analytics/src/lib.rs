use shared::event_schema::{
    AccessControlEventData, AnalyticsEventData, ContractInitializedEvent, SessionCompletedEvent,
    SessionRecordedEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::{emit_access_control_event, emit_analytics_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Error};

#[contract]
pub struct Analytics;

#[contractimpl]
impl Analytics {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        emit_access_control_event!(
            &env,
            symbol_short!("analytics"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    pub fn record_session(env: Env, user: Address, session_id: BytesN<32>) -> Result<(), Error> {
        user.require_auth();
        emit_analytics_event!(
            &env,
            symbol_short!("analytics"),
            user.clone(),
            AnalyticsEventData::SessionRecorded(SessionRecordedEvent { session_id })
        );
        Ok(())
    }

    pub fn complete_session(env: Env, user: Address, session_id: BytesN<32>) -> Result<(), Error> {
        user.require_auth();
        emit_analytics_event!(
            &env,
            symbol_short!("analytics"),
            user.clone(),
            AnalyticsEventData::SessionCompleted(SessionCompletedEvent { session_id })
        );
        Ok(())
    }

    pub fn get_session(_env: Env, session_id: BytesN<32>) -> Option<BytesN<32>> {
        Some(session_id)
    }

    pub fn get_admin(_env: Env) -> Option<Address> {
        None
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&symbol_short!("admin"));
        let report = Monitor::build_health_report(&env, symbol_short!("analytics"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}
pub mod gas_optimized;
