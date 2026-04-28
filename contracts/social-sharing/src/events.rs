use soroban_sdk::{symbol_short, Address, BytesN, Env, String};
use crate::types::SharePlatform;

/// Emit a share event when an achievement is shared.
pub fn emit_share_event(
    env: &Env,
    user: &Address,
    certificate_id: &BytesN<32>,
    platform: &SharePlatform,
    timestamp: u64,
) {
    let platform_str = match platform {
        SharePlatform::Twitter => "twitter",
        SharePlatform::LinkedIn => "linkedin",
        SharePlatform::Facebook => "facebook",
    };

    env.events().publish(
        (symbol_short!("share"), user.clone()),
        (
            certificate_id.clone(),
            String::from_slice(env, platform_str),
            timestamp,
        ),
    );
}
