use soroban_sdk::{Env, IntoVal, Symbol, Val};

/// Debugging utilities for contract development and testing.
/// These are gated behind `#[cfg(any(test, feature = "testutils"))]`
/// in `lib.rs` to ensure zero overhead in production builds.
pub struct DebugUtils;

impl DebugUtils {
    /// Inspect whether a storage key exists across all storage types.
    /// Emits a DEBUG event with the results for off-chain analysis.
    pub fn inspect_storage_key<K: IntoVal<Env, Val> + Clone>(
        env: &Env,
        key: &K,
        key_name: Symbol,
        contract_name: Symbol,
    ) {
        let key_val: Val = key.clone().into_val(env);
        let exists_persistent = env.storage().persistent().has(&key_val);
        let key_val2: Val = key.clone().into_val(env);
        let exists_instance = env.storage().instance().has(&key_val2);
        let key_val3: Val = key.clone().into_val(env);
        let exists_temp = env.storage().temporary().has(&key_val3);

        env.events().publish(
            (Symbol::new(env, "DEBUG"), contract_name, Symbol::new(env, "inspect")),
            (key_name, exists_persistent, exists_instance, exists_temp),
        );
    }

    /// Emit a snapshot of the current ledger state for debugging.
    pub fn emit_ledger_snapshot(env: &Env, contract_name: Symbol) {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();

        env.events().publish(
            (Symbol::new(env, "DEBUG"), contract_name, Symbol::new(env, "ledger")),
            (timestamp, sequence),
        );
    }
}

/// Trace function entry and exit, emitting DEBUG-level log events.
///
/// In production builds (without `testutils` feature), this module is excluded
/// entirely, so the macro will not be available. For production-safe tracing,
/// use `log_debug!` directly.
///
/// # Example
/// ```ignore
/// trace_fn!(&env, symbol_short!("token"), symbol_short!("mint"), {
///     // function body
///     do_mint(&env, &to, amount)
/// })
/// ```
#[macro_export]
macro_rules! trace_fn {
    ($env:expr, $contract:expr, $fn_name:expr, $body:expr) => {{
        let _corr_id = $env.ledger().sequence() as u64;
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Debug,
            &$crate::logger::LogContext {
                contract_name: $contract,
                function_name: $fn_name,
                correlation_id: Some(_corr_id),
            },
            soroban_sdk::symbol_short!("enter"),
            None,
        );
        let _result = $body;
        $crate::logger::Logger::log(
            $env,
            $crate::logger::LogLevel::Debug,
            &$crate::logger::LogContext {
                contract_name: $contract,
                function_name: $fn_name,
                correlation_id: Some(_corr_id),
            },
            soroban_sdk::symbol_short!("exit"),
            None,
        );
        _result
    }};
}
