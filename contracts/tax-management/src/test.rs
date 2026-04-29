#![cfg(test)]

use crate::errors::TaxError;
use crate::types::{DocumentType, TaxAdvisor, TaxDocument};
use crate::{TaxManagement, TaxManagementClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String, Vec,
};

const VALID_IPFS_HASH: &str = "QmZ4tDuvesekSs4qM5ZBKpXiZGun7S2CYtEZRB3DYXkjGx";
const VALID_IPFS_HASH_2: &str = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

fn setup() -> (Env, TaxManagementClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TaxManagement, ());
    let client = TaxManagementClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn jurisdictions(env: &Env, codes: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new(env);
    for c in codes {
        v.push_back(String::from_str(env, c));
    }
    v
}

#[test]
fn initialize_succeeds_once() {
    let (_, client, admin) = setup();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(TaxError::AlreadyInitialized)));
}

#[test]
fn upload_document_assigns_incrementing_ids_and_indexes() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let property = String::from_str(&env, "PROP-001");

    let id1 = client.upload_document(
        &owner,
        &property,
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );
    let id2 = client.upload_document(
        &owner,
        &property,
        &DocumentType::PaymentReceipt,
        &String::from_str(&env, VALID_IPFS_HASH_2),
        &2024u32,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    let owner_docs = client.get_documents_by_owner(&owner);
    assert_eq!(owner_docs.len(), 2);
    let property_docs = client.get_documents_by_property(&property);
    assert_eq!(property_docs.len(), 2);

    let doc: TaxDocument = client.get_document(&id1).unwrap();
    assert_eq!(doc.doc_type, DocumentType::TaxReturn);
    assert_eq!(doc.tax_year, 2024);
    assert!(!doc.verified);
}

#[test]
fn upload_document_rejects_invalid_ipfs_hash() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let result = client.try_upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, "short"),
        &2024u32,
    );
    assert_eq!(result, Err(Ok(TaxError::InvalidIpfsHash)));
}

#[test]
fn upload_document_rejects_invalid_year() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let result = client.try_upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &1800u32,
    );
    assert_eq!(result, Err(Ok(TaxError::InvalidTaxYear)));
}

#[test]
fn upload_document_rejects_empty_property_id() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let result = client.try_upload_document(
        &owner,
        &String::from_str(&env, ""),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );
    assert_eq!(result, Err(Ok(TaxError::InvalidPropertyId)));
}

#[test]
fn verify_ipfs_hash_compares_stored_hash() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id = client.upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );

    assert!(client.verify_ipfs_hash(&id, &String::from_str(&env, VALID_IPFS_HASH)));
    assert!(!client.verify_ipfs_hash(&id, &String::from_str(&env, VALID_IPFS_HASH_2)));
    assert!(!client.verify_ipfs_hash(&999u64, &String::from_str(&env, VALID_IPFS_HASH)));
}

#[test]
fn register_and_fetch_advisor() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme Tax LLC"),
        &String::from_str(&env, "LIC-12345"),
        &jurisdictions(&env, &["US-CA", "US-NY"]),
    );

    let record: TaxAdvisor = client.get_advisor(&advisor_addr).unwrap();
    assert_eq!(record.license_id, String::from_str(&env, "LIC-12345"));
    assert_eq!(record.jurisdictions.len(), 2);
    assert!(record.active);
}

#[test]
fn register_advisor_rejects_non_admin() {
    let (env, client, _) = setup();
    let intruder = Address::generate(&env);
    let advisor_addr = Address::generate(&env);

    let result = client.try_register_advisor(
        &intruder,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    assert_eq!(result, Err(Ok(TaxError::Unauthorized)));
}

#[test]
fn register_advisor_rejects_empty_jurisdictions() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);

    let result = client.try_register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &Vec::<String>::new(&env),
    );
    assert_eq!(result, Err(Ok(TaxError::NoJurisdictions)));
}

#[test]
fn register_advisor_rejects_duplicate() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);
    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    let result = client.try_register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    assert_eq!(result, Err(Ok(TaxError::AdvisorAlreadyRegistered)));
}

#[test]
fn deactivated_advisor_cannot_verify() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);
    let owner = Address::generate(&env);

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    let id = client.upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );
    client.deactivate_advisor(&admin, &advisor_addr);

    let result = client.try_verify_document(&advisor_addr, &id);
    assert_eq!(result, Err(Ok(TaxError::AdvisorInactive)));
}

#[test]
fn verify_document_marks_verified_with_advisor() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);
    let owner = Address::generate(&env);

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    let id = client.upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );

    client.verify_document(&advisor_addr, &id);
    let doc = client.get_document(&id).unwrap();
    assert!(doc.verified);
    assert_eq!(doc.verifier, Some(advisor_addr.clone()));

    let again = client.try_verify_document(&advisor_addr, &id);
    assert_eq!(again, Err(Ok(TaxError::DocumentAlreadyVerified)));
}

#[test]
fn verify_document_rejects_unregistered_advisor() {
    let (env, client, _) = setup();
    let stranger = Address::generate(&env);
    let owner = Address::generate(&env);
    let id = client.upload_document(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &DocumentType::TaxReturn,
        &String::from_str(&env, VALID_IPFS_HASH),
        &2024u32,
    );
    let result = client.try_verify_document(&stranger, &id);
    assert_eq!(result, Err(Ok(TaxError::AdvisorNotFound)));
}

#[test]
fn assign_and_unassign_property_advisor() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);
    let owner = Address::generate(&env);
    let property = String::from_str(&env, "PROP-001");

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    client.assign_advisor_to_property(&owner, &property, &advisor_addr);
    assert_eq!(client.get_property_advisor(&property), Some(advisor_addr.clone()));

    client.unassign_property_advisor(&owner, &property);
    assert_eq!(client.get_property_advisor(&property), None);

    let result = client.try_unassign_property_advisor(&owner, &property);
    assert_eq!(result, Err(Ok(TaxError::AdvisorNotAssigned)));
}

#[test]
fn assign_rejects_inactive_advisor() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);
    let owner = Address::generate(&env);

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    client.deactivate_advisor(&admin, &advisor_addr);

    let result = client.try_assign_advisor_to_property(
        &owner,
        &String::from_str(&env, "PROP-001"),
        &advisor_addr,
    );
    assert_eq!(result, Err(Ok(TaxError::AdvisorInactive)));
}

#[test]
fn update_advisor_jurisdictions_replaces_list() {
    let (env, client, admin) = setup();
    let advisor_addr = Address::generate(&env);

    client.register_advisor(
        &admin,
        &advisor_addr,
        &String::from_str(&env, "Acme"),
        &String::from_str(&env, "LIC-1"),
        &jurisdictions(&env, &["US-CA"]),
    );
    client.update_advisor_jurisdictions(
        &admin,
        &advisor_addr,
        &jurisdictions(&env, &["US-CA", "US-WA", "US-OR"]),
    );

    let record = client.get_advisor(&advisor_addr).unwrap();
    assert_eq!(record.jurisdictions.len(), 3);
}
