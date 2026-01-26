use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct UserAccount {
    pub balance: i128,
    pub locked: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    User(Address),
    Admin,
}
