use soroban_sdk::{Address, Env, Vec};
use crate::types::{MentorshipDataKey, MentorProfile, MentorshipSession};

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&MentorshipDataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&MentorshipDataKey::Admin).unwrap()
}

pub fn set_mentor_profile(env: &Env, address: &Address, profile: &MentorProfile) {
    env.storage().persistent().set(&MentorshipDataKey::Mentor(address.clone()), profile);
}

pub fn get_mentor_profile(env: &Env, address: &Address) -> Option<MentorProfile> {
    env.storage().persistent().get(&MentorshipDataKey::Mentor(address.clone()))
}

pub fn increment_session_counter(env: &Env) -> u64 {
    let mut count: u64 = env.storage().instance().get(&MentorshipDataKey::SessionCounter).unwrap_or(0);
    count += 1;
    env.storage().instance().set(&MentorshipDataKey::SessionCounter, &count);
    count
}

pub fn set_session(env: &Env, id: u64, session: &MentorshipSession) {
    env.storage().persistent().set(&MentorshipDataKey::Session(id), session);
}

pub fn get_session(env: &Env, id: u64) -> Option<MentorshipSession> {
    env.storage().persistent().get(&MentorshipDataKey::Session(id))
}

pub fn add_mentor_request(env: &Env, mentor: &Address, session_id: u64) {
    let mut requests: Vec<u64> = env.storage().persistent().get(&MentorshipDataKey::MentorRequests(mentor.clone())).unwrap_or_else(|| Vec::new(env));
    requests.push_back(session_id);
    env.storage().persistent().set(&MentorshipDataKey::MentorRequests(mentor.clone()), &requests);
}

pub fn add_mentee_request(env: &Env, mentee: &Address, session_id: u64) {
    let mut requests: Vec<u64> = env.storage().persistent().get(&MentorshipDataKey::MenteeRequests(mentee.clone())).unwrap_or_else(|| Vec::new(env));
    requests.push_back(session_id);
    env.storage().persistent().set(&MentorshipDataKey::MenteeRequests(mentee.clone()), &requests);
}
