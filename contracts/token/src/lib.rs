pub mod errors;

use crate::errors::TokenError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, TokenEventData, TokensMintedEvent,
    TokensTransferredEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::{emit_access_control_event, emit_token_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), TokenError> {
        emit_access_control_event!(
            &env,
            symbol_short!("token"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), TokenError> {
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

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&symbol_short!("admin"));
        let mut report = Monitor::build_health_report(&env, symbol_short!("token"), initialized);
        Monitor::add_metric(
            &mut report,
            symbol_short!("uptime"),
            1,
            env.ledger().timestamp(),
        );
        Monitor::emit_health_check(&env, &report);
        report
    }
}
pub mod gas_optimized;
