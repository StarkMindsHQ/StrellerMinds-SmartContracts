use soroban_sdk::{contract, contractimpl, Env, Address, Vec};
use crate::utils::*;
use crate::batch::batch_credit;

#[contract]
pub struct OptimizedContract;

#[contractimpl]
impl OptimizedContract {

    pub fn credit(env: Env, user: Address, amount: i128) {
        let mut account = get_user_cached(&env, &user);
        account.balance += amount;
        set_user_if_changed(&env, &user, &account);
    }

    pub fn batch_credit_users(
        env: Env,
        users: Vec<Address>,
        amount: i128,
    ) {
        batch_credit(&env, users, amount);
    }
}
