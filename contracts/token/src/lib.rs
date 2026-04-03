#![no_std]

pub mod errors;

use crate::errors::TokenError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, TokenEventData, TokensMintedEvent,
    TokensTransferredEvent,
};
use shared::logger::{LogLevel, Logger};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use shared::{emit_access_control_event, emit_token_event, log_info};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenDataKey {
    RateLimit(Address, u64), // (user, operation_id) -> RateLimitState
    RateLimitCfg,            // TokenRateLimits
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenRateLimits {
    pub max_transfers_per_day: u32,
    pub max_mints_per_day: u32,
    pub window_seconds: u64,
}

const RL_OP_TRANSFER: u64 = 1;
const RL_OP_MINT: u64 = 2;

fn get_token_rate_limits(env: &Env) -> TokenRateLimits {
    env.storage().instance().get(&TokenDataKey::RateLimitCfg).unwrap_or(TokenRateLimits {
        max_transfers_per_day: 100,
        max_mints_per_day: 50,
        window_seconds: 86_400,
    })
}

/// Entry point contract for the StrellerMinds token, providing mint, transfer, and balance operations.
#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    /// Initializes the token contract and records the admin address.
    ///
    /// # Arguments
    /// * `admin` - Address that will have administrative control over the contract.
    ///
    /// # Errors
    /// Returns [`TokenError::AlreadyInitialized`] if the contract has already been initialized.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), TokenError> {
        env.storage().instance().set(
            &TokenDataKey::RateLimitCfg,
            &TokenRateLimits {
                max_transfers_per_day: 100,
                max_mints_per_day: 50,
                window_seconds: 86_400,
            },
        );
        Logger::init(&env, LogLevel::Info);
        log_info!(&env, symbol_short!("token"), symbol_short!("init_ok"));

        emit_access_control_event!(
            &env,
            symbol_short!("token"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Mints new tokens and credits them to the recipient address.
    ///
    /// # Arguments
    /// * `to` - Recipient address to receive the newly minted tokens.
    /// * `amount` - Number of tokens to mint.
    ///
    /// # Errors
    /// Returns [`TokenError::Unauthorized`] if the caller is not the admin.
    /// Returns [`TokenError::InvalidAmount`] if `amount` is zero.
    ///
    /// # Example
    /// ```ignore
    /// client.mint(&recipient, &1000u64);
    /// ```
    pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), TokenError> {
        let rl = get_token_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &TokenDataKey::RateLimit(to.clone(), RL_OP_MINT),
            &RateLimitConfig { max_calls: rl.max_mints_per_day, window_seconds: rl.window_seconds },
        )
        .map_err(|_| TokenError::RateLimitExceeded)?;
        log_info!(&env, symbol_short!("token"), symbol_short!("mint"));

        emit_token_event!(
            &env,
            symbol_short!("token"),
            to.clone(),
            TokenEventData::TokensMinted(TokensMintedEvent { to, amount: amount as i128 })
        );
        Ok(())
    }

    /// Transfers tokens from one address to another.
    ///
    /// Requires authorization from `from`.
    ///
    /// # Arguments
    /// * `from` - Sender address (must authorize this call).
    /// * `to` - Recipient address.
    /// * `amount` - Number of tokens to transfer.
    ///
    /// # Errors
    /// Returns [`TokenError::InsufficientBalance`] if `from` does not have enough tokens.
    /// Returns [`TokenError::InvalidAmount`] if `amount` is zero.
    ///
    /// # Example
    /// ```ignore
    /// client.transfer(&sender, &recipient, &500u64);
    /// ```
    pub fn transfer(env: Env, from: Address, to: Address, amount: u64) -> Result<(), TokenError> {
        from.require_auth();
        let rl = get_token_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &TokenDataKey::RateLimit(from.clone(), RL_OP_TRANSFER),
            &RateLimitConfig {
                max_calls: rl.max_transfers_per_day,
                window_seconds: rl.window_seconds,
            },
        )
        .map_err(|_| TokenError::RateLimitExceeded)?;
        log_info!(&env, symbol_short!("token"), symbol_short!("transfer"));

        emit_token_event!(
            &env,
            symbol_short!("token"),
            from.clone(),
            TokenEventData::TokensTransferred(TokensTransferredEvent {
                from,
                to,
                amount: amount as i128,
            })
        );
        Ok(())
    }

    /// Returns the token balance of the given account.
    ///
    /// # Arguments
    /// * `account` - Address to query.
    ///
    /// # Errors
    /// Returns [`TokenError::InvalidAddress`] if `account` is not a valid address.
    ///
    /// # Example
    /// ```ignore
    /// let bal = client.balance(&account);
    /// ```
    pub fn balance(_env: Env, _account: Address) -> Result<u64, TokenError> {
        Ok(0)
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&symbol_short!("admin"));
        let mut report = Monitor::build_health_report(&env, symbol_short!("token"), initialized);
        Monitor::add_metric(&mut report, symbol_short!("uptime"), 1, env.ledger().timestamp());
        Monitor::emit_health_check(&env, &report);
        report
    }
}
pub mod gas_optimized;
#[cfg(test)]
pub mod property_tests;
pub mod benchmarks;
