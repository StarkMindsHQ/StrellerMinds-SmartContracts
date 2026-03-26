<<<<<<< HEAD
#![no_std]

pub mod validation;

pub mod access_control {
    use soroban_sdk::{Address, Env};


pub mod event_publisher;
pub mod event_replay;
>>>>>>> 3ff2e76 (fix: export actual RBAC implementation modules instead of placeholders)
    pub mod config;
    pub mod error_codes;
pub mod event_schema;
pub mod event_utils;
pub mod events;
pub mod gas_optimizer;
<<<<<<< HEAD
pub mod log_aggregator;
pub mod logger;
pub mod rate_limiter;

#[cfg(any(test, feature = "testutils"))]
pub mod debug_utils;
    pub mod gas_testing;
    pub mod log_aggregator;
/// Full validation implementation with security tests.
/// The `validation` module above is a lightweight stub used by other contracts
    pub mod permissions;
/// at the shared-crate boundary. This module exposes the complete validator.
    pub mod reentrancy_guard;
    pub mod roles;
    pub mod storage;
    pub mod upgrade;
    pub mod validation;
pub mod validation_core;
#[cfg(test)]
mod logger_tests;
    #[cfg(any(test, feature = "testutils"))]
    pub mod simple_tests;
    #[cfg(any(test, feature = "testutils"))]
    pub mod test;

#[cfg(test)]
pub mod monitoring_tests;
#[cfg(test)]
pub mod performance_tests;
