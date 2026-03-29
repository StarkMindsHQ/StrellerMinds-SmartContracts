pub mod errors;

use crate::errors::TokenError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, TokenEventData, TokensMintedEvent,
    TokensTransferredEvent,
};
use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use shared::{emit_access_control_event, emit_token_event};
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

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), TokenError> {
        env.storage().instance().set(
            &TokenDataKey::RateLimitCfg,
            &TokenRateLimits {
                max_transfers_per_day: 100,
                max_mints_per_day: 50,
                window_seconds: 86_400,
            },
        );
        emit_access_control_event!(
            &env,
            symbol_short!("token"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), TokenError> {
        let rl = get_token_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &TokenDataKey::RateLimit(to.clone(), RL_OP_MINT),
            &RateLimitConfig { max_calls: rl.max_mints_per_day, window_seconds: rl.window_seconds },
        ).map_err(|_| TokenError::RateLimitExceeded)?;
        emit_token_event!(
            &env,
            symbol_short!("token"),
            to.clone(),
            TokenEventData::TokensMinted(TokensMintedEvent { to, amount: amount as i128 })
        );
        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: u64) -> Result<(), TokenError> {
        from.require_auth();
        let rl = get_token_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &TokenDataKey::RateLimit(from.clone(), RL_OP_TRANSFER),
            &RateLimitConfig { max_calls: rl.max_transfers_per_day, window_seconds: rl.window_seconds },
        ).map_err(|_| TokenError::RateLimitExceeded)?;
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

    pub fn balance(_env: Env, _account: Address) -> Result<u64, TokenError> {
        Ok(0)
    }
}
pub mod gas_optimized;
