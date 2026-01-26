use soroban_sdk::{contracttype, Env, Symbol, String, Val};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    ErrorLevel = 3,  // Renamed to avoid ambiguity with Error type
    Metric = 4,
}

// Note: LogEntry is simplified to avoid serialization issues
// It's only used for event emission, not storage

pub struct Logger;

impl Logger {
    pub fn log(env: &Env, level: LogLevel, context: Symbol, message: String) {
        let timestamp = env.ledger().timestamp();
        // Emit log event with simple tuple data
        env.events().publish(
            (Symbol::new(env, "LOG"), context, level),
            (message, timestamp)
        );
    }


    pub fn error(env: &Env, context: Symbol, message: String) {
        Self::log(env, LogLevel::ErrorLevel, context, message);
    }

    pub fn metric(env: &Env, metric_name: Symbol, _value: Val) {
        Self::log(
            env,
            LogLevel::Metric,
            metric_name,
            String::from_str(env, "Performance Metric"),
        );
    }
}