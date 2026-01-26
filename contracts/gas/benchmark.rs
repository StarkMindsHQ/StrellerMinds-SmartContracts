use soroban_sdk::{Env, Address};

pub fn benchmark_credit(env: &Env, contract_id: &Address, user: &Address) {
    env.invoke_contract::<()>(
        contract_id,
        &"credit",
        (user.clone(), 100_i128),
    );
}

pub fn benchmark_batch(env: &Env, contract_id: &Address, users: Vec<Address>) {
    env.invoke_contract::<()>(
        contract_id,
        &"batch_credit_users",
        (users, 100_i128),
    );
}
