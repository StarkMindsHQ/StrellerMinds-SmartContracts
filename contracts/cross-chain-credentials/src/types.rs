use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    BSC,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialStatus {
    Active,
    Revoked,
    Suspended,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credential {
    pub id: String,
    pub student: Address,
    pub issuer: Address,
    pub achievement: String,
    pub issued_at: u64,
    pub chain_id: ChainId,
    pub status: CredentialStatus,
    pub metadata_hash: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainProof {
    pub credential_id: String,
    pub source_chain: ChainId,
    pub target_chain: ChainId,
    pub proof_hash: String,
    pub verified_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleAttestation {
    pub oracle: Address,
    pub credential_id: String,
    pub chain_id: ChainId,
    pub is_valid: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationRequest {
    pub id: String,
    pub credential_id: String,
    pub requester: Address,
    pub chain_id: ChainId,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transcript {
    pub student: Address,
    pub credentials: Vec<String>,
    pub total_achievements: u32,
    pub generated_at: u64,
}
