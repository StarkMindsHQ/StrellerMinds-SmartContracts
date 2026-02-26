use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Credential(String),
    Proof(String),
    Oracle(Address),
    Request(String),
    StudentCreds(Address),
    ChainBridge(u32),
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn is_oracle(env: &Env, oracle: &Address) -> bool {
    env.storage()
        .instance()
        .has(&DataKey::Oracle(oracle.clone()))
}

pub fn add_oracle(env: &Env, oracle: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Oracle(oracle.clone()), &true);
}
