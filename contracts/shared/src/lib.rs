#![no_std]
extern crate alloc;

pub mod validation;

pub mod access_control {
    use soroban_sdk::{Address, Env};

    pub struct AccessControl;

    impl AccessControl {
        pub fn initialize(_env: &Env, _admin: &Address) -> Result<(), soroban_sdk::Error> {
            Ok(())
        }
    }
}

pub mod reentrancy_guard {
    use soroban_sdk::Env;

    pub struct ReentrancyLock;

    impl ReentrancyLock {
        pub fn new(_env: &Env) -> Self {
            Self
        }
    }

    impl Default for ReentrancyLock {
        fn default() -> Self {
            Self::new(&Env::default())
        }
    }
}

pub mod roles {
    pub struct Permission;

    impl Default for Permission {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Permission {
        pub fn new() -> Self {
            Self
        }
    }
}

pub mod error_handling {
    pub struct CircuitBreakerState;

    impl Default for CircuitBreakerState {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CircuitBreakerState {
        pub fn new() -> Self {
            Self
        }
    }
}

pub mod event_schema;
pub mod event_utils;
pub mod gas_optimizer;

#[cfg(test)]
pub mod performance_tests;
