#![no_std]

use soroban_sdk::{contracttype, Address, Symbol, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningSession {
    pub id: String,
    pub student: Address,
    pub start_time: u64,
    pub course_id: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressAnalytics {
    pub completed_modules: u32,
    pub total_time: u64,
    pub score: u32,
}

pub trait AnalyticsClientTrait {
    fn record_session(session: LearningSession) -> Result<(), String>;
}
