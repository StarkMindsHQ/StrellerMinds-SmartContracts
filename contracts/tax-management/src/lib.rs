#![no_std]

pub mod errors;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::TaxError;
use crate::types::{DataKey, DocumentType, TaxAdvisor, TaxDocument};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

const MIN_TAX_YEAR: u32 = 1900;
const MAX_TAX_YEAR: u32 = 2200;
const MIN_IPFS_HASH_LEN: u32 = 16;

#[contract]
pub struct TaxManagement;

#[contractimpl]
impl TaxManagement {
    /// Initialize the contract with an admin address. Idempotent guard.
    pub fn initialize(env: Env, admin: Address) -> Result<(), TaxError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(TaxError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::DocumentCounter, &0u64);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    // Tax document storage on IPFS
    // ─────────────────────────────────────────────────────────────

    /// Upload a tax document referenced by its IPFS hash. Returns the new document id.
    pub fn upload_document(
        env: Env,
        owner: Address,
        property_id: String,
        doc_type: DocumentType,
        ipfs_hash: String,
        tax_year: u32,
    ) -> Result<u64, TaxError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        if property_id.len() == 0 {
            return Err(TaxError::InvalidPropertyId);
        }
        if ipfs_hash.len() < MIN_IPFS_HASH_LEN {
            return Err(TaxError::InvalidIpfsHash);
        }
        if tax_year < MIN_TAX_YEAR || tax_year > MAX_TAX_YEAR {
            return Err(TaxError::InvalidTaxYear);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DocumentCounter)
            .unwrap_or(0);
        let next_id = id + 1;

        let document = TaxDocument {
            id: next_id,
            owner: owner.clone(),
            property_id: property_id.clone(),
            doc_type,
            ipfs_hash,
            tax_year,
            uploaded_at: env.ledger().timestamp(),
            verified: false,
            verifier: None,
            verified_at: 0,
        };

        env.storage().persistent().set(&DataKey::Document(next_id), &document);
        env.storage().instance().set(&DataKey::DocumentCounter, &next_id);

        let mut owner_docs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::OwnerDocuments(owner.clone()))
            .unwrap_or(Vec::new(&env));
        owner_docs.push_back(next_id);
        env.storage()
            .persistent()
            .set(&DataKey::OwnerDocuments(owner), &owner_docs);

        let mut property_docs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PropertyDocuments(property_id.clone()))
            .unwrap_or(Vec::new(&env));
        property_docs.push_back(next_id);
        env.storage()
            .persistent()
            .set(&DataKey::PropertyDocuments(property_id), &property_docs);

        Ok(next_id)
    }

    /// Verify a previously uploaded tax document. Only an active registered advisor
    /// may verify; verification is one-shot.
    pub fn verify_document(env: Env, advisor: Address, document_id: u64) -> Result<(), TaxError> {
        Self::require_initialized(&env)?;
        advisor.require_auth();

        let advisor_record: TaxAdvisor = env
            .storage()
            .persistent()
            .get(&DataKey::Advisor(advisor.clone()))
            .ok_or(TaxError::AdvisorNotFound)?;
        if !advisor_record.active {
            return Err(TaxError::AdvisorInactive);
        }

        let mut document: TaxDocument = env
            .storage()
            .persistent()
            .get(&DataKey::Document(document_id))
            .ok_or(TaxError::DocumentNotFound)?;
        if document.verified {
            return Err(TaxError::DocumentAlreadyVerified);
        }

        document.verified = true;
        document.verifier = Some(advisor);
        document.verified_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Document(document_id), &document);
        Ok(())
    }

    /// Retrieve a tax document by id.
    pub fn get_document(env: Env, document_id: u64) -> Option<TaxDocument> {
        env.storage().persistent().get(&DataKey::Document(document_id))
    }

    /// Confirm that the on-chain IPFS hash for `document_id` matches the supplied one.
    pub fn verify_ipfs_hash(env: Env, document_id: u64, ipfs_hash: String) -> bool {
        match env
            .storage()
            .persistent()
            .get::<DataKey, TaxDocument>(&DataKey::Document(document_id))
        {
            Some(doc) => doc.ipfs_hash == ipfs_hash,
            None => false,
        }
    }

    pub fn get_documents_by_owner(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::OwnerDocuments(owner))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_documents_by_property(env: Env, property_id: String) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PropertyDocuments(property_id))
            .unwrap_or(Vec::new(&env))
    }

    // ─────────────────────────────────────────────────────────────
    // Tax advisor integration
    // ─────────────────────────────────────────────────────────────

    /// Register a new tax advisor. Admin-only.
    pub fn register_advisor(
        env: Env,
        admin: Address,
        advisor: Address,
        name: String,
        license_id: String,
        jurisdictions: Vec<String>,
    ) -> Result<(), TaxError> {
        Self::require_admin(&env, &admin)?;

        if license_id.len() == 0 {
            return Err(TaxError::InvalidLicense);
        }
        if jurisdictions.is_empty() {
            return Err(TaxError::NoJurisdictions);
        }
        if env.storage().persistent().has(&DataKey::Advisor(advisor.clone())) {
            return Err(TaxError::AdvisorAlreadyRegistered);
        }

        let record = TaxAdvisor {
            address: advisor.clone(),
            name,
            license_id,
            jurisdictions,
            active: true,
            registered_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&DataKey::Advisor(advisor), &record);
        Ok(())
    }

    /// Update the jurisdictions an advisor is licensed for. Admin-only.
    pub fn update_advisor_jurisdictions(
        env: Env,
        admin: Address,
        advisor: Address,
        jurisdictions: Vec<String>,
    ) -> Result<(), TaxError> {
        Self::require_admin(&env, &admin)?;
        if jurisdictions.is_empty() {
            return Err(TaxError::NoJurisdictions);
        }
        let mut record: TaxAdvisor = env
            .storage()
            .persistent()
            .get(&DataKey::Advisor(advisor.clone()))
            .ok_or(TaxError::AdvisorNotFound)?;
        record.jurisdictions = jurisdictions;
        env.storage().persistent().set(&DataKey::Advisor(advisor), &record);
        Ok(())
    }

    /// Deactivate a registered advisor. Admin-only.
    pub fn deactivate_advisor(
        env: Env,
        admin: Address,
        advisor: Address,
    ) -> Result<(), TaxError> {
        Self::require_admin(&env, &admin)?;
        let mut record: TaxAdvisor = env
            .storage()
            .persistent()
            .get(&DataKey::Advisor(advisor.clone()))
            .ok_or(TaxError::AdvisorNotFound)?;
        record.active = false;
        env.storage().persistent().set(&DataKey::Advisor(advisor), &record);
        Ok(())
    }

    pub fn get_advisor(env: Env, advisor: Address) -> Option<TaxAdvisor> {
        env.storage().persistent().get(&DataKey::Advisor(advisor))
    }

    /// Owner-authorized assignment of a registered advisor to a property.
    pub fn assign_advisor_to_property(
        env: Env,
        owner: Address,
        property_id: String,
        advisor: Address,
    ) -> Result<(), TaxError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        if property_id.len() == 0 {
            return Err(TaxError::InvalidPropertyId);
        }
        let record: TaxAdvisor = env
            .storage()
            .persistent()
            .get(&DataKey::Advisor(advisor.clone()))
            .ok_or(TaxError::AdvisorNotFound)?;
        if !record.active {
            return Err(TaxError::AdvisorInactive);
        }
        env.storage()
            .persistent()
            .set(&DataKey::PropertyAdvisor(property_id), &advisor);
        Ok(())
    }

    pub fn unassign_property_advisor(
        env: Env,
        owner: Address,
        property_id: String,
    ) -> Result<(), TaxError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        let key = DataKey::PropertyAdvisor(property_id);
        if !env.storage().persistent().has(&key) {
            return Err(TaxError::AdvisorNotAssigned);
        }
        env.storage().persistent().remove(&key);
        Ok(())
    }

    pub fn get_property_advisor(env: Env, property_id: String) -> Option<Address> {
        env.storage().persistent().get(&DataKey::PropertyAdvisor(property_id))
    }

    // ─────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────

    fn require_initialized(env: &Env) -> Result<(), TaxError> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(TaxError::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), TaxError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(TaxError::NotInitialized)?;
        if &admin != caller {
            return Err(TaxError::Unauthorized);
        }
        caller.require_auth();
        Ok(())
    }
}
