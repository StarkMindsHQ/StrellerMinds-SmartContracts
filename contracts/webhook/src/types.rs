use soroban_sdk::{contracttype, Address, Bytes, BytesN, String};

/// Maximum number of webhooks per owner
pub const MAX_WEBHOOKS_PER_OWNER: u32 = 10;
/// Maximum retry attempts for failed deliveries
pub const MAX_RETRY_ATTEMPTS: u32 = 3;
/// Retry backoff base in ledgers (~5s each)
pub const RETRY_BACKOFF_LEDGERS: u32 = 12; // ~1 minute

/// Event types that can trigger webhooks
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum WebhookEventType {
    CertificateIssued,
    StudentProgress,
    AchievementUnlocked,
}

/// Registered webhook endpoint
#[contracttype]
#[derive(Clone, Debug)]
pub struct WebhookEndpoint {
    pub id: u32,
    pub owner: Address,
    /// URL stored as Bytes (UTF-8 encoded)
    pub url: Bytes,
    /// HMAC-SHA256 signing secret (32 bytes)
    pub secret: BytesN<32>,
    pub event_types: soroban_sdk::Vec<WebhookEventType>,
    pub active: bool,
    pub created_at: u64,
}

/// Pending delivery record (stored for retry)
#[contracttype]
#[derive(Clone, Debug)]
pub struct PendingDelivery {
    pub webhook_id: u32,
    pub event_type: WebhookEventType,
    pub payload_hash: BytesN<32>,
    pub attempts: u32,
    pub next_attempt_ledger: u32,
    pub created_at: u64,
}

/// Payload for CertificateIssued event
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateIssuedPayload {
    pub certificate_id: BytesN<32>,
    pub student: Address,
    pub course_id: String,
    pub issued_at: u64,
}

/// Payload for StudentProgress event
#[contracttype]
#[derive(Clone, Debug)]
pub struct StudentProgressPayload {
    pub student: Address,
    pub course_id: String,
    pub progress_pct: u32,
    pub updated_at: u64,
}

/// Payload for AchievementUnlocked event
#[contracttype]
#[derive(Clone, Debug)]
pub struct AchievementUnlockedPayload {
    pub student: Address,
    pub achievement_id: u64,
    pub unlocked_at: u64,
}

/// Storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NextWebhookId,
    Webhook(u32),
    OwnerWebhooks(Address),
    PendingDelivery(u32, u32), // (webhook_id, delivery_seq)
    NextDeliverySeq,
}
