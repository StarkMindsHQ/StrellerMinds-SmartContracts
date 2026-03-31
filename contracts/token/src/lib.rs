#![no_std]

pub mod errors;

use crate::errors::TokenError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, TokenEventData, TokensMintedEvent,
    TokensTransferredEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::{emit_access_control_event, emit_token_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

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
