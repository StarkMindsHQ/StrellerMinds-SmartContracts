use soroban_sdk::{contracttype, Address, String, Vec};

/// Supported blockchain networks for cross-chain credential operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChainId {
    /// The Stellar network.
    Stellar,
    /// The Ethereum mainnet.
    Ethereum,
    /// The Polygon (MATIC) network.
    Polygon,
    /// The Binance Smart Chain network.
    Bsc,
}

impl ChainId {
    pub fn to_u32(&self) -> u32 {
        match self {
            ChainId::Stellar => 0,
            ChainId::Ethereum => 1,
            ChainId::Polygon => 2,
            ChainId::Bsc => 3,
        }
    }
}

/// Current validity status of a cross-chain credential.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialStatus {
    /// Credential is valid and in use.
    Active,
    /// Credential has been permanently revoked.
    Revoked,
    /// Credential is temporarily suspended.
    Suspended,
}

/// An educational credential anchored to a specific blockchain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credential {
    /// Unique string identifier for the credential.
    pub id: String,
    /// Address of the student who holds this credential.
    pub student: Address,
    /// Address of the entity that issued the credential.
    pub issuer: Address,
    /// Human-readable description of the achievement this credential represents.
    pub achievement: String,
    /// Unix timestamp (seconds) when the credential was issued.
    pub issued_at: u64,
    /// Identifier of the chain on which this credential was originally issued.
    pub chain_id: ChainId,
    /// Current validity status of the credential.
    pub status: CredentialStatus,
    /// Hash of the off-chain metadata associated with this credential.
    pub metadata_hash: String,
}

/// Cryptographic proof that a credential has been verified on a target chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainProof {
    /// Identifier of the credential this proof relates to.
    pub credential_id: String,
    /// Chain on which the credential was originally issued.
    pub source_chain: ChainId,
    /// Chain on which the credential was verified.
    pub target_chain: ChainId,
    /// Hash of the cross-chain verification proof.
    pub proof_hash: String,
    /// Unix timestamp (seconds) when the verification was completed.
    pub verified_at: u64,
}

/// An attestation from an oracle confirming the validity of a credential on a given chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleAttestation {
    /// Address of the oracle that produced this attestation.
    pub oracle: Address,
    /// Identifier of the credential being attested.
    pub credential_id: String,
    /// Chain on which the oracle performed the check.
    pub chain_id: ChainId,
    /// Whether the oracle found the credential to be valid.
    pub is_valid: bool,
    /// Unix timestamp (seconds) when the attestation was recorded.
    pub timestamp: u64,
}

/// A request to verify a credential on a specific chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationRequest {
    /// Unique string identifier for this verification request.
    pub id: String,
    /// Identifier of the credential to be verified.
    pub credential_id: String,
    /// Address of the party that submitted the verification request.
    pub requester: Address,
    /// Chain on which verification is requested.
    pub chain_id: ChainId,
    /// Unix timestamp (seconds) when the request was created.
    pub created_at: u64,
}

/// An aggregated transcript listing all credentials earned by a student.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transcript {
    /// Address of the student this transcript belongs to.
    pub student: Address,
    /// List of credential identifiers included in the transcript.
    pub credentials: Vec<String>,
    /// Total number of achievements represented in the transcript.
    pub total_achievements: u32,
    /// Unix timestamp (seconds) when the transcript was generated.
    pub generated_at: u64,
}
