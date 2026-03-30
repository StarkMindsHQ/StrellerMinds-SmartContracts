use soroban_sdk::contracttype;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_seconds: u64,
    pub half_open_max_calls: u32,
    pub half_open_success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout_seconds: 300,
            half_open_max_calls: 1,
            half_open_success_threshold: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CircuitBreakerStatus {
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_failure_at: u64,
    pub last_transition_at: u64,
    pub half_open_calls: u32,
    pub total_failures: u64,
    pub total_successes: u64,
    pub alert_count: u32,
}

impl CircuitBreakerStatus {
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_failure_at: 0,
            last_transition_at: 0,
            half_open_calls: 0,
            total_failures: 0,
            total_successes: 0,
            alert_count: 0,
        }
    }
}

impl Default for CircuitBreakerStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CircuitTransition {
    Opened,
    HalfOpened,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CircuitCheckError {
    Open,
    HalfOpenCapacityReached,
}

pub fn ensure_call_allowed(
    status: &mut CircuitBreakerStatus,
    config: &CircuitBreakerConfig,
    now: u64,
) -> Result<Option<CircuitTransition>, CircuitCheckError> {
    match status.state {
        CircuitState::Closed => Ok(None),
        CircuitState::Open => {
            if now.saturating_sub(status.last_transition_at) >= config.recovery_timeout_seconds {
                status.state = CircuitState::HalfOpen;
                status.consecutive_successes = 0;
                status.half_open_calls = 0;
                status.last_transition_at = now;
                Ok(Some(CircuitTransition::HalfOpened))
            } else {
                Err(CircuitCheckError::Open)
            }
        }
        CircuitState::HalfOpen => {
            if status.half_open_calls >= config.half_open_max_calls {
                Err(CircuitCheckError::HalfOpenCapacityReached)
            } else {
                status.half_open_calls += 1;
                Ok(None)
            }
        }
    }
}

pub fn record_success(
    status: &mut CircuitBreakerStatus,
    config: &CircuitBreakerConfig,
    now: u64,
) -> Option<CircuitTransition> {
    status.total_successes += 1;
    status.consecutive_failures = 0;

    match status.state {
        CircuitState::Closed => None,
        CircuitState::Open => None,
        CircuitState::HalfOpen => {
            status.consecutive_successes += 1;
            if status.consecutive_successes >= config.half_open_success_threshold {
                status.state = CircuitState::Closed;
                status.consecutive_successes = 0;
                status.half_open_calls = 0;
                status.last_transition_at = now;
                Some(CircuitTransition::Closed)
            } else {
                None
            }
        }
    }
}

pub fn record_failure(
    status: &mut CircuitBreakerStatus,
    config: &CircuitBreakerConfig,
    now: u64,
) -> Option<CircuitTransition> {
    status.total_failures += 1;
    status.last_failure_at = now;

    match status.state {
        CircuitState::Closed => {
            status.consecutive_failures += 1;
            if status.consecutive_failures >= config.failure_threshold {
                status.state = CircuitState::Open;
                status.consecutive_successes = 0;
                status.half_open_calls = 0;
                status.last_transition_at = now;
                status.alert_count += 1;
                Some(CircuitTransition::Opened)
            } else {
                None
            }
        }
        CircuitState::Open => None,
        CircuitState::HalfOpen => {
            status.state = CircuitState::Open;
            status.consecutive_successes = 0;
            status.half_open_calls = 0;
            status.last_transition_at = now;
            status.alert_count += 1;
            Some(CircuitTransition::Opened)
        }
    }
}

pub fn force_reset_closed(status: &mut CircuitBreakerStatus, now: u64) {
    status.state = CircuitState::Closed;
    status.consecutive_failures = 0;
    status.consecutive_successes = 0;
    status.half_open_calls = 0;
    status.last_transition_at = now;
}
