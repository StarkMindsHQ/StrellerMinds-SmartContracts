use crate::logger::LogLevel;
use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

/// Storage key for log statistics
const LOG_STATS_KEY: Symbol = symbol_short!("LOGSTATS");

/// TTL for log stats in temporary storage (~30 days in ledgers at 5s/ledger)
const LOG_STATS_TTL: u32 = 518_400;

/// Aggregated log statistics stored in temporary storage.
/// Individual log entries are emitted as events for off-chain indexers;
/// this struct tracks only lightweight counters on-chain.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LogStats {
    pub debug_count: u32,
    pub info_count: u32,
    pub warn_count: u32,
    pub error_count: u32,
    pub metric_count: u32,
    pub last_error_timestamp: u64,
    pub total_count: u32,
    pub window_start: u64,
}

impl LogStats {
    pub fn new(window_start: u64) -> Self {
        LogStats {
            debug_count: 0,
            info_count: 0,
            warn_count: 0,
            error_count: 0,
            metric_count: 0,
            last_error_timestamp: 0,
            total_count: 0,
            window_start,
        }
    }
}

/// Log aggregation engine that maintains lightweight counters in temporary storage.
pub struct LogAggregator;

impl LogAggregator {
    /// Record a log entry by incrementing the appropriate counter.
    /// Called automatically by `Logger::log()` for non-Debug levels.
    pub fn record(env: &Env, level: LogLevel) {
        let mut stats = Self::get_stats(env);

        match level {
            LogLevel::Debug => stats.debug_count += 1,
            LogLevel::Info => stats.info_count += 1,
            LogLevel::Warn => stats.warn_count += 1,
            LogLevel::Err => {
                stats.error_count += 1;
                stats.last_error_timestamp = env.ledger().timestamp();
            }
            LogLevel::Metric => stats.metric_count += 1,
        }
        stats.total_count += 1;

        Self::save_stats(env, &stats);
    }

    /// Retrieve current log statistics.
    pub fn get_stats(env: &Env) -> LogStats {
        env.storage()
            .temporary()
            .get::<Symbol, LogStats>(&LOG_STATS_KEY)
            .unwrap_or_else(|| LogStats::new(env.ledger().timestamp()))
    }

    /// Reset all counters. Intended for admin use.
    pub fn reset(env: &Env) {
        let stats = LogStats::new(env.ledger().timestamp());
        Self::save_stats(env, &stats);
    }

    /// Calculate error rate as a percentage of total logs (0-100).
    /// Returns 0 if no logs have been recorded.
    pub fn get_error_rate(env: &Env) -> u32 {
        let stats = Self::get_stats(env);
        if stats.total_count == 0 {
            return 0;
        }
        (stats.error_count * 100) / stats.total_count
    }

    /// Get counts for a specific log level.
    pub fn get_count(env: &Env, level: LogLevel) -> u32 {
        let stats = Self::get_stats(env);
        match level {
            LogLevel::Debug => stats.debug_count,
            LogLevel::Info => stats.info_count,
            LogLevel::Warn => stats.warn_count,
            LogLevel::Err => stats.error_count,
            LogLevel::Metric => stats.metric_count,
        }
    }

    fn save_stats(env: &Env, stats: &LogStats) {
        env.storage()
            .temporary()
            .set::<Symbol, LogStats>(&LOG_STATS_KEY, stats);
        env.storage()
            .temporary()
            .extend_ttl(&LOG_STATS_KEY, LOG_STATS_TTL / 2, LOG_STATS_TTL);
    }
}
