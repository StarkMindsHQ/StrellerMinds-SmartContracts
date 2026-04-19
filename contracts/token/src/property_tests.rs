#[cfg(test)]
mod tests {
    use crate::incentives::IncentiveManager;
    use crate::types::{IncentiveDataKey, StreakData, TokenomicsConfig};
    use proptest::prelude::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[allow(dead_code)]
    fn setup_env() -> (Env, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        (env, admin)
    }

    proptest! {
        #[test]
    #[ignore]
    fn test_reward_calculation(
            completion_percentage in 0..100u32,
            streak_days in 0..100u32,
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            // Initialize
            IncentiveManager::initialize(&env, &admin).unwrap();

            // Setup streak
            let streak_data = StreakData {
                user: user.clone(),
                current_streak: streak_days,
                max_streak: streak_days,
                last_activity_date: env.ledger().timestamp(),
                streak_rewards_earned: 0,
            };
            env.storage().persistent().set(&IncentiveDataKey::UserStats(user.clone()), &streak_data);

            let course_id = String::from_str(&env, "course1");
            let reward = IncentiveManager::reward_course_completion(&env, &user, &course_id, completion_percentage).unwrap();

            // Manual calculation logic from incentives.rs
            let config = TokenomicsConfig::default(); // Simplified as we just initialized with default
            let mut expected = config.base_course_reward;
            if completion_percentage >= 90 {
                expected = expected * 150 / 100;
            } else if completion_percentage >= 80 {
                expected = expected * 125 / 100;
            }

            let _bonus = (streak_days + 1) * config.streak_bonus_rate / 10000; // +1 because update_user_streak is called inside reward_course_completion
            // Wait, streak_multiplier is calculated BEFORE update_user_streak in reward_course_completion
            // Let's re-verify incentives.rs:
            // 78: let streak_multiplier = Self::get_streak_multiplier(env, user);
            // 98: Self::update_user_streak(env, user)?;

            let bonus_calc = streak_days * config.streak_bonus_rate / 10000;
            let streak_multiplier = (100 + bonus_calc).min(config.max_streak_multiplier);

            expected = expected * streak_multiplier as i128 / 100;

            // Event multiplier is 100 by default
            assert_eq!(reward, expected, "Reward mismatch for percentage {} and streak {}", completion_percentage, streak_days);
        }

        #[test]
    #[ignore]
    fn test_transfer_conservation(
            amount in 0..5000u64,
            initial_from in 5000..10000u64,
            initial_to in 0..5000u64,
        ) {
            let env = Env::default();
            env.mock_all_auths();
            let admin = Address::generate(&env);
            let from = Address::generate(&env);
            let to = Address::generate(&env);

            // Mint initial balances
            crate::gas_optimized::mint_optimized(&env, &admin, &from, initial_from);
            crate::gas_optimized::mint_optimized(&env, &admin, &to, initial_to);

            let bal_from_before = crate::gas_optimized::balance_of(&env, &from);
            let bal_to_before = crate::gas_optimized::balance_of(&env, &to);
            let total_before = bal_from_before + bal_to_before;

            // Transfer
            crate::gas_optimized::transfer_optimized(&env, &from, &to, amount);

            let bal_from_after = crate::gas_optimized::balance_of(&env, &from);
            let bal_to_after = crate::gas_optimized::balance_of(&env, &to);
            let total_after = bal_from_after + bal_to_after;

            assert_eq!(total_before, total_after, "Total balance not conserved");
            assert_eq!(bal_from_before - amount, bal_from_after, "Sender balance not decreased correctly");
            assert_eq!(bal_to_before + amount, bal_to_after, "Recipient balance not increased correctly");
        }
    }
}
