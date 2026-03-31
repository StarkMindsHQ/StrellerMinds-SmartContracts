use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

use crate::types::*;
use crate::{Community, CommunityClient};

fn create_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    (env, admin, user1, user2, user3)
}

fn setup_community<'a>(env: &Env, admin: &Address) -> CommunityClient<'a> {
    let contract_id = env.register(Community, ());
    let client = CommunityClient::new(env, &contract_id);
    client.initialize(admin);
    client
}

// ══════════════════════════════════════════════════════════════════════
//  Initialization Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_initialize() {
    let (env, admin, _, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let config = client.get_config();
    assert_eq!(config.post_xp_reward, 10);
    assert_eq!(config.reply_xp_reward, 5);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize() {
    let (env, admin, _, _, _) = create_test_env();
    let client = setup_community(&env, &admin);
    client.initialize(&admin);
}

// ══════════════════════════════════════════════════════════════════════
//  Forum Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_post() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let post_id = client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Test Post"),
        &String::from_str(&env, "This is a test post"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    assert_eq!(post_id, 1);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.author, user1);
    assert_eq!(post.id, 1);
}

#[test]
fn test_create_reply() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let post_id = client.create_post(
        &user1,
        &ForumCategory::TechnicalHelp,
        &String::from_str(&env, "Need Help"),
        &String::from_str(&env, "How do I solve this?"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    let reply_id =
        client.create_reply(&user2, &post_id, &String::from_str(&env, "Here's the solution"), &0);

    assert_eq!(reply_id, 1);

    let replies = client.get_post_replies(&post_id);
    assert_eq!(replies.len(), 1);
}

#[test]
fn test_mark_solution() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let post_id = client.create_post(
        &user1,
        &ForumCategory::TechnicalHelp,
        &String::from_str(&env, "Question"),
        &String::from_str(&env, "I need help with this problem"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    let reply_id = client.create_reply(
        &user2,
        &post_id,
        &String::from_str(&env, "Here is the answer to your question"),
        &0,
    );

    client.mark_solution(&user1, &post_id, &reply_id);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.status, PostStatus::Resolved);
}

#[test]
fn test_vote_post() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let post_id = client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Test Post"),
        &String::from_str(&env, "This is test content for voting"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    client.vote_post(&user2, &post_id, &true);

    let post = client.get_post(&post_id).unwrap();
    assert_eq!(post.upvotes, 1);
}

// ══════════════════════════════════════════════════════════════════════
//  Mentorship Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_register_mentor() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let mut expertise = Vec::new(&env);
    expertise.push_back(String::from_str(&env, "Rust"));

    client.register_mentor(
        &user1,
        &expertise,
        &MentorExpertise::Expert,
        &5,
        &String::from_str(&env, "Experienced Rust developer"),
    );

    let profile = client.get_mentor_profile(&user1).unwrap();
    assert_eq!(profile.mentor, user1);
    assert_eq!(profile.max_mentees, 5);
}

#[test]
fn test_mentorship_flow() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    // Register mentor
    let mut expertise = Vec::new(&env);
    expertise.push_back(String::from_str(&env, "Blockchain"));

    client.register_mentor(
        &user1,
        &expertise,
        &MentorExpertise::Advanced,
        &3,
        &String::from_str(&env, "Blockchain expert"),
    );

    // Request mentorship
    let request_id = client.request_mentorship(
        &user2,
        &user1,
        &String::from_str(&env, "Smart Contracts"),
        &String::from_str(&env, "Need help with Soroban"),
    );

    assert_eq!(request_id, 1);

    // Accept mentorship
    client.accept_mentorship(&user1, &request_id);

    // Complete session
    let session_id = client.complete_session(
        &user1,
        &request_id,
        &3600,
        &String::from_str(&env, "Covered basics of Soroban"),
    );

    assert_eq!(session_id, 1);

    // Rate session
    client.rate_session(&user2, &session_id, &5);
}

// ══════════════════════════════════════════════════════════════════════
//  Knowledge Base Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_submit_contribution() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let contrib_id = client.submit_contribution(
        &user1,
        &ContributionType::Tutorial,
        &String::from_str(&env, "Soroban Tutorial"),
        &String::from_str(&env, "Complete guide to Soroban"),
        &ForumCategory::General,
        &Vec::new(&env),
    );

    assert_eq!(contrib_id, 1);

    let contrib = client.get_contribution(&contrib_id).unwrap();
    assert_eq!(contrib.contributor, user1);
    assert_eq!(contrib.status, ContributionStatus::Submitted);
}

#[test]
fn test_review_contribution() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let contrib_id = client.submit_contribution(
        &user1,
        &ContributionType::Article,
        &String::from_str(&env, "Article Title"),
        &String::from_str(&env, "This is the article content for review"),
        &ForumCategory::General,
        &Vec::new(&env),
    );

    client.review_contribution(&admin, &contrib_id, &true);

    let contrib = client.get_contribution(&contrib_id).unwrap();
    assert_eq!(contrib.status, ContributionStatus::Approved);
    assert!(contrib.xp_reward > 0);
}

// ══════════════════════════════════════════════════════════════════════
//  Event Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_event() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let event_id = client.create_event(
        &user1,
        &EventType::Workshop,
        &String::from_str(&env, "Soroban Workshop"),
        &String::from_str(&env, "Learn Soroban development"),
        &1000,
        &2000,
        &50,
        &true,
        &25,
    );

    assert_eq!(event_id, 1);

    let event = client.get_event(&event_id).unwrap();
    assert_eq!(event.organizer, user1);
    assert_eq!(event.max_participants, 50);
}

#[test]
fn test_event_registration() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let event_id = client.create_event(
        &user1,
        &EventType::Webinar,
        &String::from_str(&env, "Webinar Session"),
        &String::from_str(&env, "A webinar about blockchain technology"),
        &1000,
        &2000,
        &10,
        &true,
        &20,
    );

    client.register_for_event(&user2, &event_id);

    let event = client.get_event(&event_id).unwrap();
    assert_eq!(event.current_participants, 1);
}

// ══════════════════════════════════════════════════════════════════════
//  Governance Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_proposal() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    // Build reputation through activity
    // Create posts to build reputation
    for _i in 0..10 {
        client.create_post(
            &user1,
            &ForumCategory::General,
            &String::from_str(&env, "Test Post"),
            &String::from_str(&env, "This is test content for building reputation"),
            &Vec::new(&env),
            &String::from_str(&env, ""),
        );
    }

    // Calculate reputation
    let reputation = client.calculate_reputation(&user1);
    assert!(reputation >= 100); // Should have enough reputation now

    let proposal_id = client.create_proposal(
        &user1,
        &ProposalType::FeatureRequest,
        &String::from_str(&env, "New Feature"),
        &String::from_str(&env, "Add new functionality to the platform"),
        &86400,
        &10,
    );

    assert_eq!(proposal_id, 1);
}

// ══════════════════════════════════════════════════════════════════════
//  Analytics Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_community_metrics() {
    let (env, admin, _, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let metrics = client.get_community_metrics();
    assert_eq!(metrics.total_posts, 0);
    assert_eq!(metrics.total_replies, 0);
}

#[test]
fn test_user_stats() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    // Create some activity
    client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Test Post"),
        &String::from_str(&env, "This is test content for user stats"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    let stats = client.get_user_stats(&user1);
    assert_eq!(stats.posts_created, 1);
}

// ══════════════════════════════════════════════════════════════════════
//  Input Validation Tests
// ══════════════════════════════════════════════════════════════════════

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_post_empty_title() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "AB"),
        &String::from_str(&env, "This is valid content"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_post_short_content() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, "Short"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_post_too_many_tags() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let mut tags = Vec::new(&env);
    let tag = String::from_str(&env, "tag");
    for _ in 0..21u32 {
        tags.push_back(tag.clone());
    }

    client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, "This is valid content for testing"),
        &tags,
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_reply_short_content() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let post_id = client.create_post(
        &user1,
        &ForumCategory::General,
        &String::from_str(&env, "Test Post"),
        &String::from_str(&env, "This is valid test content"),
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    client.create_reply(&user2, &post_id, &String::from_str(&env, "Short"), &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_event_invalid_time_range() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    client.create_event(
        &user1,
        &EventType::Workshop,
        &String::from_str(&env, "Workshop Title"),
        &String::from_str(&env, "Workshop description for testing"),
        &2000,
        &1000, // end before start
        &50,
        &true,
        &25,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_proposal_zero_voting_duration() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    // Build reputation first
    for _i in 0..10 {
        client.create_post(
            &user1,
            &ForumCategory::General,
            &String::from_str(&env, "Test Post"),
            &String::from_str(&env, "This is test content for building reputation"),
            &Vec::new(&env),
            &String::from_str(&env, ""),
        );
    }
    client.calculate_reputation(&user1);

    client.create_proposal(
        &user1,
        &ProposalType::FeatureRequest,
        &String::from_str(&env, "Proposal Title"),
        &String::from_str(&env, "Proposal description with enough detail"),
        &0, // zero voting duration
        &10,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_register_mentor_too_many_expertise_areas() {
    let (env, admin, user1, _, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let mut expertise = Vec::new(&env);
    let skill = String::from_str(&env, "skill");
    for _ in 0..11u32 {
        expertise.push_back(skill.clone());
    }

    client.register_mentor(
        &user1,
        &expertise,
        &MentorExpertise::Expert,
        &5,
        &String::from_str(&env, "Experienced developer with many skills"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_rate_session_out_of_range() {
    let (env, admin, user1, user2, _) = create_test_env();
    let client = setup_community(&env, &admin);

    let mut expertise = Vec::new(&env);
    expertise.push_back(String::from_str(&env, "Rust"));

    client.register_mentor(
        &user1,
        &expertise,
        &MentorExpertise::Expert,
        &5,
        &String::from_str(&env, "Experienced Rust developer"),
    );

    let request_id = client.request_mentorship(
        &user2,
        &user1,
        &String::from_str(&env, "Smart Contracts"),
        &String::from_str(&env, "Need help with Soroban development"),
    );

    client.accept_mentorship(&user1, &request_id);

    let session_id = client.complete_session(
        &user1,
        &request_id,
        &3600,
        &String::from_str(&env, "Covered basics of Soroban development"),
    );

    client.rate_session(&user2, &session_id, &6); // > MAX_RATING of 5
}
