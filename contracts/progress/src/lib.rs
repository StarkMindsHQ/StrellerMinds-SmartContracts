use soroban_sdk::{contract, contractimpl, Address, Env, Error, Symbol, Vec};

#[contract]
pub struct Progress;

#[contractimpl]
impl Progress {
    pub fn initialize(_env: Env, _admin: Address) -> Result<(), Error> {
        Ok(())
    }

    pub fn record_progress(
        _env: Env,
        _student: Address,
        _course_id: Symbol,
        _progress: u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_progress(_env: Env, _student: Address, _course_id: Symbol) -> Result<u32, Error> {
        Ok(0)
    }

    pub fn get_student_courses(_env: Env, _student: Address) -> Vec<Symbol> {
        Vec::new(&_env)
    }
}
