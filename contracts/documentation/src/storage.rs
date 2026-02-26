use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

pub struct Storage;

impl Storage {
    pub fn increment_counter(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().persistent().get(key).unwrap_or(0);
        let new_value = current + 1;
        env.storage().persistent().set(key, &new_value);
        new_value
    }

    pub fn get_counter(env: &Env, key: &DataKey) -> u64 {
        env.storage().persistent().get(key).unwrap_or(0)
    }

    pub fn add_to_category(env: &Env, category: &String, doc_id: &String) {
        let key = DataKey::CategoryDocs(category.clone());
        let mut docs: Vec<String> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        docs.push_back(doc_id.clone());
        env.storage().persistent().set(&key, &docs);
    }

    pub fn add_to_user_contributions(env: &Env, user: &Address, contribution_id: &String) {
        let key = DataKey::UserContributions(user.clone());
        let mut contributions: Vec<String> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        contributions.push_back(contribution_id.clone());
        env.storage().persistent().set(&key, &contributions);
    }

    pub fn add_to_author_docs(env: &Env, author: &Address, doc_id: &String) {
        let key = DataKey::DocumentsByAuthor(author.clone());
        let mut docs: Vec<String> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        docs.push_back(doc_id.clone());
        env.storage().persistent().set(&key, &docs);
    }
}
