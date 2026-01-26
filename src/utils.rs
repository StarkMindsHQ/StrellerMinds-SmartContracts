use soroban_sdk::{Env, Address};
use crate::storage::{DataKey, UserAccount};

pub fn get_user_cached(env: &Env, user: &Address) -> UserAccount {
    let storage = env.storage().persistent();
    storage
        .get(&DataKey::User(user.clone()))
        .unwrap_or(UserAccount {
            balance: 0,
            locked: 0,
        })
}

pub fn set_user_if_changed(
    env: &Env,
    user: &Address,
    updated: &UserAccount,
) {
    let storage = env.storage().persistent();
    let key = DataKey::User(user.clone());

    let current: Option<UserAccount> = storage.get(&key);
    if current.as_ref() != Some(updated) {
        storage.set(&key, updated);
    }
}
