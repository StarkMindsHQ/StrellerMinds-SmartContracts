#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct Progress {
    module_id: Symbol,
    percent: u32,
}

#[contracttype]
enum DataKey {
    Progress(Address, Symbol), // (student, course_id)
    Admin,
}

#[contract]
pub struct ProgressTracker;

#[contractimpl]
impl ProgressTracker {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
    }
    pub fn update_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
        module_id: Symbol,
        percent: u32,
    ) {
        if percent > 100 {
            panic!("percentage cannot be more than 100");
        }
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not set");
        if student != admin {
            student.require_auth();
        } else {
            admin.require_auth();
        }
        let key = DataKey::Progress(student.clone(), course_id.clone());

        let mut progress_map: Map<Symbol, u32> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(&env));


        progress_map.set(module_id.clone(), percent);
        env.storage().persistent().set(&key, &progress_map);

        env.events().publish(
            (symbol_short!("progress"),),
            (
                symbol_short!("updated"),
                student,
                course_id,
                module_id,
                percent,
            ),
        );
    }

    pub fn get_progress(env: Env, student: Address, course_id: Symbol) -> Map<Symbol, u32> {
        let key = DataKey::Progress(student, course_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(&env)) 
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not set")
    }
}

#[cfg(test)]
mod test;
