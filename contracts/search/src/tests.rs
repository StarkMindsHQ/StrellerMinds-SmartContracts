#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_test_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AdvancedSearchContract);

    (env, admin, contract_id)
}

#[test]
fn test_initialize() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);

    // Initialize contract
    let result = client.try_initialize(&admin);
    assert!(result.is_ok());

    // Try to initialize again - should fail
    let result2 = client.try_initialize(&admin);
    assert!(result2.is_err());
}
