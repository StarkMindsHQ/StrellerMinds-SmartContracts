use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env, Symbol, Vec};

fn setup() -> (Env, AssessmentClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Assessment, ());
    let client = AssessmentClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_create_and_publish_assessment() {
    let (env, client, admin) = setup();
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

    let id = client.create_assessment(&instructor, &course_id, &module_id, &config);

    let meta = client.get_assessment_metadata(&id).unwrap();
    assert_eq!(meta.assessment_id, id);
    assert!(!meta.published);

    client.publish_assessment(&admin, &id);
    let meta = client.get_assessment_metadata(&id).unwrap();
    assert!(meta.published);
}

#[test]
fn test_single_choice_grading() {
    let (env, client, admin) = setup();
    let instructor = admin.clone();
    let course_id = Symbol::new(&env, "C1");
    let module_id = Symbol::new(&env, "M1");

    let config = AssessmentConfig {
        time_limit_seconds: 0,
        max_attempts: 1,
        pass_score: 1,
        allow_review: false,
        is_adaptive: false,
        proctoring_required: false,
    };

    let id = client.create_assessment(&instructor, &course_id, &module_id, &config);
    client.publish_assessment(&admin, &id);

    let content_hash: BytesN<32> = env.crypto().sha256(&Bytes::new(&env)).into();
    let options: Vec<QuestionOption> = Vec::new(&env);

    let qid = client.add_question(
        &admin,
        &id,
        &QuestionType::SingleChoice,
        &1u32,
        &1u32,
        &content_hash,
        &options,
        &AnswerKey::SingleChoice(1),
    );

    let student = Address::generate(&env);
    let submission_id = client.start_submission(&student, &id);

    let mut answers: Vec<SubmittedAnswer> = Vec::new(&env);
    answers.push_back(SubmittedAnswer {
        question_id: qid,
        value: SubmittedAnswerValue::SingleChoice(1),
    });

    let submission = client.submit_answers(&student, &submission_id, &answers);

    assert_eq!(submission.score, 1);
    assert_eq!(submission.max_score, 1);
    assert!(submission.passed);
}

#[test]
fn test_adaptive_state_updates() {
    let (env, client, admin) = setup();
    let instructor = admin.clone();
    let course_id = Symbol::new(&env, "C2");
    let module_id = Symbol::new(&env, "M2");

    let config = AssessmentConfig {
        time_limit_seconds: 0,
        max_attempts: 3,
        pass_score: 1,
        allow_review: false,
        is_adaptive: true,
        proctoring_required: false,
    };

    let id = client.create_assessment(&instructor, &course_id, &module_id, &config);
    client.publish_assessment(&admin, &id);

    let options: Vec<QuestionOption> = Vec::new(&env);
    for difficulty in 1u32..=3u32 {
        let content_hash: BytesN<32> = env.crypto().sha256(&Bytes::new(&env)).into();
        client.add_question(
            &admin,
            &id,
            &QuestionType::SingleChoice,
            &1u32,
            &difficulty,
            &content_hash,
            &options,
            &AnswerKey::SingleChoice(1),
        );
    }

    let student = Address::generate(&env);
    let maybe_q = client.get_next_question(&student, &id);
    assert!(maybe_q.is_some());

    let q = maybe_q.unwrap();
    client.update_adaptive_state(&student, &id, &q.question_id, &true);
}
