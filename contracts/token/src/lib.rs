use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, TokenEventData, TokensMintedEvent,
    TokensTransferredEvent,
};
use shared::logger::{LogLevel, Logger};
use shared::{emit_access_control_event, emit_token_event, log_info};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Error};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
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

    pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), Error> {
        log_info!(&env, symbol_short!("token"), symbol_short!("mint"));

        emit_token_event!(
            &env,
            symbol_short!("token"),
            to.clone(),
            TokenEventData::TokensMinted(TokensMintedEvent { to, amount: amount as i128 })
        );
        Ok(())
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: u64) -> Result<(), Error> {
        from.require_auth();
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

    pub fn balance(_env: Env, _account: Address) -> Result<u64, Error> {
        Ok(0)
    }
}
pub mod gas_optimized;
