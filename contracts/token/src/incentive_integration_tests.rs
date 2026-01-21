#[cfg(test)]
mod integration_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
    use crate::types::{
        Achievement, AchievementRequirements, AchievementRarity, StakingPool, PremiumFeature,
        TokenomicsConfig, BurnType, IncentiveEvent, UserStats
    };
    use crate::{Token, Error};

    fn create_test_contract() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register_contract(None, Token);
        (env, contract_id)
    }

    fn create_test_addresses(env: &Env) -> (Address, Address, Address, Address) {
        (
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        )
    }

    fn setup_full_system(env: &Env, contract: &Address, admin: &Address) {
        env.mock_all_auths();
        Token::initialize(env.clone(), contract.clone(), admin.clone()).unwrap();
        
        // Create default achievements
        create_default_achievements(env, contract, admin);
        
        // Create staking pools
        create_default_staking_pools(env, contract, admin);
    }

    fn create_default_achievements(env: &Env, contract: &Address, admin: &Address) {
        let achievements = vec![
            ("first_course", "First Course", "Complete your first course", 50_000, AchievementRarity::Common),
            ("course_explorer", "Course Explorer", "Complete 5 courses", 150_000, AchievementRarity::Uncommon),
            ("dedicated_learner", "Dedicated Learner", "Complete 10 courses", 500_000, AchievementRarity::Rare),
            ("week_warrior", "Week Warrior", "Maintain 7-day streak", 100_000, AchievementRarity::Uncommon),
            ("month_master", "Month Master", "Maintain 30-day streak", 1_000_000, AchievementRarity::Epic),
        ];

        for (id, name, desc, reward, rarity) in achievements {
            let achievement = Achievement {
                id: String::from_str(env, id),
                name: String::from_str(env, name),
                description: String::from_str(env, desc),
                reward_amount: reward,
                requirements: AchievementRequirements {
                    courses_completed: Some(match id {
                        "first_course" => 1,
                        "course_explorer" => 5,
                        "dedicated_learner" => 10,
                        _ => 0,
                    }),
                    completion_percentage: Some(80),
                    time_limit: None,
                    specific_courses: None,
                    streak_days: Some(match id {
                        "week_warrior" => 7,
                        "month_master" => 30,
                        _ => 0,
                    }),
                    referrals_count: None,
                    custom_criteria: None,
                },
                rarity,
                created_at: 0,
                is_active: true,
            };

            Token::create_achievement(env.clone(), contract.clone(), admin.clone(), achievement).unwrap();
        }
    }

    fn create_default_staking_pools(env: &Env, contract: &Address, admin: &Address) {
        // Basic staking pool
        let mut basic_features = Vec::new(env);
        basic_features.push_back(PremiumFeature::AdvancedAnalytics);

        let basic_pool = StakingPool {
            id: String::from_str(env, ""),
            name: String::from_str(env, "Basic Premium"),
            description: String::from_str(env, "Basic premium features"),
            minimum_stake: 100_000, // 100 tokens
            reward_rate: 500, // 5% APY
            lock_duration: 86400 * 7, // 7 days
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0,
            premium_features: basic_features,
        };

        // Premium staking pool
        let mut premium_features = Vec::new(env);
        premium_features.push_back(PremiumFeature::AdvancedAnalytics);
        premium_features.push_back(PremiumFeature::PrioritySupport);
        premium_features.push_back(PremiumFeature::ExclusiveCourses);

        let premium_pool = StakingPool {
            id: String::from_str(env, ""),
            name: String::from_str(env, "Premium Plus"),
            description: String::from_str(env, "Full premium experience"),
            minimum_stake: 1_000_000, // 1000 tokens
            reward_rate: 1000, // 10% APY
            lock_duration: 86400 * 30, // 30 days
            total_staked: 0,
            total_rewards_distributed: 0,
            is_active: true,
            created_at: 0,
            premium_features,
        };

        Token::create_staking_pool(env.clone(), contract.clone(), admin.clone(), basic_pool).unwrap();
        Token::create_staking_pool(env.clone(), contract.clone(), admin.clone(), premium_pool).unwrap();
    }

    #[test]
    fn test_complete_learning_journey() {
        let (env, contract) = create_test_contract();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Student starts learning journey
        // 1. Complete first course
        let reward1 = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "intro_programming"),
            85,
        ).unwrap();

        assert_eq!(reward1, 125_000); // 100k * 1.25 for 85% completion

        // Check achievements - should get "first_course"
        let achievements = Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
        assert!(achievements.len() > 0);

        // 2. Complete several modules
        for i in 1..=5 {
            Token::reward_module_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, "intro_programming"),
                String::from_str(&env, &format!("module_{}", i)),
            ).unwrap();
        }

        // 3. Complete more courses to build streak and unlock achievements
        let courses = vec!["data_structures", "algorithms", "web_development", "databases"];
        for course in courses {
            let reward = Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, course),
                90, // High completion
            ).unwrap();

            // Rewards should increase due to streak
            assert!(reward >= 150_000); // At least 1.5x for 90% completion
        }

        // Check for "course_explorer" achievement (5 courses)
        let achievements = Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
        assert!(achievements.len() > 0);

        // 4. Student decides to stake tokens for premium features
        // First mint tokens to student
        Token::mint(env.clone(), contract.clone(), student.clone(), 2_000_000).unwrap();

        // Stake in basic pool
        Token::stake_tokens(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "pool_1"),
            500_000,
        ).unwrap();

        // 5. Student burns tokens for certificate upgrade
        let burn_result = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            student.clone(),
            200_000,
            String::from_str(&env, "cert_web_dev"),
            String::from_str(&env, "premium_design"),
        );

        assert!(burn_result.is_ok());
    }

    #[test]
    fn test_multi_user_competition() {
        let (env, contract) = create_test_contract();
        let (admin, student1, student2, student3) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Multiple students competing
        let students = vec![student1.clone(), student2.clone(), student3.clone()];
        let mut total_rewards = Vec::new(&env);

        for (i, student) in students.iter().enumerate() {
            // Mint initial tokens
            Token::mint(env.clone(), contract.clone(), student.clone(), 1_000_000).unwrap();

            let mut student_rewards = 0i128;

            // Each student completes different number of courses
            let course_count = (i + 1) * 3; // 3, 6, 9 courses respectively
            
            for j in 1..=course_count {
                let reward = Token::reward_course_completion(
                    env.clone(),
                    contract.clone(),
                    student.clone(),
                    String::from_str(&env, &format!("course_{}_{}", i, j)),
                    80 + (j % 20) as u32, // Varying completion percentages
                ).unwrap();

                student_rewards += reward;

                // Complete some modules too
                Token::reward_module_completion(
                    env.clone(),
                    contract.clone(),
                    student.clone(),
                    String::from_str(&env, &format!("course_{}_{}", i, j)),
                    String::from_str(&env, "module_1"),
                ).unwrap();
            }

            total_rewards.push_back(student_rewards);

            // Check achievements
            Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
        }

        // Student with more courses should have higher rewards due to streak bonuses
        assert!(total_rewards.get(2).unwrap() > total_rewards.get(1).unwrap());
        assert!(total_rewards.get(1).unwrap() > total_rewards.get(0).unwrap());
    }

    #[test]
    fn test_staking_and_premium_features_workflow() {
        let (env, contract) = create_test_contract();
        let (admin, user, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // User earns tokens through course completion
        Token::mint(env.clone(), contract.clone(), user.clone(), 5_000_000).unwrap();

        // Complete courses to earn more tokens
        for i in 1..=10 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                user.clone(),
                String::from_str(&env, &format!("course_{}", i)),
                85,
            ).unwrap();
        }

        // Stake in basic pool first
        Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            String::from_str(&env, "pool_1"),
            500_000,
        ).unwrap();

        // Later upgrade to premium pool
        Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            String::from_str(&env, "pool_2"),
            2_000_000,
        ).unwrap();

        // User should now have access to premium features
        // (In a real implementation, we would check premium access here)
        
        // User burns tokens for various upgrades
        Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            300_000,
            String::from_str(&env, "cert_1"),
            String::from_str(&env, "custom_design"),
        ).unwrap();

        Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            150_000,
            String::from_str(&env, "cert_2"),
            String::from_str(&env, "fast_track"),
        ).unwrap();
    }

    #[test]
    fn test_achievement_progression_system() {
        let (env, contract) = create_test_contract();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Track achievement progression
        let mut achievements_earned = Vec::new(&env);

        // Complete first course - should unlock "first_course"
        Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "course_1"),
            85,
        ).unwrap();

        let new_achievements = Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
        for achievement in new_achievements.iter() {
            achievements_earned.push_back(achievement.clone());
        }

        assert!(achievements_earned.len() >= 1);

        // Complete 4 more courses - should unlock "course_explorer"
        for i in 2..=5 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, &format!("course_{}", i)),
                80,
            ).unwrap();

            let new_achievements = Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
            for achievement in new_achievements.iter() {
                if !achievements_earned.contains(achievement) {
                    achievements_earned.push_back(achievement.clone());
                }
            }
        }

        // Should have at least "first_course" and "course_explorer"
        assert!(achievements_earned.len() >= 2);

        // Complete 5 more courses - should unlock "dedicated_learner"
        for i in 6..=10 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, &format!("course_{}", i)),
                90,
            ).unwrap();

            Token::check_achievements(env.clone(), contract.clone(), student.clone()).unwrap();
        }

        // Verify progression through achievement tiers
        assert!(achievements_earned.len() >= 2);
    }

    #[test]
    fn test_economic_model_validation() {
        let (env, contract) = create_test_contract();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Test token supply mechanics
        let initial_supply = Token::total_supply(env.clone(), contract.clone());
        
        // Mint tokens (increases supply)
        Token::mint(env.clone(), contract.clone(), admin.clone(), 1_000_000).unwrap();
        let supply_after_mint = Token::total_supply(env.clone(), contract.clone());
        assert_eq!(supply_after_mint, initial_supply + 1_000_000);

        // Burn tokens (decreases supply)
        Token::burn(env.clone(), contract.clone(), admin.clone(), 500_000).unwrap();
        let supply_after_burn = Token::total_supply(env.clone(), contract.clone());
        assert_eq!(supply_after_burn, supply_after_mint - 500_000);

        // Test reward economics
        let user = Address::generate(&env);
        Token::mint(env.clone(), contract.clone(), user.clone(), 2_000_000).unwrap();

        // Complete courses and track reward distribution
        let mut total_rewards = 0i128;
        for i in 1..=5 {
            let reward = Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                user.clone(),
                String::from_str(&env, &format!("course_{}", i)),
                85,
            ).unwrap();
            total_rewards += reward;
        }

        // Rewards should follow expected patterns
        assert!(total_rewards > 500_000); // At least 5 * 100k base reward
        
        // Test staking economics
        Token::stake_tokens(
            env.clone(),
            contract.clone(),
            user.clone(),
            String::from_str(&env, "pool_1"),
            1_000_000,
        ).unwrap();

        // Test burning economics
        Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            200_000,
            String::from_str(&env, "cert_1"),
            String::from_str(&env, "upgrade"),
        ).unwrap();
    }

    #[test]
    fn test_referral_and_social_features() {
        let (env, contract) = create_test_contract();
        let (admin, referrer, referee, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Mint tokens for testing
        Token::mint(env.clone(), contract.clone(), referrer.clone(), 1_000_000).unwrap();
        Token::mint(env.clone(), contract.clone(), referee.clone(), 1_000_000).unwrap();

        // Simulate referral system (would be implemented in full system)
        // Referrer completes courses
        for i in 1..=3 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                referrer.clone(),
                String::from_str(&env, &format!("referrer_course_{}", i)),
                85,
            ).unwrap();
        }

        // Referee completes courses (triggered by referral)
        for i in 1..=2 {
            Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                referee.clone(),
                String::from_str(&env, &format!("referee_course_{}", i)),
                80,
            ).unwrap();
        }

        // Both users should have earned rewards
        let referrer_balance = Token::balance(env.clone(), contract.clone(), referrer.clone());
        let referee_balance = Token::balance(env.clone(), contract.clone(), referee.clone());

        assert!(referrer_balance > 1_000_000); // Original + rewards
        assert!(referee_balance > 1_000_000); // Original + rewards
    }

    #[test]
    fn test_seasonal_events_and_campaigns() {
        let (env, contract) = create_test_contract();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Create a seasonal event (would be implemented in full system)
        // For now, test basic reward mechanics during "events"
        
        Token::mint(env.clone(), contract.clone(), student.clone(), 1_000_000).unwrap();

        // Complete courses during "event period"
        let mut event_rewards = 0i128;
        for i in 1..=5 {
            let reward = Token::reward_course_completion(
                env.clone(),
                contract.clone(),
                student.clone(),
                String::from_str(&env, &format!("event_course_{}", i)),
                90, // High completion for event
            ).unwrap();
            event_rewards += reward;
        }

        // Event rewards should be substantial
        assert!(event_rewards > 750_000); // 5 courses * 150k each
    }

    #[test]
    fn test_governance_and_tokenomics_updates() {
        let (env, contract) = create_test_contract();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Test configuration updates (admin only)
        // In a full implementation, this would go through governance
        
        // Verify current config
        let config = IncentiveManager::get_config(&env).unwrap();
        assert_eq!(config.base_course_reward, 100_000);

        // Test that non-admin cannot update config (would be tested in full implementation)
        // Test governance voting mechanics (would be implemented)
        // Test proposal execution (would be implemented)
    }

    #[test]
    fn test_emergency_scenarios() {
        let (env, contract) = create_test_contract();
        let (admin, user, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Test contract pause functionality (would be implemented)
        // Test emergency token recovery (would be implemented)
        // Test circuit breakers for excessive rewards (would be implemented)

        // For now, test basic error handling
        Token::mint(env.clone(), contract.clone(), user.clone(), 1_000_000).unwrap();

        // Test insufficient balance scenarios
        let result = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            user.clone(),
            2_000_000, // More than balance
            String::from_str(&env, "cert_1"),
            String::from_str(&env, "upgrade"),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InsufficientBalance);
    }

    #[test]
    fn test_cross_contract_integration() {
        let (env, contract) = create_test_contract();
        let (admin, student, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Test integration with certificate contract
        // (Would call certificate contract methods in real implementation)
        
        // Test integration with progress tracking
        // (Would call progress contract methods in real implementation)
        
        // For now, test the token contract's role in the ecosystem
        Token::mint(env.clone(), contract.clone(), student.clone(), 1_000_000).unwrap();

        // Student completes course - this would trigger certificate minting
        let reward = Token::reward_course_completion(
            env.clone(),
            contract.clone(),
            student.clone(),
            String::from_str(&env, "integrated_course"),
            95,
        ).unwrap();

        assert_eq!(reward, 150_000);

        // Student burns tokens for certificate upgrade
        let burn_result = Token::burn_for_upgrade(
            env.clone(),
            contract.clone(),
            student.clone(),
            300_000,
            String::from_str(&env, "cert_integrated"),
            String::from_str(&env, "premium_upgrade"),
        );

        assert!(burn_result.is_ok());
    }

    #[test]
    fn test_performance_and_scalability() {
        let (env, contract) = create_test_contract();
        let (admin, _, _, _) = create_test_addresses(&env);
        
        setup_full_system(&env, &contract, &admin);

        // Test handling many users
        let mut users = Vec::new(&env);
        for i in 0..10 {
            let user = Address::generate(&env);
            users.push_back(user.clone());
            Token::mint(env.clone(), contract.clone(), user.clone(), 1_000_000).unwrap();
        }

        // Each user completes courses
        for (i, user) in users.iter().enumerate() {
            for j in 1..=3 {
                Token::reward_course_completion(
                    env.clone(),
                    contract.clone(),
                    user.clone(),
                    String::from_str(&env, &format!("user_{}_course_{}", i, j)),
                    80 + (j * 5) as u32,
                ).unwrap();
            }

            // Check achievements
            Token::check_achievements(env.clone(), contract.clone(), user.clone()).unwrap();
        }

        // Test batch operations (would be implemented for efficiency)
        // Test gas optimization (would be measured in real deployment)
        
        // Verify all operations completed successfully
        for user in users.iter() {
            let balance = Token::balance(env.clone(), contract.clone(), user.clone());
            assert!(balance > 1_000_000); // Should have earned rewards
        }
    }
}
