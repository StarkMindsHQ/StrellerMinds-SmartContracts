#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, String, Vec, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedPath {
    pub id: String,
    pub instructor: Address,
    pub name: String,
    pub description: String,
    pub price: i128,
    pub rating_sum: u32,
    pub rating_count: u32,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Path(String),
    UserPaths(Address),
    Admin,
}

#[contract]
pub struct LearningPathMarketplace;

#[contractimpl]
impl LearningPathMarketplace {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn list_path(
        env: Env,
        instructor: Address,
        id: String,
        name: String,
        description: String,
        price: i128,
    ) {
        instructor.require_auth();
        
        let path = ListedPath {
            id: id.clone(),
            instructor,
            name,
            description,
            price,
            rating_sum: 0,
            rating_count: 0,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Path(id), &path);
    }

    pub fn buy_path(env: Env, buyer: Address, id: String) {
        buyer.require_auth();
        
        let path: ListedPath = env.storage().persistent().get(&DataKey::Path(id.clone())).expect("Path not found");
        
        if path.price > 0 {
            // In a real implementation, we would transfer tokens from buyer to instructor
            // For this mock, we assume the transfer is handled or we just record the purchase
        }

        let mut user_paths: Vec<String> = env.storage().persistent().get(&DataKey::UserPaths(buyer.clone())).unwrap_or(Vec::new(&env));
        user_paths.push_back(id);
        env.storage().persistent().set(&DataKey::UserPaths(buyer), &user_paths);
    }

    pub fn rate_path(env: Env, user: Address, id: String, rating: u32) {
        user.require_auth();
        if rating < 1 || rating > 5 {
            panic!("Invalid rating");
        }

        let mut path: ListedPath = env.storage().persistent().get(&DataKey::Path(id.clone())).expect("Path not found");
        path.rating_sum += rating;
        path.rating_count += 1;
        env.storage().persistent().set(&DataKey::Path(id), &path);
    }

    pub fn get_path(env: Env, id: String) -> Option<ListedPath> {
        env.storage().persistent().get(&DataKey::Path(id))
    }

    pub fn get_user_paths(env: Env, user: Address) -> Vec<String> {
        env.storage().persistent().get(&DataKey::UserPaths(user)).unwrap_or(Vec::new(&env))
    }
}
