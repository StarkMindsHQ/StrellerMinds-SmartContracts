use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
}

#[test]
fn test_register_deployment() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    env.mock_all_auths();

    client.initialize(&admin);
    client.register_deployment(&admin, &target, &wasm_hash, &1);
}

#[test]
fn test_register_compressed_deployment() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let wasm_hash = BytesN::from_array(&env, &[2u8; 32]);
    env.mock_all_auths();

    client.initialize(&admin);
    client.register_compressed_deployment(&admin, &target, &wasm_hash, &1, &5000, &10000);
}

#[test]
fn test_deploy_incremental() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let delta_hash = BytesN::from_array(&env, &[3u8; 32]);
    env.mock_all_auths();

    client.initialize(&admin);
    client.deploy_incremental(&admin, &target, &delta_hash, &1);
}

#[test]
fn test_offline_deployment() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
    let bundle_hash = client.prepare_offline_deployment(&admin, &target);
    client.confirm_offline_deployment(&admin, &bundle_hash);
}

#[test]
fn test_rollback_deployment() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
    client.rollback_deployment(&admin, &target);
}

#[test]
fn test_verify_deployment() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let target = Address::generate(&env);

    let verified = client.verify_deployment(&target);
    assert!(verified);
}

#[test]
fn test_deployment_history() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let target = Address::generate(&env);

    let history = client.get_deployment_history(&target);
    assert_eq!(history.len(), 0);
}

#[test]
fn test_bandwidth_usage() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let target = Address::generate(&env);

    let usage = client.get_bandwidth_usage(&target);
    assert_eq!(usage, 0);
}

#[test]
fn test_estimate_deployment_size() {
    let env = Env::default();
    let contract_id = env.register(MobileOptimizerContract, ());
    let client = MobileOptimizerContractClient::new(&env, &contract_id);
    let wasm_hash = BytesN::from_array(&env, &[4u8; 32]);

    let size = client.estimate_deployment_size(&wasm_hash);
    assert_eq!(size, 0);
}
