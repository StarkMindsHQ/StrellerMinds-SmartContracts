#![no_std]

mod errors;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;
mod types;

use errors::WebhookError;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Vec};
use types::*;

// ---------------------------------------------------------------------------
// HMAC-SHA256 signing (pure Soroban, no external crates)
// ---------------------------------------------------------------------------
// Soroban exposes SHA-256 via env.crypto().sha256(). We implement HMAC using
// the standard construction: HMAC(K,m) = H((K⊕opad) || H((K⊕ipad) || m))
// where ipad = 0x36 repeated and opad = 0x5c repeated, block size = 64 bytes.

fn hmac_sha256(env: &Env, key: &BytesN<32>, message: &Bytes) -> BytesN<32> {
    const BLOCK: usize = 64;
    const IPAD: u8 = 0x36;
    const OPAD: u8 = 0x5c;

    let mut ikey = [0u8; BLOCK];
    let mut okey = [0u8; BLOCK];
    for i in 0..32usize {
        let k = key.get(i as u32).unwrap_or(0);
        ikey[i] = k ^ IPAD;
        okey[i] = k ^ OPAD;
    }
    for i in 32..BLOCK {
        ikey[i] = IPAD;
        okey[i] = OPAD;
    }

    let mut inner_input = Bytes::new(env);
    inner_input.extend_from_array(&ikey);
    inner_input.append(message);
    let inner_hash: BytesN<32> = env.crypto().sha256(&inner_input).into();

    let mut outer_input = Bytes::new(env);
    outer_input.extend_from_array(&okey);
    let inner_bytes: Bytes = inner_hash.into();
    outer_input.append(&inner_bytes);
    env.crypto().sha256(&outer_input).into()
}

// ---------------------------------------------------------------------------
// Payload hashing helpers
// ---------------------------------------------------------------------------

fn hash_certificate_payload(env: &Env, p: &CertificateIssuedPayload) -> BytesN<32> {
    let mut b = Bytes::new(env);
    let cert_bytes: Bytes = p.certificate_id.clone().into();
    b.append(&cert_bytes);
    b.append(&p.course_id.clone().to_xdr(env));
    env.crypto().sha256(&b).into()
}

fn hash_progress_payload(env: &Env, p: &StudentProgressPayload) -> BytesN<32> {
    let mut b = Bytes::new(env);
    b.append(&p.course_id.clone().to_xdr(env));
    let pct_bytes = p.progress_pct.to_be_bytes();
    b.extend_from_array(&pct_bytes);
    env.crypto().sha256(&b).into()
}

fn hash_achievement_payload(env: &Env, p: &AchievementUnlockedPayload) -> BytesN<32> {
    let mut b = Bytes::new(env);
    let id_bytes = p.achievement_id.to_be_bytes();
    b.extend_from_array(&id_bytes);
    let ts_bytes = p.unlocked_at.to_be_bytes();
    b.extend_from_array(&ts_bytes);
    env.crypto().sha256(&b).into()
}

// ---------------------------------------------------------------------------
// Storage helpers
// ---------------------------------------------------------------------------

fn get_admin(env: &Env) -> Result<Address, WebhookError> {
    env.storage().instance().get(&DataKey::Admin).ok_or(WebhookError::NotInitialized)
}

fn next_webhook_id(env: &Env) -> u32 {
    let id: u32 = env.storage().instance().get(&DataKey::NextWebhookId).unwrap_or(0);
    env.storage().instance().set(&DataKey::NextWebhookId, &(id + 1));
    id
}

fn next_delivery_seq(env: &Env) -> u32 {
    let seq: u32 = env.storage().instance().get(&DataKey::NextDeliverySeq).unwrap_or(0);
    env.storage().instance().set(&DataKey::NextDeliverySeq, &(seq + 1));
    seq
}

fn get_webhook(env: &Env, id: u32) -> Result<WebhookEndpoint, WebhookError> {
    env.storage().persistent().get(&DataKey::Webhook(id)).ok_or(WebhookError::WebhookNotFound)
}

fn save_webhook(env: &Env, wh: &WebhookEndpoint) {
    env.storage().persistent().set(&DataKey::Webhook(wh.id), wh);
}

fn owner_webhooks(env: &Env, owner: &Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerWebhooks(owner.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

fn save_owner_webhooks(env: &Env, owner: &Address, ids: &Vec<u32>) {
    env.storage().persistent().set(&DataKey::OwnerWebhooks(owner.clone()), ids);
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct WebhookContract;

#[contractimpl]
impl WebhookContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), WebhookError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(WebhookError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.events().publish((symbol_short!("wh_init"),), admin);
        Ok(())
    }

    /// Register a new webhook endpoint.
    pub fn register(
        env: Env,
        owner: Address,
        url: Bytes,
        secret: BytesN<32>,
        event_types: Vec<WebhookEventType>,
    ) -> Result<u32, WebhookError> {
        get_admin(&env)?;
        owner.require_auth();

        if url.is_empty() {
            return Err(WebhookError::InvalidUrl);
        }
        if event_types.is_empty() {
            return Err(WebhookError::NoEventTypesSpecified);
        }

        let mut ids = owner_webhooks(&env, &owner);
        if ids.len() >= MAX_WEBHOOKS_PER_OWNER {
            return Err(WebhookError::TooManyWebhooks);
        }

        let id = next_webhook_id(&env);
        let wh = WebhookEndpoint {
            id,
            owner: owner.clone(),
            url,
            secret,
            event_types,
            active: true,
            created_at: env.ledger().timestamp(),
        };
        save_webhook(&env, &wh);
        ids.push_back(id);
        save_owner_webhooks(&env, &owner, &ids);

        env.events().publish((symbol_short!("wh_reg"), owner), id);
        Ok(id)
    }

    /// Deactivate a webhook. Only the owner may do this.
    pub fn unregister(env: Env, owner: Address, webhook_id: u32) -> Result<(), WebhookError> {
        owner.require_auth();
        let mut wh = get_webhook(&env, webhook_id)?;
        if wh.owner != owner {
            return Err(WebhookError::Unauthorized);
        }
        wh.active = false;
        save_webhook(&env, &wh);
        env.events().publish((symbol_short!("wh_unreg"), owner), webhook_id);
        Ok(())
    }

    /// Dispatch a CertificateIssued event to all matching active webhooks.
    pub fn dispatch_certificate_issued(
        env: Env,
        caller: Address,
        payload: CertificateIssuedPayload,
    ) -> Result<Vec<u32>, WebhookError> {
        caller.require_auth();
        get_admin(&env)?;
        let payload_hash = hash_certificate_payload(&env, &payload);
        let deliveries =
            Self::create_deliveries(&env, WebhookEventType::CertificateIssued, &payload_hash);
        env.events().publish(
            (symbol_short!("wh_cert"), payload.student.clone()),
            payload.certificate_id.clone(),
        );
        Ok(deliveries)
    }

    /// Dispatch a StudentProgress event to all matching active webhooks.
    pub fn dispatch_student_progress(
        env: Env,
        caller: Address,
        payload: StudentProgressPayload,
    ) -> Result<Vec<u32>, WebhookError> {
        caller.require_auth();
        get_admin(&env)?;
        let payload_hash = hash_progress_payload(&env, &payload);
        let deliveries =
            Self::create_deliveries(&env, WebhookEventType::StudentProgress, &payload_hash);
        env.events()
            .publish((symbol_short!("wh_prog"), payload.student.clone()), payload.progress_pct);
        Ok(deliveries)
    }

    /// Dispatch an AchievementUnlocked event to all matching active webhooks.
    pub fn dispatch_achievement_unlocked(
        env: Env,
        caller: Address,
        payload: AchievementUnlockedPayload,
    ) -> Result<Vec<u32>, WebhookError> {
        caller.require_auth();
        get_admin(&env)?;
        let payload_hash = hash_achievement_payload(&env, &payload);
        let deliveries =
            Self::create_deliveries(&env, WebhookEventType::AchievementUnlocked, &payload_hash);
        env.events()
            .publish((symbol_short!("wh_ach"), payload.student.clone()), payload.achievement_id);
        Ok(deliveries)
    }

    /// Retry a pending delivery. Enforces backoff: next_attempt_ledger must have passed.
    pub fn retry_delivery(
        env: Env,
        webhook_id: u32,
        delivery_seq: u32,
    ) -> Result<(), WebhookError> {
        let key = DataKey::PendingDelivery(webhook_id, delivery_seq);
        let mut delivery: PendingDelivery =
            env.storage().persistent().get(&key).ok_or(WebhookError::DeliveryNotFound)?;

        if delivery.attempts >= MAX_RETRY_ATTEMPTS {
            return Err(WebhookError::RetryLimitExceeded);
        }
        let current_ledger = env.ledger().sequence();
        if current_ledger < delivery.next_attempt_ledger {
            return Err(WebhookError::RetryTooEarly);
        }

        let wh = get_webhook(&env, webhook_id)?;
        if !wh.active {
            return Err(WebhookError::WebhookInactive);
        }

        let payload_bytes: Bytes = delivery.payload_hash.clone().into();
        let _signature = hmac_sha256(&env, &wh.secret, &payload_bytes);

        delivery.attempts += 1;
        delivery.next_attempt_ledger =
            current_ledger + RETRY_BACKOFF_LEDGERS * (1u32 << delivery.attempts.min(4));

        if delivery.attempts >= MAX_RETRY_ATTEMPTS {
            env.storage().persistent().remove(&key);
            env.events().publish((symbol_short!("wh_fail"), webhook_id), delivery_seq);
        } else {
            env.storage().persistent().set(&key, &delivery);
            env.events().publish(
                (symbol_short!("wh_retry"), webhook_id),
                (delivery_seq, delivery.attempts),
            );
        }

        Ok(())
    }

    /// Compute HMAC-SHA256 signature for a given webhook and message.
    pub fn compute_signature(
        env: Env,
        webhook_id: u32,
        owner: Address,
        message: Bytes,
    ) -> Result<BytesN<32>, WebhookError> {
        owner.require_auth();
        let wh = get_webhook(&env, webhook_id)?;
        if wh.owner != owner {
            return Err(WebhookError::Unauthorized);
        }
        Ok(hmac_sha256(&env, &wh.secret, &message))
    }

    pub fn get_webhook(env: Env, webhook_id: u32) -> Result<WebhookEndpoint, WebhookError> {
        get_webhook(&env, webhook_id)
    }

    pub fn get_owner_webhooks(env: Env, owner: Address) -> Vec<u32> {
        owner_webhooks(&env, &owner)
    }

    pub fn get_pending_delivery(
        env: Env,
        webhook_id: u32,
        delivery_seq: u32,
    ) -> Option<PendingDelivery> {
        env.storage().persistent().get(&DataKey::PendingDelivery(webhook_id, delivery_seq))
    }

    fn create_deliveries(
        env: &Env,
        event_type: WebhookEventType,
        payload_hash: &BytesN<32>,
    ) -> Vec<u32> {
        let next_id: u32 = env.storage().instance().get(&DataKey::NextWebhookId).unwrap_or(0);
        let mut seqs = Vec::new(env);

        for id in 0..next_id {
            let wh: Option<WebhookEndpoint> = env.storage().persistent().get(&DataKey::Webhook(id));
            let wh = match wh {
                Some(w) if w.active => w,
                _ => continue,
            };

            let mut subscribed = false;
            for et in wh.event_types.iter() {
                if et == event_type {
                    subscribed = true;
                    break;
                }
            }
            if !subscribed {
                continue;
            }

            let payload_bytes: Bytes = payload_hash.clone().into();
            let _sig = hmac_sha256(env, &wh.secret, &payload_bytes);

            let seq = next_delivery_seq(env);
            let delivery = PendingDelivery {
                webhook_id: id,
                event_type: event_type.clone(),
                payload_hash: payload_hash.clone(),
                attempts: 1,
                next_attempt_ledger: env.ledger().sequence() + RETRY_BACKOFF_LEDGERS,
                created_at: env.ledger().timestamp(),
            };
            env.storage().persistent().set(&DataKey::PendingDelivery(id, seq), &delivery);
            seqs.push_back(seq);
        }

        seqs
    }
}
