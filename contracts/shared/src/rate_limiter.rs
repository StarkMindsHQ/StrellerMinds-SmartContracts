use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Configuration for a rate limit on a specific operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitConfig {
    /// Maximum number of calls allowed per window.
    pub max_calls: u32,
    /// Window size in seconds (e.g., 86_400 for daily).
    pub window_seconds: u64,
}

/// Tracks how many calls a user has made in the current time bucket.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitState {
    /// Number of calls made in the current bucket.
    pub count: u32,
    /// The bucket identifier (timestamp / window_seconds).
    pub bucket: u64,
}

/// Pure logic: checks the rate limit and returns the updated state.
///
/// The caller is responsible for loading/storing the state.
/// Returns `Ok(new_state)` if the call is allowed, or `Err(())` if the limit is exceeded.
pub fn check_and_increment(
    current_timestamp: u64,
    state: Option<RateLimitState>,
    config: &RateLimitConfig,
) -> Result<RateLimitState, ()> {
    let current_bucket = current_timestamp / config.window_seconds;

    let count = match &state {
        Some(s) if s.bucket == current_bucket => s.count,
        _ => 0,
    };

    if count >= config.max_calls {
        return Err(());
    }

    Ok(RateLimitState {
        count: count + 1,
        bucket: current_bucket,
    })
}

/// Convenience wrapper: enforces a rate limit using persistent storage.
///
/// Reads the current `RateLimitState` from `storage_key`, checks the limit,
/// and writes back the updated state. Returns `Ok(())` if allowed, `Err(())` if exceeded.
pub fn enforce_rate_limit<K>(env: &Env, storage_key: &K, config: &RateLimitConfig) -> Result<(), ()>
where
    K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>
        + soroban_sdk::TryFromVal<Env, soroban_sdk::Val>,
{
    let current_timestamp = env.ledger().timestamp();
    let state: Option<RateLimitState> = env.storage().persistent().get(storage_key);

    let new_state = check_and_increment(current_timestamp, state, config)?;
    env.storage().persistent().set(storage_key, &new_state);

    Ok(())
}

/// Emits an event when a rate limit is exceeded, useful for off-chain monitoring.
pub fn emit_rate_limit_event(
    env: &Env,
    user: &Address,
    operation: &Symbol,
    count: u32,
    limit: u32,
) {
    env.events().publish(
        (
            Symbol::new(env, "rate_limit"),
            Symbol::new(env, "exceeded"),
        ),
        (user.clone(), operation.clone(), count, limit),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_call_succeeds() {
        let config = RateLimitConfig {
            max_calls: 5,
            window_seconds: 86_400,
        };
        let result = check_and_increment(1_000_000, None, &config);
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.count, 1);
        assert_eq!(state.bucket, 1_000_000 / 86_400);
    }

    #[test]
    fn test_calls_within_limit_succeed() {
        let config = RateLimitConfig {
            max_calls: 3,
            window_seconds: 86_400,
        };
        let ts = 1_000_000u64;
        let bucket = ts / 86_400;

        let mut state = check_and_increment(ts, None, &config).unwrap();
        assert_eq!(state.count, 1);

        state = check_and_increment(ts + 10, Some(state), &config).unwrap();
        assert_eq!(state.count, 2);

        state = check_and_increment(ts + 20, Some(state), &config).unwrap();
        assert_eq!(state.count, 3);
        assert_eq!(state.bucket, bucket);
    }

    #[test]
    fn test_call_exceeding_limit_fails() {
        let config = RateLimitConfig {
            max_calls: 2,
            window_seconds: 86_400,
        };
        let ts = 1_000_000u64;

        let state = check_and_increment(ts, None, &config).unwrap();
        let state = check_and_increment(ts + 1, Some(state), &config).unwrap();
        let result = check_and_increment(ts + 2, Some(state), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_window_resets_count() {
        let config = RateLimitConfig {
            max_calls: 2,
            window_seconds: 86_400,
        };
        let ts = 1_000_000u64;

        let state = check_and_increment(ts, None, &config).unwrap();
        let state = check_and_increment(ts + 1, Some(state), &config).unwrap();
        // Limit reached
        assert!(check_and_increment(ts + 2, Some(state.clone()), &config).is_err());

        // Advance to next window
        let next_window_ts = ts + 86_400;
        let result = check_and_increment(next_window_ts, Some(state), &config);
        assert!(result.is_ok());
        let new_state = result.unwrap();
        assert_eq!(new_state.count, 1);
        assert_eq!(new_state.bucket, next_window_ts / 86_400);
    }

    #[test]
    fn test_different_buckets_are_independent() {
        let config = RateLimitConfig {
            max_calls: 1,
            window_seconds: 3600, // 1 hour
        };

        // First call in bucket 0
        let state = check_and_increment(100, None, &config).unwrap();
        assert_eq!(state.count, 1);

        // Limit reached in same bucket
        assert!(check_and_increment(200, Some(state.clone()), &config).is_err());

        // New bucket allows calls again
        let state2 = check_and_increment(3700, Some(state), &config).unwrap();
        assert_eq!(state2.count, 1);
        assert_eq!(state2.bucket, 1);
    }

    #[test]
    fn test_max_calls_one() {
        let config = RateLimitConfig {
            max_calls: 1,
            window_seconds: 86_400,
        };
        let ts = 1_000_000u64;

        let state = check_and_increment(ts, None, &config).unwrap();
        assert_eq!(state.count, 1);

        assert!(check_and_increment(ts + 1, Some(state), &config).is_err());
    }
}
