use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{
    Achievement, AchievementCategory, AchievementRequirements, AchievementTier, ActivityRecord,
    ActivityType, AdaptiveDifficulty, ChallengeDifficulty, GamificationKey, GamificationProfile,
    UserAchievement,
};

/// First 25 IDs are reserved for milestone achievements seeded at init.
const MILESTONE_RESERVE: u64 = 25;

pub struct AchievementManager;

impl AchievementManager {
    // ── Admin: create custom achievement ───────────────────────────────────

    pub fn create(env: &Env, mut achievement: Achievement) -> Result<u64, Error> {
        let id = GamificationStorage::next_id(env, &GamificationKey::AchievementCounter);
        achievement.id = id;
        achievement.created_at = env.ledger().timestamp();
        achievement.is_active = true;

        env.storage()
            .persistent()
            .set(&GamificationKey::Achievement(id), &achievement);

        Ok(id)
    }

    // ── Core: process a learning activity ──────────────────────────────────

    pub fn process_activity(
        env: &Env,
        user: &Address,
        activity: &ActivityRecord,
    ) -> Result<Vec<u64>, Error> {
        let config = GamificationStorage::get_config(env);
        let mut profile = GamificationStorage::get_profile(env, user);

        // ── 1. Base XP for activity type ──────────────────────────────────
        let base_xp: u32 = match activity.activity_type {
            ActivityType::ModuleCompleted => config.base_module_xp,
            ActivityType::CourseCompleted => config.base_course_xp,
            ActivityType::AssessmentPassed => {
                // Score-weighted: 0-100 % of base module XP
                config.base_module_xp * activity.score.min(100) / 100
            }
            ActivityType::StudySession => {
                // 1 XP per minute, capped at 30
                ((activity.time_spent / 60) as u32).min(30)
            }
            ActivityType::PeerHelped => config.help_xp,
            ActivityType::ChallengeProgress => 0, // handled by ChallengeManager
        };

        // ── 2. Update activity-specific counters ──────────────────────────
        match activity.activity_type {
            ActivityType::CourseCompleted => {
                profile.courses_completed += 1;
                profile.modules_completed += 1;
            }
            ActivityType::ModuleCompleted => {
                profile.modules_completed += 1;
            }
            _ => {}
        }

        // ── 3. Streak update ──────────────────────────────────────────────
        let streak_bonus_xp = Self::update_streak(env, &mut profile, activity.timestamp, &config);

        // ── 4. Season multiplier ──────────────────────────────────────────
        let season_mult = crate::seasons::SeasonManager::get_xp_multiplier(env);
        let base_with_season = (base_xp + streak_bonus_xp) * season_mult / 100;

        // ── 5. Final XP accumulation ──────────────────────────────────────
        let final_xp = base_with_season;
        profile.total_xp += final_xp;
        profile.last_activity = activity.timestamp;
        let prev_level = profile.level;
        profile.level = Self::calculate_level(profile.total_xp);
        let leveled_up = profile.level > prev_level;

        // ── 6. Save profile ───────────────────────────────────────────────
        GamificationStorage::set_profile(env, user, &profile);

        // ── 7. Season XP ──────────────────────────────────────────────────
        let new_season_xp = crate::seasons::SeasonManager::add_season_xp(env, user, final_xp);
        if new_season_xp > 0 {
            let mut p = GamificationStorage::get_profile(env, user);
            p.season_xp = new_season_xp;
            GamificationStorage::set_profile(env, user, &p);
        }

        // ── 8. Guild contribution ─────────────────────────────────────────
        crate::guilds::GuildManager::add_contribution(env, user, final_xp);

        // ── 9. Reputation ─────────────────────────────────────────────────
        crate::reputation::ReputationManager::update_from_activity(env, user, activity);

        // ── 10. Leaderboard ───────────────────────────────────────────────
        let updated_profile = GamificationStorage::get_profile(env, user);
        crate::leaderboard::LeaderboardManager::update_user_score(env, &updated_profile);

        // ── 11. Achievement check ─────────────────────────────────────────
        let final_profile = GamificationStorage::get_profile(env, user);
        let new_achievements = Self::check_and_award_achievements(env, user, &final_profile);

        // ── 12. Events ────────────────────────────────────────────────────
        if final_xp > 0 {
            GamificationEvents::emit_xp_earned(env, user, final_xp);
        }
        if leveled_up {
            GamificationEvents::emit_level_up(env, user, profile.level);
        }

        Ok(new_achievements)
    }

    // ── Achievement check & award ──────────────────────────────────────────

    /// Evaluate which milestone achievements the user now qualifies for and
    /// award any that haven't been earned yet.  Returns IDs of newly earned ones.
    pub fn check_and_award_achievements(
        env: &Env,
        user: &Address,
        profile: &GamificationProfile,
    ) -> Vec<u64> {
        let qualifying = Self::qualifying_milestones(env, profile);
        let mut awarded = Vec::new(env);

        for id in qualifying.iter() {
            let earned_key = GamificationKey::UserAchievement(user.clone(), id);
            if env.storage().persistent().has(&earned_key) {
                continue; // already earned
            }

            // Load the achievement definition (may not exist if not seeded)
            let ach_opt: Option<Achievement> = env
                .storage()
                .persistent()
                .get(&GamificationKey::Achievement(id));

            if let Some(ach) = ach_opt {
                if !ach.is_active {
                    continue;
                }
                let ua = UserAchievement {
                    user: user.clone(),
                    achievement_id: id,
                    earned_at: env.ledger().timestamp(),
                    token_reward_claimed: false,
                    xp_reward: ach.xp_reward,
                    token_reward: ach.token_reward,
                };

                env.storage().persistent().set(&earned_key, &ua);

                // Append to user's achievement list
                let list_key = GamificationKey::UserAchievements(user.clone());
                let mut list: Vec<u64> = env
                    .storage()
                    .persistent()
                    .get(&list_key)
                    .unwrap_or_else(|| Vec::new(env));
                list.push_back(id);
                env.storage().persistent().set(&list_key, &list);

                awarded.push_back(id);
                GamificationEvents::emit_achievement_earned(env, user, id, ach.xp_reward);
            }
        }

        awarded
    }

    // ── Claim token reward ─────────────────────────────────────────────────

    pub fn claim_reward(env: &Env, user: &Address, achievement_id: u64) -> Result<i128, Error> {
        let key = GamificationKey::UserAchievement(user.clone(), achievement_id);
        let mut ua: UserAchievement = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        if ua.token_reward_claimed {
            return Err(Error::AchievementAlreadyClaimed);
        }

        ua.token_reward_claimed = true;
        env.storage().persistent().set(&key, &ua);

        let mut profile = GamificationStorage::get_profile(env, user);
        profile.total_tokens_earned += ua.token_reward;
        profile.achievements_count += 1;
        GamificationStorage::set_profile(env, user, &profile);

        GamificationEvents::emit_achievement_claimed(env, user, achievement_id, ua.token_reward);
        Ok(ua.token_reward)
    }

    // ── User achievement list ──────────────────────────────────────────────

    pub fn get_user_achievements(env: &Env, user: &Address) -> Vec<UserAchievement> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&GamificationKey::UserAchievements(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut out = Vec::new(env);
        for id in ids.iter() {
            let key = GamificationKey::UserAchievement(user.clone(), id);
            if let Some(ua) = env
                .storage()
                .persistent()
                .get::<GamificationKey, UserAchievement>(&key)
            {
                out.push_back(ua);
            }
        }
        out
    }

    // ── Adaptive difficulty ────────────────────────────────────────────────

    pub fn get_adaptive_difficulty(env: &Env, user: &Address) -> AdaptiveDifficulty {
        if let Some(ad) = env
            .storage()
            .persistent()
            .get::<GamificationKey, AdaptiveDifficulty>(&GamificationKey::UserDifficulty(
                user.clone(),
            ))
        {
            return ad;
        }

        // Default: infer from level
        let profile = GamificationStorage::get_profile(env, user);
        let difficulty = match profile.level {
            1..=4 => ChallengeDifficulty::Beginner,
            5..=14 => ChallengeDifficulty::Intermediate,
            15..=29 => ChallengeDifficulty::Advanced,
            30..=59 => ChallengeDifficulty::Expert,
            _ => ChallengeDifficulty::Legendary,
        };

        AdaptiveDifficulty {
            user: user.clone(),
            recommended_difficulty: difficulty,
            performance_score: profile.level.min(100) * 2,
            completion_rate: 0,
            avg_score: 0,
            last_calculated: env.ledger().timestamp(),
        }
    }

    /// Update the adaptive difficulty model after a challenge completion.
    pub fn update_adaptive_difficulty(
        env: &Env,
        user: &Address,
        challenge_completed: bool,
        score: u32,
    ) {
        let mut ad = Self::get_adaptive_difficulty(env, user);
        let now = env.ledger().timestamp();

        // EMA-style update
        let alpha = 20u32; // 20% weight on new data
        if challenge_completed {
            ad.completion_rate = (ad.completion_rate * (100 - alpha) + 100 * alpha) / 100;
        } else {
            ad.completion_rate = ad.completion_rate * (100 - alpha) / 100;
        }
        ad.avg_score = (ad.avg_score * (100 - alpha) + score * alpha) / 100;
        ad.performance_score = (ad.completion_rate + ad.avg_score) / 2;

        // Recommend next difficulty tier based on performance
        ad.recommended_difficulty = match ad.performance_score {
            0..=39 => ChallengeDifficulty::Beginner,
            40..=59 => ChallengeDifficulty::Intermediate,
            60..=74 => ChallengeDifficulty::Advanced,
            75..=89 => ChallengeDifficulty::Expert,
            _ => ChallengeDifficulty::Legendary,
        };
        ad.last_calculated = now;

        env.storage()
            .persistent()
            .set(&GamificationKey::UserDifficulty(user.clone()), &ad);
    }

    // ── Milestone seeding ──────────────────────────────────────────────────

    /// Called once during contract `initialize` to seed pre-defined milestones.
    pub fn seed_default_achievements(env: &Env) {
        let ts = env.ledger().timestamp();

        // Course completion milestones (IDs 1-5)
        Self::seed_one(
            env,
            1,
            "First Step",
            "Complete your first course",
            AchievementTier::Bronze,
            AchievementCategory::Learning,
            100,
            1_000,
            AchievementRequirements {
                courses_completed: 1,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            2,
            "Course Explorer",
            "Complete 5 courses",
            AchievementTier::Silver,
            AchievementCategory::Learning,
            500,
            5_000,
            AchievementRequirements {
                courses_completed: 5,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            3,
            "Dedicated Learner",
            "Complete 10 courses",
            AchievementTier::Gold,
            AchievementCategory::Learning,
            1_000,
            10_000,
            AchievementRequirements {
                courses_completed: 10,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            4,
            "Knowledge Seeker",
            "Complete 25 courses",
            AchievementTier::Platinum,
            AchievementCategory::Learning,
            2_500,
            25_000,
            AchievementRequirements {
                courses_completed: 25,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            5,
            "Master Student",
            "Complete 50 courses",
            AchievementTier::Diamond,
            AchievementCategory::Learning,
            5_000,
            50_000,
            AchievementRequirements {
                courses_completed: 50,
                ..Self::zero_req()
            },
            ts,
        );

        // Streak milestones (IDs 6-10)
        Self::seed_one(
            env,
            6,
            "Week Warrior",
            "Maintain a 7-day learning streak",
            AchievementTier::Bronze,
            AchievementCategory::Streak,
            150,
            1_500,
            AchievementRequirements {
                streak_days: 7,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            7,
            "Month Master",
            "Maintain a 30-day learning streak",
            AchievementTier::Silver,
            AchievementCategory::Streak,
            600,
            6_000,
            AchievementRequirements {
                streak_days: 30,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            8,
            "Century Scholar",
            "Maintain a 100-day learning streak",
            AchievementTier::Gold,
            AchievementCategory::Streak,
            2_000,
            20_000,
            AchievementRequirements {
                streak_days: 100,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            9,
            "Relentless",
            "Maintain a 365-day learning streak",
            AchievementTier::Diamond,
            AchievementCategory::Streak,
            10_000,
            100_000,
            AchievementRequirements {
                streak_days: 365,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            10,
            "Comeback Kid",
            "Rebuild a streak to 30 days after breaking it",
            AchievementTier::Silver,
            AchievementCategory::Streak,
            600,
            6_000,
            AchievementRequirements {
                streak_days: 30,
                ..Self::zero_req()
            },
            ts,
        );

        // XP milestones (IDs 11-15)
        Self::seed_one(
            env,
            11,
            "XP Beginner",
            "Earn 1,000 XP",
            AchievementTier::Bronze,
            AchievementCategory::Learning,
            50,
            500,
            AchievementRequirements {
                total_xp: 1_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            12,
            "XP Enthusiast",
            "Earn 5,000 XP",
            AchievementTier::Silver,
            AchievementCategory::Learning,
            250,
            2_500,
            AchievementRequirements {
                total_xp: 5_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            13,
            "XP Expert",
            "Earn 10,000 XP",
            AchievementTier::Gold,
            AchievementCategory::Learning,
            500,
            5_000,
            AchievementRequirements {
                total_xp: 10_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            14,
            "XP Legend",
            "Earn 50,000 XP",
            AchievementTier::Platinum,
            AchievementCategory::Learning,
            2_500,
            25_000,
            AchievementRequirements {
                total_xp: 50_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            15,
            "XP Grandmaster",
            "Earn 100,000 XP",
            AchievementTier::Diamond,
            AchievementCategory::Learning,
            5_000,
            50_000,
            AchievementRequirements {
                total_xp: 100_000,
                ..Self::zero_req()
            },
            ts,
        );

        // Social milestones (IDs 16-20)
        Self::seed_one(
            env,
            16,
            "First Fan",
            "Receive your first peer endorsement",
            AchievementTier::Bronze,
            AchievementCategory::Social,
            100,
            1_000,
            AchievementRequirements {
                endorsements_received: 1,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            17,
            "Well Respected",
            "Receive 10 peer endorsements",
            AchievementTier::Silver,
            AchievementCategory::Social,
            500,
            5_000,
            AchievementRequirements {
                endorsements_received: 10,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            18,
            "Community Pillar",
            "Receive 50 peer endorsements",
            AchievementTier::Gold,
            AchievementCategory::Social,
            2_000,
            20_000,
            AchievementRequirements {
                endorsements_received: 50,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            19,
            "Challenge Accepted",
            "Complete your first challenge",
            AchievementTier::Bronze,
            AchievementCategory::Challenge,
            150,
            1_500,
            AchievementRequirements {
                challenges_completed: 1,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            20,
            "Challenge Champion",
            "Complete 10 challenges",
            AchievementTier::Gold,
            AchievementCategory::Challenge,
            1_500,
            15_000,
            AchievementRequirements {
                challenges_completed: 10,
                ..Self::zero_req()
            },
            ts,
        );

        // Guild milestones (IDs 21-25)
        Self::seed_one(
            env,
            21,
            "Team Player",
            "Contribute 1,000 XP to your guild",
            AchievementTier::Bronze,
            AchievementCategory::Guild,
            200,
            2_000,
            AchievementRequirements {
                guild_contributions: 1_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            22,
            "Guild Pillar",
            "Contribute 10,000 XP to your guild",
            AchievementTier::Silver,
            AchievementCategory::Guild,
            1_000,
            10_000,
            AchievementRequirements {
                guild_contributions: 10_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            23,
            "Guild Legend",
            "Contribute 50,000 XP to your guild",
            AchievementTier::Platinum,
            AchievementCategory::Guild,
            5_000,
            50_000,
            AchievementRequirements {
                guild_contributions: 50_000,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            24,
            "Season Warrior",
            "Participate in a season",
            AchievementTier::Bronze,
            AchievementCategory::Season,
            100,
            1_000,
            AchievementRequirements {
                seasons_completed: 1,
                ..Self::zero_req()
            },
            ts,
        );
        Self::seed_one(
            env,
            25,
            "Season Veteran",
            "Participate in 3 seasons",
            AchievementTier::Silver,
            AchievementCategory::Season,
            500,
            5_000,
            AchievementRequirements {
                seasons_completed: 3,
                ..Self::zero_req()
            },
            ts,
        );

        // Counter starts past the reserved block
        env.storage()
            .persistent()
            .set(&GamificationKey::AchievementCounter, &MILESTONE_RESERVE);
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    pub fn calculate_level(total_xp: u32) -> u32 {
        // Simple linear: every 100 XP = 1 level, capped at 100
        (total_xp / 100 + 1).min(100)
    }

    fn update_streak(
        env: &Env,
        profile: &mut GamificationProfile,
        now: u64,
        config: &crate::types::GamificationConfig,
    ) -> u32 {
        let one_day = 86_400u64;
        let two_days = one_day * 2;

        if profile.current_streak == 0 {
            // Very first activity ever recorded for this user
            profile.current_streak = 1;
            profile.max_streak = 1;
            return 0;
        }

        let elapsed = now.saturating_sub(profile.last_activity);

        if elapsed < one_day {
            // Same day — streak already counted, no change, no bonus
        } else if elapsed <= two_days {
            // Consecutive day — extend streak
            profile.current_streak += 1;
            if profile.current_streak > profile.max_streak {
                profile.max_streak = profile.current_streak;
            }
            // Milestone notifications
            if profile.current_streak == 7
                || profile.current_streak == 30
                || profile.current_streak == 100
                || profile.current_streak == 365
            {
                GamificationEvents::emit_streak_milestone(
                    env,
                    &profile.user,
                    profile.current_streak,
                );
            }
        } else {
            // Streak broken
            profile.current_streak = 1;
        }

        // Bonus XP: +25 XP per complete week of streak, capped
        let weeks = profile.current_streak / 7;
        (weeks * config.streak_weekly_bonus).min(config.max_streak_bonus_xp)
    }

    /// Returns the set of milestone IDs the user now qualifies for.
    fn qualifying_milestones(env: &Env, profile: &GamificationProfile) -> Vec<u64> {
        let mut q = Vec::new(env);

        // Courses
        if profile.courses_completed >= 1 {
            q.push_back(1u64);
        }
        if profile.courses_completed >= 5 {
            q.push_back(2u64);
        }
        if profile.courses_completed >= 10 {
            q.push_back(3u64);
        }
        if profile.courses_completed >= 25 {
            q.push_back(4u64);
        }
        if profile.courses_completed >= 50 {
            q.push_back(5u64);
        }

        // Streaks (current and max)
        if profile.current_streak >= 7 {
            q.push_back(6u64);
        }
        if profile.current_streak >= 30 {
            q.push_back(7u64);
        }
        if profile.current_streak >= 100 {
            q.push_back(8u64);
        }
        if profile.current_streak >= 365 {
            q.push_back(9u64);
        }
        if profile.max_streak >= 30 {
            q.push_back(10u64);
        }

        // XP
        if profile.total_xp >= 1_000 {
            q.push_back(11u64);
        }
        if profile.total_xp >= 5_000 {
            q.push_back(12u64);
        }
        if profile.total_xp >= 10_000 {
            q.push_back(13u64);
        }
        if profile.total_xp >= 50_000 {
            q.push_back(14u64);
        }
        if profile.total_xp >= 100_000 {
            q.push_back(15u64);
        }

        // Social
        if profile.endorsements_received >= 1 {
            q.push_back(16u64);
        }
        if profile.endorsements_received >= 10 {
            q.push_back(17u64);
        }
        if profile.endorsements_received >= 50 {
            q.push_back(18u64);
        }

        // Challenges
        if profile.challenges_completed >= 1 {
            q.push_back(19u64);
        }
        if profile.challenges_completed >= 10 {
            q.push_back(20u64);
        }

        // Guild contributions (stored in GuildMember)
        let guild_contrib = Self::guild_contribution(env, &profile.user);
        if guild_contrib >= 1_000 {
            q.push_back(21u64);
        }
        if guild_contrib >= 10_000 {
            q.push_back(22u64);
        }
        if guild_contrib >= 50_000 {
            q.push_back(23u64);
        }

        q
    }

    fn guild_contribution(env: &Env, user: &Address) -> u32 {
        let member: Option<crate::types::GuildMember> = env
            .storage()
            .persistent()
            .get(&GamificationKey::GuildMember(user.clone()));
        member.map(|m| m.contribution_xp).unwrap_or(0)
    }

    fn zero_req() -> AchievementRequirements {
        AchievementRequirements {
            courses_completed: 0,
            modules_completed: 0,
            streak_days: 0,
            total_xp: 0,
            challenges_completed: 0,
            endorsements_received: 0,
            guild_contributions: 0,
            seasons_completed: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn seed_one(
        env: &Env,
        id: u64,
        name: &str,
        description: &str,
        tier: AchievementTier,
        category: AchievementCategory,
        xp_reward: u32,
        token_reward: i128,
        requirements: AchievementRequirements,
        ts: u64,
    ) {
        let ach = Achievement {
            id,
            name: String::from_str(env, name),
            description: String::from_str(env, description),
            tier,
            category,
            xp_reward,
            token_reward,
            requirements,
            created_at: ts,
            is_active: true,
            is_cross_course: true,
        };
        env.storage()
            .persistent()
            .set(&GamificationKey::Achievement(id), &ach);
    }
}
