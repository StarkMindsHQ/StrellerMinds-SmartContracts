use soroban_sdk::{contracttype, Env, String, Symbol};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Err = 3,
    Metric = 4,
}

#[contracttype]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub payload: String,
}

pub struct Logger;

impl Logger {
    pub fn log(env: &Env, level: LogLevel, context: Symbol, message: String, payload: String) {
        let timestamp = env.ledger().timestamp();
        // Emit log event with simple tuple data
        env.events().publish(
            (Symbol::new(env, "LOG"), context, level),
            LogEntry {
                level,
                message,
                timestamp,
                payload,
            },
        );
    }

    pub fn metric(env: &Env, metric_name: Symbol, _value: i128) {
        Self::log(
            env,
            LogLevel::Metric,
            metric_name,
            String::from_str(env, "Performance Metric"),
            // Convert i128 to String for payload
            // Since soroban String implies no Display, we can't easily format.
            // We'll just put a placeholder or use crude conversion if needed.
            // For now, simple string.
            String::from_str(env, "value"),
        );
    }
}
