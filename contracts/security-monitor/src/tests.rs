#![cfg(test)]

use crate::{SecurityMonitor, SecurityMonitorClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};
use crate::types::{SecurityConfig, ThreatType};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SecurityMonitor);
    let client = SecurityMonitorClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = SecurityConfig::default_config();

    let result = client.initialize(&admin, &config);
    assert!(result.is_ok());
}

#[test]
fn test_get_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SecurityMonitor);
    let client = SecurityMonitorClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = SecurityConfig::default_config();

    client.initialize(&admin, &config);

    let retrieved_config = client.get_config();
    assert_eq!(retrieved_config.burst_detection_threshold, 100);
}

#[test]
fn test_scan_for_threats() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SecurityMonitor);
    let client = SecurityMonitorClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let config = SecurityConfig::default_config();

    client.initialize(&admin, &config);

    let contract = Symbol::new(&env, "test_contract");
    let threats = client.scan_for_threats(&contract, &60);

    // Should not error even with no events
    assert!(threats.len() >= 0);
}
