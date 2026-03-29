# Gamification Contract

## Overview

The `gamification` contract powers achievements, guilds, challenges, seasons, leaderboards, reputation, and social recognition features across the learning platform.

## Quick Start

```bash
cargo test -p gamification
```

Initialize the contract, configure the active season, then create challenges, achievements, and guild experiences using the public API in [lib.rs](./src/lib.rs).

## Usage Examples

```rust
// Initialize the contract
GamificationContract::initialize(env, admin)?;

// Create a challenge
GamificationContract::create_challenge(env, creator, challenge_id, metadata)?;

// Join and complete challenge activities
GamificationContract::join_challenge(env, participant, challenge_id)?;
GamificationContract::record_activity(env, participant, challenge_id, activity)?;
```

## Contribution Guide

- Keep changes limited to gamification behavior, ranking logic, or social interaction flows.
- Reuse existing storage helpers and types instead of introducing duplicate state paths.
- Extend `src/tests.rs` for new season, challenge, or leaderboard edge cases.

## Troubleshooting

- `AlreadyInitialized`: initialize only once per deployment.
- `ChallengeInactive` or `ChallengeExpired`: verify challenge timing and state before allowing joins or submissions.
- `InsufficientXP`: confirm progression prerequisites before executing reward-gated actions.

## Related Files

- `src/lib.rs`: public contract interface
- `src/achievements.rs`: achievement and reward logic
- `src/leaderboard.rs`: ranking and leaderboard calculations
- `src/tests.rs`: feature and regression tests
