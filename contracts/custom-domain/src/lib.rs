//! Custom Domain Support Contract (fixes #436)
//!
//! Allows institutions to register and manage custom domains for their
//! credential portals.  The contract stores domain configuration on-chain
//! (domain name, SSL status, DNS verification state, subdomain support) and
//! exposes admin-gated functions to register, verify, and remove domains.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Address, Env, String, Vec};

// ─────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────

/// Lifecycle state of a registered custom domain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainStatus {
    /// Domain has been registered but DNS has not been verified yet.
    Pending,
    /// DNS verification passed; domain is active.
    Active,
    /// Domain verification failed or was manually suspended.
    Suspended,
    /// Domain has been removed from the registry.
    Removed,
}

/// SSL certificate provisioning status for a domain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SslStatus {
    /// No SSL certificate has been provisioned yet.
    None,
    /// Certificate provisioning is in progress.
    Provisioning,
    /// A valid SSL certificate is active.
    Active,
    /// The SSL certificate has expired and needs renewal.
    Expired,
}

/// On-chain record for a registered custom domain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomDomain {
    /// The fully-qualified domain name (e.g. `credentials.university.edu`).
    pub domain: String,
    /// Institution address that owns this domain registration.
    pub owner: Address,
    /// Current lifecycle status.
    pub status: DomainStatus,
    /// SSL certificate status.
    pub ssl_status: SslStatus,
    /// Whether subdomain routing is enabled for this domain.
    pub subdomain_support: bool,
    /// Expected DNS TXT record value used for ownership verification.
    pub dns_verification_token: String,
    /// Unix timestamp when the domain was registered.
    pub registered_at: u64,
    /// Unix timestamp of the last status update.
    pub updated_at: u64,
}

/// Storage keys for the custom-domain contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainDataKey {
    Admin,
    Initialized,
    /// Domain record keyed by domain name string.
    Domain(String),
    /// List of all registered domain names.
    DomainList,
    /// List of domain names owned by a specific institution.
    OwnerDomains(Address),
}

/// Errors returned by the custom-domain contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DomainError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    DomainAlreadyRegistered = 4,
    DomainNotFound = 5,
    DomainNotPending = 6,
    InvalidDomain = 7,
}

// ─────────────────────────────────────────────────────────────
// Contract
// ─────────────────────────────────────────────────────────────

#[contract]
pub struct CustomDomainContract;

#[contractimpl]
impl CustomDomainContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), DomainError> {
        if env.storage().instance().has(&DomainDataKey::Initialized) {
            return Err(DomainError::AlreadyInitialized);
        }
        env.storage().instance().set(&DomainDataKey::Admin, &admin);
        env.storage().instance().set(&DomainDataKey::Initialized, &true);
        Ok(())
    }

    /// Register a new custom domain for an institution.
    ///
    /// The domain is created in `Pending` status.  The caller must call
    /// `verify_domain` once DNS has been configured to activate it.
    ///
    /// # Arguments
    /// * `caller`             - Must be the contract admin.
    /// * `owner`              - Institution address that will own the domain.
    /// * `domain`             - Fully-qualified domain name to register.
    /// * `subdomain_support`  - Whether to enable subdomain routing.
    /// * `verification_token` - DNS TXT record value for ownership proof.
    pub fn register_domain(
        env: Env,
        caller: Address,
        owner: Address,
        domain: String,
        subdomain_support: bool,
        verification_token: String,
    ) -> Result<(), DomainError> {
        Self::require_admin(&env, &caller)?;

        if domain.len() == 0 {
            return Err(DomainError::InvalidDomain);
        }

        let key = DomainDataKey::Domain(domain.clone());
        if env.storage().persistent().has(&key) {
            return Err(DomainError::DomainAlreadyRegistered);
        }

        let now = env.ledger().timestamp();
        let record = CustomDomain {
            domain: domain.clone(),
            owner: owner.clone(),
            status: DomainStatus::Pending,
            ssl_status: SslStatus::None,
            subdomain_support,
            dns_verification_token: verification_token,
            registered_at: now,
            updated_at: now,
        };

        env.storage().persistent().set(&key, &record);
        Self::add_to_domain_list(&env, &domain);
        Self::add_to_owner_domains(&env, &owner, &domain);

        Ok(())
    }

    /// Mark a pending domain as verified and activate it.
    ///
    /// In a production system the caller would supply proof that the DNS TXT
    /// record is live; here the admin attests to that fact on-chain.
    ///
    /// # Arguments
    /// * `caller` - Must be the contract admin.
    /// * `domain` - Domain name to verify.
    /// * `ssl`    - SSL status to record at activation time.
    pub fn verify_domain(
        env: Env,
        caller: Address,
        domain: String,
        ssl: SslStatus,
    ) -> Result<(), DomainError> {
        Self::require_admin(&env, &caller)?;

        let key = DomainDataKey::Domain(domain.clone());
        let mut record: CustomDomain =
            env.storage().persistent().get(&key).ok_or(DomainError::DomainNotFound)?;

        if record.status != DomainStatus::Pending {
            return Err(DomainError::DomainNotPending);
        }

        record.status = DomainStatus::Active;
        record.ssl_status = ssl;
        record.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &record);

        Ok(())
    }

    /// Update the SSL status of an active domain.
    ///
    /// # Arguments
    /// * `caller` - Must be the contract admin.
    /// * `domain` - Domain name to update.
    /// * `ssl`    - New SSL status.
    pub fn update_ssl_status(
        env: Env,
        caller: Address,
        domain: String,
        ssl: SslStatus,
    ) -> Result<(), DomainError> {
        Self::require_admin(&env, &caller)?;

        let key = DomainDataKey::Domain(domain.clone());
        let mut record: CustomDomain =
            env.storage().persistent().get(&key).ok_or(DomainError::DomainNotFound)?;

        record.ssl_status = ssl;
        record.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &record);

        Ok(())
    }

    /// Remove a domain registration and free its storage.
    ///
    /// # Arguments
    /// * `caller` - Must be the contract admin.
    /// * `domain` - Domain name to remove.
    pub fn remove_domain(env: Env, caller: Address, domain: String) -> Result<(), DomainError> {
        Self::require_admin(&env, &caller)?;

        let key = DomainDataKey::Domain(domain.clone());
        if !env.storage().persistent().has(&key) {
            return Err(DomainError::DomainNotFound);
        }

        env.storage().persistent().remove(&key);
        Self::remove_from_domain_list(&env, &domain);

        Ok(())
    }

    /// Retrieve a domain record by name.
    pub fn get_domain(env: Env, domain: String) -> Option<CustomDomain> {
        env.storage().persistent().get(&DomainDataKey::Domain(domain))
    }

    /// List all registered domain names.
    pub fn list_domains(env: Env) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DomainDataKey::DomainList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// List domain names owned by a specific institution.
    pub fn list_owner_domains(env: Env, owner: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DomainDataKey::OwnerDomains(owner))
            .unwrap_or_else(|| Vec::new(&env))
    }
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

impl CustomDomainContract {
    fn require_admin(env: &Env, caller: &Address) -> Result<(), DomainError> {
        if !env.storage().instance().has(&DomainDataKey::Initialized) {
            return Err(DomainError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&DomainDataKey::Admin).unwrap();
        if *caller != admin {
            return Err(DomainError::Unauthorized);
        }
        Ok(())
    }

    fn add_to_domain_list(env: &Env, domain: &String) {
        let mut list: Vec<String> = env
            .storage()
            .persistent()
            .get(&DomainDataKey::DomainList)
            .unwrap_or_else(|| Vec::new(env));
        list.push_back(domain.clone());
        env.storage().persistent().set(&DomainDataKey::DomainList, &list);
    }

    fn remove_from_domain_list(env: &Env, domain: &String) {
        let list: Vec<String> = env
            .storage()
            .persistent()
            .get(&DomainDataKey::DomainList)
            .unwrap_or_else(|| Vec::new(env));
        let mut new_list: Vec<String> = Vec::new(env);
        for d in list.iter() {
            if d != *domain {
                new_list.push_back(d);
            }
        }
        env.storage().persistent().set(&DomainDataKey::DomainList, &new_list);
    }

    fn add_to_owner_domains(env: &Env, owner: &Address, domain: &String) {
        let key = DomainDataKey::OwnerDomains(owner.clone());
        let mut list: Vec<String> =
            env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
        list.push_back(domain.clone());
        env.storage().persistent().set(&key, &list);
    }
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    fn setup() -> (Env, CustomDomainContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CustomDomainContract);
        let client = CustomDomainContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_register_and_verify_domain() {
        let (env, client, admin) = setup();
        let owner = Address::generate(&env);
        let domain = soroban_sdk::String::from_str(&env, "credentials.uni.edu");
        let token = soroban_sdk::String::from_str(&env, "verify-abc123");

        client.register_domain(&admin, &owner, &domain, &true, &token);

        let record = client.get_domain(&domain).unwrap();
        assert_eq!(record.status, DomainStatus::Pending);
        assert_eq!(record.subdomain_support, true);

        client.verify_domain(&admin, &domain, &SslStatus::Active);

        let record = client.get_domain(&domain).unwrap();
        assert_eq!(record.status, DomainStatus::Active);
        assert_eq!(record.ssl_status, SslStatus::Active);
    }

    #[test]
    fn test_remove_domain() {
        let (env, client, admin) = setup();
        let owner = Address::generate(&env);
        let domain = soroban_sdk::String::from_str(&env, "test.example.com");
        let token = soroban_sdk::String::from_str(&env, "tok");

        client.register_domain(&admin, &owner, &domain, &false, &token);
        client.remove_domain(&admin, &domain);

        assert!(client.get_domain(&domain).is_none());
    }

    #[test]
    fn test_duplicate_registration_fails() {
        let (env, client, admin) = setup();
        let owner = Address::generate(&env);
        let domain = soroban_sdk::String::from_str(&env, "dup.example.com");
        let token = soroban_sdk::String::from_str(&env, "tok");

        client.register_domain(&admin, &owner, &domain, &false, &token);
        let result = client.try_register_domain(&admin, &owner, &domain, &false, &token);
        assert!(result.is_err());
    }
}
