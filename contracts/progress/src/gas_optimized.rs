use shared::gas_optimizer::{BatchResult, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

const KEY_PROGRESS: Symbol = symbol_short!("PROG");

#[contracttype]
#[derive(Clone, PartialEq, Default)]
pub struct PackedProgress {
    pub module_flags: u64,
    pub score_and_meta: u64,
}

impl PackedProgress {
    pub fn is_module_complete(&self, idx: u8) -> bool {
        (self.module_flags >> idx) & 1 == 1
    }
    pub fn mark_module_complete(&mut self, idx: u8) -> bool {
        let bit = 1u64 << idx;
        if self.module_flags & bit != 0 {
            return false;
        }
        self.module_flags |= bit;
        true
    }
    pub fn completed_module_count(&self) -> u32 {
        self.module_flags.count_ones()
    }
    pub fn score_x10(&self) -> u16 {
        (self.score_and_meta >> 48) as u16
    }
    pub fn completion_pct(&self) -> u8 {
        ((self.score_and_meta >> 40) & 0xFF) as u8
    }
    pub fn started_ledger(&self) -> u32 {
        (self.score_and_meta & 0xFFFF_FFFF) as u32
    }
    pub fn set_score_x10(&mut self, score: u16) {
        self.score_and_meta = (self.score_and_meta & !(0xFFFFu64 << 48)) | ((score as u64) << 48);
    }
    pub fn set_completion_pct(&mut self, pct: u8) {
        self.score_and_meta = (self.score_and_meta & !(0xFFu64 << 40)) | ((pct as u64) << 40);
    }
    pub fn set_started_ledger(&mut self, ledger: u32) {
        self.score_and_meta = (self.score_and_meta & !0xFFFF_FFFFu64) | (ledger as u64);
    }
    pub fn is_completed(&self, total: u8) -> bool {
        self.completed_module_count() >= total as u32
    }
}

fn progress_key(learner: &Address, course_id: u32) -> (Symbol, Address, u32) {
    (KEY_PROGRESS, learner.clone(), course_id)
}

fn load_progress(env: &Env, learner: &Address, course_id: u32) -> PackedProgress {
    env.storage()
        .persistent()
        .get(&progress_key(learner, course_id))
        .unwrap_or_default()
}

fn save_progress(env: &Env, learner: &Address, course_id: u32, prog: &PackedProgress) {
    let key = progress_key(learner, course_id);
    env.storage().persistent().set(&key, prog);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}

pub fn start_course_optimized(env: &Env, learner: &Address, course_id: u32) {
    learner.require_auth();
    if env
        .storage()
        .persistent()
        .has(&progress_key(learner, course_id))
    {
        return;
    }
    let mut prog = PackedProgress::default();
    prog.set_started_ledger(env.ledger().sequence());
    save_progress(env, learner, course_id, &prog);
}

pub fn complete_module_optimized(
    env: &Env,
    learner: &Address,
    course_id: u32,
    module_idx: u8,
    total_modules: u8,
) -> bool {
    learner.require_auth();
    let mut prog = load_progress(env, learner, course_id);
    if !prog.mark_module_complete(module_idx) {
        return false;
    }
    let pct = ((prog.completed_module_count() as u64 * 100) / total_modules as u64) as u8;
    prog.set_completion_pct(pct);
    save_progress(env, learner, course_id, &prog);
    true
}

pub fn batch_complete_modules(
    env: &Env,
    learner: &Address,
    course_id: u32,
    module_indices: &Vec<u32>,
    total_modules: u8,
) -> BatchResult {
    learner.require_auth();
    let mut prog = load_progress(env, learner, course_id);
    let mut result = BatchResult::new();
    for i in 0..module_indices.len() {
        if let Some(idx) = module_indices.get(i) {
            if idx >= 64 {
                result.skipped += 1;
                continue;
            }
            if prog.mark_module_complete(idx as u8) {
                result.processed += 1;
            } else {
                result.skipped += 1;
            }
        }
    }
    if result.processed > 0 {
        let pct = ((prog.completed_module_count() as u64 * 100) / total_modules as u64) as u8;
        prog.set_completion_pct(pct);
        save_progress(env, learner, course_id, &prog);
    }
    result
}

pub fn update_score_optimized(env: &Env, learner: &Address, course_id: u32, score_x10: u16) {
    learner.require_auth();
    let mut prog = load_progress(env, learner, course_id);
    prog.set_score_x10(score_x10);
    save_progress(env, learner, course_id, &prog);
}

pub fn get_progress(env: &Env, learner: &Address, course_id: u32) -> PackedProgress {
    load_progress(env, learner, course_id)
}

pub fn is_course_complete(env: &Env, learner: &Address, course_id: u32, total_modules: u8) -> bool {
    load_progress(env, learner, course_id).is_completed(total_modules)
}
