//! Comprehensive Integration Test Suite for Analytics Contract
//!
//! This test suite validates real-world usage scenarios for the analytics contract,
//! including learning session tracking, progress analytics, leaderboard generation,
//! and performance metrics aggregation.

use anyhow::Result;
use e2e_tests::test_data::*;
use e2e_tests::test_utils::*;
use e2e_tests::{setup_test_harness, E2ETestHarness};
use std::time::{SystemTime, UNIX_EPOCH};

/// Test learning session tracking end-to-end workflow
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_learning_session_tracking_e2e() -> Result<()> {
    let harness = setup_test_harness!();

    // Get contract addresses
    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;
    let student_address = harness.client.get_account_address("alice")?;

    // Initialize analytics contract
    let config = create_test_config();
    let init_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !init_result.trim().is_empty(),
        "Analytics initialization failed"
    );

    // Test 1: Record individual learning sessions
    let sessions = create_realistic_learning_sessions(&student_address);
    let mut session_ids = Vec::new();

    for session in &sessions {
        let session_args = format!("--session '{}'", serde_json::to_string(session)?);

        let result = harness
            .client
            .invoke_contract(analytics_id, "record_session", &[session_args], "alice")
            .await?;

        assert!(!result.trim().is_empty(), "Failed to record session");
        session_ids.push(session.session_id.clone());
    }

    // Test 2: Verify session storage and retrieval
    for session_id in &session_ids {
        let retrieved_session = harness
            .client
            .invoke_contract(
                analytics_id,
                "get_session",
                &[format!("--session_id {session_id}")],
                "alice",
            )
            .await?;

        assert!(
            !retrieved_session.trim().is_empty(),
            "Failed to retrieve session"
        );

        // Parse and validate session data
        let session_data: LearningSession = serde_json::from_str(&retrieved_session)?;
        assert_eq!(session_data.student, student_address);
        assert!(session_data.time_spent > 0);
    }

    // Test 3: Complete sessions with final data
    let completion_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    for (i, session_id) in session_ids.iter().enumerate() {
        let final_score = 75 + (i as u32 * 5); // Scores from 75-95
        let completion_percentage = if i < 3 { 100 } else { 80 + (i as u32 * 5) };

        let result = harness
            .client
            .invoke_contract(
                analytics_id,
                "complete_session",
                &[
                    format!("--session_id {session_id}"),
                    format!("--end_time {completion_time}"),
                    format!("--final_score {final_score}"),
                    format!("--completion_percentage {completion_percentage}"),
                ],
                "alice",
            )
            .await?;

        assert!(!result.trim().is_empty(), "Failed to complete session");
    }

    // Test 4: Batch session updates
    let batch_sessions = create_batch_learning_sessions(&student_address);
    let batch_update = BatchSessionUpdate {
        sessions: batch_sessions,
        update_analytics: true,
        update_leaderboards: false,
    };

    let batch_args = format!("--batch '{}'", serde_json::to_string(&batch_update)?);

    let batch_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "batch_update_sessions",
            &[batch_args],
            "alice",
        )
        .await?;

    assert!(
        !batch_result.trim().is_empty(),
        "Failed to update batch sessions"
    );

    // Test 5: Validate student session history
    let course_id = "intro_to_rust".to_string();
    let student_sessions = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_student_sessions",
            &[
                format!("--student {student_address}"),
                format!("--course_id {course_id}"),
            ],
            "alice",
        )
        .await?;

    assert!(
        !student_sessions.trim().is_empty(),
        "Failed to get student sessions"
    );

    // Parse and validate session count
    let session_list: Vec<String> = serde_json::from_str(&student_sessions)?;
    assert!(
        session_list.len() >= sessions.len(),
        "Not all sessions were stored"
    );

    println!("✅ Learning session tracking E2E test passed");
    Ok(())
}

/// Test progress analytics calculations with real-world scenarios
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_progress_analytics_calculations() -> Result<()> {
    let harness = setup_test_harness!();

    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;
    let student_address = harness.client.get_account_address("bob")?;

    // Initialize contract
    let config = create_test_config();
    harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Create diverse learning scenarios
    let scenarios = create_diverse_learning_scenarios(&student_address);

    for scenario in &scenarios {
        // Record sessions for this scenario
        for session in &scenario.sessions {
            let session_args = format!("--session '{}'", serde_json::to_string(session)?);

            harness
                .client
                .invoke_contract(analytics_id, "record_session", &[session_args], "bob")
                .await?;

            // Complete the session
            let completion_time = session.start_time + session.time_spent;
            harness
                .client
                .invoke_contract(
                    analytics_id,
                    "complete_session",
                    &[
                        format!("--session_id {}", session.session_id),
                        format!("--end_time {completion_time}"),
                        format!("--final_score {}", session.score.unwrap_or(85)),
                        format!("--completion_percentage {}", session.completion_percentage),
                    ],
                    "bob",
                )
                .await?;
        }

        // Test progress analytics calculation
        let progress_analytics = harness
            .client
            .invoke_contract(
                analytics_id,
                "get_progress_analytics",
                &[
                    format!("--student {student_address}"),
                    format!("--course_id {}", scenario.course_id),
                ],
                "bob",
            )
            .await?;

        assert!(
            !progress_analytics.trim().is_empty(),
            "Failed to get progress analytics"
        );

        // Parse and validate analytics
        let analytics: ProgressAnalytics = serde_json::from_str(&progress_analytics)?;

        // Validate calculated metrics
        assert_eq!(analytics.student.to_string(), student_address);
        assert_eq!(analytics.course_id, scenario.course_id);
        assert!(analytics.total_sessions > 0);
        assert!(analytics.total_time_spent > 0);
        assert!(analytics.completion_percentage <= 100);

        // Validate average calculations
        if analytics.total_sessions > 0 {
            assert_eq!(
                analytics.average_session_time,
                analytics.total_time_spent / analytics.total_sessions as u64
            );
        }

        // Validate score calculations
        if let Some(avg_score) = analytics.average_score {
            assert!(avg_score <= 100);
        }

        // Test performance trend calculation
        match analytics.performance_trend {
            PerformanceTrend::Improving
            | PerformanceTrend::Stable
            | PerformanceTrend::Declining
            | PerformanceTrend::Insufficient => {
                // Valid trend
            }
        }

        println!(
            "✅ Progress analytics validated for course {}: {}% completion, {} sessions",
            scenario.course_id, analytics.completion_percentage, analytics.total_sessions
        );
    }

    // Test course-wide analytics
    let course_id = "advanced_blockchain".to_string();
    let course_analytics = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_course_analytics",
            &[format!("--course_id {course_id}")],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !course_analytics.trim().is_empty(),
        "Failed to get course analytics"
    );

    let course_analytics: CourseAnalytics = serde_json::from_str(&course_analytics)?;
    assert_eq!(course_analytics.course_id, course_id);
    assert!(course_analytics.total_students > 0);
    assert!(course_analytics.completion_rate <= 100);

    println!("✅ Progress analytics calculations test passed");
    Ok(())
}

/// Test leaderboard generation and ranking system
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_leaderboard_generation() -> Result<()> {
    let harness = setup_test_harness!();

    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;

    // Initialize contract
    let config = create_test_config();
    harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Create competitive learning scenarios
    let students = ["alice", "bob", "charlie", "dave", "eve"];
    let course_id = "competitive_programming".to_string();

    for (rank, student_name) in students.iter().enumerate() {
        let student_address = harness.client.get_account_address(student_name)?;

        // Create sessions with varying performance (higher rank = better performance)
        let sessions = create_competitive_sessions(&student_address, &course_id, rank);

        for session in &sessions {
            let session_args = format!("--session '{}'", serde_json::to_string(session)?);

            harness
                .client
                .invoke_contract(
                    analytics_id,
                    "record_session",
                    &[session_args],
                    student_name,
                )
                .await?;

            // Complete session
            let completion_time = session.start_time + session.time_spent;
            harness
                .client
                .invoke_contract(
                    analytics_id,
                    "complete_session",
                    &[
                        format!("--session_id {}", session.session_id),
                        format!("--end_time {completion_time}"),
                        format!("--final_score {}", session.score.unwrap()),
                        format!("--completion_percentage {}", session.completion_percentage),
                    ],
                    student_name,
                )
                .await?;
        }
    }

    // Test different leaderboard metrics
    let metrics = vec![
        LeaderboardMetric::TotalScore,
        LeaderboardMetric::CompletionSpeed,
        LeaderboardMetric::TimeSpent,
        LeaderboardMetric::ConsistencyScore,
    ];

    for metric in &metrics {
        // Generate leaderboard
        let leaderboard_result = harness
            .client
            .invoke_contract(
                analytics_id,
                "generate_leaderboard",
                &[
                    format!("--course_id {course_id}"),
                    format!("--metric {metric:?}"),
                    "--limit 10".to_string(),
                ],
                &harness.client.config.admin_account,
            )
            .await?;

        assert!(
            !leaderboard_result.trim().is_empty(),
            "Failed to generate leaderboard"
        );

        // Parse and validate leaderboard
        let leaderboard: Vec<LeaderboardEntry> = serde_json::from_str(&leaderboard_result)?;
        assert!(!leaderboard.is_empty(), "Leaderboard should not be empty");

        // Validate ranking (should be in descending order)
        for i in 1..leaderboard.len() {
            let current = &leaderboard[i];
            let previous = &leaderboard[i - 1];

            assert_eq!(current.rank, (i + 1) as u32);
            assert_eq!(current.course_id, course_id);
            assert_eq!(current.metric_type, *metric);

            // Scores should be in descending order
            assert!(current.score <= previous.score);
        }

        println!(
            "✅ Leaderboard generated for {:?}: {} students ranked",
            metric,
            leaderboard.len()
        );
    }

    // Test top performers retrieval
    let top_performers = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_top_performers",
            &[
                format!("--course_id {course_id}"),
                format!("--metric {:?}", LeaderboardMetric::TotalScore),
                "--limit 3".to_string(),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !top_performers.trim().is_empty(),
        "Failed to get top performers"
    );

    let top_list: Vec<LeaderboardEntry> = serde_json::from_str(&top_performers)?;
    assert!(
        top_list.len() <= 3,
        "Should return at most 3 top performers"
    );

    // Test struggling students identification
    let struggling_students = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_struggling_students",
            &[
                format!("--course_id {course_id}"),
                "--threshold 50".to_string(),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !struggling_students.trim().is_empty(),
        "Failed to get struggling students"
    );

    println!("✅ Leaderboard generation test passed");
    Ok(())
}

/// Test performance metrics aggregation and reporting
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_performance_metrics_aggregation() -> Result<()> {
    let harness = setup_test_harness!();

    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;

    // Initialize contract
    let config = create_test_config();
    harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Create time-based learning activity
    let base_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - (7 * 24 * 60 * 60); // 7 days ago

    let course_id = "data_science_fundamentals".to_string();
    let students = vec!["alice", "bob", "charlie"];

    // Generate activity over the past week
    for day in 0..7 {
        let day_start = base_time + (day * 24 * 60 * 60);

        for student_name in &students {
            let student_address = harness.client.get_account_address(student_name)?;

            // Create 2-4 sessions per day per student
            let sessions_per_day = 2 + (day % 3);
            for session_num in 0..sessions_per_day {
                let session = create_time_based_session(
                    &student_address,
                    &course_id,
                    day_start + (session_num * 4 * 60 * 60), // 4 hours apart
                    day as usize,
                );

                let session_args = format!("--session '{}'", serde_json::to_string(&session)?);

                harness
                    .client
                    .invoke_contract(
                        analytics_id,
                        "record_session",
                        &[session_args],
                        student_name,
                    )
                    .await?;

                // Complete session
                let completion_time = session.start_time + session.time_spent;
                harness
                    .client
                    .invoke_contract(
                        analytics_id,
                        "complete_session",
                        &[
                            format!("--session_id {}", session.session_id),
                            format!("--end_time {completion_time}"),
                            format!("--final_score {}", session.score.unwrap()),
                            format!("--completion_percentage {}", session.completion_percentage),
                        ],
                        student_name,
                    )
                    .await?;
            }
        }
    }

    // Test daily metrics aggregation
    for day in 0..7 {
        let day_timestamp = base_time + (day * 24 * 60 * 60);

        let daily_metrics = harness
            .client
            .invoke_contract(
                analytics_id,
                "generate_daily_metrics",
                &[
                    format!("--course_id {course_id}"),
                    format!("--date {day_timestamp}"),
                ],
                &harness.client.config.admin_account,
            )
            .await?;

        assert!(
            !daily_metrics.trim().is_empty(),
            "Failed to generate daily metrics"
        );

        let metrics: AggregatedMetrics = serde_json::from_str(&daily_metrics)?;
        assert_eq!(metrics.course_id, course_id);
        assert_eq!(metrics.date, day_timestamp);
        assert!(metrics.active_students > 0);
        assert!(metrics.total_sessions > 0);
        assert!(metrics.total_time > 0);

        println!(
            "Day {}: {} active students, {} sessions, {} total time",
            day, metrics.active_students, metrics.total_sessions, metrics.total_time
        );
    }

    // Test weekly summary
    let week_start = base_time;
    let weekly_summary = harness
        .client
        .invoke_contract(
            analytics_id,
            "generate_weekly_summary",
            &[
                format!("--course_id {course_id}"),
                format!("--week_start {week_start}"),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !weekly_summary.trim().is_empty(),
        "Failed to generate weekly summary"
    );

    let weekly_data: Vec<AggregatedMetrics> = serde_json::from_str(&weekly_summary)?;
    assert_eq!(
        weekly_data.len(),
        7,
        "Weekly summary should have 7 days of data"
    );

    // Test completion trends
    let trends = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_completion_trends",
            &[
                format!("--course_id {course_id}"),
                format!("--start_date {base_time}"),
                format!("--end_date {}", base_time + (7 * 24 * 60 * 60)),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(!trends.trim().is_empty(), "Failed to get completion trends");

    let trend_data: Vec<AggregatedMetrics> = serde_json::from_str(&trends)?;
    assert!(!trend_data.is_empty(), "Trend data should not be empty");

    // Test progress reports for individual students
    for student_name in &students {
        let student_address = harness.client.get_account_address(student_name)?;

        let progress_report = harness
            .client
            .invoke_contract(
                analytics_id,
                "generate_progress_report",
                &[
                    format!("--student {student_address}"),
                    format!("--course_id {course_id}"),
                    "--period Weekly".to_string(),
                    format!("--start_date {base_time}"),
                    format!("--end_date {}", base_time + (7 * 24 * 60 * 60)),
                ],
                student_name,
            )
            .await?;

        assert!(
            !progress_report.trim().is_empty(),
            "Failed to generate progress report"
        );

        let report: ProgressReport = serde_json::from_str(&progress_report)?;
        assert_eq!(report.student.to_string(), student_address);
        assert_eq!(report.course_id, course_id);
        assert!(report.sessions_count > 0);
        assert!(report.total_time > 0);

        println!(
            "Student {} report: {} sessions, {} hours, {}% consistency",
            student_name,
            report.sessions_count,
            report.total_time / 3600,
            report.consistency_score
        );
    }

    println!("✅ Performance metrics aggregation test passed");
    Ok(())
}

/// Test data consistency across contract operations
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_data_consistency_validation() -> Result<()> {
    let harness = setup_test_harness!();

    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;
    let student_address = harness.client.get_account_address("alice")?;

    // Initialize contract
    let config = create_test_config();
    harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Create test sessions
    let sessions = create_consistency_test_sessions(&student_address);
    let mut session_ids = Vec::new();

    // Record sessions
    for session in &sessions {
        let session_args = format!("--session '{}'", serde_json::to_string(session)?);

        harness
            .client
            .invoke_contract(analytics_id, "record_session", &[session_args], "alice")
            .await?;

        session_ids.push(session.session_id.clone());
    }

    // Complete sessions
    for session_id in &session_ids {
        let completion_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        harness
            .client
            .invoke_contract(
                analytics_id,
                "complete_session",
                &[
                    format!("--session_id {session_id}"),
                    format!("--end_time {completion_time}"),
                    "--final_score 85".to_string(),
                    "--completion_percentage 100".to_string(),
                ],
                "alice",
            )
            .await?;
    }

    // Validate data consistency across different views

    // 1. Check that student sessions match individual session data
    let course_id = "consistency_test_course".to_string();
    let student_sessions = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_student_sessions",
            &[
                format!("--student {student_address}"),
                format!("--course_id {course_id}"),
            ],
            "alice",
        )
        .await?;

    let session_list: Vec<String> = serde_json::from_str(&student_sessions)?;
    assert_eq!(session_list.len(), sessions.len(), "Session count mismatch");

    // 2. Verify progress analytics match session data
    let progress_analytics = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_progress_analytics",
            &[
                format!("--student {student_address}"),
                format!("--course_id {course_id}"),
            ],
            "alice",
        )
        .await?;

    let analytics: ProgressAnalytics = serde_json::from_str(&progress_analytics)?;
    assert_eq!(analytics.total_sessions, sessions.len() as u32);

    // 3. Validate course analytics aggregate student data correctly
    let course_analytics = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_course_analytics",
            &[format!("--course_id {course_id}")],
            &harness.client.config.admin_account,
        )
        .await?;

    let course_data: CourseAnalytics = serde_json::from_str(&course_analytics)?;
    assert!(course_data.total_students >= 1);

    // 4. Test filtered session queries
    let filter = AnalyticsFilter {
        course_id: Some(course_id.clone()),
        student: Some(student_address.clone()),
        start_date: Some(0),
        end_date: Some(u64::MAX),
        session_type: OptionalSessionType::None,
        min_score: Some(80),
    };

    let filtered_sessions = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_filtered_sessions",
            &[format!("--filter '{}'", serde_json::to_string(&filter)?)],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !filtered_sessions.trim().is_empty(),
        "Filtered sessions should not be empty"
    );

    let _filtered_data: Vec<LearningSession> = serde_json::from_str(&filtered_sessions)?;

    // 5. Verify leaderboard consistency
    let leaderboard = harness
        .client
        .invoke_contract(
            analytics_id,
            "generate_leaderboard",
            &[
                format!("--course_id {course_id}"),
                format!("--metric {:?}", LeaderboardMetric::TotalScore),
                "--limit 10".to_string(),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    let leaderboard_data: Vec<LeaderboardEntry> = serde_json::from_str(&leaderboard)?;

    // Find our student in the leaderboard
    let student_in_leaderboard = leaderboard_data
        .iter()
        .find(|entry| entry.student == student_address);

    assert!(
        student_in_leaderboard.is_some(),
        "Student should appear in leaderboard"
    );

    // 6. Test data integrity after recalculation
    harness
        .client
        .invoke_contract(
            analytics_id,
            "recalculate_course_analytics",
            &[
                format!("--admin {admin_address}"),
                format!("--course_id {course_id}"),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Verify analytics remain consistent after recalculation
    let recalculated_analytics = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_progress_analytics",
            &[
                format!("--student {student_address}"),
                format!("--course_id {course_id}"),
            ],
            "alice",
        )
        .await?;

    let recalculated_data: ProgressAnalytics = serde_json::from_str(&recalculated_analytics)?;
    assert_eq!(analytics.total_sessions, recalculated_data.total_sessions);
    assert_eq!(
        analytics.completion_percentage,
        recalculated_data.completion_percentage
    );

    println!("✅ Data consistency validation test passed");
    Ok(())
}

/// Test edge cases and error conditions
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_edge_cases_and_error_conditions() -> Result<()> {
    let harness = setup_test_harness!();

    let analytics_id = harness.get_contract_id("analytics").unwrap();
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;
    let student_address = harness.client.get_account_address("alice")?;

    // Initialize contract
    let config = create_test_config();
    harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    // Test 1: Duplicate session recording
    let session = create_edge_case_session(&student_address);
    let session_args = format!("--session '{}'", serde_json::to_string(&session)?);

    // Record session first time (should succeed)
    harness
        .client
        .invoke_contract(
            analytics_id,
            "record_session",
            std::slice::from_ref(&session_args),
            "alice",
        )
        .await?;

    // Try to record same session again (should fail)
    let duplicate_result = harness
        .client
        .invoke_contract(analytics_id, "record_session", &[session_args], "alice")
        .await;

    assert!(duplicate_result.is_err(), "Duplicate session should fail");

    // Test 2: Invalid session data
    let invalid_session = LearningSession {
        session_id: hex::encode([0u8; 32]),
        student: student_address.clone(),
        course_id: "test_course".to_string(),
        module_id: "test_module".to_string(),
        start_time: 1000,
        end_time: 500,              // Invalid: end before start
        completion_percentage: 150, // Invalid: > 100
        time_spent: 0,
        interactions: 0,
        score: Some(150), // Invalid: > 100
        session_type: SessionType::Study,
    };

    let invalid_session_args = format!("--session '{}'", serde_json::to_string(&invalid_session)?);

    let invalid_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "record_session",
            &[invalid_session_args],
            "alice",
        )
        .await;

    assert!(invalid_result.is_err(), "Invalid session should fail");

    // Test 3: Non-existent session retrieval
    let non_existent_id = hex::encode([1u8; 32]);
    let retrieval_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_session",
            &[format!("--session_id {non_existent_id}")],
            "alice",
        )
        .await?;

    assert!(
        retrieval_result.trim().is_empty() || retrieval_result.contains("null"),
        "Non-existent session should return empty or null"
    );

    // Test 4: Analytics for non-existent student/course
    let non_existent_student =
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAF2".to_string();
    let non_existent_course = "non_existent_course".to_string();

    let analytics_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_progress_analytics",
            &[
                format!("--student {non_existent_student}"),
                format!("--course_id {non_existent_course}"),
            ],
            "alice",
        )
        .await;

    assert!(
        analytics_result.is_err(),
        "Analytics for non-existent data should fail"
    );

    // Test 5: Batch operations with invalid data
    let invalid_batch = vec![session, invalid_session];

    let batch_update = BatchSessionUpdate {
        sessions: invalid_batch,
        update_analytics: true,
        update_leaderboards: false,
    };

    let batch_args = format!("--batch '{}'", serde_json::to_string(&batch_update)?);

    let batch_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "batch_update_sessions",
            &[batch_args],
            "alice",
        )
        .await;

    // Should either fail completely or process only valid sessions
    match batch_result {
        Ok(result) => {
            let processed: u32 = serde_json::from_str(&result)?;
            assert!(processed <= 1, "Should process at most 1 valid session");
        }
        Err(_) => {
            // Complete failure is also acceptable
        }
    }

    // Test 6: Unauthorized operations
    let unauthorized_address = harness.client.get_account_address("bob")?;

    let unauthorized_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "update_config",
            &[
                format!("--admin {unauthorized_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            "bob",
        )
        .await;

    assert!(
        unauthorized_result.is_err(),
        "Unauthorized config update should fail"
    );

    // Test 7: Edge case analytics calculations
    // Create a session with minimal valid data
    let minimal_session = LearningSession {
        session_id: hex::encode([2u8; 32]),
        student: student_address.clone(),
        course_id: "minimal_course".to_string(),
        module_id: "minimal_module".to_string(),
        start_time: 1000,
        end_time: 2000,
        completion_percentage: 0, // No completion
        time_spent: 1000,
        interactions: 1,
        score: None,
        session_type: SessionType::Study,
    };

    let minimal_session_args = format!("--session '{}'", serde_json::to_string(&minimal_session)?);

    harness
        .client
        .invoke_contract(
            analytics_id,
            "record_session",
            &[minimal_session_args],
            "alice",
        )
        .await?;

    // Complete with minimal score
    harness
        .client
        .invoke_contract(
            analytics_id,
            "complete_session",
            &[
                format!("--session_id {}", minimal_session.session_id),
                "--end_time 2000".to_string(),
                "--final_score 0".to_string(),
                "--completion_percentage 0".to_string(),
            ],
            "alice",
        )
        .await?;

    // Test analytics with minimal data
    let minimal_analytics = harness
        .client
        .invoke_contract(
            analytics_id,
            "get_progress_analytics",
            &[
                format!("--student {student_address}"),
                format!("--course_id {}", "minimal_course"),
            ],
            "alice",
        )
        .await?;

    assert!(
        !minimal_analytics.trim().is_empty(),
        "Minimal analytics should not be empty"
    );

    let minimal_data: ProgressAnalytics = serde_json::from_str(&minimal_analytics)?;
    assert_eq!(minimal_data.total_sessions, 1);
    assert_eq!(minimal_data.completion_percentage, 0);
    assert!(minimal_data.average_score.is_none());

    println!("✅ Edge cases and error conditions test passed");
    Ok(())
}

/// Test CI/CD pipeline integration
#[tokio::test]
#[ignore = "requires running Soroban localnet at localhost:8000"]
async fn test_cicd_pipeline_integration() -> Result<()> {
    let harness = setup_test_harness!();

    // Test that all contracts deploy successfully
    assert!(
        harness.deployed_contracts.contains_key("analytics"),
        "Analytics contract should be deployed"
    );

    let analytics_id = harness.get_contract_id("analytics").unwrap();

    // Test contract initialization
    let admin_address = harness
        .client
        .get_account_address(&harness.client.config.admin_account)?;
    let config = create_test_config();

    let init_result = harness
        .client
        .invoke_contract(
            analytics_id,
            "initialize",
            &[
                format!("--admin {admin_address}"),
                format!("--config {}", serde_json::to_string(&config)?),
            ],
            &harness.client.config.admin_account,
        )
        .await?;

    assert!(
        !init_result.trim().is_empty(),
        "Contract initialization should succeed"
    );

    // Test basic functionality smoke test
    let student_address = harness.client.get_account_address("alice")?;
    let session = create_smoke_test_session(&student_address);

    let session_args = format!("--session '{}'", serde_json::to_string(&session)?);

    let record_result = harness
        .client
        .invoke_contract(analytics_id, "record_session", &[session_args], "alice")
        .await?;

    assert!(
        !record_result.trim().is_empty(),
        "Session recording should succeed"
    );

    // Test that the contract responds to health checks
    let config_result = harness
        .client
        .invoke_contract(analytics_id, "get_config", &[], "alice")
        .await?;

    assert!(
        !config_result.trim().is_empty(),
        "Config retrieval should succeed"
    );

    // Test that all major functions are callable
    let course_id = "smoke_test_course".to_string();

    // Test analytics functions
    let functions_to_test = vec![
        (
            "get_course_analytics",
            vec![format!("--course_id {}", course_id.to_string())],
        ),
        (
            "get_progress_analytics",
            vec![
                format!("--student {}", student_address),
                format!("--course_id {}", course_id.to_string()),
            ],
        ),
        (
            "get_student_sessions",
            vec![
                format!("--student {}", student_address),
                format!("--course_id {}", course_id.to_string()),
            ],
        ),
    ];

    for (function, args) in functions_to_test {
        let result = harness
            .client
            .invoke_contract(analytics_id, function, &args, "alice")
            .await;

        // Some functions might fail due to insufficient data, but they should not crash
        match result {
            Ok(_) => println!("✅ {function} function responds"),
            Err(e) => println!("⚠️ {function} function returns error (expected): {e}"),
        }
    }

    println!("✅ CI/CD pipeline integration test passed");
    Ok(())
}
