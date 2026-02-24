#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    Assessment::initialize(env.clone(), admin.clone()).unwrap();
    (env, admin)
}

#[test]
fn test_create_and_publish_assessment() {
    let (env, admin) = setup();
    let instructor = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE1");
    let module_id = Symbol::new(&env, "MOD1");

    let config = AssessmentConfig {
        time_limit_seconds: 3600,
        max_attempts: 2,
        pass_score: 50,
        allow_review: true,
        is_adaptive: false,
        proctoring_required: false,
    };

    let id = Assessment::create_assessment(
        env.clone(),
        instructor.clone(),
        course_id.clone(),
        module_id.clone(),
        config,
    )
    .unwrap();

    let meta = Assessment::get_assessment_metadata(env.clone(), id).unwrap();
    assert_eq!(meta.assessment_id, id);
    assert!(!meta.published);

    Assessment::publish_assessment(env.clone(), admin.clone(), id).unwrap();
    let meta = Assessment::get_assessment_metadata(env.clone(), id).unwrap();
    assert!(meta.published);
}

#[test]
fn test_single_choice_grading() {
    let (env, admin) = setup();
    let instructor = admin.clone();
    let course_id = Symbol::short("C1");
    let module_id = Symbol::short("M1");

    let config = AssessmentConfig {
        time_limit_seconds: 0,
        max_attempts: 1,
        pass_score: 1,
        allow_review: false,
        is_adaptive: false,
        proctoring_required: false,
    };

    let id = Assessment::create_assessment(
        env.clone(),
        instructor.clone(),
        course_id.clone(),
        module_id.clone(),
        config,
    )
    .unwrap();
    Assessment::publish_assessment(env.clone(), admin.clone(), id).unwrap();

    let qid = Assessment::add_question(
        env.clone(),
        admin.clone(),
        id,
        QuestionType::SingleChoice,
        1,
        1,
        env.crypto().sha256(&Vec::new(&env)),
        Vec::new(&env),
        AnswerKey::SingleChoice(1),
    )
    .unwrap();

    let student = Address::generate(&env);
    let submission_id = Assessment::start_submission(env.clone(), student.clone(), id).unwrap();

    let mut answers: Vec<SubmittedAnswer> = Vec::new(&env);
    answers.push_back(SubmittedAnswer {
        question_id: qid,
        value: SubmittedAnswerValue::SingleChoice(1),
    });

    let submission =
        Assessment::submit_answers(env.clone(), student.clone(), submission_id.clone(), answers)
            .unwrap();

    assert_eq!(submission.score, 1);
    assert_eq!(submission.max_score, 1);
    assert!(submission.passed);
}

#[test]
fn test_adaptive_state_updates() {
    let (env, admin) = setup();
    let instructor = admin.clone();
    let course_id = Symbol::short("C2");
    let module_id = Symbol::short("M2");

    let config = AssessmentConfig {
        time_limit_seconds: 0,
        max_attempts: 3,
        pass_score: 1,
        allow_review: false,
        is_adaptive: true,
        proctoring_required: false,
    };

    let id = Assessment::create_assessment(
        env.clone(),
        instructor.clone(),
        course_id.clone(),
        module_id.clone(),
        config,
    )
    .unwrap();
    Assessment::publish_assessment(env.clone(), admin.clone(), id).unwrap();

    for difficulty in 1..=3 {
        Assessment::add_question(
            env.clone(),
            admin.clone(),
            id,
            QuestionType::SingleChoice,
            1,
            difficulty,
            env.crypto().sha256(&Vec::new(&env)),
            Vec::new(&env),
            AnswerKey::SingleChoice(1),
        )
        .unwrap();
    }

    let student = Address::generate(&env);
    let maybe_q = Assessment::get_next_question(env.clone(), student.clone(), id).unwrap();
    assert!(maybe_q.is_some());

    let q = maybe_q.unwrap();
    Assessment::update_adaptive_state(env.clone(), student.clone(), id, q.question_id, true)
        .unwrap();
}

