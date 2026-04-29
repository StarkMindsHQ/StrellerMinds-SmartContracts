use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentType {
    TaxReturn,
    PaymentReceipt,
    AssessmentNotice,
    ExemptionCertificate,
    AppealFiling,
    SupportingEvidence,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaxDocument {
    pub id: u64,
    pub owner: Address,
    pub property_id: String,
    pub doc_type: DocumentType,
    pub ipfs_hash: String,
    pub tax_year: u32,
    pub uploaded_at: u64,
    pub verified: bool,
    pub verifier: Option<Address>,
    pub verified_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaxAdvisor {
    pub address: Address,
    pub name: String,
    pub license_id: String,
    pub jurisdictions: Vec<String>,
    pub active: bool,
    pub registered_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DocumentCounter,
    Document(u64),
    OwnerDocuments(Address),
    PropertyDocuments(String),

    Advisor(Address),
    PropertyAdvisor(String),
}
