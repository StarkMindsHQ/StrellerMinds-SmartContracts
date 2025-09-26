#![cfg(test)]

use crate::{SimpleCertificateParams, GasTestCertificate, GasTestCertificateTrait, SimpleCertificateError};
use shared::gas_testing::{GasTester, StandardThresholds};
use soroban_sdk::{
    testutils::Address as _,
    Address, BytesN, Env, String, Vec,
};

fn setup_contract() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    // Initialize the contract
    GasTestCertificate::initialize(&env, admin.clone()).unwrap();
    
    (env, admin, issuer)
}

fn create_test_params(env: &Env, issuer: &Address, index: u32) -> SimpleCertificateParams {
    SimpleCertificateParams {
        certificate_id: BytesN::from_array(env, &[index; 32]),
        student: Address::generate(env),
        course_id: String::from_str(env, "COURSE_001"),
        title: String::from_str(env, "Test Certificate"),
        description: String::from_str(env, "Test Description"),
        metadata_uri: String::from_str(env, "https://example.com/metadata"),
        expiry_date: env.ledger().timestamp() + 86400, // 1 day from now
    }
}

#[test]
fn test_single_certificate_mint_gas_regression() {
    let (env, _admin, issuer) = setup_contract();
    let threshold = StandardThresholds::simple_storage_operation(&env);
    
    let params = create_test_params(&env, &issuer, 1);
    
    let (_result, measurement) = GasTester::measure_gas(&env, "single_certificate_mint", || {
        GasTestCertificate::mint_certificate(&env, issuer.clone(), params.clone())
            .map_err(|_| shared::errors::AccessControlError::PermissionDenied)
    }).unwrap();
    
    // Validate gas usage is within threshold
    let validation_result = GasTester::validate_against_threshold(&measurement, &threshold);
    assert!(validation_result.passed, "Gas regression detected in single certificate mint");
    
    // Verify the operation was successful
    assert_eq!(GasTestCertificate::get_certificate_count(&env), 1);
}

#[test]
fn test_batch_certificate_mint_gas_regression() {
    let (env, _admin, issuer) = setup_contract();
    let threshold = StandardThresholds::batch_operation(&env);
    
    // Create a batch of 5 certificates
    let mut params_list = Vec::new(&env);
    for i in 1..=5 {
        params_list.push_back(create_test_params(&env, &issuer, i));
    }
    
    let (_result, measurement) = GasTester::measure_gas(&env, "batch_certificate_mint", || {
        GasTestCertificate::mint_certificates_batch(&env, issuer.clone(), params_list.clone())
            .map_err(|_| shared::errors::AccessControlError::PermissionDenied)
    }).unwrap();
    
    // Validate gas usage is within threshold
    let validation_result = GasTester::validate_against_threshold(&measurement, &threshold);
    assert!(validation_result.passed, "Gas regression detected in batch certificate mint");
    
    // Verify the operation was successful
    assert_eq!(GasTestCertificate::get_certificate_count(&env), 5);
}

#[test]
fn test_storage_operations_gas_regression() {
    let (env, _admin, issuer) = setup_contract();
    let threshold = StandardThresholds::simple_storage_operation(&env);
    
    let (_result, measurement) = GasTester::measure_gas(&env, "storage_operations", || {
        // Perform some storage operations
        let count = GasTestCertificate::get_certificate_count(&env);
        assert_eq!(count, 0);
        Ok::<(), shared::errors::AccessControlError>(())
    }).unwrap();
    
    // Validate gas usage is within threshold
    let validation_result = GasTester::validate_against_threshold(&measurement, &threshold);
    assert!(validation_result.passed, "Gas regression detected in storage operations");
}

#[test]
fn test_contract_initialization_gas_regression() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let threshold = StandardThresholds::simple_storage_operation(&env);
    
    let (_result, measurement) = GasTester::measure_gas(&env, "contract_initialization", || {
        GasTestCertificate::initialize(&env, admin.clone())
            .map_err(|_| shared::errors::AccessControlError::PermissionDenied)
    }).unwrap();
    
    // Validate gas usage is within threshold
    let validation_result = GasTester::validate_against_threshold(&measurement, &threshold);
    assert!(validation_result.passed, "Gas regression detected in contract initialization");
}