use super::*;
use crate::errors::CrossChainError;
use shared::monitoring::ContractHealthStatus;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use types::{ChainId, CredentialStatus};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert!(!client.is_oracle(&admin));
}

#[test]
fn test_issue_credential() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let achievement = String::from_str(&env, "Blockchain Fundamentals");
    let metadata = String::from_str(&env, "hash123");

    let cred_id = client.issue_credential(&student, &achievement, &metadata, &ChainId::Stellar);

    let credential = client.get_credential(&cred_id);
    assert_eq!(credential.student, student);
    assert_eq!(credential.achievement, achievement);
    assert_eq!(credential.status, CredentialStatus::Active);
}

#[test]
fn test_revoke_credential() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Test Achievement"),
        &String::from_str(&env, "hash"),
        &ChainId::Stellar,
    );

    client.revoke_credential(&cred_id);

    let credential = client.get_credential(&cred_id);
    assert_eq!(credential.status, CredentialStatus::Revoked);
}

#[test]
fn test_cross_chain_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "DeFi Mastery"),
        &String::from_str(&env, "hash456"),
        &ChainId::Stellar,
    );

    let proof = client.verify_cross_chain(&cred_id, &ChainId::Ethereum);

    assert_eq!(proof.credential_id, cred_id);
    assert_eq!(proof.source_chain, ChainId::Stellar);
    assert_eq!(proof.target_chain, ChainId::Ethereum);
}

#[test]
fn test_oracle_management() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    client.initialize(&admin);

    assert!(!client.is_oracle(&oracle));

    client.add_oracle(&oracle);
    assert!(client.is_oracle(&oracle));

    client.remove_oracle(&oracle);
    assert!(!client.is_oracle(&oracle));
}

#[test]
fn test_generate_transcript() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    client.issue_credential(
        &student,
        &String::from_str(&env, "Course 1"),
        &String::from_str(&env, "hash1"),
        &ChainId::Stellar,
    );

    client.issue_credential(
        &student,
        &String::from_str(&env, "Course 2"),
        &String::from_str(&env, "hash2"),
        &ChainId::Polygon,
    );

    let transcript = client.generate_transcript(&student);

    assert_eq!(transcript.student, student);
    assert_eq!(transcript.total_achievements, 2);
}

#[test]
fn test_verification_request() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let requester = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Achievement"),
        &String::from_str(&env, "hash"),
        &ChainId::Stellar,
    );

    let request_id = client.request_verification(&cred_id, &ChainId::Bsc, &requester);
    let request = client.get_verification_request(&request_id);

    assert_eq!(request.credential_id, cred_id);
    assert_eq!(request.chain_id, ChainId::Bsc);
    assert_eq!(request.requester, requester);
}

#[test]
fn test_health_check_before_init() {
    let env = Env::default();
    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let report = client.health_check();
    assert_eq!(report.status, ContractHealthStatus::Unknown);
    assert!(!report.initialized);
}

#[test]
fn test_health_check_after_init() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let report = client.health_check();
    assert_eq!(report.status, ContractHealthStatus::Healthy);
    assert!(report.initialized);
}

// ─── Error Scenario Tests ─────────────────────────────────────────────────────

#[test]
fn test_initialize_already_initialized_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(CrossChainError::AlreadyInitialized)));
}

#[test]
fn test_get_nonexistent_credential_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let nonexistent_id = String::from_str(&env, "NONEXISTENT");
    let result = client.try_get_credential(&nonexistent_id);
    assert_eq!(result, Err(Ok(CrossChainError::CredentialNotFound)));
}

#[test]
fn test_verify_revoked_credential_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Test Achievement"),
        &String::from_str(&env, "hash"),
        &ChainId::Stellar,
    );

    client.revoke_credential(&cred_id);

    let result = client.try_verify_cross_chain(&cred_id, &ChainId::Ethereum);
    assert_eq!(result, Err(Ok(CrossChainError::CredentialNotActive)));
}

#[test]
fn test_get_nonexistent_proof_returns_error() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_get_proof(&String::from_str(&env, "NONEXISTENT"));
    assert_eq!(result, Err(Ok(CrossChainError::ProofNotFound)));
}

#[test]
fn test_error_variants_are_ordered() {
    assert!(CrossChainError::AlreadyInitialized < CrossChainError::Unauthorized);
    assert!(CrossChainError::CredentialNotFound < CrossChainError::CredentialNotActive);
    assert_ne!(CrossChainError::CredentialNotFound, CrossChainError::ProofNotFound);
}

#[test]
fn test_arbitrum_chain_support() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Arbitrum DeFi"),
        &String::from_str(&env, "hash_arb"),
        &ChainId::Arbitrum,
    );

    let credential = client.get_credential(&cred_id);
    assert_eq!(credential.chain_id, ChainId::Arbitrum);
    assert_eq!(credential.status, CredentialStatus::Active);
}

#[test]
fn test_initiate_bridge_to_arbitrum() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Smart Contract Dev"),
        &String::from_str(&env, "hash_bridge"),
        &ChainId::Stellar,
    );

    let bridge_req = client.initiate_bridge(&cred_id, &ChainId::Arbitrum);
    assert_eq!(bridge_req.credential_id, cred_id);
    assert_eq!(bridge_req.source_chain, ChainId::Stellar);
    assert_eq!(bridge_req.target_chain, ChainId::Arbitrum);
    assert!(bridge_req.gas_estimate > 0);
}

#[test]
fn test_migrate_credential_to_arbitrum() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Layer2 Expert"),
        &String::from_str(&env, "hash_migrate"),
        &ChainId::Ethereum,
    );

    client.migrate_credential(&cred_id, &ChainId::Arbitrum);

    let updated_cred = client.get_credential(&cred_id);
    assert_eq!(updated_cred.chain_id, ChainId::Arbitrum);
}

#[test]
fn test_gas_estimate_per_chain() {
    let env = Env::default();
    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let eth_gas = client.estimate_bridge_gas(&ChainId::Ethereum);
    let arb_gas = client.estimate_bridge_gas(&ChainId::Arbitrum);
    let poly_gas = client.estimate_bridge_gas(&ChainId::Polygon);

    assert!(arb_gas < eth_gas, "Arbitrum should be cheaper than Ethereum");
    assert!(poly_gas < eth_gas, "Polygon should be cheaper than Ethereum");
    assert!(arb_gas > 0);
}

#[test]
fn test_bridge_revoked_credential_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrossChainCredentials, ());
    let client = CrossChainCredentialsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    client.initialize(&admin);

    let cred_id = client.issue_credential(
        &student,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "hash"),
        &ChainId::Stellar,
    );

    client.revoke_credential(&cred_id);

    let result = client.try_initiate_bridge(&cred_id, &ChainId::Arbitrum);
    assert_eq!(result, Err(Ok(CrossChainError::CredentialNotActive)));
}
