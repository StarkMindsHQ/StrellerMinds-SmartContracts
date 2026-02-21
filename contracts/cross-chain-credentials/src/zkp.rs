use soroban_sdk::{Env, String, Bytes, format};

pub struct ZKProof {
    pub commitment: String,
    pub challenge: String,
    pub response: String,
}

pub fn generate_commitment(env: &Env, secret: &String) -> String {
    let bytes = Bytes::from(secret.clone());
    let hash = env.crypto().sha256(&bytes);
    String::from_bytes(env, &hash.into())
}

pub fn verify_zkp(env: &Env, proof: &ZKProof, public_input: &String) -> bool {
    let commitment_bytes = Bytes::from(proof.commitment.clone());
    let challenge_bytes = Bytes::from(proof.challenge.clone());
    let response_bytes = Bytes::from(proof.response.clone());
    
    let mut combined = Bytes::new(env);
    combined.append(&commitment_bytes);
    combined.append(&challenge_bytes);
    combined.append(&response_bytes);
    
    let verification_hash = env.crypto().sha256(&combined);
    let expected_hash = env.crypto().sha256(&Bytes::from(public_input.clone()));
    
    verification_hash == expected_hash
}

pub fn create_privacy_proof(env: &Env, credential_data: &String) -> ZKProof {
    let commitment = generate_commitment(env, credential_data);
    let challenge = String::from_str(env, "challenge");
    let combined = format!(&env, "{}-{}", credential_data, challenge);
    let response = generate_commitment(env, &combined);
    
    ZKProof {
        commitment,
        challenge,
        response,
    }
}
