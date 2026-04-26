#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Symbol, Vec};
use shared::event_publisher::EventPublisher;
use shared::event_schema::{EventData, CertificateEventData, CertificateMintedEvent, StandardEvent};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Certificate {
    pub id: BytesN<32>,
    pub student: Address,
    pub issuer: Address,
    pub course_id: String,
    pub issue_date: u64,
    pub expiry_date: u64,
    pub metadata_uri: String,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Certificate(BytesN<32>), // Granular key to prevent concurrent update corruption
}

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    pub fn mint(
        env: Env,
        student: Address,
        course_id: String,
        metadata_uri: String,
        expiry_date: u64,
    ) -> BytesN<32> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // Generate unique certificate ID
        let mut id_data = [0u8; 32];
        env.crypto().keccak256(&(student.clone(), course_id.clone(), env.ledger().timestamp()).into_val(&env)).copy_into_slice(&mut id_data);
        let id = BytesN::from_array(&env, &id_data);

        let certificate = Certificate {
            id: id.clone(),
            student: student.clone(),
            issuer: admin.clone(),
            course_id,
            issue_date: env.ledger().timestamp(),
            expiry_date,
            metadata_uri: metadata_uri.clone(),
        };

        // Use granular storage to avoid corruption of a global map during concurrent updates
        let key = DataKey::Certificate(id.clone());
        env.storage().persistent().set(&key, &certificate);

        // Publish event
        let event_data = EventData::Certificate(CertificateEventData::CertificateMinted(CertificateMintedEvent {
            certificate_id: id.clone(),
            student,
            issuer: admin,
            token_id: id.clone(), // Placeholder
            metadata_hash: metadata_uri,
        }));
        
        let event = StandardEvent::new(&env, Symbol::new(&env, "certificate"), certificate.issuer, event_data);
        let _ = EventPublisher::publish(&env, event);

        id
    }

    pub fn get_certificate(env: Env, id: BytesN<32>) -> Option<Certificate> {
        let key = DataKey::Certificate(id);
        env.storage().persistent().get(&key)
    }

    pub fn update_metadata(env: Env, id: BytesN<32>, new_uri: String) {
        let key = DataKey::Certificate(id.clone());
        let mut certificate: Certificate = env.storage().persistent().get(&key).expect("Certificate not found");
        
        certificate.issuer.require_auth();
        
        certificate.metadata_uri = new_uri;
        env.storage().persistent().set(&key, &certificate);
    }
}
