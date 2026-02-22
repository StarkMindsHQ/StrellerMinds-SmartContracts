use shared::gas_optimizer::{
    extend_instance_if_needed, pack_bool_u32, unpack_bool_u32, BatchResult, TTL_BUMP_THRESHOLD,
    TTL_PERSISTENT_YEAR,
};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

const KEY_SUPPLY: Symbol = symbol_short!("SUPPLY");

#[contracttype]
#[derive(Clone, PartialEq, Default)]
pub struct PackedAccount {
    pub balance: u64,
    pub stake_packed: u64,
}

impl PackedAccount {
    pub fn staked_amount(&self) -> u32 {
        unpack_bool_u32(self.stake_packed).1
    }
    pub fn is_locked(&self) -> bool {
        unpack_bool_u32(self.stake_packed).0
    }
    pub fn set_stake(&mut self, locked: bool, amount: u32) {
        self.stake_packed = pack_bool_u32(locked, amount);
    }
}

fn account_key(owner: &Address) -> (Symbol, Address) {
    (symbol_short!("ACC"), owner.clone())
}

fn load_account(env: &Env, owner: &Address) -> PackedAccount {
    env.storage()
        .persistent()
        .get(&account_key(owner))
        .unwrap_or_default()
}

fn save_account(env: &Env, owner: &Address, acc: &PackedAccount) {
    let key = account_key(owner);
    env.storage().persistent().set(&key, acc);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}

fn load_supply(env: &Env) -> u64 {
    env.storage().instance().get(&KEY_SUPPLY).unwrap_or(0u64)
}

fn save_supply(env: &Env, supply: u64) {
    env.storage().instance().set(&KEY_SUPPLY, &supply);
}

pub fn transfer_optimized(env: &Env, from: &Address, to: &Address, amount: u64) {
    from.require_auth();
    let mut sender = load_account(env, from);
    assert!(sender.balance >= amount, "insufficient balance");
    let mut recipient = load_account(env, to);
    sender.balance -= amount;
    recipient.balance += amount;
    save_account(env, from, &sender);
    save_account(env, to, &recipient);
}

pub fn batch_transfer(env: &Env, from: &Address, recipients: &Vec<(Address, u64)>) -> BatchResult {
    from.require_auth();
    let mut result = BatchResult::new();
    let mut sender = load_account(env, from);
    let mut total: u64 = 0;
    for i in 0..recipients.len() {
        if let Some((_, amount)) = recipients.get(i) {
            total = total.saturating_add(amount);
        }
    }
    assert!(sender.balance >= total, "insufficient balance for batch");
    for i in 0..recipients.len() {
        if let Some((addr, amount)) = recipients.get(i) {
            if amount == 0 {
                result.skipped += 1;
                continue;
            }
            let mut recipient = load_account(env, &addr);
            sender.balance -= amount;
            recipient.balance += amount;
            save_account(env, &addr, &recipient);
            result.processed += 1;
        }
    }
    save_account(env, from, &sender);
    result
}

pub fn stake_optimized(env: &Env, staker: &Address, amount: u32) {
    staker.require_auth();
    let mut acc = load_account(env, staker);
    assert!(acc.balance >= amount as u64, "insufficient balance");
    assert!(!acc.is_locked(), "account locked");
    let current = acc.staked_amount();
    acc.balance -= amount as u64;
    acc.set_stake(false, current.saturating_add(amount));
    save_account(env, staker, &acc);
}

pub fn unstake_optimized(env: &Env, staker: &Address, amount: u32) {
    staker.require_auth();
    let mut acc = load_account(env, staker);
    assert!(!acc.is_locked(), "stake locked");
    assert!(acc.staked_amount() >= amount, "insufficient staked");
    let current = acc.staked_amount();
    acc.balance += amount as u64;
    acc.set_stake(false, current - amount);
    save_account(env, staker, &acc);
}

pub fn mint_optimized(env: &Env, admin: &Address, to: &Address, amount: u64) {
    admin.require_auth();
    let mut supply = load_supply(env);
    let mut recipient = load_account(env, to);
    supply += amount;
    recipient.balance += amount;
    save_supply(env, supply);
    save_account(env, to, &recipient);
    extend_instance_if_needed(env);
}

pub fn burn_optimized(env: &Env, from: &Address, amount: u64) {
    from.require_auth();
    let mut supply = load_supply(env);
    let mut acc = load_account(env, from);
    assert!(acc.balance >= amount, "insufficient balance");
    acc.balance -= amount;
    supply -= amount;
    save_account(env, from, &acc);
    save_supply(env, supply);
    extend_instance_if_needed(env);
}

pub fn balance_of(env: &Env, owner: &Address) -> u64 {
    load_account(env, owner).balance
}

pub fn total_supply(env: &Env) -> u64 {
    load_supply(env)
}
