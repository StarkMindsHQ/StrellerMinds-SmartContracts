use crate::types::{CertificateMetadata, Role};
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

/// Contract event emissions
pub struct CertificateEvents;

impl CertificateEvents {
    /// Emits event when contract is initialized
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `admin` - Address of the initial admin
    pub fn emit_contract_initialized(env: &Env, admin: &Address) {
        let topics = (Symbol::new(env, "contract_initialized"),);
        env.events().publish(topics, admin);
    }

    /// Emits event when a role is added
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    /// * `role` - The role that was added
    pub fn emit_role_added(env: &Env, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_added"), user);
        let data = (role.can_issue, role.can_revoke);
        env.events().publish(topics, data);
    }

    /// Emits event when a role is removed
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    pub fn emit_role_removed(env: &Env, user: &Address) {
        let topics = (Symbol::new(env, "role_removed"), user);
        env.events().publish(topics, ());
    }

    /// Emits event when a role is updated
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    /// * `new_role` - The updated role
    pub fn emit_role_updated(env: &Env, user: &Address, new_role: &Role) {
        let topics = (Symbol::new(env, "role_updated"), user);
        let data = (new_role.can_issue, new_role.can_revoke);
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is minted
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `metadata` - Certificate metadata
    /// * `student` - Address of the student
    /// * `issuer` - Address of the issuer
    /// * `token_id` - Unique token ID for the certificate
    pub fn emit_certificate_minted(
        env: &Env,
        certificate_id: &BytesN<32>,
        metadata: &CertificateMetadata,
        student: &Address,
        issuer: &Address,
        token_id: &BytesN<32>,
    ) {
        let topics = (Symbol::new(env, "nft_certificate_minted"), certificate_id);
        let data = (
            metadata.clone(),
            student.clone(),
            issuer.clone(),
            token_id.clone(),
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is revoked
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `metadata` - Certificate metadata
    /// * `revoker` - Address of the revoker
    /// * `timestamp` - Time of revocation
    pub fn emit_certificate_revoked(
        env: &Env,
        certificate_id: &BytesN<32>,
        metadata: &CertificateMetadata,
        revoker: &Address,
        timestamp: u64,
    ) {
        let topics = (Symbol::new(env, "nft_certificate_revoked"), certificate_id);
        let data = (metadata.clone(), revoker.clone(), timestamp);
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is transferred
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `from` - Address of the previous owner
    /// * `to` - Address of the new owner
    #[allow(dead_code)]
    pub fn emit_certificate_transferred(
        env: &Env,
        certificate_id: &BytesN<32>,
        from: &Address,
        to: &Address,
    ) {
        let topics = (Symbol::new(env, "certificate_transferred"), certificate_id);
        let data = (from, to);
        env.events().publish(topics, data);
    }

    /// Emits event when certificate metadata is updated
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `updater` - Address of the user who updated metadata
    /// * `old_uri` - Previous metadata URI
    /// * `new_uri` - New metadata URI
    pub fn emit_metadata_updated(
        env: &Env,
        certificate_id: &BytesN<32>,
        updater: &Address,
        old_uri: &String,
        new_uri: &String,
    ) {
        let topics = (Symbol::new(env, "metadata_updated"), certificate_id);
        let data = (
            updater.clone(),
            old_uri.clone(),
            new_uri.clone(),
            env.ledger().timestamp(),
        );
        env.events().publish(topics, data);
    }
}
