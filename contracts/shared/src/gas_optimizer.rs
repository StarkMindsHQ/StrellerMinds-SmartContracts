

use soroban_sdk::{Env, Symbol, Val, symbol_short, IntoVal, TryFromVal};

pub const TTL_PERSISTENT_YEAR: u32 = 535_680;
pub const TTL_PERSISTENT_MONTH: u32 = 44_640;
pub const TTL_INSTANCE_DAY: u32 = 17_280;
pub const TTL_BUMP_THRESHOLD: u32 = 10_080;
pub const TTL_TEMP_MAX: u32 = 2_073_600;

#[inline(always)]
pub fn pack_u32(a: u32, b: u32) -> u64 {
    ((a as u64) << 32) | (b as u64)
}

#[inline(always)]
pub fn unpack_u32(packed: u64) -> (u32, u32) {
    ((packed >> 32) as u32, (packed & 0xFFFF_FFFF) as u32)
}

#[inline(always)]
pub fn pack_bool_u32(flag: bool, value: u32) -> u64 {
    ((flag as u64) << 32) | (value as u64)
}

#[inline(always)]
pub fn unpack_bool_u32(packed: u64) -> (bool, u32) {
    ((packed >> 32) != 0, (packed & 0xFFFF_FFFF) as u32)
}

pub fn extend_persistent_if_needed(env: &Env, key: &impl IntoVal<Env, Val>) {
    let key_val: Val = key.into_val(env);
    env.storage()
        .persistent()
        .extend_ttl(&key_val, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}

pub fn extend_instance_if_needed(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_BUMP_THRESHOLD, TTL_INSTANCE_DAY * 30);
}

pub fn set_if_changed<
    K: IntoVal<Env, Val> + Clone,
    V: IntoVal<Env, Val> + TryFromVal<Env, Val> + PartialEq + Clone,
>(
    env: &Env,
    key: &K,
    new_val: &V,
) -> bool {
    let existing: Option<V> = env.storage().persistent().get(key);
    if existing.as_ref() == Some(new_val) {
        return false;
    }
    env.storage().persistent().set(key, new_val);
    true
}

pub const SYM_ADMIN:    Symbol = symbol_short!("ADMIN");
pub const SYM_PAUSED:   Symbol = symbol_short!("PAUSED");
pub const SYM_SUPPLY:   Symbol = symbol_short!("SUPPLY");
pub const SYM_BALANCE:  Symbol = symbol_short!("BAL");
pub const SYM_PROGRESS: Symbol = symbol_short!("PROG");
pub const SYM_METRICS:  Symbol = symbol_short!("METRICS");
pub const SYM_CONFIG:   Symbol = symbol_short!("CFG");

use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchResult {
    pub processed: u32,
    pub skipped:   u32,
    pub failed:    u32,
}

impl BatchResult {
    pub fn new() -> Self {
        BatchResult { processed: 0, skipped: 0, failed: 0 }
    }
}

