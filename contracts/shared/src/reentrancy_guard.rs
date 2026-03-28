use soroban_sdk::{symbol_short, Env, Symbol};
use std::collections::HashMap;

const REENTRANCY_GUARD_KEY: Symbol = symbol_short!("REENTRANT");

pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Call at the start of a protected function. Panics if already entered.
    pub fn enter(env: &Env) {
        let mut storage = env.storage().instance();
        if storage.has(&REENTRANCY_GUARD_KEY) {
            panic!("ReentrancyGuard: reentrant call");
        }
        storage.set(&REENTRANCY_GUARD_KEY, &true);
    }

    /// Call at the end of a protected function to clear the lock.
    pub fn exit(env: &Env) {
        env.storage().instance().remove(&REENTRANCY_GUARD_KEY);
    }

    /// Checks if the reentrancy guard is currently active.
    pub fn is_active(env: &Env) -> bool {
        env.storage().instance().has(&REENTRANCY_GUARD_KEY)
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

pub struct ReentrancyTracker {
    call_stack: HashMap<Symbol, usize>,
}

impl ReentrancyTracker {
    pub fn new() -> Self {
        Self {
            call_stack: HashMap::new(),
        }
    }

    pub fn track_entry(&mut self, key: Symbol) {
        let count = self.call_stack.entry(key).or_insert(0);
        *count += 1;
    }

    pub fn track_exit(&mut self, key: Symbol) {
        if let Some(count) = self.call_stack.get_mut(&key) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub fn is_reentrant(&self, key: Symbol) -> bool {
        self.call_stack.get(&key).copied().unwrap_or(0) > 1
    }

    pub fn assert_no_reentrancy(&self, key: Symbol) {
        if self.is_reentrant(key) {
            panic!("Reentrancy detected for key: {:?}", key);
        }
    }
}

/// Documentation for ReentrancyGuard usage:
/// - Always use `ReentrancyGuard::enter` at the start of a protected function.
/// - Use `ReentrancyGuard::exit` at the end of the function to clear the lock.
/// - Utilize `ReentrancyTracker` for tracking nested calls and detecting reentrancy.
/// - Ensure that all functions interacting with shared state are protected.
/// - Refer to the tests in `tests/reentrancy_guard_tests.rs` for example usage.
