use soroban_sdk::{Env, Address, BytesN, Symbol};
use crate::events::CertificateEvent;

pub fn mint_certificate(env: Env, owner: Address, cert_id: BytesN<32>, metadata: String) {
    env.events().publish(
        (Symbol::short("CertificateMinted"), cert_id.clone()),
        CertificateEvent::CertificateMinted {
            cert_id,
            owner,
            issued_at: env.ledger().timestamp(),
            metadata,
        },
    );
}

pub fn revoke_certificate(env: Env, cert_id: BytesN<32>, reason: String, revoked_by: Address) {
    env.events().publish(
        (Symbol::short("CertificateRevoked"), cert_id.clone()),
        CertificateEvent::CertificateRevoked {
            cert_id,
            revoked_by,
            revoked_at: env.ledger().timestamp(),
            reason,
        },
    );
}

pub fn update_certificate(env: Env, cert_id: BytesN<32>, changes: String, updated_by: Address) {
    env.events().publish(
        (Symbol::short("CertificateUpdated"), cert_id.clone()),
        CertificateEvent::CertificateUpdated {
            cert_id,
            updated_by,
            updated_at: env.ledger().timestamp(),
            changes,
        },
    );
}
