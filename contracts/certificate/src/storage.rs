use crate::types::{CertificateMetadata, DataKey, Role};
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Storage operations for the Certificate contract
pub struct CertificateStorage;

impl CertificateStorage {
    /// Sets the contract admin
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `admin` - Address to set as admin
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    /// Retrieves the current admin
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    ///
    /// # Returns
    /// * `Address` - Current admin address
    pub fn get_admin(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Marks the contract as initialized
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// Checks if the contract is initialized
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    ///
    /// # Returns
    /// * `bool` - True if initialized, false otherwise
    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Sets a role for a user
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    /// * `role` - Role to set
    pub fn set_role(env: &Env, user: &Address, role: &Role) {
        let key = DataKey::Role(user.clone());
        env.storage().instance().set(&key, role);
    }

    /// Gets a role for a user
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    ///
    /// # Returns
    /// * `Option<Role>` - User's role if found
    pub fn get_role(env: &Env, user: &Address) -> Option<Role> {
        let key = DataKey::Role(user.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key)
        } else {
            None
        }
    }

    /// Removes a role for a user
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    pub fn remove_role(env: &Env, user: &Address) {
        let key = DataKey::Role(user.clone());
        env.storage().instance().remove(&key);
    }

    /// Stores certificate metadata
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Certificate identifier
    /// * `metadata` - Certificate metadata
    pub fn set_certificate(env: &Env, certificate_id: &BytesN<32>, metadata: &CertificateMetadata) {
        let key = DataKey::Certificates(certificate_id.clone());
        env.storage().instance().set(&key, metadata);
    }

    /// Retrieves certificate metadata
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Certificate identifier
    ///
    /// # Returns
    /// * `Option<CertificateMetadata>` - Certificate metadata if found
    pub fn get_certificate(env: &Env, certificate_id: &BytesN<32>) -> Option<CertificateMetadata> {
        let key = DataKey::Certificates(certificate_id.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key)
        } else {
            None
        }
    }

    /// Checks if a certificate exists
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Certificate identifier
    ///
    /// # Returns
    /// * `bool` - True if certificate exists, false otherwise
    pub fn has_certificate(env: &Env, certificate_id: &BytesN<32>) -> bool {
        let key = DataKey::Certificates(certificate_id.clone());
        env.storage().instance().has(&key)
    }

    /// Gets all certificates owned by a user
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    ///
    /// # Returns
    /// * `Vec<BytesN<32>>` - Vector of certificate IDs owned by the user
    pub fn get_user_certificates(env: &Env, user: &Address) -> Vec<BytesN<32>> {
        let key = DataKey::UserCerts(user.clone());
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Sets the certificates owned by a user
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    /// * `certificate_ids` - Vector of certificate IDs owned by the user
    pub fn set_user_certificates(env: &Env, user: &Address, certificate_ids: &Vec<BytesN<32>>) {
        let key = DataKey::UserCerts(user.clone());
        env.storage().instance().set(&key, certificate_ids);
    }

    /// Adds a certificate to a user's owned certificates
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    /// * `certificate_id` - Certificate ID to add
    pub fn add_user_certificate(env: &Env, user: &Address, certificate_id: &BytesN<32>) {
        let mut certificates = Self::get_user_certificates(env, user);

        // Only add if not already present
        for existing_id in certificates.iter() {
            if &existing_id == certificate_id {
                return;
            }
        }

        certificates.push_back(certificate_id.clone());
        Self::set_user_certificates(env, user, &certificates);
    }

    /// Removes a certificate from a user's owned certificates
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - User address
    /// * `certificate_id` - Certificate ID to remove
    pub fn remove_user_certificate(env: &Env, user: &Address, certificate_id: &BytesN<32>) {
        let mut certificates = Self::get_user_certificates(env, user);
        let mut index_to_remove = None;

        for (i, existing_id) in certificates.iter().enumerate() {
            if &existing_id == certificate_id {
                index_to_remove = Some(i);
                break;
            }
        }
        if let Some(index) = index_to_remove {
            certificates.remove(index as u32);
            Self::set_user_certificates(env, user, &certificates);
        }
    }
}
