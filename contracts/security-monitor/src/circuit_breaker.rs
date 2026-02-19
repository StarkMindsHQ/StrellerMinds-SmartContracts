use crate::errors::SecurityError;
use crate::events::SecurityEvents;
use crate::storage::SecurityStorage;
use crate::types::{BreakerState, CircuitBreakerState};
use soroban_sdk::{Env, Symbol};

/// Circuit breaker implementation for automated fault handling
pub struct CircuitBreaker;

impl CircuitBreaker {
    /// Check circuit breaker state and record an operation result
    /// Returns true if the operation should be allowed, false if circuit is open
    pub fn check_and_record(
        env: &Env,
        contract: &Symbol,
        function: &Symbol,
        success: bool,
    ) -> Result<bool, SecurityError> {
        let config = SecurityStorage::get_config(env).ok_or(SecurityError::NotInitialized)?;

        let mut state = SecurityStorage::get_circuit_breaker_state(env, contract, function)
            .unwrap_or_else(|| {
                CircuitBreakerState::new(
                    contract.clone(),
                    function.clone(),
                    config.circuit_breaker_threshold,
                    config.circuit_breaker_timeout,
                )
            });

        let current_time = env.ledger().timestamp();
        state.last_checked = current_time;

        match state.state {
            BreakerState::Open => {
                // Check if timeout has passed
                if let Some(opened_at) = state.opened_at {
                    if current_time.saturating_sub(opened_at) > state.timeout_duration {
                        // Transition to HalfOpen
                        state.state = BreakerState::HalfOpen;
                        SecurityStorage::set_circuit_breaker_state(env, &state);
                        SecurityEvents::emit_circuit_breaker_state_changed(
                            env,
                            contract,
                            function,
                            &state.state,
                        );
                        Ok(true) // Allow this call through to test
                    } else {
                        // Circuit still open
                        Ok(false)
                    }
                } else {
                    // Invalid state - no opened_at timestamp
                    Ok(false)
                }
            }

            BreakerState::HalfOpen => {
                if success {
                    // Success in HalfOpen -> Close circuit
                    state.state = BreakerState::Closed;
                    state.failure_count = 0;
                    state.opened_at = None;
                    SecurityStorage::set_circuit_breaker_state(env, &state);

                    SecurityEvents::emit_circuit_breaker_closed(env, contract, function);
                    SecurityEvents::emit_circuit_breaker_state_changed(
                        env,
                        contract,
                        function,
                        &state.state,
                    );
                    Ok(true)
                } else {
                    // Failure in HalfOpen -> Reopen circuit
                    state.state = BreakerState::Open;
                    state.opened_at = Some(current_time);
                    state.failure_count += 1;
                    state.last_failure_time = current_time;
                    SecurityStorage::set_circuit_breaker_state(env, &state);

                    SecurityEvents::emit_circuit_breaker_opened(
                        env,
                        contract,
                        function,
                        state.failure_count,
                    );
                    SecurityEvents::emit_circuit_breaker_state_changed(
                        env,
                        contract,
                        function,
                        &state.state,
                    );
                    Ok(false)
                }
            }

            BreakerState::Closed => {
                if !success {
                    // Record failure
                    state.failure_count += 1;
                    state.last_failure_time = current_time;

                    if state.failure_count >= state.failure_threshold {
                        // Open the circuit
                        state.state = BreakerState::Open;
                        state.opened_at = Some(current_time);

                        SecurityStorage::set_circuit_breaker_state(env, &state);

                        SecurityEvents::emit_circuit_breaker_opened(
                            env,
                            contract,
                            function,
                            state.failure_count,
                        );
                        SecurityEvents::emit_circuit_breaker_state_changed(
                            env,
                            contract,
                            function,
                            &state.state,
                        );
                        Ok(false)
                    } else {
                        // Still within threshold
                        SecurityStorage::set_circuit_breaker_state(env, &state);
                        Ok(true)
                    }
                } else {
                    // Success - reset failure count if there were previous failures
                    if state.failure_count > 0 {
                        state.failure_count = 0;
                        SecurityStorage::set_circuit_breaker_state(env, &state);
                    }
                    Ok(true)
                }
            }
        }
    }

    /// Manually reset a circuit breaker (admin function)
    pub fn reset(env: &Env, contract: &Symbol, function: &Symbol) -> Result<(), SecurityError> {
        let config = SecurityStorage::get_config(env).ok_or(SecurityError::NotInitialized)?;

        let mut state = SecurityStorage::get_circuit_breaker_state(env, contract, function)
            .ok_or(SecurityError::CircuitBreakerNotFound)?;

        state.state = BreakerState::Closed;
        state.failure_count = 0;
        state.opened_at = None;
        state.last_checked = env.ledger().timestamp();

        SecurityStorage::set_circuit_breaker_state(env, &state);

        SecurityEvents::emit_circuit_breaker_closed(env, contract, function);

        Ok(())
    }

    /// Get the current state of a circuit breaker
    pub fn get_state(
        env: &Env,
        contract: &Symbol,
        function: &Symbol,
    ) -> Option<CircuitBreakerState> {
        SecurityStorage::get_circuit_breaker_state(env, contract, function)
    }
}
