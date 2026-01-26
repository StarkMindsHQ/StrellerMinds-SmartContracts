use soroban_sdk::{Env, Address, Vec};
use crate::utils::{get_user_cached, set_user_if_changed};

pub fn batch_credit(
    env: &Env,
    users: Vec<Address>,
    amount: i128,
) {
    // Hard limit to prevent DoS
    assert!(users.len() <= 50, "Batch too large");

    for user in users.iter() {
        let mut account = get_user_cached(env, user);
        account.balance += amount;
        set_user_if_changed(env, user, &account);
    }
}
