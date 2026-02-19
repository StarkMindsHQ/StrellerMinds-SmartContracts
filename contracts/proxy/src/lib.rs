use soroban_sdk::{
    contract, contractimpl, Address, Env, Error,
};

#[contract]
pub struct Proxy;

#[contractimpl]
impl Proxy {
    pub fn initialize(_env: Env, _admin: Address, _implementation: Address) -> Result<(), Error> {
        Ok(())
    }

    pub fn upgrade(_env: Env, _new_implementation: Address) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_admin(_env: Env) -> Result<Address, Error> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }

    pub fn get_implementation(_env: Env) -> Result<Address, Error> {
        Ok(Address::from_str(&_env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"))
    }
}
