use super::*;
use crate::errors::OutcomeError;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

fn setup() -> (Env, OutcomeTrackerClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(OutcomeTracker, ());
    let client = OutcomeTrackerClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

fn init(env: &Env, client: &OutcomeTrackerClient, admin: &Address) {
    env.mock_auths(&[MockAuth {
        address: admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(admin);
}

#[test]
fn test_initialize() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_double_initialize_fails() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);
    // Second init should return AlreadyInitialized error
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(OutcomeError::AlreadyInitialized)));
}

#[test]
fn test_record_and_get_outcome() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);

    let student_id = symbol_short!("STU001");

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "record_outcome",
            args: (
                student_id.clone(),
                EmploymentStatus::Employed,
                1u32,
                8u32,
                symbol_short!("promoted"),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.record_outcome(
        &student_id,
        &EmploymentStatus::Employed,
        &1u32,
        &8u32,
        &symbol_short!("promoted"),
    );

    let outcome = client.get_outcome(&student_id);
    assert_eq!(outcome.employment_status, EmploymentStatus::Employed);
    assert_eq!(outcome.salary_range, 1u32);
    assert_eq!(outcome.satisfaction_score, 8u32);
}

#[test]
fn test_invalid_satisfaction_score() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);

    let student_id = symbol_short!("STU002");

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "record_outcome",
            args: (
                student_id.clone(),
                EmploymentStatus::Employed,
                1u32,
                11u32, // invalid: > 10
                symbol_short!("none"),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_record_outcome(
        &student_id,
        &EmploymentStatus::Employed,
        &1u32,
        &11u32,
        &symbol_short!("none"),
    );
    assert_eq!(
        result,
        Err(Ok(OutcomeError::InvalidSatisfactionScore))
    );
}

#[test]
fn test_invalid_salary_range() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);

    let student_id = symbol_short!("STU003");

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "record_outcome",
            args: (
                student_id.clone(),
                EmploymentStatus::Employed,
                5u32, // invalid: > 3 and != u32::MAX
                8u32,
                symbol_short!("none"),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_record_outcome(
        &student_id,
        &EmploymentStatus::Employed,
        &5u32,
        &8u32,
        &symbol_short!("none"),
    );
    assert_eq!(result, Err(Ok(OutcomeError::InvalidSalary)));
}

#[test]
fn test_outcome_not_found() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);

    let result = client.try_get_outcome(&symbol_short!("NOONE"));
    assert_eq!(result, Err(Ok(OutcomeError::OutcomeNotFound)));
}

#[test]
fn test_undisclosed_salary() {
    let (env, client, admin) = setup();
    init(&env, &client, &admin);

    let student_id = symbol_short!("STU004");

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "record_outcome",
            args: (
                student_id.clone(),
                EmploymentStatus::SelfEmployed,
                u32::MAX, // undisclosed
                7u32,
                symbol_short!("startup"),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.record_outcome(
        &student_id,
        &EmploymentStatus::SelfEmployed,
        &u32::MAX,
        &7u32,
        &symbol_short!("startup"),
    );

    let outcome = client.get_outcome(&student_id);
    assert_eq!(outcome.salary_range, u32::MAX);
}
