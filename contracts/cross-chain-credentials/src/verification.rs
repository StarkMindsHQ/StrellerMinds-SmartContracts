use soroban_sdk::{Env, String, Bytes, format};
use crate::types::{ChainId, Credential, CrossChainProof, OracleAttestation};
use crate::storage::DataKey;

pub fn generate_proof(env: &Env, credential: &Credential, target_chain: &ChainId) -> CrossChainProof {
    let proof_data = format!(
        &env,
        "{}-{}-{}-{}",
        credential.id,
        credential.student,
        credential.issued_at,
        credential.metadata_hash
    );
    
    let proof_hash = hash_proof(env, &proof_data);
    
    CrossChainProof {
        credential_id: credential.id.clone(),
        source_chain: credential.chain_id.clone(),
        target_chain: target_chain.clone(),
        proof_hash,
        verified_at: env.ledger().timestamp(),
    }
}

fn hash_proof(env: &Env, data: &String) -> String {
    let bytes = Bytes::from(data.clone());
    let hash = env.crypto().sha256(&bytes);
    String::from_bytes(env, &hash.into())
}

pub fn store_attestation(env: &Env, attestation: &OracleAttestation) {
    let key = format!(&env, "ATT-{}-{}", attestation.credential_id, attestation.oracle);
    env.storage().persistent().set(&DataKey::Proof(key), attestation);
}

pub fn verify_attestations(_env: &Env, _credential_id: &String, min_attestations: u32) -> bool {
    let valid_count = 0u32;
    valid_count >= min_attestations
}
