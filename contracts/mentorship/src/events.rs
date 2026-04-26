use soroban_sdk::{symbol_short, Address, Env, String};

pub fn emit_mentor_registered(env: &Env, mentor: &Address) {
    env.events().publish((symbol_short!("mentor"), symbol_short!("reg")), mentor.clone());
}

pub fn emit_session_created(env: &Env, session_id: u64, mentor: &Address, mentee: &Address) {
    env.events().publish((symbol_short!("session"), symbol_short!("create")), (session_id, mentor.clone(), mentee.clone()));
}

pub fn emit_session_updated(env: &Env, session_id: u64, status: u32) {
    env.events().publish((symbol_short!("session"), symbol_short!("update")), (session_id, status));
}

pub fn emit_review_submitted(env: &Env, reviewer: &Address, target: &Address, rating: u32) {
    env.events().publish((symbol_short!("review"), symbol_short!("submit")), (reviewer.clone(), target.clone(), rating));
}
