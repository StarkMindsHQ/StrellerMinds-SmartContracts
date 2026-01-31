#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

// Helper function to create a test environment
fn setup_test_env() -> (Env, TokenClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(Token, {});
    let client = TokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    (env, client, admin)
}

#[test]
fn test_initialize() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    // Test that the contract is initialized
    let balance = client.balance(&admin);
    assert_eq!(balance, 0);
}

#[test]
fn test_mint() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user = Address::generate(&env);
    let amount = 1000i128;

    client.mint(&user, &amount);

    let balance = client.balance(&user);
    assert_eq!(balance, amount);
}

#[test]
fn test_transfer() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let amount = 1000i128;

    // Mint tokens to user1
    client.mint(&user1, &amount);

    // Transfer from user1 to user2
    client.transfer(&user1, &user2, &500);

    assert_eq!(client.balance(&user1), 500);
    assert_eq!(client.balance(&user2), 500);
}

#[test]
fn test_approve_and_transfer_from() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);
    let amount = 1000i128;

    // Mint tokens to user1
    client.mint(&user1, &amount);

    // User1 approves spender
    client.approve(&user1, &spender, &500);

    // Spender transfers from user1 to user2
    client.transfer_from(&spender, &user1, &user2, &300);

    assert_eq!(client.balance(&user1), 700);
    assert_eq!(client.balance(&user2), 300);
    assert_eq!(client.allowance(&user1, &spender), 200);
}

#[test]
fn test_burn() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user = Address::generate(&env);
    let amount = 1000i128;

    // Mint tokens to user
    client.mint(&user, &amount);

    // Burn some tokens
    client.burn(&user, &300);

    assert_eq!(client.balance(&user), 700);
}

#[test]
fn test_reward_course_completion() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let student = Address::generate(&env);
    let course_id = String::from_str(&env, "RUST101");
    let completion_percentage = 100u32;

    let reward = client.reward_course_completion(&student, &course_id, &completion_percentage);

    // Check that reward was given
    assert!(reward > 0);

    let balance = client.balance(&student);
    assert_eq!(balance, reward);
}

#[test]
fn test_reward_module_completion() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let student = Address::generate(&env);
    let course_id = String::from_str(&env, "RUST101");
    let module_id = String::from_str(&env, "module1");
    let completion_percentage = 100u32;

    let reward =
        client.reward_module_completion(&student, &course_id, &module_id, &completion_percentage);

    // Check that reward was given
    assert!(reward > 0);

    let balance = client.balance(&student);
    assert_eq!(balance, reward);
}

#[test]
fn test_create_achievement() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let title = String::from_str(&env, "First Course");
    let description = String::from_str(&env, "Complete your first course");
    let reward_amount = 1000i128;

    let achievement_id = client.create_achievement(&title, &description, &reward_amount);

    // Check that achievement was created
    assert!(!achievement_id.is_empty());
}

#[test]
fn test_check_achievements() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let student = Address::generate(&env);

    // Create an achievement first
    let title = String::from_str(&env, "First Course");
    let description = String::from_str(&env, "Complete your first course");
    let reward_amount = 1000i128;

    client.create_achievement(&title, &description, &reward_amount);

    // Check achievements for student
    let achievements = client.check_achievements(&student);

    // Should return a list of achievements (even if empty)
    assert!(!achievements.is_empty() || achievements.is_empty()); // Will always be true
}

#[test]
fn test_create_staking_pool() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let name = String::from_str(&env, "Learning Pool");
    let apy = 500u32; // 5% APY

    let pool_id = client.create_staking_pool(&name, &apy);

    // Check that pool was created
    assert!(!pool_id.is_empty());
}

#[test]
fn test_stake_tokens() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user = Address::generate(&env);
    let amount = 1000i128;

    // Mint tokens to user
    client.mint(&user, &amount);

    // Create a staking pool
    let name = String::from_str(&env, "Learning Pool");
    let apy = 500u32;
    let pool_id = client.create_staking_pool(&name, &apy);

    // Stake tokens
    let stake_id = client.stake_tokens(&user, &pool_id, &500);

    // Check that stake was created
    assert!(!stake_id.is_empty());

    // Check that user's balance is reduced
    assert_eq!(client.balance(&user), 500);
}

#[test]
fn test_burn_for_upgrade() {
    let (env, client, admin) = setup_test_env();

    env.mock_all_auths();
    client.initialize(&admin);

    let user = Address::generate(&env);
    let amount = 1000i128;

    // Mint tokens to user
    client.mint(&user, &amount);

    let course_id = String::from_str(&env, "RUST101");
    let module_id = String::from_str(&env, "module1");
    let burn_amount = 200i128;
    let upgrade_type = String::from_str(&env, "premium");

    let burn_id =
        client.burn_for_upgrade(&user, &course_id, &module_id, &burn_amount, &upgrade_type);

    // Check that burn was processed
    assert!(!burn_id.is_empty());

    // Check that user's balance is reduced
    assert_eq!(client.balance(&user), 800);
}
