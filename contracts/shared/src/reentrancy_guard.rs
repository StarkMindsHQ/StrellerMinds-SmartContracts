use soroban_sdk::{Env, Symbol, symbol_short};

const REENTRANCY_GUARD_KEY: Symbol = symbol_short!("REENTRANT");

pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Call at the start of a protected function. Panics if already entered.
    pub fn enter(env: &Env) {
        if env.storage().instance().has(&REENTRANCY_GUARD_KEY) {
            panic!("ReentrancyGuard: reentrant call");
        }
        env.storage().instance().set(&REENTRANCY_GUARD_KEY, &true);
    }

    /// Call at the end of a protected function to clear the lock.
    pub fn exit(env: &Env) {
        env.storage().instance().remove(&REENTRANCY_GUARD_KEY);
    }
}

/// Helper RAII-style guard for use with early returns
pub struct ReentrancyLock<'a> {
    env: &'a Env,
}

impl<'a> ReentrancyLock<'a> {
    pub fn new(env: &'a Env) -> Self {
        ReentrancyGuard::enter(env);
        Self { env }
    }
}

impl<'a> Drop for ReentrancyLock<'a> {
    fn drop(&mut self) {
        ReentrancyGuard::exit(self.env);
    }
} 