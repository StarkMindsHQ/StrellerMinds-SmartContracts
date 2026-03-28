use shared::circuit_breaker::{
    ensure_call_allowed, force_reset_closed, record_failure, record_success, CircuitBreakerConfig,
    CircuitBreakerStatus, CircuitCheckError, CircuitTransition,
};
use shared::event_schema::{
    AccessControlEventData, AnalyticsEventData, ContractInitializedEvent, SessionCompletedEvent,
    SessionRecordedEvent,
};
use shared::{emit_access_control_event, emit_analytics_event};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Error, Symbol,
};

use crate::errors::AnalyticsError;

#[contracttype]
enum DataKey {
    Admin,
    Session(BytesN<32>),
    CircuitConfig,
    CircuitStatus,
}

#[contract]
pub struct Analytics;

#[contractimpl]
impl Analytics {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Self::as_error(AnalyticsError::AlreadyInitialized));
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CircuitConfig, &CircuitBreakerConfig::default());
        env.storage().instance().set(&DataKey::CircuitStatus, &CircuitBreakerStatus::new());

        emit_access_control_event!(
            &env,
            symbol_short!("analytics"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );

        Ok(())
    }

    pub fn record_session(env: Env, user: Address, session_id: BytesN<32>) -> Result<(), Error> {
        user.require_auth();
        Self::ensure_operational(&env)?;

        let result = if env.storage().persistent().has(&DataKey::Session(session_id.clone())) {
            Err(Self::as_error(AnalyticsError::SessionAlreadyExists))
        } else {
            env.storage().persistent().set(&DataKey::Session(session_id.clone()), &user);

            emit_analytics_event!(
                &env,
                symbol_short!("analytics"),
                user.clone(),
                AnalyticsEventData::SessionRecorded(SessionRecordedEvent {
                    session_id: session_id.clone()
                })
            );
            Ok(())
        };

        if result.is_ok() {
            Self::record_operation_result(&env, true);
        }
        result
    }

    pub fn complete_session(env: Env, user: Address, session_id: BytesN<32>) -> Result<(), Error> {
        user.require_auth();
        Self::ensure_operational(&env)?;

        let result = if !env.storage().persistent().has(&DataKey::Session(session_id.clone())) {
            Err(Self::as_error(AnalyticsError::SessionNotFound))
        } else {
            emit_analytics_event!(
                &env,
                symbol_short!("analytics"),
                user.clone(),
                AnalyticsEventData::SessionCompleted(SessionCompletedEvent {
                    session_id: session_id.clone()
                })
            );
            Ok(())
        };

        if result.is_ok() {
            Self::record_operation_result(&env, true);
        }
        result
    }

    pub fn configure_circuit_breaker(
        env: Env,
        admin: Address,
        failure_threshold: u32,
        recovery_timeout_seconds: u64,
        half_open_max_calls: u32,
        half_open_success_threshold: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        if failure_threshold == 0
            || recovery_timeout_seconds == 0
            || half_open_max_calls == 0
            || half_open_success_threshold == 0
        {
            return Err(Self::as_error(AnalyticsError::InvalidConfiguration));
        }

        let config = CircuitBreakerConfig {
            failure_threshold,
            recovery_timeout_seconds,
            half_open_max_calls,
            half_open_success_threshold,
        };

        env.storage().instance().set(&DataKey::CircuitConfig, &config);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("config")),
            (symbol_short!("analytics"), failure_threshold, recovery_timeout_seconds),
        );

        Ok(())
    }

    pub fn reset_circuit_breaker(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut status = Self::get_circuit_status_internal(&env);
        force_reset_closed(&mut status, env.ledger().timestamp());
        env.storage().instance().set(&DataKey::CircuitStatus, &status);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("reset")),
            (symbol_short!("analytics"), admin),
        );

        Ok(())
    }

    pub fn report_operation_failure(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::record_operation_result(&env, false);
        env.events().publish(
            (symbol_short!("circuit"), symbol_short!("failure")),
            (Symbol::new(&env, "analytics"), admin),
        );
        Ok(())
    }

    pub fn get_circuit_breaker_status(env: Env) -> CircuitBreakerStatus {
        Self::get_circuit_status_internal(&env)
    }

    pub fn get_session(env: Env, session_id: BytesN<32>) -> Option<BytesN<32>> {
        if env.storage().persistent().has(&DataKey::Session(session_id.clone())) {
            Some(session_id)
        } else {
            None
        }
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    fn as_error(err: AnalyticsError) -> Error {
        Error::from_contract_error(err as u32)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or_else(|| Self::as_error(AnalyticsError::NotInitialized))?;

        if admin != *caller {
            return Err(Self::as_error(AnalyticsError::Unauthorized));
        }

        Ok(())
    }

    fn get_circuit_config_internal(env: &Env) -> CircuitBreakerConfig {
        env.storage()
            .instance()
            .get(&DataKey::CircuitConfig)
            .unwrap_or(CircuitBreakerConfig::default())
    }

    fn get_circuit_status_internal(env: &Env) -> CircuitBreakerStatus {
        env.storage().instance().get(&DataKey::CircuitStatus).unwrap_or(CircuitBreakerStatus::new())
    }

    fn emit_circuit_transition(
        env: &Env,
        transition: CircuitTransition,
        status: &CircuitBreakerStatus,
    ) {
        let action = match transition {
            CircuitTransition::Opened => symbol_short!("open"),
            CircuitTransition::HalfOpened => symbol_short!("halfopen"),
            CircuitTransition::Closed => symbol_short!("closed"),
        };

        env.events().publish(
            (symbol_short!("circuit"), action),
            (
                Symbol::new(env, "analytics"),
                status.total_failures,
                status.total_successes,
                status.alert_count,
            ),
        );
    }

    fn ensure_operational(env: &Env) -> Result<(), Error> {
        let mut status = Self::get_circuit_status_internal(env);
        let config = Self::get_circuit_config_internal(env);
        let now = env.ledger().timestamp();

        match ensure_call_allowed(&mut status, &config, now) {
            Ok(transition) => {
                env.storage().instance().set(&DataKey::CircuitStatus, &status);
                if let Some(state_change) = transition {
                    Self::emit_circuit_transition(env, state_change, &status);
                }
                Ok(())
            }
            Err(CircuitCheckError::Open) => {
                env.events().publish(
                    (symbol_short!("circuit"), symbol_short!("blocked")),
                    (Symbol::new(env, "analytics"), Symbol::new(env, "open")),
                );
                Err(Self::as_error(AnalyticsError::CircuitBreakerOpen))
            }
            Err(CircuitCheckError::HalfOpenCapacityReached) => {
                env.events().publish(
                    (symbol_short!("circuit"), symbol_short!("blocked")),
                    (Symbol::new(env, "analytics"), Symbol::new(env, "halfopen")),
                );
                Err(Self::as_error(AnalyticsError::CircuitBreakerHalfOpenLimit))
            }
        }
    }

    fn record_operation_result(env: &Env, success: bool) {
        let mut status = Self::get_circuit_status_internal(env);
        let config = Self::get_circuit_config_internal(env);
        let now = env.ledger().timestamp();

        let transition = if success {
            record_success(&mut status, &config, now)
        } else {
            record_failure(&mut status, &config, now)
        };

        env.storage().instance().set(&DataKey::CircuitStatus, &status);
        if let Some(state_change) = transition {
            Self::emit_circuit_transition(env, state_change, &status);
        }
    }
}

pub mod errors;
pub mod gas_optimized;

#[cfg(test)]
mod circuit_breaker_tests;
