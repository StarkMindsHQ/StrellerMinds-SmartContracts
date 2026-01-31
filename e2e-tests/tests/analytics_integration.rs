#[test]
fn test_progress_analytics_calculation() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);

    let sessions = vec![
        LearningSession {
            student: student.clone(),
            session_id: BytesN::from_array(&env, &[1; 32]),
            course_id: Symbol::new(&env, "course1"),
            module_id: Symbol::new(&env, "mod1"),
            start_time: 0,
            end_time: 50,
            interactions: 5,
        },
        LearningSession {
            student: student.clone(),
            session_id: BytesN::from_array(&env, &[2; 32]),
            course_id: Symbol::new(&env, "course1"),
            module_id: Symbol::new(&env, "mod2"),
            start_time: 60,
            end_time: 120,
            interactions: 3,
        },
    ];

    for s in sessions.clone() {
        analytics.record_session(s);
    }

    let progress = analytics.calculate_progress(student.clone(), Symbol::new(&env, "course1"));
    assert!(progress.total_time > 0);
    assert_eq!(progress.completed_modules.len(), 2);
}

#[test]
fn test_module_analytics_calculation() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student1 = create_student(&env, 1);
    let student2 = create_student(&env, 2);

    let session1 = LearningSession {
        student: student1.clone(),
        session_id: BytesN::from_array(&env, &[1; 32]),
        course_id: Symbol::new(&env, "course1"),
        module_id: Symbol::new(&env, "module1"),
        start_time: 0,
        end_time: 100,
        interactions: 5,
    };
    analytics.record_session(session1);

    let session2 = LearningSession {
        student: student2.clone(),
        session_id: BytesN::from_array(&env, &[2; 32]),
        course_id: Symbol::new(&env, "course1"),
        module_id: Symbol::new(&env, "module1"),
        start_time: 10,
        end_time: 110,
        interactions: 7,
    };
    analytics.record_session(session2);

    let module_stats = analytics.calculate_module_analytics(Symbol::new(&env, "course1"), Symbol::new(&env, "module1"));
    assert_eq!(module_stats.total_students, 2);
    assert!(module_stats.avg_interactions > 0);
}

#[test]
fn test_generate_progress_report() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);
    let session = LearningSession {
        student: student.clone(),
        session_id: BytesN::from_array(&env, &[1; 32]),
        course_id: Symbol::new(&env, "course1"),
        module_id: Symbol::new(&env, "module1"),
        start_time: 0,
        end_time: 50,
        interactions: 5,
    };
    analytics.record_session(session);

    let report = analytics.generate_progress_report(student.clone(), Symbol::new(&env, "course1"));
    assert!(report.completed_modules.len() > 0);
}

#[test]
fn test_request_ml_insight() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);
    let insight = analytics.request_ml_insight(student.clone(), Symbol::new(&env, "course1"));
    assert!(insight.predicted_score >= 0);
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    let admin = create_admin(&env);
    let new_admin = create_student(&env, 99);

    let analytics = init_contract(&env, &admin);
    analytics.transfer_admin(new_admin.clone());

    let current_admin = analytics.get_admin();
    assert_eq!(current_admin, new_admin);
}

#[test]
fn test_get_student_sessions() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);
    let session = LearningSession {
        student: student.clone(),
        session_id: BytesN::from_array(&env, &[1; 32]),
        course_id: Symbol::new(&env, "course1"),
        module_id: Symbol::new(&env, "module1"),
        start_time: 0,
        end_time: 50,
        interactions: 5,
    };
    analytics.record_session(session);

    let sessions = analytics.get_student_sessions(student.clone());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].student, student);
}

#[test]
fn test_complete_session() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);
    let mut session = LearningSession {
        student: student.clone(),
        session_id: BytesN::from_array(&env, &[1; 32]),
        course_id: Symbol::new(&env, "course1"),
        module_id: Symbol::new(&env, "module1"),
        start_time: 0,
        end_time: 0,
        interactions: 0,
    };

    session.complete(100, 5); // helper method to finalize session
    analytics.record_session(session);

    let sessions = analytics.get_student_sessions(student.clone());
    assert_eq!(sessions[0].end_time, 100);
}

#[test]
fn test_batch_size_limit() {
    let env = Env::default();
    let admin = create_admin(&env);
    let analytics = init_contract(&env, &admin);

    let student = create_student(&env, 1);
    let mut sessions = vec![];
    for i in 0..20 {
        sessions.push(LearningSession {
            student: student.clone(),
            session_id: BytesN::from_array(&env, &[i; 32]),
            course_id: Symbol::new(&env, "course1"),
            module_id: Symbol::new(&env, "mod1"),
            start_time: i * 10,
            end_time: i * 10 + 5,
            interactions: i,
        });
    }

    analytics.batch_record_sessions(sessions.clone());

    let recorded = analytics.get_student_sessions(student);
    assert_eq!(recorded.len(), 20);
}
