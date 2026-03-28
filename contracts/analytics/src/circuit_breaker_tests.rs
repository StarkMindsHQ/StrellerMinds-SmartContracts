#![cfg(test)]

extern crate std;

use crate::errors::AnalyticsError;
use crate::{Analytics, AnalyticsClient};
use shared::circuit_breaker::CircuitState;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env, Error as SorobanError};

fn setup() -> (Env, AnalyticsClient<'static>, Address, Address) {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 10_000);
    env.mock_all_auths();

    let contract_id = env.register(Analytics, ());
    let client = AnalyticsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    (env, client, admin, user)
}

#[test]
fn circuit_opens_after_repeated_failures() {
    let (_env, client, admin, _user) = setup();

    client.initialize(&admin);
    client.configure_circuit_breaker(&admin, &2, &60, &1, &1);

    client.report_operation_failure(&admin);
    client.report_operation_failure(&admin);

    let status = client.get_circuit_breaker_status();
    assert_eq!(status.state, CircuitState::Open);
    assert_eq!(status.consecutive_failures, 2);
}

#[test]
fn open_circuit_blocks_requests_until_reset_or_timeout() {
    let (env, client, admin, user) = setup();

    client.initialize(&admin);
    client.configure_circuit_breaker(&admin, &1, &60, &1, &1);

    client.report_operation_failure(&admin);

    let another_session = BytesN::from_array(&env, &[10u8; 32]);
    let blocked = client.try_record_session(&user, &another_session);
    assert_eq!(
        blocked,
        Err(Ok(SorobanError::from_contract_error(AnalyticsError::CircuitBreakerOpen as u32)))
    );

    client.reset_circuit_breaker(&admin);
    let recovered = client.try_record_session(&user, &another_session);
    assert!(recovered.is_ok());
}

#[test]
fn timeout_moves_open_to_half_open_then_closed_on_success() {
    let (env, client, admin, user) = setup();

    client.initialize(&admin);
    client.configure_circuit_breaker(&admin, &1, &30, &1, &1);

    client.report_operation_failure(&admin);

    env.ledger().with_mut(|li| li.timestamp += 31);

    let session_id = BytesN::from_array(&env, &[12u8; 32]);
    let result = client.try_record_session(&user, &session_id);
    assert!(result.is_ok());

    let status = client.get_circuit_breaker_status();
    assert_eq!(status.state, CircuitState::Closed);
}
