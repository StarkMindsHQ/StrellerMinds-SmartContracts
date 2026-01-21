//! Test data generators for realistic learning scenarios

use soroban_sdk::{Address, BytesN, Symbol, Env};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::test_utils::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningScenario {
    pub course_id: Symbol,
    pub sessions: Vec<LearningSession>,
    pub expected_outcome: ScenarioOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioOutcome {
    Excellent,
    Good,
    Average,
    Poor,
}

/// Create realistic learning sessions for a student
pub fn create_realistic_learning_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::from_entropy();
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (30 * 24 * 60 * 60); // 30 days ago
    
    let mut sessions = Vec::new();
    
    // Create sessions over a month with realistic patterns
    for day in 0..30 {
        // Skip some days to simulate breaks
        if rng.gen_range(0..10) < 2 {
            continue;
        }
        
        let day_start = base_time + (day * 24 * 60 * 60);
        
        // 1-3 sessions per day
        let sessions_today = rng.gen_range(1..4);
        for session_num in 0..sessions_today {
            let session_start = day_start + (session_num * 6 * 60 * 60) + rng.gen_range(0..2 * 60 * 60);
            
            let session = LearningSession {
                session_id: generate_session_id(&mut rng, day, session_num),
                student: Address::from_string(student_address),
                course_id: Symbol::from_str(&Env::default(), "intro_to_rust"),
                module_id: Symbol::from_str(&Env::default(), &format!("module_{}", day % 8 + 1)),
                start_time: session_start,
                end_time: 0, // Will be set when completed
                completion_percentage: 0, // Will be set when completed
                time_spent: 0, // Will be calculated
                interactions: rng.gen_range(5..25),
                score: None, // Will be set when completed
                session_type: match session_num {
                    0 => SessionType::Study,
                    1 => SessionType::Practice,
                    _ => SessionType::Review,
                },
            };
            
            sessions.push(session);
        }
    }
    
    sessions
}

/// Create batch learning sessions for testing
pub fn create_batch_learning_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::from_entropy();
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut sessions = Vec::new();
    
    // Create 20 sessions for batch testing
    for i in 0..20 {
        let session = LearningSession {
            session_id: generate_session_id(&mut rng, i, 0),
            student: Address::from_string(student_address),
            course_id: Symbol::from_str(&Env::default(), "batch_test_course"),
            module_id: Symbol::from_str(&Env::default(), &format!("batch_module_{}", i % 5 + 1)),
            start_time: base_time + (i * 60 * 60),
            end_time: base_time + (i * 60 * 60) + 3600,
            completion_percentage: rng.gen_range(70..100),
            time_spent: 3600,
            interactions: rng.gen_range(10..30),
            score: Some(rng.gen_range(65..95)),
            session_type: SessionType::Study,
        };
        
        sessions.push(session);
    }
    
    sessions
}

/// Create diverse learning scenarios for testing analytics
pub fn create_diverse_learning_scenarios(student_address: &str) -> Vec<LearningScenario> {
    let mut scenarios = Vec::new();
    
    // Scenario 1: Excellent student - high completion, consistent study
    scenarios.push(LearningScenario {
        course_id: Symbol::from_str(&Env::default(), "advanced_blockchain"),
        sessions: create_excellent_student_sessions(student_address),
        expected_outcome: ScenarioOutcome::Excellent,
    });
    
    // Scenario 2: Good student - steady progress, occasional breaks
    scenarios.push(LearningScenario {
        course_id: Symbol::from_str(&Env::default(), "smart_contract_development"),
        sessions: create_good_student_sessions(student_address),
        expected_outcome: ScenarioOutcome::Good,
    });
    
    // Scenario 3: Average student - inconsistent progress
    scenarios.push(LearningScenario {
        course_id: Symbol::from_str(&Env::default(), "rust_programming"),
        sessions: create_average_student_sessions(student_address),
        expected_outcome: ScenarioOutcome::Average,
    });
    
    // Scenario 4: Struggling student - low completion, long gaps
    scenarios.push(LearningScenario {
        course_id: Symbol::from_str(&Env::default(), "cryptography_basics"),
        sessions: create_struggling_student_sessions(student_address),
        expected_outcome: ScenarioOutcome::Poor,
    });
    
    scenarios
}

/// Create sessions for an excellent student
fn create_excellent_student_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(12345);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (14 * 24 * 60 * 60); // 2 weeks ago
    
    let mut sessions = Vec::new();
    
    for day in 0..14 {
        // Excellent students study almost every day
        if rng.gen_range(0..10) < 9 {
            let day_start = base_time + (day * 24 * 60 * 60);
            
            // 2-3 sessions per day, high engagement
            let sessions_today = rng.gen_range(2..4);
            for session_num in 0..sessions_today {
                let session_start = day_start + (session_num * 4 * 60 * 60);
                let session_duration = rng.gen_range(7200..14400); // 2-4 hours
                
                let session = LearningSession {
                    session_id: generate_session_id(&mut rng, day, session_num),
                    student: Address::from_string(student_address),
                    course_id: Symbol::from_str(&Env::default(), "advanced_blockchain"),
                    module_id: Symbol::from_str(&Env::default(), &format!("module_{}", day % 10 + 1)),
                    start_time: session_start,
                    end_time: session_start + session_duration,
                    completion_percentage: rng.gen_range(85..100),
                    time_spent: session_duration,
                    interactions: rng.gen_range(20..40),
                    score: Some(rng.gen_range(88..98)),
                    session_type: match session_num {
                        0 => SessionType::Study,
                        1 => SessionType::Practice,
                        _ => SessionType::Assessment,
                    },
                };
                
                sessions.push(session);
            }
        }
    }
    
    sessions
}

/// Create sessions for a good student
fn create_good_student_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(67890);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (14 * 24 * 60 * 60);
    
    let mut sessions = Vec::new();
    
    for day in 0..14 {
        // Good students study most days
        if rng.gen_range(0..10) < 7 {
            let day_start = base_time + (day * 24 * 60 * 60);
            
            // 1-2 sessions per day
            let sessions_today = rng.gen_range(1..3);
            for session_num in 0..sessions_today {
                let session_start = day_start + (session_num * 6 * 60 * 60);
                let session_duration = rng.gen_range(3600..7200); // 1-2 hours
                
                let session = LearningSession {
                    session_id: generate_session_id(&mut rng, day, session_num),
                    student: Address::from_string(student_address),
                    course_id: Symbol::from_str(&Env::default(), "smart_contract_development"),
                    module_id: Symbol::from_str(&Env::default(), &format!("module_{}", day % 8 + 1)),
                    start_time: session_start,
                    end_time: session_start + session_duration,
                    completion_percentage: rng.gen_range(75..95),
                    time_spent: session_duration,
                    interactions: rng.gen_range(15..30),
                    score: Some(rng.gen_range(80..92)),
                    session_type: match session_num {
                        0 => SessionType::Study,
                        _ => SessionType::Practice,
                    },
                };
                
                sessions.push(session);
            }
        }
    }
    
    sessions
}

/// Create sessions for an average student
fn create_average_student_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(11111);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (14 * 24 * 60 * 60);
    
    let mut sessions = Vec::new();
    
    for day in 0..14 {
        // Average students study irregularly
        if rng.gen_range(0..10) < 5 {
            let day_start = base_time + (day * 24 * 60 * 60);
            
            // Usually 1 session per day
            let session_start = day_start + rng.gen_range(0..4 * 60 * 60);
            let session_duration = rng.gen_range(1800..5400); // 30 min - 1.5 hours
            
            let session = LearningSession {
                session_id: generate_session_id(&mut rng, day, 0),
                student: Address::from_string(student_address),
                course_id: Symbol::from_str(&Env::default(), "rust_programming"),
                module_id: Symbol::from_str(&Env::default(), &format!("module_{}", day % 6 + 1)),
                start_time: session_start,
                end_time: session_start + session_duration,
                completion_percentage: rng.gen_range(60..85),
                time_spent: session_duration,
                interactions: rng.gen_range(10..20),
                score: Some(rng.gen_range(70..85)),
                session_type: SessionType::Study,
            };
            
            sessions.push(session);
        }
    }
    
    sessions
}

/// Create sessions for a struggling student
fn create_struggling_student_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(22222);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (14 * 24 * 60 * 60);
    
    let mut sessions = Vec::new();
    
    for day in 0..14 {
        // Struggling students study infrequently
        if rng.gen_range(0..10) < 3 {
            let day_start = base_time + (day * 24 * 60 * 60);
            
            // Short, infrequent sessions
            let session_start = day_start + rng.gen_range(0..6 * 60 * 60);
            let session_duration = rng.gen_range(900..3600); // 15 min - 1 hour
            
            let session = LearningSession {
                session_id: generate_session_id(&mut rng, day, 0),
                student: Address::from_string(student_address),
                course_id: Symbol::from_str(&Env::default(), "cryptography_basics"),
                module_id: Symbol::from_str(&Env::default(), &format!("module_{}", day % 4 + 1)),
                start_time: session_start,
                end_time: session_start + session_duration,
                completion_percentage: rng.gen_range(30..65),
                time_spent: session_duration,
                interactions: rng.gen_range(5..15),
                score: Some(rng.gen_range(55..75)),
                session_type: SessionType::Study,
            };
            
            sessions.push(session);
        }
    }
    
    sessions
}

/// Create competitive learning sessions for leaderboard testing
pub fn create_competitive_sessions(
    student_address: &str,
    course_id: Symbol,
    performance_rank: usize,
) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(performance_rank as u64);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (7 * 24 * 60 * 60);
    
    let mut sessions = Vec::new();
    
    // Higher rank = better performance
    let base_score = match performance_rank {
        0 => 95, // Best
        1 => 88,
        2 => 82,
        3 => 75,
        _ => 68, // Worst
    };
    
    for day in 0..7 {
        let day_start = base_time + (day * 24 * 60 * 60);
        
        // 2 sessions per day
        for session_num in 0..2 {
            let session_start = day_start + (session_num * 4 * 60 * 60);
            let session_duration = rng.gen_range(3600..7200);
            
            let session = LearningSession {
                session_id: generate_session_id(&mut rng, day, session_num),
                student: Address::from_string(student_address),
                course_id,
                module_id: Symbol::from_str(&Env::default(), &format!("comp_module_{}", day % 5 + 1)),
                start_time: session_start,
                end_time: session_start + session_duration,
                completion_percentage: (base_score - (day * 2)) as u32,
                time_spent: session_duration,
                interactions: rng.gen_range(15..35),
                score: Some((base_score - (day * 2)) as u32),
                session_type: SessionType::Assessment,
            };
            
            sessions.push(session);
        }
    }
    
    sessions
}

/// Create time-based sessions for metrics aggregation
pub fn create_time_based_session(
    student_address: &str,
    course_id: Symbol,
    start_time: u64,
    day_offset: usize,
) -> LearningSession {
    let mut rng = StdRng::seed_from_u64(day_offset as u64);
    let session_duration = rng.gen_range(1800..7200);
    
    LearningSession {
        session_id: generate_session_id(&mut rng, day_offset, 0),
        student: Address::from_string(student_address),
        course_id,
        module_id: Symbol::from_str(&Env::default(), &format!("metrics_module_{}", day_offset % 7 + 1)),
        start_time,
        end_time: 0, // Will be set when completed
        completion_percentage: 0, // Will be set when completed
        time_spent: 0, // Will be calculated
        interactions: rng.gen_range(10..25),
        score: None, // Will be set when completed
        session_type: SessionType::Study,
    }
}

/// Create sessions for consistency testing
pub fn create_consistency_test_sessions(student_address: &str) -> Vec<LearningSession> {
    let mut rng = StdRng::seed_from_u64(33333);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - (5 * 24 * 60 * 60);
    
    let mut sessions = Vec::new();
    
    // Create 10 sessions over 5 days
    for day in 0..5 {
        let day_start = base_time + (day * 24 * 60 * 60);
        
        // 2 sessions per day
        for session_num in 0..2 {
            let session_start = day_start + (session_num * 6 * 60 * 60);
            let session_duration = 3600; // 1 hour
            
            let session = LearningSession {
                session_id: generate_session_id(&mut rng, day, session_num),
                student: Address::from_string(student_address),
                course_id: Symbol::from_str(&Env::default(), "consistency_test_course"),
                module_id: Symbol::from_str(&Env::default(), &format!("consistency_module_{}", day % 3 + 1)),
                start_time: session_start,
                end_time: 0, // Will be set when completed
                completion_percentage: 0, // Will be set when completed
                time_spent: 0, // Will be calculated
                interactions: 15,
                score: None, // Will be set when completed
                session_type: SessionType::Study,
            };
            
            sessions.push(session);
        }
    }
    
    sessions
}

/// Create edge case session for error testing
pub fn create_edge_case_session(student_address: &str) -> LearningSession {
    let mut rng = StdRng::seed_from_u64(44444);
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    LearningSession {
        session_id: generate_session_id(&mut rng, 0, 0),
        student: Address::from_string(student_address),
        course_id: Symbol::from_str(&Env::default(), "edge_case_course"),
        module_id: Symbol::from_str(&Env::default(), "edge_case_module"),
        start_time: base_time,
        end_time: 0, // Will be set when completed
        completion_percentage: 0, // Will be set when completed
        time_spent: 0, // Will be calculated
        interactions: 10,
        score: None, // Will be set when completed
        session_type: SessionType::Study,
    }
}

/// Create smoke test session for CI/CD testing
pub fn create_smoke_test_session(student_address: &str) -> LearningSession {
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    LearningSession {
        session_id: BytesN::from_array(&[5u8; 32]),
        student: Address::from_string(student_address),
        course_id: Symbol::from_str(&Env::default(), "smoke_test_course"),
        module_id: Symbol::from_str(&Env::default(), "smoke_test_module"),
        start_time: base_time,
        end_time: 0, // Will be set when completed
        completion_percentage: 0, // Will be set when completed
        time_spent: 0, // Will be calculated
        interactions: 5,
        score: None, // Will be set when completed
        session_type: SessionType::Study,
    }
}

/// Generate unique session ID
fn generate_session_id(rng: &mut StdRng, day: usize, session_num: usize) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    let seed = (day as u64) << 32 | (session_num as u64);
    let mut local_rng = StdRng::seed_from_u64(seed);
    
    for byte in bytes.iter_mut() {
        *byte = local_rng.gen_range(0..255);
    }
    
    BytesN::from_array(&bytes)
}

/// Create test configuration for analytics contract
pub fn create_test_config() -> AnalyticsConfig {
    AnalyticsConfig {
        min_session_time: 300, // 5 minutes
        max_session_time: 14400, // 4 hours
        streak_threshold: 48, // 48 hours
        active_threshold: 30, // 30 days
        difficulty_thresholds: DifficultyThresholds {
            easy_completion_rate: 80,
            medium_completion_rate: 60,
            hard_completion_rate: 40,
        },
    }
}
