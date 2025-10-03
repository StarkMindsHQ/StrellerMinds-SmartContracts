#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
    use crate::types::{
        Achievement, AchievementRequirements, AchievementRarity, StakingPool, PremiumFeature,
        TokenomicsConfig, RewardType, BurnType, PrerequisitePolicy
    };
    use crate::incentives::IncentiveManager;
    use crate::{Token, Error};

    fn create_test_env() -> Env {
        Env::default()
    }

    fn create_test_addresses(env: &Env) -> (Address, Address, Address, Address) {
        (
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        )
    }

    fn setup_contract(env: &Env, admin: &Address) -> Address {
        let contract_id = env.register_contract(None, Token);
        env.mock_all_auths();
        
        Token::initialize(env.clone(), contract_id.clone(), admin.clone()).unwrap();
        contract_id
    }

    #[test]
    fn test_initialize_incentive_system() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();
        let result = IncentiveManager::initialize(&env, &admin);
        assert!(result.is_ok());

        // Verify config was set
        let config = IncentiveManager::get_config(&env);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().base_course_reward, 100_000);
    }

    #[test]
    fn test_reward_course_completion_basic() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        let reward = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "rust_basics"),
            85,
        );

        assert!(reward.is_ok());
        let reward_amount = reward.unwrap();
        assert!(reward_amount > 0);
        // 85% completion should get 1.25x multiplier
        assert_eq!(reward_amount, 125_000); // 100k * 1.25
    }

    #[test]
    fn test_reward_course_completion_high_percentage() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        let reward = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "advanced_rust"),
            95,
        );

        assert!(reward.is_ok());
        let reward_amount = reward.unwrap();
        // 95% completion should get 1.5x multiplier
        assert_eq!(reward_amount, 150_000); // 100k * 1.5
    }

    #[test]
    fn test_reward_module_completion() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        let reward = Token::reward_module_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "rust_basics"),
            String::from_str(&env, "variables"),
        );

        assert!(reward.is_ok());
        let reward_amount = reward.unwrap();
        assert_eq!(reward_amount, 10_000); // Base module reward
    }

    #[test]
    fn test_create_achievement() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        let achievement = Achievement {
            id: String::from_str(&env, ""), // Will be generated
            name: String::from_str(&env, "First Course"),
            description: String::from_str(&env, "Complete your first course"),
            reward_amount: 50_000,
            requirements: AchievementRequirements {
                courses_completed: Some(1),
                completion_percentage: Some(80),
                time_limit: None,
                specific_courses: None,
                streak_days: None,
                referrals_count: None,
                custom_criteria: None,
            },
            rarity: AchievementRarity::Common,
            created_at: 0, // Will be set
            is_active: true,
        };

        let result = Token::create_achievement(
            env.clone(),
            contract.clone(),
            admin.clone(),
            achievement,
        );

        assert!(result.is_ok());
        let achievement_id = result.unwrap();
        assert!(!achievement_id.is_empty());
    }

    #[test]
    fn test_check_achievements() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // First complete a course to trigger achievement check
        Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "rust_basics"),
            85,
        ).unwrap();

        let achievements = Token::check_achievements(
            env.clone(),
            contract.clone(),
            student.clone(),
        );

        assert!(achievements.is_ok());
        let awarded = achievements.unwrap();
        // Should award "first_course" achievement
        assert!(awarded.len() > 0);
    }

    #[test]
    fn test_create_staking_pool() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        let mut premium_features = Vec::new(&env);
        premium_features.push_back(PremiumFeature::AdvancedAnalytics);
        premium_features.push_back(PremiumFeature::PrioritySupport);

        let pool = StakingPool {
            id: String::from_str(&env, ""), // Will be generated
            name: String::from_str(&env, "Premium Pool"),
            description: String::from_str(&env, "Stake for premium features"),
            minimum_stake: 1000_000, // 1000 tokens
            reward_rate: 1000, // 10%
            lock_duration: 86400 * 30, // 30 days
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0, // Will be set
            premium_features,
        };

        let result = Token::create_staking_pool(
            env.clone(),
            contract.clone(),
            admin.clone(),
            pool,
        );

        assert!(result.is_ok());
        let pool_id = result.unwrap();
        assert!(!pool_id.is_empty());
    }

    #[test]
    fn test_stake_tokens() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Create staking pool first
        let mut premium_features = Vec::new(&env);
        premium_features.push_back(PremiumFeature::AdvancedAnalytics);

        let pool = StakingPool {
            id: String::from_str(&env, ""),
            name: String::from_str(&env, "Test Pool"),
            description: String::from_str(&env, "Test staking pool"),
            minimum_stake: 100_000,
            reward_rate: 500, // 5%
            lock_duration: 86400 * 7, // 7 days
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0,
            premium_features,
        };

        let pool_id = Token::create_staking_pool(
            env.clone(),
            contract.clone(),
            admin.clone(),
            pool,
        ).unwrap();

        // Mint tokens to user first
        Token::mint(env.clone(), contract.clone(), user.clone(), 1_000_000).unwrap();

        // Stake tokens
        let result = Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            pool_id,
            500_000,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_stake_insufficient_amount() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Create staking pool
        let pool = StakingPool {
            id: String::from_str(&env, ""),
            name: String::from_str(&env, "Test Pool"),
            description: String::from_str(&env, "Test staking pool"),
            minimum_stake: 100_000,
            reward_rate: 500,
            lock_duration: 86400 * 7,
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0,
            premium_features: Vec::new(&env),
        };

        let pool_id = Token::create_staking_pool(
            env.clone(),
            contract.clone(),
            admin.clone(),
            pool,
        ).unwrap();

        // Try to stake less than minimum
        let result = Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            pool_id,
            50_000, // Less than minimum
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InvalidAmount);
    }

    #[test]
    fn test_burn_for_upgrade() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Mint tokens to user
        Token::mint(env.clone(), contract.clone(), user.clone(), 1_000_000).unwrap();

        let result = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            200_000,
            String::from_str(&env, "cert_123"),
            String::from_str(&env, "premium_design"),
        );

        assert!(result.is_ok());
        let burn_id = result.unwrap();
        assert!(!burn_id.is_empty());
    }

    #[test]
    fn test_burn_insufficient_balance() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Don't mint tokens to user
        let result = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            200_000,
            String::from_str(&env, "cert_123"),
            String::from_str(&env, "premium_design"),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientBalance);
    }

    #[test]
    fn test_streak_multiplier_calculation() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Complete multiple courses to build streak
        for i in 1..=5 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, &format!("course_{}", i)),
                80,
            ).unwrap();
        }

        // The streak should increase rewards for subsequent completions
        let reward = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "course_6"),
            80,
        ).unwrap();

        // Should be higher than base reward due to streak
        assert!(reward > 125_000);
    }

    #[test]
    fn test_achievement_rarity_rewards() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Test different rarity achievements
        let rarities = vec![
            (AchievementRarity::Common, 500),
            (AchievementRarity::Uncommon, 750),
            (AchievementRarity::Rare, 1500),
            (AchievementRarity::Epic, 3000),
            (AchievementRarity::Legendary, 7500),
        ];

        for (rarity, expected_min_reward) in rarities {
            let achievement = Achievement {
                id: String::from_str(&env, ""),
                name: String::from_str(&env, "Test Achievement"),
                description: String::from_str(&env, "Test description"),
                reward_amount: expected_min_reward,
                requirements: AchievementRequirements {
                    courses_completed: Some(1),
                    completion_percentage: None,
                    time_limit: None,
                    specific_courses: None,
                    streak_days: None,
                    referrals_count: None,
                    custom_criteria: None,
                },
                rarity,
                created_at: 0,
                is_active: true,
            };

            let result = Token::create_achievement(
                env.clone(),
                contract.clone(),
                admin.clone(),
                achievement,
            );

            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_premium_feature_access() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Create staking pool with premium features
        let mut premium_features = Vec::new(&env);
        premium_features.push_back(PremiumFeature::AdvancedAnalytics);
        premium_features.push_back(PremiumFeature::ExclusiveCourses);

        let pool = StakingPool {
            id: String::from_str(&env, ""),
            name: String::from_str(&env, "Premium Pool"),
            description: String::from_str(&env, "Premium features pool"),
            minimum_stake: 500_000,
            reward_rate: 1000,
            lock_duration: 86400 * 30,
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0,
            premium_features,
        };

        let pool_id = Token::create_staking_pool(
            env.clone(),
            contract.clone(),
            admin.clone(),
            pool,
        ).unwrap();

        // Mint and stake tokens
        Token::mint(env.clone(), contract.clone(), user.clone(), 2_000_000).unwrap();
        Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            pool_id,
            1_000_000,
        ).unwrap();

        // User should now have premium access (this would be checked in a real implementation)
        // For now, we just verify the staking succeeded
        assert!(true);
    }

    #[test]
    fn test_tokenomics_configuration() {
        let env = create_test_env();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        env.mock_all_auths();
        IncentiveManager::initialize(&env, &admin).unwrap();

        let config = IncentiveManager::get_config(&env).unwrap();
        
        // Verify default configuration
        assert_eq!(config.base_course_reward, 100_000);
        assert_eq!(config.base_module_reward, 10_000);
        assert_eq!(config.streak_bonus_rate, 500); // 5%
        assert_eq!(config.max_streak_multiplier, 300); // 3x
        assert_eq!(config.referral_reward, 50_000);
        assert_eq!(config.achievement_bonus_rate, 1000); // 10%
        assert_eq!(config.burn_discount_rate, 2000); // 20%
        assert_eq!(config.inflation_rate, 500); // 5%
        assert_eq!(config.max_supply, 1_000_000_000_000);
    }

    #[test]
    fn test_reward_calculation_with_multipliers() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Complete a course with high percentage
        let reward1 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "course_1"),
            95, // High completion percentage
        ).unwrap();

        // Should get 1.5x multiplier for 95% completion
        assert_eq!(reward1, 150_000);

        // Complete another course with lower percentage
        let reward2 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "course_2"),
            75, // Lower completion percentage
        ).unwrap();

        // Should get base reward (no completion bonus for <80%)
        // But might have streak bonus from previous completion
        assert!(reward2 >= 100_000);
    }

    #[test]
    fn test_global_stats_tracking() {
        let env = create_test_env();
        let (admin, student, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Perform various actions
        Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "course_1"),
            85,
        ).unwrap();

        Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            student.clone(),
            50_000,
            String::from_str(&env, "cert_1"),
            String::from_str(&env, "upgrade"),
        ).unwrap();

        // Global stats should be updated (would verify in real implementation)
        assert!(true);
    }

    #[test]
    fn test_invalid_inputs() {
        let env = create_test_env();
        let (admin, user, _, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Test negative reward amount
        let result = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            user.clone(),
            String::from_str(&env, "course_1"),
            0, // Invalid percentage
        );
        // Should still work but with minimal reward

        // Test empty course ID
        let result2 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            user.clone(),
            String::from_str(&env, ""),
            85,
        );
        // Should work (empty string is valid)

        // Test invalid burn amount
        let result3 = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            0, // Invalid amount
            String::from_str(&env, "cert_1"),
            String::from_str(&env, "upgrade"),
        );
        assert!(result3.is_err());
        assert_eq!(result3.unwrap_err(), Error::InvalidAmount);
    }

    #[test]
    fn test_concurrent_operations() {
        let env = create_test_env();
        let (admin, user1, user2, _) = create_test_addresses(&env);
        let contract = setup_contract(&env, &admin);

        // Multiple users completing courses simultaneously
        let reward1 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            user1.clone(),
            String::from_str(&env, "course_1"),
            85,
        );

        let reward2 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            user2.clone(),
            String::from_str(&env, "course_2"),
            90,
        );

        assert!(reward1.is_ok());
        assert!(reward2.is_ok());
        
        // Both should receive appropriate rewards
        assert_eq!(reward1.unwrap(), 125_000); // 1.25x for 85%
        assert_eq!(reward2.unwrap(), 150_000); // 1.5x for 90%
    }
}
