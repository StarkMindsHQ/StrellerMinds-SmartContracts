use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env, String};

use crate::types::{
    Achievement, AchievementCategory, AchievementRequirements, AchievementTier, ActivityRecord,
    ActivityType, Challenge, ChallengeDifficulty, ChallengeType, LeaderboardCategory,
    RecognitionType, Season,
};
use crate::{Gamification, GamificationClient};

// ─── Test Helpers ────────────────────────────────────────────────────────────

fn setup_env() -> (Env, GamificationClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Gamification, ());
    let client = GamificationClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    (env, client, admin)
}

fn make_activity(env: &Env, activity_type: ActivityType, ts: u64) -> ActivityRecord {
    ActivityRecord {
        activity_type,
        course_id: String::from_str(env, "COURSE_001"),
        module_id: String::from_str(env, "MODULE_001"),
        score: 85,
        time_spent: 3600,
        timestamp: ts,
    }
}

fn make_challenge(env: &Env, admin: &Address, now: u64) -> Challenge {
    Challenge {
        id: 0,
        name: String::from_str(env, "Speed Learner"),
        description: String::from_str(env, "Complete 3 modules in 7 days"),
        challenge_type: ChallengeType::Individual,
        difficulty: ChallengeDifficulty::Intermediate,
        xp_reward: 300,
        token_reward: 3_000,
        start_time: now,
        end_time: now + 7 * 86_400,
        target_progress: 3,
        max_participants: 0,
        current_participants: 0,
        is_active: false,
        prerequisite_challenge_id: 0,
        created_by: admin.clone(),
        created_at: 0,
    }
}

// ─── Initialization ───────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Gamification, ());
    let client = GamificationClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, Some(admin));
}

#[test]
#[should_panic]
fn test_double_initialize_panics() {
    let (_, client, admin) = setup_env();
    // Second call should fail
    client.initialize(&admin);
}

// ─── Activity & XP ───────────────────────────────────────────────────────────

#[test]
fn test_module_completion_awards_xp() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let activity = make_activity(&env, ActivityType::ModuleCompleted, 1_000_000);
    client.record_activity(&student, &activity);

    let profile = client.get_user_profile(&student);
    assert!(profile.total_xp > 0, "student should have earned XP");
    assert_eq!(profile.modules_completed, 1);
}

#[test]
fn test_course_completion_awards_more_xp_than_module() {
    let (env, client, _admin) = setup_env();
    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let module_act = make_activity(&env, ActivityType::ModuleCompleted, 1_000_000);
    let course_act = make_activity(&env, ActivityType::CourseCompleted, 1_000_000);

    client.record_activity(&s1, &module_act);
    client.record_activity(&s2, &course_act);

    let p1 = client.get_user_profile(&s1);
    let p2 = client.get_user_profile(&s2);

    assert!(
        p2.total_xp > p1.total_xp,
        "course completion should award more XP than module"
    );
}

#[test]
fn test_study_session_awards_time_based_xp() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 2_000_000);
    let activity = ActivityRecord {
        activity_type: ActivityType::StudySession,
        course_id: String::from_str(&env, "COURSE_001"),
        module_id: String::from_str(&env, ""),
        score: 0,
        time_spent: 3600, // 1 hour = 60 mins → capped at 30 XP
        timestamp: 2_000_000,
    };
    client.record_activity(&student, &activity);

    let profile = client.get_user_profile(&student);
    assert_eq!(profile.total_xp, 30);
}

#[test]
fn test_assessment_score_scales_xp() {
    let (env, client, _admin) = setup_env();
    let s_high = Address::generate(&env);
    let s_low = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let high_score = ActivityRecord {
        activity_type: ActivityType::AssessmentPassed,
        course_id: String::from_str(&env, "C1"),
        module_id: String::from_str(&env, "M1"),
        score: 100,
        time_spent: 1800,
        timestamp: 1_000_000,
    };
    let low_score = ActivityRecord {
        activity_type: ActivityType::AssessmentPassed,
        course_id: String::from_str(&env, "C1"),
        module_id: String::from_str(&env, "M1"),
        score: 50,
        time_spent: 1800,
        timestamp: 1_000_000,
    };

    client.record_activity(&s_high, &high_score);
    client.record_activity(&s_low, &low_score);

    let ph = client.get_user_profile(&s_high);
    let pl = client.get_user_profile(&s_low);
    assert!(
        ph.total_xp > pl.total_xp,
        "higher score should give more XP"
    );
}

// ─── Streak ───────────────────────────────────────────────────────────────────

#[test]
fn test_consecutive_day_streak_grows() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    // Day 1
    env.ledger().with_mut(|l| l.timestamp = 0);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::ModuleCompleted, 0),
    );

    // Day 2 (next day)
    let day2 = 86_400u64;
    env.ledger().with_mut(|l| l.timestamp = day2);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::ModuleCompleted, day2),
    );

    let profile = client.get_user_profile(&student);
    assert_eq!(profile.current_streak, 2);
    assert_eq!(profile.max_streak, 2);
}

#[test]
fn test_missed_day_resets_streak() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    // Day 1
    env.ledger().with_mut(|l| l.timestamp = 0);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::ModuleCompleted, 0),
    );

    // Day 2
    env.ledger().with_mut(|l| l.timestamp = 86_400);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::ModuleCompleted, 86_400),
    );

    // Day 5 (2-day gap → streak broken)
    let day5 = 86_400 * 4;
    env.ledger().with_mut(|l| l.timestamp = day5);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::ModuleCompleted, day5),
    );

    let profile = client.get_user_profile(&student);
    assert_eq!(profile.current_streak, 1, "streak should reset after gap");
    assert_eq!(profile.max_streak, 2, "max_streak should be preserved");
}

// ─── Achievements ─────────────────────────────────────────────────────────────

#[test]
fn test_first_course_achievement_awarded() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let activity = make_activity(&env, ActivityType::CourseCompleted, 1_000_000);
    let new_achievements = client.record_activity(&student, &activity);

    // Achievement ID 1 = "First Step" (complete first course)
    assert!(
        new_achievements.contains(1u64),
        "should award 'First Step' achievement on first course"
    );
}

#[test]
fn test_admin_can_create_custom_achievement() {
    let (env, client, admin) = setup_env();

    let ach = Achievement {
        id: 0,
        name: String::from_str(&env, "Blockchain Pioneer"),
        description: String::from_str(&env, "Complete the blockchain fundamentals course"),
        tier: AchievementTier::Gold,
        category: AchievementCategory::Learning,
        xp_reward: 1_000,
        token_reward: 10_000,
        requirements: AchievementRequirements {
            courses_completed: 1,
            modules_completed: 0,
            streak_days: 0,
            total_xp: 0,
            challenges_completed: 0,
            endorsements_received: 0,
            guild_contributions: 0,
            seasons_completed: 0,
        },
        created_at: 0,
        is_active: false,
        is_cross_course: true,
    };

    let id = client.create_achievement(&admin, &ach);
    assert!(
        id > 25,
        "custom achievement ID should be beyond reserved block"
    );
}

#[test]
fn test_claim_achievement_reward() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let activity = make_activity(&env, ActivityType::CourseCompleted, 1_000_000);
    let new_achievements = client.record_activity(&student, &activity);

    if !new_achievements.is_empty() {
        let ach_id = new_achievements.get(0).unwrap();
        let tokens = client.claim_achievement_reward(&student, &ach_id);
        assert!(tokens > 0, "claimed achievement should yield tokens");

        // Double-claim should fail
        let result = client.try_claim_achievement_reward(&student, &ach_id);
        assert!(result.is_err(), "second claim should be rejected");
    }
}

// ─── Leaderboard ─────────────────────────────────────────────────────────────

#[test]
fn test_leaderboard_populates_after_activity() {
    let (env, client, _admin) = setup_env();
    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // s2 completes a course (more XP); s1 completes a module
    client.record_activity(
        &s1,
        &make_activity(&env, ActivityType::ModuleCompleted, 1_000_000),
    );
    client.record_activity(
        &s2,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );

    let board = client.get_leaderboard(&LeaderboardCategory::TotalXP, &10u32);
    assert!(
        board.len() >= 2,
        "leaderboard should have at least 2 entries"
    );

    // Top entry should be s2 (more XP)
    let top = board.get(0).unwrap();
    assert_eq!(top.user, s2, "student with course completion should be #1");
    assert_eq!(top.rank, 1);
}

#[test]
fn test_leaderboard_limit_respected() {
    let (env, client, _admin) = setup_env();

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // Register 5 students
    for _ in 0..5 {
        let s = Address::generate(&env);
        client.record_activity(
            &s,
            &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
        );
    }

    let board = client.get_leaderboard(&LeaderboardCategory::TotalXP, &3u32);
    assert!(board.len() <= 3, "leaderboard should respect limit");
}

// ─── Challenges ───────────────────────────────────────────────────────────────

#[test]
fn test_create_and_join_challenge() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let now = 1_000_000u64;

    let challenge = make_challenge(&env, &admin, now);
    let challenge_id = client.create_challenge(&admin, &challenge);

    client.join_challenge(&student, &challenge_id);

    let status = client.get_user_challenge_status(&student, &challenge_id);
    assert!(status.is_some(), "student should have challenge status");
    let uc = status.unwrap();
    assert_eq!(uc.challenge_id, challenge_id);
    assert!(!uc.completed);
    assert_eq!(uc.current_progress, 0);
}

#[test]
fn test_challenge_completion_awards_xp() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let now = 1_000_000u64;

    let challenge = make_challenge(&env, &admin, now);
    let challenge_id = client.create_challenge(&admin, &challenge);

    client.join_challenge(&student, &challenge_id);

    let before = client.get_user_profile(&student);
    let completed = client.update_challenge_progress(&student, &challenge_id, &3u32);
    assert!(completed, "challenge should be completed");

    let after = client.get_user_profile(&student);
    assert!(
        after.total_xp > before.total_xp,
        "completing a challenge should award XP"
    );
    assert_eq!(after.challenges_completed, 1);
}

#[test]
fn test_cannot_join_challenge_twice() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let now = 1_000_000u64;

    let challenge = make_challenge(&env, &admin, now);
    let challenge_id = client.create_challenge(&admin, &challenge);

    client.join_challenge(&student, &challenge_id);

    let result = client.try_join_challenge(&student, &challenge_id);
    assert!(result.is_err(), "joining twice should fail");
}

#[test]
fn test_quest_chain_prerequisite_enforced() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let now = 1_000_000u64;

    // Create parent challenge
    let parent = make_challenge(&env, &admin, now);
    let parent_id = client.create_challenge(&admin, &parent);

    // Create child that requires parent
    let mut child = make_challenge(&env, &admin, now);
    child.prerequisite_challenge_id = parent_id;
    let child_id = client.create_challenge(&admin, &child);

    // Should fail: prerequisite not met
    let result = client.try_join_challenge(&student, &child_id);
    assert!(
        result.is_err(),
        "should not join child without completing parent"
    );

    // Complete parent, then join child
    client.join_challenge(&student, &parent_id);
    client.update_challenge_progress(&student, &parent_id, &3u32);

    let result2 = client.try_join_challenge(&student, &child_id);
    assert!(result2.is_ok(), "should join child after completing parent");
}

// ─── Guild System ─────────────────────────────────────────────────────────────

#[test]
fn test_create_guild() {
    let (env, client, _admin) = setup_env();
    let creator = Address::generate(&env);

    let guild_id = client.create_guild(
        &creator,
        &String::from_str(&env, "Alpha Guild"),
        &String::from_str(&env, "Top learners"),
        &20u32,
        &true,
    );

    let guild = client.get_guild(&guild_id).expect("guild should exist");
    assert_eq!(guild.id, guild_id);
    assert_eq!(guild.member_count, 1);
    assert_eq!(guild.leader, creator);
}

#[test]
fn test_join_guild() {
    let (env, client, _admin) = setup_env();
    let creator = Address::generate(&env);
    let joiner = Address::generate(&env);

    let guild_id = client.create_guild(
        &creator,
        &String::from_str(&env, "Beta Guild"),
        &String::from_str(&env, "Open guild"),
        &20u32,
        &true,
    );

    client.join_guild(&joiner, &guild_id);

    let guild = client.get_guild(&guild_id).unwrap();
    assert_eq!(guild.member_count, 2);

    let profile = client.get_user_profile(&joiner);
    assert_eq!(profile.guild_id, guild_id);
}

#[test]
fn test_leave_guild() {
    let (env, client, _admin) = setup_env();
    let creator = Address::generate(&env);
    let joiner = Address::generate(&env);

    let guild_id = client.create_guild(
        &creator,
        &String::from_str(&env, "Gamma Guild"),
        &String::from_str(&env, "Test guild"),
        &10u32,
        &true,
    );
    client.join_guild(&joiner, &guild_id);
    client.leave_guild(&joiner);

    let guild = client.get_guild(&guild_id).unwrap();
    assert_eq!(guild.member_count, 1);

    let profile = client.get_user_profile(&joiner);
    assert_eq!(
        profile.guild_id, 0,
        "guild_id should be cleared after leaving"
    );
}

#[test]
fn test_guild_xp_accumulates_from_members() {
    let (env, client, _admin) = setup_env();
    let creator = Address::generate(&env);

    let guild_id = client.create_guild(
        &creator,
        &String::from_str(&env, "XP Guild"),
        &String::from_str(&env, "Guild for XP tests"),
        &10u32,
        &true,
    );

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    client.record_activity(
        &creator,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );

    let guild = client.get_guild(&guild_id).unwrap();
    assert!(
        guild.total_xp > 0,
        "guild should accumulate XP from member activity"
    );
}

#[test]
fn test_cannot_join_two_guilds() {
    let (env, client, _admin) = setup_env();
    let creator = Address::generate(&env);

    let g1 = client.create_guild(
        &creator,
        &String::from_str(&env, "G1"),
        &String::from_str(&env, "First guild"),
        &10u32,
        &true,
    );

    let other_creator = Address::generate(&env);
    let g2 = client.create_guild(
        &other_creator,
        &String::from_str(&env, "G2"),
        &String::from_str(&env, "Second guild"),
        &10u32,
        &true,
    );

    let joiner = Address::generate(&env);
    client.join_guild(&joiner, &g1);

    let result = client.try_join_guild(&joiner, &g2);
    assert!(result.is_err(), "cannot be in two guilds simultaneously");
}

// ─── Seasons ─────────────────────────────────────────────────────────────────

#[test]
fn test_create_season_and_earn_season_xp() {
    let (env, client, admin) = setup_env();

    let now = 2_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);

    let season = Season {
        id: 0,
        name: String::from_str(&env, "Spring Season"),
        description: String::from_str(&env, "Double XP event"),
        start_time: now,
        end_time: now + 30 * 86_400,
        xp_multiplier: 200, // 2×
        is_active: false,
        total_participants: 0,
        reward_pool: 1_000_000,
    };

    let season_id = client.create_season(&admin, &season);
    let active = client.get_active_season();
    assert!(active.is_some(), "season should be active");
    assert_eq!(active.unwrap().id, season_id);

    let student = Address::generate(&env);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::CourseCompleted, now),
    );

    let profile = client.get_user_profile(&student);
    assert!(
        profile.season_xp > 0,
        "should earn season XP during active season"
    );

    // Season XP should reflect the 2× multiplier
    let board = client.get_season_leaderboard(&season_id);
    assert!(!board.is_empty());
}

#[test]
fn test_season_xp_multiplier_applies() {
    let (env, client, admin) = setup_env();

    // Record activity WITHOUT a season
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let student_no_season = Address::generate(&env);
    client.record_activity(
        &student_no_season,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );
    let xp_no_season = client.get_user_profile(&student_no_season).total_xp;

    // Activate a 2× season
    let now = 2_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    let season = Season {
        id: 0,
        name: String::from_str(&env, "Double XP"),
        description: String::from_str(&env, "2x season"),
        start_time: now,
        end_time: now + 30 * 86_400,
        xp_multiplier: 200,
        is_active: false,
        total_participants: 0,
        reward_pool: 0,
    };
    client.create_season(&admin, &season);

    let student_season = Address::generate(&env);
    client.record_activity(
        &student_season,
        &make_activity(&env, ActivityType::CourseCompleted, now),
    );
    let xp_with_season = client.get_user_profile(&student_season).total_xp;

    assert!(
        xp_with_season >= xp_no_season * 2,
        "2× season should at least double XP (got: {xp_with_season} vs {xp_no_season})"
    );
}

// ─── Social & Endorsements ───────────────────────────────────────────────────

#[test]
fn test_peer_endorsement() {
    let (env, client, _admin) = setup_env();
    let endorser = Address::generate(&env);
    let endorsee = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    client.endorse_peer(
        &endorser,
        &endorsee,
        &String::from_str(&env, "Rust Programming"),
    );

    let profile = client.get_user_profile(&endorsee);
    assert_eq!(profile.endorsements_received, 1);
    assert!(profile.total_xp > 0, "endorsee should receive XP");

    let endorsements = client.get_user_endorsements(&endorsee);
    assert_eq!(endorsements.len(), 1);
}

#[test]
fn test_self_endorsement_rejected() {
    let (env, client, _admin) = setup_env();
    let user = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let result = client.try_endorse_peer(&user, &user, &String::from_str(&env, "Self-Taught"));
    assert!(result.is_err(), "self-endorsement should be rejected");
}

#[test]
fn test_endorsement_daily_rate_limit() {
    let (env, client, _admin) = setup_env();
    let endorser = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // Default limit is 5 per day
    for _ in 0..5 {
        let endorsee = Address::generate(&env);
        client.endorse_peer(&endorser, &endorsee, &String::from_str(&env, "Rust"));
    }

    // 6th should fail
    let endorsee6 = Address::generate(&env);
    let result = client.try_endorse_peer(&endorser, &endorsee6, &String::from_str(&env, "Rust"));
    assert!(
        result.is_err(),
        "6th endorsement in same day should be rate limited"
    );
}

#[test]
fn test_endorsement_unlocks_next_day() {
    let (env, client, _admin) = setup_env();
    let endorser = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // Use up daily limit
    for _ in 0..5 {
        let endorsee = Address::generate(&env);
        client.endorse_peer(&endorser, &endorsee, &String::from_str(&env, "Python"));
    }

    // Advance to next day
    env.ledger().with_mut(|l| l.timestamp = 1_000_000 + 86_400);

    let endorsee_new = Address::generate(&env);
    // Should succeed on next day
    client.endorse_peer(&endorser, &endorsee_new, &String::from_str(&env, "Python"));
    let endorsements = client.get_user_endorsements(&endorsee_new);
    assert_eq!(endorsements.len(), 1);
}

#[test]
fn test_peer_recognition() {
    let (env, client, _admin) = setup_env();
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // Should succeed without error
    client.recognize_peer(
        &from,
        &to,
        &RecognitionType::HelpfulAnswer,
        &String::from_str(&env, "Answered my question perfectly!"),
    );
}

// ─── Reputation ───────────────────────────────────────────────────────────────

#[test]
fn test_reputation_grows_with_activity() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    let rep_before = client.get_reputation(&student);
    assert_eq!(rep_before.total_score, 0);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    client.record_activity(
        &student,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );

    let rep_after = client.get_reputation(&student);
    assert!(
        rep_after.total_score > 0,
        "reputation should grow after activity"
    );
}

#[test]
fn test_high_score_boosts_quality_reputation() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let activity = ActivityRecord {
        activity_type: ActivityType::AssessmentPassed,
        course_id: String::from_str(&env, "C1"),
        module_id: String::from_str(&env, "M1"),
        score: 95,
        time_spent: 3600,
        timestamp: 1_000_000,
    };
    client.record_activity(&student, &activity);

    let rep = client.get_reputation(&student);
    assert!(
        rep.quality_points > 0,
        "high score should add quality reputation points"
    );
}

// ─── Adaptive Difficulty ─────────────────────────────────────────────────────

#[test]
fn test_adaptive_difficulty_defaults_to_beginner() {
    let (env, client, _admin) = setup_env();
    let student = Address::generate(&env);

    let ad = client.get_adaptive_difficulty(&student);
    // Fresh student should get beginner recommendation (level 1 < 5)
    assert!(matches!(
        ad.recommended_difficulty,
        crate::types::ChallengeDifficulty::Beginner
    ));
}

#[test]
fn test_adaptive_difficulty_updates_after_challenge() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let now = 1_000_000u64;

    let challenge = make_challenge(&env, &admin, now);
    let challenge_id = client.create_challenge(&admin, &challenge);
    client.join_challenge(&student, &challenge_id);
    client.update_challenge_progress(&student, &challenge_id, &3u32);

    let ad = client.get_adaptive_difficulty(&student);
    assert!(
        ad.completion_rate > 0,
        "completion rate should update after challenge"
    );
}

// ─── Guild Leaderboard ───────────────────────────────────────────────────────

#[test]
fn test_guild_leaderboard_updates() {
    let (env, client, _admin) = setup_env();
    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);

    let g1 = client.create_guild(
        &c1,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "desc"),
        &10u32,
        &true,
    );
    let _g2 = client.create_guild(
        &c2,
        &String::from_str(&env, "Beta"),
        &String::from_str(&env, "desc"),
        &10u32,
        &true,
    );

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    // c1 earns more XP → g1 should rank above g2
    client.record_activity(
        &c1,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );
    client.record_activity(
        &c1,
        &make_activity(&env, ActivityType::CourseCompleted, 1_000_000),
    );

    let board = client.get_guild_leaderboard();
    assert!(board.len() >= 2);
    assert_eq!(board.get(0).unwrap().guild_id, g1, "guild 1 should lead");
}
