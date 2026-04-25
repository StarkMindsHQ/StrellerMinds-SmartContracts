#![no_std]
extern crate alloc;

pub mod access_control;
pub mod config;
pub mod error_codes;
pub mod error_handling;
pub mod errors;
pub mod event_aggregator;
pub mod event_filter;
pub mod event_manager;
pub mod event_publisher;
pub mod event_replay;
pub mod event_schema;
pub mod event_utils;
pub mod events;
pub mod gas_optimizer;
pub mod gas_testing;
pub mod gdpr_types;
pub mod log_aggregator;
pub mod logger;
pub mod monitoring;
pub mod permissions;
pub mod rate_limiter;
pub mod reentrancy_guard;
pub mod roles;
pub mod storage;
pub mod two_factor_auth;
pub mod upgrade;
pub mod validation;
pub mod validation_core;

#[cfg(any(test, feature = "testutils"))]
pub mod debug_utils;
#[cfg(test)]
pub mod simple_tests;
#[cfg(test)]
pub mod test;

#[cfg(test)]
mod logger_tests;
#[cfg(test)]
pub mod monitoring_tests;
#[cfg(test)]
pub mod performance_tests;
