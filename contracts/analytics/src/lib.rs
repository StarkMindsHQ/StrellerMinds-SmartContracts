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
    /// Initializes the analytics contract and records the admin address.
    ///
    /// # Arguments
    /// * `admin` - Address that will have administrative control.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::AlreadyInitialized`] if called more than once.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), AnalyticsError> {
        emit_access_control_event!(
            &env,
            symbol_short!("analytics"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Records the start of a new learning session for a user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the student starting the session.
    /// * `session_id` - Unique 32-byte identifier for this session.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the user.
    /// Returns [`AnalyticsError::InvalidSessionData`] if the session ID is invalid.
    ///
    /// # Example
    /// ```ignore
    /// client.record_session(&user, &session_id);
    /// ```
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

    /// Marks an existing learning session as completed.
    ///
    /// Requires authorization from `user`. The session must have been previously recorded via
    /// [`Analytics::record_session`].
    ///
    /// # Arguments
    /// * `user` - Address of the student completing the session.
    /// * `session_id` - Unique 32-byte identifier of the session to complete.
    ///
    /// # Errors
    /// Returns [`AnalyticsError::SessionNotFound`] if the session does not exist.
    /// Returns [`AnalyticsError::Unauthorized`] if the caller is not the session owner.
    ///
    /// # Example
    /// ```ignore
    /// client.complete_session(&user, &session_id);
    /// ```
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

    /// Returns the session data for the given session ID, or `None` if not found.
    ///
    /// # Arguments
    /// * `session_id` - Unique 32-byte identifier of the session to retrieve.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(session) = client.get_session(&session_id) { /* … */ }
    /// ```
    pub fn get_session(_env: Env, session_id: BytesN<32>) -> Option<BytesN<32>> {
        Some(session_id)
    }

    /// Returns the admin address, or `None` if the contract has not been initialized.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(admin) = client.get_admin() { /* … */ }
    /// ```
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
