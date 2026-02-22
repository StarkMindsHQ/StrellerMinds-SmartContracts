use shared::gas_optimizer::{
    pack_u32, unpack_u32, BatchResult, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR,
};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

const PFX_STUDENT: Symbol = symbol_short!("STU");
const PFX_COURSE: Symbol = symbol_short!("CRS");

#[contracttype]
#[derive(Clone, PartialEq, Default)]
pub struct StudentAggregate {
    pub starts_and_completions: u64,
    pub streak_and_level: u64,
}

impl StudentAggregate {
    pub fn total_started(&self) -> u32 {
        unpack_u32(self.starts_and_completions).0
    }
    pub fn total_completed(&self) -> u32 {
        unpack_u32(self.starts_and_completions).1
    }
    pub fn current_streak(&self) -> u32 {
        unpack_u32(self.streak_and_level).0
    }
    pub fn level(&self) -> u32 {
        unpack_u32(self.streak_and_level).1
    }
    pub fn increment_started(&mut self) {
        let (s, c) = unpack_u32(self.starts_and_completions);
        self.starts_and_completions = pack_u32(s.saturating_add(1), c);
    }
    pub fn increment_completed(&mut self) {
        let (s, c) = unpack_u32(self.starts_and_completions);
        self.starts_and_completions = pack_u32(s, c.saturating_add(1));
    }
    pub fn set_streak_and_level(&mut self, streak: u32, level: u32) {
        self.streak_and_level = pack_u32(streak, level);
    }
}

#[contracttype]
#[derive(Clone, PartialEq, Default)]
pub struct CourseProgress {
    pub module_flags: u64,
    pub score_and_meta: u64,
}

impl CourseProgress {
    pub fn mark_module(&mut self, idx: u8) -> bool {
        let bit = 1u64 << (idx.min(63));
        if self.module_flags & bit != 0 {
            return false;
        }
        self.module_flags |= bit;
        true
    }
    pub fn modules_done(&self) -> u32 {
        self.module_flags.count_ones()
    }
    pub fn best_score_x10(&self) -> u16 {
        (self.score_and_meta >> 48) as u16
    }
    pub fn completion_pct(&self) -> u8 {
        ((self.score_and_meta >> 40) & 0xFF) as u8
    }
    pub fn update_meta(&mut self, score_x10: u16, pct: u8, ledger: u32) {
        self.score_and_meta = ((score_x10 as u64) << 48) | ((pct as u64) << 40) | (ledger as u64);
    }
}

fn student_key(learner: &Address) -> (Symbol, Address) {
    (PFX_STUDENT, learner.clone())
}
fn course_key(learner: &Address, course_id: u32) -> (Symbol, Address, u32) {
    (PFX_COURSE, learner.clone(), course_id)
}

fn load_student(env: &Env, learner: &Address) -> StudentAggregate {
    env.storage()
        .persistent()
        .get(&student_key(learner))
        .unwrap_or_default()
}
fn save_student(env: &Env, learner: &Address, agg: &StudentAggregate) {
    let key = student_key(learner);
    env.storage().persistent().set(&key, agg);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}
fn load_course(env: &Env, learner: &Address, course_id: u32) -> CourseProgress {
    env.storage()
        .persistent()
        .get(&course_key(learner, course_id))
        .unwrap_or_default()
}
fn save_course(env: &Env, learner: &Address, course_id: u32, prog: &CourseProgress) {
    let key = course_key(learner, course_id);
    env.storage().persistent().set(&key, prog);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}

pub fn enroll_student(env: &Env, learner: &Address, course_id: u32) {
    learner.require_auth();
    if env
        .storage()
        .persistent()
        .has(&course_key(learner, course_id))
    {
        return;
    }
    let mut agg = load_student(env, learner);
    agg.increment_started();
    save_student(env, learner, &agg);
    let mut prog = CourseProgress::default();
    prog.update_meta(0, 0, env.ledger().sequence());
    save_course(env, learner, course_id, &prog);
}

pub fn complete_module_with_score(
    env: &Env,
    learner: &Address,
    course_id: u32,
    module_idx: u8,
    score_x10: u16,
    total_modules: u8,
) -> bool {
    learner.require_auth();
    let mut prog = load_course(env, learner, course_id);
    if !prog.mark_module(module_idx) {
        return false;
    }
    let done = prog.modules_done();
    let pct = ((done as u64 * 100) / total_modules as u64) as u8;
    prog.update_meta(
        prog.best_score_x10().max(score_x10),
        pct,
        env.ledger().sequence(),
    );
    save_course(env, learner, course_id, &prog);
    if pct == 100 {
        let mut agg = load_student(env, learner);
        agg.increment_completed();
        let completed = agg.total_completed();
        agg.set_streak_and_level(agg.current_streak(), completed / 5);
        save_student(env, learner, &agg);
    }
    true
}

pub fn batch_complete_modules(
    env: &Env,
    learner: &Address,
    course_id: u32,
    modules: &Vec<(u32, u64)>,
    total_modules: u8,
) -> BatchResult {
    learner.require_auth();
    let mut prog = load_course(env, learner, course_id);
    let mut result = BatchResult::new();
    let mut best_score = prog.best_score_x10();
    for i in 0..modules.len() {
        if let Some((idx, score)) = modules.get(i) {
            if idx >= 64 {
                result.skipped += 1;
                continue;
            }
            if prog.mark_module(idx as u8) {
                best_score = best_score.max(score as u16);
                result.processed += 1;
            } else {
                result.skipped += 1;
            }
        }
    }
    if result.processed > 0 {
        let pct = ((prog.modules_done() as u64 * 100) / total_modules as u64) as u8;
        prog.update_meta(best_score, pct, env.ledger().sequence());
        save_course(env, learner, course_id, &prog);
        if pct == 100 {
            let mut agg = load_student(env, learner);
            agg.increment_completed();
            let completed = agg.total_completed();
            agg.set_streak_and_level(agg.current_streak(), completed / 5);
            save_student(env, learner, &agg);
        }
    }
    result
}

pub fn get_course_progress(env: &Env, learner: &Address, course_id: u32) -> CourseProgress {
    load_course(env, learner, course_id)
}
