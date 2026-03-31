#![no_std]

pub mod validation;

pub mod access_control {
    use soroban_sdk::{Address, Env};

    pub struct AccessControl;

    impl AccessControl {
        /// Initialize the access control module with the given admin address.
        pub fn initialize(_env: &Env, _admin: &Address) -> Result<(), soroban_sdk::Error> {
            Ok(())
        }
    }
}

pub mod reentrancy_guard {
    use soroban_sdk::Env;

    pub struct ReentrancyLock;

    impl ReentrancyLock {
        /// Create a new reentrancy lock bound to the given environment.
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
        /// Create a new default Permission instance.
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
        /// Create a new default CircuitBreakerState instance.
        pub fn new() -> Self {
            Self
        }
    }
}

pub mod config;
pub mod error_codes;
pub mod event_schema;
pub mod event_utils;
pub mod gas_optimizer;
pub mod monitoring;

#[cfg(test)]
pub mod performance_tests;
