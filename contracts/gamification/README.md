# Gamification Contract

## Purpose

The Gamification contract is the engagement engine of the StrellerMinds platform. It provides a comprehensive on-chain system for motivating learners through experience points (XP), milestone achievements, time-bound challenges, collaborative guilds, competitive seasons, peer endorsements, public recognition, reputation scoring, and multi-category leaderboards. All state transitions emit structured events that feed into the platform's analytics pipeline.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — wires together all managers and exposes the 29 public functions |
| `achievements.rs` | `AchievementManager` — seeds 25 default milestones, evaluates activity records, awards XP and tokens |
| `challenges.rs` | `ChallengeManager` — creates time-bound challenges, handles enrollment and progress tracking |
| `guilds.rs` | `GuildManager` — manages guild creation, membership (join / leave), and per-guild XP aggregation |
| `leaderboard.rs` | `LeaderboardManager` — maintains category and guild leaderboards, capped at 50 entries |
| `reputation.rs` | `ReputationManager` — computes composite reputation scores from XP, endorsements, and activity |
| `seasons.rs` | `SeasonManager` — lifecycle management of competitive seasons with final leaderboard snapshots |
| `social.rs` | `SocialManager` — peer endorsements with daily limits, public recognition badges |
| `storage.rs` | `GamificationStorage` — all persistent state access through a typed `GamificationKey` enum |
| `types.rs` | Shared `contracttype`-derived structs: `GamificationProfile`, `Achievement`, `Challenge`, `Guild`, `Season`, `PeerEndorsement`, etc. |
| `events.rs` | `GamificationEvents` — standardized event emission for all state changes |
| `errors.rs` | `GamificationError` — 25 typed error variants |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; seeds 25 default achievements and default config | Admin |
| `record_activity(user, activity)` | Records a learning activity; returns newly awarded achievement IDs | User |
| `get_user_profile(user)` | Returns (or creates) the full gamification profile for a user | None |
| `get_adaptive_difficulty(user)` | Returns computed adaptive difficulty settings for a user | None |
| `create_achievement(admin, achievement)` | Creates a custom achievement beyond the 25 seeded milestones | Admin |
| `get_user_achievements(user)` | Lists all achievements earned by a user | None |
| `claim_achievement_reward(user, achievement_id)` | Claims the token reward attached to an earned achievement | User |
| `get_leaderboard(category, limit)` | Returns up to `limit` leaderboard entries for the given category | None |
| `get_guild_leaderboard()` | Returns the global guild leaderboard sorted by total XP | None |
| `create_challenge(admin, challenge)` | Creates a new time-bound challenge | Admin |
| `join_challenge(user, challenge_id)` | Enrolls a user in an active challenge | User |
| `update_challenge_progress(user, challenge_id, progress)` | Updates progress; returns `true` when challenge is completed | User |
| `get_active_challenges()` | Lists all currently active (non-expired) challenges | None |
| `get_user_challenge_status(user, challenge_id)` | Returns enrollment details for a user on a specific challenge | None |
| `get_user_active_challenges(user)` | Returns IDs of all challenges a user is enrolled in | None |
| `create_guild(creator, name, description, max_members, is_public)` | Creates a new guild with the caller as founding member | User |
| `join_guild(user, guild_id)` | Joins an existing public guild | User |
| `leave_guild(user)` | Removes a user from their current guild | User |
| `get_guild(guild_id)` | Fetches a guild by ID | None |
| `get_guild_members(guild_id)` | Lists all current members of a guild | None |
| `create_season(admin, season)` | Creates a new competitive season (only one may be active at a time) | Admin |
| `get_active_season()` | Returns the currently active season, or `None` | None |
| `end_season(admin)` | Ends the current season after its `end_time` has passed | Admin |
| `get_season_leaderboard(season_id)` | Returns the final leaderboard snapshot for a completed season | None |
| `endorse_peer(endorser, endorsee, skill)` | Endorses a peer for a skill; awards XP to both parties | User |
| `recognize_peer(from, to, recognition_type, message)` | Sends a public recognition badge to another user | User |
| `get_user_endorsements(user)` | Lists all endorsements received by a user | None |
| `get_reputation(user)` | Returns the computed reputation score for a user | None |
| `get_admin()` | Returns the stored admin address | None |

## Usage Example

```
# 1. Admin initializes the contract
gamification.initialize(admin_address)

# 2. Record a learning activity to trigger achievement checks
awarded_ids = gamification.record_activity(student, {
    activity_type: "ModuleCompleted",
    course_id: "RUST101",
    module_id: "M1"
})

# 3. Student claims the token reward for an earned achievement
reward_amount = gamification.claim_achievement_reward(student, awarded_ids[0])

# 4. Admin creates a week-long challenge
challenge_id = gamification.create_challenge(admin, {
    title: "7-Day Sprint",
    target_progress: 100,
    xp_reward: 500,
    start_time: now,
    end_time: now + 604800
})

# 5. Student joins and tracks progress
gamification.join_challenge(student, challenge_id)
completed = gamification.update_challenge_progress(student, challenge_id, 50)

# 6. Students form a guild and endorse each other
guild_id = gamification.create_guild(student, "Rust Masters", "...", 20, true)
gamification.join_guild(other_student, guild_id)
gamification.endorse_peer(student, other_student, "Rust")

# 7. Check leaderboard
top_users = gamification.get_leaderboard(LeaderboardCategory::XP, 10)
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Contract has already been initialized |
| `NotInitialized` | 2 | Contract has not been initialized yet |
| `Unauthorized` | 3 | Caller lacks required admin or ownership privileges |
| `InvalidAmount` | 4 | Token or XP amount is zero or invalid |
| `InvalidInput` | 5 | One or more input fields fail validation |
| `NotFound` | 6 | Requested resource was not found |
| `AlreadyExists` | 7 | A resource with the same identifier already exists |
| `ChallengeFull` | 8 | Challenge has reached maximum participant count |
| `ChallengeInactive` | 9 | Challenge is not currently active |
| `ChallengeExpired` | 10 | Challenge deadline has passed |
| `ChallengeNotStarted` | 11 | Challenge start time has not been reached |
| `GuildFull` | 12 | Guild has reached maximum member count |
| `AlreadyInGuild` | 13 | User is already a member of a guild |
| `NotInGuild` | 14 | User is not a member of any guild |
| `SeasonAlreadyActive` | 15 | A season is already running |
| `SeasonInactive` | 16 | No active season to interact with |
| `SelfEndorsement` | 17 | User attempted to endorse themselves |
| `EndorsementLimitReached` | 18 | Daily endorsement limit has been reached |
| `AchievementAlreadyClaimed` | 19 | Reward for this achievement was already claimed |
| `PrerequisiteNotMet` | 20 | A prerequisite achievement has not been earned |
| `AlreadyJoinedChallenge` | 21 | User has already joined this challenge |
| `NotJoinedChallenge` | 22 | User has not joined this challenge |
| `GuildNameTooLong` | 23 | Guild name exceeds the maximum allowed length |
| `SeasonNotEnded` | 24 | Season end time has not been reached |
| `InsufficientXP` | 25 | User does not have enough XP for this action |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `token` | Distributes token rewards when achievement rewards are claimed |
| `analytics` | Activity records and XP milestones feed into the analytics pipeline via events |
| `community` | Peer endorsement XP is reflected in community reputation scores |
| `progress` | Learning activity events can be cross-referenced with the progress tracker |
