use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use types::{ChainId, CredentialStatus};

#[test]
fn test_initialize() {
    let env = Env::default();
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
