use soroban_sdk::{contracttype, symbol_short, Env, String, Symbol};

/// Storage key for the configured minimum log level
const LOG_LEVEL_KEY: Symbol = symbol_short!("LOGLVL");

/// Log severity levels, ordered by increasing severity.
/// The `repr(u32)` ordering is used for level filtering.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Err = 3,
    Metric = 4,
}

/// Structured log entry emitted as an event.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub payload: String,
}

/// Context attached to each log emission for tracing and filtering.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LogContext {
    /// Short identifier for the emitting contract (e.g. `symbol_short!("token")`)
    pub contract_name: Symbol,
    /// Name of the function emitting the log
    pub function_name: Symbol,
    /// Optional correlation ID for grouping related log entries within a transaction
    pub correlation_id: Option<u64>,
}

/// Structured logger that emits log events through Soroban's event system.
///
/// Supports configurable minimum log levels stored in instance storage,
/// context-aware logging with contract/function identification, and
/// optional integration with the log aggregation module.
pub struct Logger;

impl Logger {
    /// Initialize logging with a minimum log level.
    /// Call this once during contract initialization.
    pub fn init(env: &Env, min_level: LogLevel) {
        env.storage().instance().set::<Symbol, u32>(&LOG_LEVEL_KEY, &(min_level as u32));
    }

    /// Update the minimum log level at runtime.
    pub fn set_level(env: &Env, min_level: LogLevel) {
        env.storage().instance().set::<Symbol, u32>(&LOG_LEVEL_KEY, &(min_level as u32));
    }

    /// Read the current minimum log level. Defaults to `Info` if not set.
    pub fn get_level(env: &Env) -> LogLevel {
        let raw: u32 = env.storage().instance().get::<Symbol, u32>(&LOG_LEVEL_KEY).unwrap_or(1); // default Info
        match raw {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Err,
            4 => LogLevel::Metric,
            _ => LogLevel::Info,
        }
    }

    /// Returns true if the given level meets or exceeds the configured minimum.
    #[inline]
    pub fn should_log(env: &Env, level: LogLevel) -> bool {
        (level as u32) >= (Self::get_level(env) as u32)
    }

    /// Emit a structured log event with context.
    ///
    /// The event is published with topic `("LOG", contract_name, level, function_name)`
    /// and data `(timestamp, message, payload, correlation_id)`.
    ///
    /// If the level is below the configured minimum, this is a no-op.
    pub fn log(
        env: &Env,
        level: LogLevel,
        ctx: &LogContext,
        message: Symbol,
        payload: Option<String>,
    ) {
        if !Self::should_log(env, level) {
            return;
        }

        let timestamp = env.ledger().timestamp();

        // Publish with structured topics for off-chain filtering
        env.events().publish(
            (Symbol::new(env, "LOG"), ctx.contract_name.clone(), level, ctx.function_name.clone()),
            (timestamp, message, payload, ctx.correlation_id),
        );

        // Record in aggregator for non-Debug levels (to save gas)
        #[cfg(not(test))]
        if level as u32 >= LogLevel::Info as u32 {
            crate::log_aggregator::LogAggregator::record(env, level);
        }
    }

    /// Emit a simple log without full context (backward-compatible convenience).
    pub fn log_simple(
        env: &Env,
        level: LogLevel,
        context: Symbol,
        message: String,
        payload: String,
    ) {
        if !Self::should_log(env, level) {
            return;
        }
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (Symbol::new(env, "LOG"), context, level),
            LogEntry { level, message, timestamp, payload },
        );
    }

    /// Record a performance metric.
    pub fn metric(env: &Env, metric_name: Symbol, value: i128) {
        if !Self::should_log(env, LogLevel::Metric) {
            return;
        }
        let timestamp = env.ledger().timestamp();
        env.events().publish(
            (Symbol::new(env, "LOG"), Symbol::new(env, "metric"), LogLevel::Metric, metric_name),
            (timestamp, value),
        );
    }
}

// ---------------------------------------------------------------------------
// Convenience macros
// ---------------------------------------------------------------------------

/// Create a `LogContext` for the current contract and function.
///
/// # Example
/// ```ignore
/// let ctx = log_ctx!(symbol_short!("token"), symbol_short!("mint"));
/// ```
#[macro_export]
macro_rules! log_ctx {
    ($contract:expr, $function:expr) => {
        $crate::logger::LogContext {
            contract_name: $contract,
            function_name: $function,
            correlation_id: None,
        }
    };
    ($contract:expr, $function:expr, $corr_id:expr) => {
        $crate::logger::LogContext {
            contract_name: $contract,
            function_name: $function,
            correlation_id: Some($corr_id),
        }
    };
}

/// Emit a Debug-level log event.
#[macro_export]
macro_rules! log_debug {
    ($env:expr, $contract:expr, $msg:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Debug,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            None,
        )
    };
    ($env:expr, $contract:expr, $msg:expr, $payload:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Debug,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            Some($payload),
        )
    };
}

/// Emit an Info-level log event.
#[macro_export]
macro_rules! log_info {
    ($env:expr, $contract:expr, $msg:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Info,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            None,
        )
    };
    ($env:expr, $contract:expr, $msg:expr, $payload:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Info,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            Some($payload),
        )
    };
}

/// Emit a Warn-level log event.
#[macro_export]
macro_rules! log_warn {
    ($env:expr, $contract:expr, $msg:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Warn,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            None,
        )
    };
    ($env:expr, $contract:expr, $msg:expr, $payload:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Warn,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            Some($payload),
        )
    };
}

/// Emit an Error-level log event.
#[macro_export]
macro_rules! log_error {
    ($env:expr, $contract:expr, $msg:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Err,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            None,
        )
    };
    ($env:expr, $contract:expr, $msg:expr, $payload:expr) => {
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Err,
            &$crate::log_ctx!($contract, soroban_sdk::symbol_short!("_")),
            $msg,
            Some($payload),
        )
    };
}

/// Emit a Metric-level log event with a numeric value.
#[macro_export]
macro_rules! log_metric {
    ($env:expr, $metric_name:expr, $value:expr) => {
        $crate::logger::Logger::metric($env, $metric_name, $value)
    };
}
