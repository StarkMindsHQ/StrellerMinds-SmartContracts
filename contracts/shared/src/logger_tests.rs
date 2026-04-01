#[cfg(test)]
mod tests {
    use crate::log_aggregator::LogAggregator;
    use crate::logger::{LogContext, LogLevel, Logger};
    use crate::{log_ctx, log_debug, log_error, log_info, log_metric, log_warn};
    use soroban_sdk::testutils::Events;
    use soroban_sdk::{contract, contractimpl, symbol_short, Env};

    #[contract]
    struct LogTestContract;

    #[contractimpl]
    impl LogTestContract {}

    fn setup() -> (Env, soroban_sdk::Address) {
        let env = Env::default();
        let id = env.register(LogTestContract, ());
        (env, id)
    }

    // ---------------------------------------------------------------
    // Logger core
    // ---------------------------------------------------------------

    #[test]
    fn test_default_level_is_info() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            assert_eq!(Logger::get_level(&env), LogLevel::Info);
        });
    }

    #[test]
    fn test_init_sets_level() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Warn);
            assert_eq!(Logger::get_level(&env), LogLevel::Warn);
        });
    }

    #[test]
    fn test_set_level_updates() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Debug);
            assert_eq!(Logger::get_level(&env), LogLevel::Debug);

            Logger::set_level(&env, LogLevel::Err);
            assert_eq!(Logger::get_level(&env), LogLevel::Err);
        });
    }

    #[test]
    fn test_should_log_respects_level() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Warn);

            assert!(!Logger::should_log(&env, LogLevel::Debug));
            assert!(!Logger::should_log(&env, LogLevel::Info));
            assert!(Logger::should_log(&env, LogLevel::Warn));
            assert!(Logger::should_log(&env, LogLevel::Err));
            assert!(Logger::should_log(&env, LogLevel::Metric));
        });
    }

    #[test]
    fn test_log_suppressed_below_level() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Warn);
            let events_before = env.events().all().len();

            let ctx = LogContext {
                contract_name: symbol_short!("test"),
                function_name: symbol_short!("fn1"),
                correlation_id: None,
            };
            Logger::log(&env, LogLevel::Debug, &ctx, symbol_short!("msg"), None);
            Logger::log(&env, LogLevel::Info, &ctx, symbol_short!("msg"), None);

            assert_eq!(env.events().all().len(), events_before);
        });
    }

    #[test]
    fn test_log_emitted_at_or_above_level() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Warn);
            let events_before = env.events().all().len();

            let ctx = LogContext {
                contract_name: symbol_short!("test"),
                function_name: symbol_short!("fn1"),
                correlation_id: None,
            };
            Logger::log(&env, LogLevel::Warn, &ctx, symbol_short!("warning"), None);

            assert!(env.events().all().len() > events_before);
        });
    }

    #[test]
    fn test_log_with_payload() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Debug);
            let events_before = env.events().all().len();

            let ctx = LogContext {
                contract_name: symbol_short!("test"),
                function_name: symbol_short!("fn1"),
                correlation_id: Some(42),
            };
            let payload = soroban_sdk::String::from_str(&env, "extra data");
            Logger::log(&env, LogLevel::Info, &ctx, symbol_short!("detail"), Some(payload));

            assert!(env.events().all().len() > events_before);
        });
    }

    // ---------------------------------------------------------------
    // Convenience macros
    // ---------------------------------------------------------------

    #[test]
    fn test_log_ctx_macro() {
        let ctx = log_ctx!(symbol_short!("tok"), symbol_short!("mint"));
        assert_eq!(ctx.contract_name, symbol_short!("tok"));
        assert_eq!(ctx.function_name, symbol_short!("mint"));
        assert_eq!(ctx.correlation_id, None);
    }

    #[test]
    fn test_log_ctx_macro_with_correlation() {
        let ctx = log_ctx!(symbol_short!("tok"), symbol_short!("mint"), 99);
        assert_eq!(ctx.correlation_id, Some(99));
    }

    #[test]
    fn test_log_info_macro() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Info);
            let events_before = env.events().all().len();
            log_info!(&env, symbol_short!("test"), symbol_short!("hello"));
            assert!(env.events().all().len() > events_before);
        });
    }

    #[test]
    fn test_log_debug_macro_suppressed() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Info);
            let events_before = env.events().all().len();
            log_debug!(&env, symbol_short!("test"), symbol_short!("dbg"));
            assert_eq!(env.events().all().len(), events_before);
        });
    }

    #[test]
    fn test_log_warn_macro() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Debug);
            let events_before = env.events().all().len();
            log_warn!(&env, symbol_short!("test"), symbol_short!("caution"));
            assert!(env.events().all().len() > events_before);
        });
    }

    #[test]
    fn test_log_error_macro() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Debug);
            let events_before = env.events().all().len();
            log_error!(&env, symbol_short!("test"), symbol_short!("fail"));
            assert!(env.events().all().len() > events_before);
        });
    }

    #[test]
    fn test_log_metric_macro() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            Logger::init(&env, LogLevel::Debug);
            let events_before = env.events().all().len();
            log_metric!(&env, symbol_short!("gas_used"), 12345_i128);
            assert!(env.events().all().len() > events_before);
        });
    }

    // ---------------------------------------------------------------
    // Log Aggregator
    // ---------------------------------------------------------------

    #[test]
    fn test_aggregator_initial_stats() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            let stats = LogAggregator::get_stats(&env);
            assert_eq!(stats.total_count, 0);
            assert_eq!(stats.error_count, 0);
        });
    }

    #[test]
    fn test_aggregator_record_increments() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            LogAggregator::record(&env, LogLevel::Info);
            LogAggregator::record(&env, LogLevel::Info);
            LogAggregator::record(&env, LogLevel::Err);

            let stats = LogAggregator::get_stats(&env);
            assert_eq!(stats.info_count, 2);
            assert_eq!(stats.error_count, 1);
            assert_eq!(stats.total_count, 3);
        });
    }

    #[test]
    fn test_aggregator_error_rate() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            LogAggregator::record(&env, LogLevel::Info);
            LogAggregator::record(&env, LogLevel::Info);
            LogAggregator::record(&env, LogLevel::Info);
            LogAggregator::record(&env, LogLevel::Err);

            assert_eq!(LogAggregator::get_error_rate(&env), 25);
        });
    }

    #[test]
    fn test_aggregator_error_rate_zero_when_empty() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            assert_eq!(LogAggregator::get_error_rate(&env), 0);
        });
    }

    #[test]
    fn test_aggregator_reset() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            LogAggregator::record(&env, LogLevel::Warn);
            LogAggregator::record(&env, LogLevel::Err);
            assert_eq!(LogAggregator::get_stats(&env).total_count, 2);

            LogAggregator::reset(&env);
            let stats = LogAggregator::get_stats(&env);
            assert_eq!(stats.total_count, 0);
            assert_eq!(stats.warn_count, 0);
            assert_eq!(stats.error_count, 0);
        });
    }

    #[test]
    fn test_aggregator_get_count() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            LogAggregator::record(&env, LogLevel::Debug);
            LogAggregator::record(&env, LogLevel::Debug);
            LogAggregator::record(&env, LogLevel::Warn);

            assert_eq!(LogAggregator::get_count(&env, LogLevel::Debug), 2);
            assert_eq!(LogAggregator::get_count(&env, LogLevel::Warn), 1);
            assert_eq!(LogAggregator::get_count(&env, LogLevel::Info), 0);
        });
    }

    #[test]
    fn test_aggregator_tracks_last_error_timestamp() {
        let (env, id) = setup();
        env.as_contract(&id, || {
            LogAggregator::record(&env, LogLevel::Err);
            let stats = LogAggregator::get_stats(&env);
            assert_eq!(stats.error_count, 1);
        });
    }

    // ---------------------------------------------------------------
    // Debug utils (only available in test/testutils)
    // ---------------------------------------------------------------

    #[test]
    fn test_debug_inspect_storage_key() {
        use crate::debug_utils::DebugUtils;

        let (env, id) = setup();
        env.as_contract(&id, || {
            let key = symbol_short!("mykey");
            let events_before = env.events().all().len();
            DebugUtils::inspect_storage_key(
                &env,
                &key,
                symbol_short!("mykey"),
                symbol_short!("test"),
            );
            assert!(env.events().all().len() > events_before);
        });
    }

    #[test]
    fn test_debug_ledger_snapshot() {
        use crate::debug_utils::DebugUtils;

        let (env, id) = setup();
        env.as_contract(&id, || {
            let events_before = env.events().all().len();
            DebugUtils::emit_ledger_snapshot(&env, symbol_short!("test"));
            assert!(env.events().all().len() > events_before);
        });
    }
}
