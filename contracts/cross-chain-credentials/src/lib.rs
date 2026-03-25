#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

mod storage;
mod types;

use storage::{get_admin, is_oracle, DataKey};
use types::{
    ChainId, Credential, CredentialStatus, CrossChainProof, Transcript, VerificationRequest,
};
const CIRCUIT_FAILURE_THRESHOLD: u32 = 3;
const CIRCUIT_RESET_TIMEOUT_SECONDS: u64 = 300;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
struct CircuitState {
    state: BreakerState,
    failures: u32,
    opened_at: u64,
}

fn key_for_operation(env: &Env, operation: &str) -> DataKey {
    DataKey::CircuitState(String::from_str(env, operation))
}

fn get_or_init_circuit(env: &Env, operation: &str) -> CircuitState {
    env.storage()
        .instance()
        .get(&key_for_operation(env, operation))
        .unwrap_or(CircuitState {
            state: BreakerState::Closed,
            failures: 0,
            opened_at: 0,
        })
}

fn can_proceed(env: &Env, operation: &str) -> bool {
    let mut state = get_or_init_circuit(env, operation);
    match state.state {
        BreakerState::Closed => true,
        BreakerState::Open => {
            if env.ledger().timestamp() >= state.opened_at + CIRCUIT_RESET_TIMEOUT_SECONDS {
                state.state = BreakerState::HalfOpen;
                env.storage()
                    .instance()
                    .set(&key_for_operation(env, operation), &state);
                true
            } else {
                false
            }
        }
        BreakerState::HalfOpen => true,
    }
}

fn record_success(env: &Env, operation: &str) {
    let state = CircuitState {
        state: BreakerState::Closed,
        failures: 0,
        opened_at: 0,
    };
    env.storage()
        .instance()
        .set(&key_for_operation(env, operation), &state);
    env.events().publish(
        (Symbol::new(env, "circuit_closed"), Symbol::new(env, operation)),
        env.ledger().timestamp(),
    );
}

fn record_failure(env: &Env, operation: &str, reason: &str) {
    let mut state = get_or_init_circuit(env, operation);
    state.failures += 1;
    if matches!(state.state, BreakerState::HalfOpen) || state.failures >= CIRCUIT_FAILURE_THRESHOLD {
        state.state = BreakerState::Open;
        state.opened_at = env.ledger().timestamp();
        env.events().publish(
            (
                Symbol::new(env, "circuit_opened"),
                Symbol::new(env, operation),
                state.failures,
            ),
            String::from_str(env, reason),
        );
    }
    env.storage()
        .instance()
        .set(&key_for_operation(env, operation), &state);
}

#[contract]
pub struct CrossChainCredentials;

#[contractimpl]
impl CrossChainCredentials {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        storage::set_admin(&env, &admin);
    }

    pub fn issue_credential(
        env: Env,
        student: Address,
        achievement: String,
        metadata_hash: String,
        chain_id: ChainId,
    ) -> String {
        if !can_proceed(&env, "issue_cred") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();

        let credential_id = String::from_str(&env, "CRED");
        let credential = Credential {
            id: credential_id.clone(),
            student: student.clone(),
            issuer: admin,
            achievement,
            issued_at: env.ledger().timestamp(),
            chain_id,
            status: CredentialStatus::Active,
            metadata_hash,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id.clone()), &credential);

        let mut student_creds = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<String>>(&DataKey::StudentCreds(student.clone()))
            .unwrap_or(Vec::new(&env));
        student_creds.push_back(credential_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::StudentCreds(student), &student_creds);

        record_success(&env, "issue_cred");
        credential_id
    }

    pub fn revoke_credential(env: Env, credential_id: String) {
        if !can_proceed(&env, "revoke_cred") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Revoked;
        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id), &credential);
        record_success(&env, "revoke_cred");
    }

    pub fn suspend_credential(env: Env, credential_id: String) {
        if !can_proceed(&env, "suspend_cred") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Suspended;
        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id), &credential);
        record_success(&env, "suspend_cred");
    }

    pub fn reactivate_credential(env: Env, credential_id: String) {
        if !can_proceed(&env, "reactivate_cred") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Active;
        env.storage()
            .persistent()
            .set(&DataKey::Credential(credential_id), &credential);
        record_success(&env, "reactivate_cred");
    }

    pub fn get_credential(env: Env, credential_id: String) -> Credential {
        env.storage()
            .persistent()
            .get(&DataKey::Credential(credential_id))
            .unwrap()
    }

    pub fn verify_cross_chain(
        env: Env,
        credential_id: String,
        target_chain: ChainId,
    ) -> CrossChainProof {
        let credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();

        if credential.status != CredentialStatus::Active {
            panic!("Credential not active");
        }

        let proof = CrossChainProof {
            credential_id: credential.id.clone(),
            source_chain: credential.chain_id.clone(),
            target_chain,
            proof_hash: String::from_str(&env, "proof_hash"),
            verified_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proof(credential_id), &proof);
        proof
    }

    pub fn get_proof(env: Env, credential_id: String) -> CrossChainProof {
        env.storage()
            .persistent()
            .get(&DataKey::Proof(credential_id))
            .unwrap()
    }

    pub fn request_verification(
        env: Env,
        credential_id: String,
        chain_id: ChainId,
        requester: Address,
    ) -> String {
        let request_id = String::from_str(&env, "REQ");

        let request = VerificationRequest {
            id: request_id.clone(),
            credential_id,
            requester,
            chain_id,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id.clone()), &request);
        request_id
    }

    pub fn get_verification_request(env: Env, request_id: String) -> VerificationRequest {
        env.storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .unwrap()
    }

    pub fn generate_transcript(env: Env, student: Address) -> Transcript {
        let credentials = Self::get_student_credentials(env.clone(), student.clone());

        Transcript {
            student,
            credentials: credentials.clone(),
            total_achievements: credentials.len(),
            generated_at: env.ledger().timestamp(),
        }
    }

    pub fn get_student_credentials(env: Env, student: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::StudentCreds(student))
            .unwrap_or(Vec::new(&env))
    }

    pub fn add_oracle(env: Env, oracle: Address) {
        if !can_proceed(&env, "add_oracle") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();
        storage::add_oracle(&env, &oracle);
        record_success(&env, "add_oracle");
    }

    pub fn remove_oracle(env: Env, oracle: Address) {
        if !can_proceed(&env, "remove_oracle") {
            panic!("Circuit open");
        }
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage().instance().remove(&DataKey::Oracle(oracle));
        record_success(&env, "remove_oracle");
    }

    pub fn is_oracle(env: Env, oracle: Address) -> bool {
        is_oracle(&env, &oracle)
    }
}

#[cfg(test)]
mod tests;
