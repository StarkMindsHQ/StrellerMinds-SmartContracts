#![no_std]

pub mod errors;

use crate::errors::AnalyticsError;
use shared::event_schema::{
    AccessControlEventData, AnalyticsEventData, ContractInitializedEvent, SessionCompletedEvent,
    SessionRecordedEvent,
};
use shared::{emit_access_control_event, emit_analytics_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env};

#[contract]
pub struct Analytics;

#[contractimpl]
impl Analytics {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AnalyticsError> {
        emit_access_control_event!(
            &env,
            symbol_short!("analytics"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    pub fn record_session(
        env: Env,
        user: Address,
        session_id: BytesN<32>,
    ) -> Result<(), AnalyticsError> {
        user.require_auth();
        emit_analytics_event!(
            &env,
            symbol_short!("analytics"),
            user.clone(),
            AnalyticsEventData::SessionRecorded(SessionRecordedEvent { session_id })
        );
        Ok(())
    }

    pub fn complete_session(
        env: Env,
        user: Address,
        session_id: BytesN<32>,
    ) -> Result<(), AnalyticsError> {
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
}
pub mod gas_optimized;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AnalyticsError;
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    fn setup() -> (Env, AnalyticsClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(Analytics, ());
        let client = AnalyticsClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        (env, client, admin)
    }

    #[test]
    fn test_initialize_returns_analytics_error_type() {
        let (_, client, admin) = setup();
        // verify initialize succeeds and returns the correct error type
        let result = client.try_initialize(&admin);
        assert_eq!(result, Ok(Ok(())));
    }

    #[test]
    fn test_record_session_success() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let user = Address::generate(&env);
        let session_id = BytesN::from_array(&env, &[1u8; 32]);
        let result = client.try_record_session(&user, &session_id);
        assert_eq!(result, Ok(Ok(())));
    }

    #[test]
    fn test_complete_session_success() {
        let (env, client, admin) = setup();
        client.initialize(&admin);
        let user = Address::generate(&env);
        let session_id = BytesN::from_array(&env, &[2u8; 32]);
        client.record_session(&user, &session_id);
        let result = client.try_complete_session(&user, &session_id);
        assert_eq!(result, Ok(Ok(())));
    }

    #[test]
    fn test_error_variants_are_distinct() {
        assert_ne!(AnalyticsError::NotInitialized, AnalyticsError::Unauthorized);
        assert_ne!(AnalyticsError::SessionNotFound, AnalyticsError::CourseNotFound);
        assert!(AnalyticsError::NotInitialized < AnalyticsError::Unauthorized);
    }
}
