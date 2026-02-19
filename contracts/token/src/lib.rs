use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, Error, Symbol, Vec,
};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(_env: Env, _admin: Address) -> Result<(), Error> {
        Ok(())
    }

    pub fn mint(_env: Env, _to: Address, _amount: u64) -> Result<(), Error> {
        Ok(())
    }

    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: u64) -> Result<(), Error> {
        Ok(())
    }

    pub fn balance(_env: Env, _account: Address) -> Result<u64, Error> {
        Ok(0)
    }
}
