#![no_std]
use soroban_sdk::{contracttype, Env, Symbol, String, Val};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Metric = 4,
}

#[contracttype]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub payload: Val,
}

pub struct Logger;

impl Logger {
    pub fn log(env: &Env, level: LogLevel, context: Symbol, message: String, payload: Val) {
    
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (Symbol::new(env, "LOG"), context, level), 
            LogEntry {
                level,
                message,
                timestamp,
                payload,
            }
        );
    }


    pub fn metric(env: &Env, metric_name: Symbol, value: Val) {
        Self::log(
            env, 
            LogLevel::Metric, 
            metric_name, 
            String::from_str(env, "Performance Metric"), 
            value
        );
    }
}