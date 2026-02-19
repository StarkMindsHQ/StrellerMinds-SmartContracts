use soroban_sdk::{contract, contractimpl, Address, Env, Error};

#[contract]
pub struct Diagnostics;

#[contractimpl]
impl Diagnostics {
    pub fn initialize(_env: Env, _admin: Address) -> Result<(), Error> {
        Ok(())
    }

    pub fn run_diagnostic(_env: Env) -> Result<(), Error> {
        Ok(())
    }
}
