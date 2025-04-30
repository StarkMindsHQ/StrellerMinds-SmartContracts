#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Env, BytesN, Address};
use super::{CertificateContract, CertificateContractClient};
use crate::certificate::{CertificateData, CertificateType};

#[test]
fn test_basic_contract_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Create admin, issuer, and owner addresses
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let owner = Address::generate(&env);
    let max_batch_size = 10u32;

    // Deploy contract
    #[allow(deprecated)]
    let contract_id = env.register_contract(None, CertificateContract);
    let client = CertificateContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &max_batch_size);

    // Add issuer
    env.mock_all_auths();
    client.add_issuer(&admin, &issuer);

    // Verify issuer was added
    assert!(client.is_issuer(&issuer));

    // Create certificate data
    let cert_id = 1u64;
    let metadata_hash = BytesN::from_array(&env, &[1; 32]);
    let certificate = CertificateData {
        id: cert_id,
        metadata_hash: metadata_hash.clone(),
        valid_from: env.ledger().timestamp(),
        valid_until: env.ledger().timestamp() + 86400,
        revocable: true,
        cert_type: CertificateType::Standard,
    };

    // Mint certificate
    env.mock_all_auths();
    client.mint_single_certificate(&issuer, &owner, &certificate);

    // Verify certificate exists
    let stored_cert = client.get_certificate(&cert_id);
    assert!(stored_cert.is_some());
    
    let stored_cert = stored_cert.unwrap();
    assert_eq!(stored_cert.id, cert_id);
    assert_eq!(stored_cert.metadata_hash, metadata_hash);
}
