# StrellerMinds Smart Contracts API Documentation

## Overview

This document provides comprehensive API documentation for all StrellerMinds smart contracts, including detailed request/response examples, error handling, and integration patterns.

## Base Configuration

### Network Endpoints

```javascript
// Mainnet
const mainnetConfig = {
  rpcUrl: 'https://horizon.stellar.org',
  networkPassphrase: 'Public Global Stellar Network ; September 2015',
};

// Testnet
const testnetConfig = {
  rpcUrl: 'https://horizon-testnet.stellar.org',
  networkPassphrase: 'Test SDF Network ; September 2015',
};

// Futurenet
const futurenetConfig = {
  rpcUrl: 'https://horizon-futurenet.stellar.org',
  networkPassphrase: 'Test SDF Future Network ; October 2022',
};
```

### Contract Addresses

```javascript
// Example contract addresses (replace with actual deployed addresses)
const contracts = {
  shared: 'CD3ZC2LSL3ZE3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q',
  analytics: 'CA3ZC2LSL3ZE3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q',
  token: 'CB3ZC2LSL3ZE3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q3Q',
};
```

## Shared Contract API

### Access Control Endpoints

#### Initialize Access Control

**Endpoint**: `initialize`

**Description**: Initializes the access control system with an administrator.

**Request**:
```json
{
  "method": "initialize",
  "params": {
    "admin": "GD5... (Stellar Address)"
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "message": "Access control initialized successfully"
  }
}
```

**Error Response**:
```json
{
  "error": {
    "code": 4001,
    "message": "Contract already initialized",
    "type": "AccessControlError"
  }
}
```

**Example Usage**:
```javascript
const result = await sharedContract.call(
  "initialize",
  new Address("GD5...ADMIN_ADDRESS...")
);
```

#### Grant Role

**Endpoint**: `grant_role`

**Description**: Grants a specific role to a user.

**Request**:
```json
{
  "method": "grant_role",
  "params": {
    "caller": "GD5...ADMIN_ADDRESS...",
    "role": "Student",
    "user": "GD5...USER_ADDRESS..."
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "role_granted": {
      "user": "GD5...USER_ADDRESS...",
      "role": "Student",
      "granted_at": 1640995200,
      "granted_by": "GD5...ADMIN_ADDRESS..."
    }
  }
}
```

**Error Response**:
```json
{
  "error": {
    "code": 4003,
    "message": "Insufficient privilege to grant this role",
    "type": "AccessControlError"
  }
}
```

**Example Usage**:
```javascript
const result = await sharedContract.call(
  "grant_role",
  adminAddress,
  Role.Student,
  userAddress
);
```

#### Revoke Role

**Endpoint**: `revoke_role`

**Description**: Revokes a specific role from a user.

**Request**:
```json
{
  "method": "revoke_role",
  "params": {
    "caller": "GD5...ADMIN_ADDRESS...",
    "role": "Instructor",
    "user": "GD5...USER_ADDRESS..."
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "role_revoked": {
      "user": "GD5...USER_ADDRESS...",
      "role": "Instructor",
      "revoked_at": 1640995200,
      "revoked_by": "GD5...ADMIN_ADDRESS..."
    }
  }
}
```

#### Check Role

**Endpoint**: `has_role`

**Description**: Checks if a user has a specific role.

**Request**:
```json
{
  "method": "has_role",
  "params": {
    "user": "GD5...USER_ADDRESS...",
    "role": "Student"
  }
}
```

**Response**:
```json
{
  "result": {
    "has_role": true,
    "user": "GD5...USER_ADDRESS...",
    "role": "Student"
  }
}
```

**Example Usage**:
```javascript
const hasStudentRole = await sharedContract.call(
  "has_role",
  userAddress,
  Role.Student
);
```

### Reentrancy Guard Endpoints

#### Initialize Reentrancy Guard

**Endpoint**: `initialize_reentrancy_guard`

**Description**: Initializes the reentrancy protection system.

**Request**:
```json
{
  "method": "initialize_reentrancy_guard",
  "params": {}
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "guard_initialized": true
  }
}
```

#### Enter Protected Section

**Endpoint**: `enter_protected`

**Description**: Enters a reentrancy-protected section.

**Request**:
```json
{
  "method": "enter_protected",
  "params": {
    "function_name": "transfer_tokens"
  }
}
```

**Response**:
```json
{
  "result": {
    "lock_acquired": true,
    "lock_id": "abc123...",
    "function_name": "transfer_tokens"
  }
}
```

**Error Response**:
```json
{
  "error": {
    "code": 5001,
    "message": "Reentrant call detected",
    "type": "ReentrancyError"
  }
}
```

#### Exit Protected Section

**Endpoint**: `exit_protected`

**Description**: Exits a reentrancy-protected section.

**Request**:
```json
{
  "method": "exit_protected",
  "params": {
    "lock_id": "abc123..."
  }
}
```

**Response**:
```json
{
  "result": {
    "lock_released": true,
    "lock_id": "abc123..."
  }
}
```

### Validation Endpoints

#### Validate Address

**Endpoint**: `validate_address`

**Description**: Validates a Stellar address format.

**Request**:
```json
{
  "method": "validate_address",
  "params": {
    "address": "GD5...ADDRESS..."
  }
}
```

**Response**:
```json
{
  "result": {
    "is_valid": true,
    "address": "GD5...ADDRESS...",
    "validation_details": {
      "format_valid": true,
      "checksum_valid": true,
      "network_valid": true
    }
  }
}
```

**Error Response**:
```json
{
  "error": {
    "code": 3001,
    "message": "Invalid address format",
    "type": "ValidationError"
  }
}
```

#### Validate Amount

**Endpoint**: `validate_amount`

**Description**: Validates an amount within specified range.

**Request**:
```json
{
  "method": "validate_amount",
  "params": {
    "amount": 1000,
    "min_amount": 1,
    "max_amount": 1000000
  }
}
```

**Response**:
```json
{
  "result": {
    "is_valid": true,
    "amount": 1000,
    "within_range": true
  }
}
```

## Analytics Contract API

### Session Management Endpoints

#### Record Session

**Endpoint**: `record_session`

**Description**: Records the start of a new learning session.

**Request**:
```json
{
  "method": "record_session",
  "params": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "module_id": "basics",
    "session_type": "Lecture",
    "learning_objectives": ["understand_syntax", "master_ownership"]
  }
}
```

**Response**:
```json
{
  "result": {
    "session_id": "abc123def456...",
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "start_time": 1640995200,
    "status": "Active"
  }
}
```

**Example Usage**:
```javascript
const sessionId = await analyticsContract.call(
  "record_session",
  studentAddress,
  Symbol.new("RUST101"),
  Symbol.new("basics"),
  SessionType.Lecture,
  [Symbol.new("understand_syntax"), Symbol.new("master_ownership")]
);
```

#### Update Session

**Endpoint**: `update_session`

**Description**: Updates an existing learning session with new metrics.

**Request**:
```json
{
  "method": "update_session",
  "params": {
    "session_id": "abc123def456...",
    "metrics": {
      "time_spent": 1800,
      "pages_viewed": 5,
      "exercises_completed": 3,
      "interaction_count": 25,
      "engagement_score": 85
    },
    "events": [
      {
        "type": "page_view",
        "timestamp": 1640995300,
        "data": {"page": "introduction"}
      },
      {
        "type": "exercise_complete",
        "timestamp": 1640995400,
        "data": {"exercise_id": "ex1", "score": 95}
      }
    ]
  }
}
```

**Response**:
```json
{
  "result": {
    "session_updated": true,
    "session_id": "abc123def456...",
    "updated_at": 1640995500,
    "metrics_summary": {
      "total_time": 1800,
      "total_interactions": 25,
      "current_engagement": 85
    }
  }
}
```

#### Complete Session

**Endpoint**: `complete_session`

**Description**: Completes a learning session with final metrics.

**Request**:
```json
{
  "method": "complete_session",
  "params": {
    "session_id": "abc123def456...",
    "final_metrics": {
      "score": 92.5,
      "completion_percentage": 100,
      "engagement_score": 90,
      "difficulty_rating": 6,
      "satisfaction_level": 85
    },
    "learning_outcomes": ["understand_syntax", "master_ownership"],
    "feedback": {
      "rating": 5,
      "comments": "Great learning experience!"
    }
  }
}
```

**Response**:
```json
{
  "result": {
    "session_completed": true,
    "session_id": "abc123def456...",
    "completion_data": {
      "end_time": 1640997000,
      "total_duration": 1800,
      "final_score": 92.5,
      "achievements_unlocked": ["first_session", "high_engagement"]
    }
  }
}
```

### Progress Analytics Endpoints

#### Get Progress Analytics

**Endpoint**: `get_progress_analytics`

**Description**: Retrieves comprehensive progress analytics for a student.

**Request**:
```json
{
  "method": "get_progress_analytics",
  "params": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101"
  }
}
```

**Response**:
```json
{
  "result": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "completion_percentage": 75,
    "average_score": 85.5,
    "total_time_spent": 10800,
    "modules_completed": 6,
    "total_modules": 8,
    "learning_streak": 5,
    "performance_trend": {
      "direction": "Improving",
      "strength": 75,
      "recent_average": 87.2,
      "change_percentage": 8.5
    },
    "predicted_completion": 1643568000,
    "current_difficulty": {
      "level": 6,
      "adaptive_factor": 1.1,
      "recommended_level": 7
    },
    "mastery_level": "Intermediate",
    "engagement_metrics": {
      "average_session_time": 1800,
      "interaction_rate": 0.85,
      "consistency_score": 90
    },
    "knowledge_gaps": [
      {
        "topic": "lifetimes",
        "severity": "Medium",
        "recommended_resources": ["lifetime_tutorial", "lifetime_exercises"]
      }
    ],
    "recent_achievements": [
      {
        "id": "week_streak",
        "name": "Week Warrior",
        "earned_at": 1640995200
      }
    ]
  }
}
```

#### Update Progress Analytics

**Endpoint**: `update_progress_analytics`

**Description**: Updates progress analytics based on new session data.

**Request**:
```json
{
  "method": "update_progress_analytics",
  "params": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "session_data": {
      "session_id": "abc123def456...",
      "score": 92.5,
      "completion_percentage": 100,
      "time_spent": 1800
    }
  }
}
```

**Response**:
```json
{
  "result": {
    "analytics_updated": true,
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "updated_fields": [
      "completion_percentage",
      "average_score",
      "learning_streak",
      "performance_trend"
    ],
    "new_achievements": ["module_complete", "high_score"]
  }
}
```

### Course Analytics Endpoints

#### Get Course Analytics

**Endpoint**: `get_course_analytics`

**Description**: Retrieves comprehensive analytics for an entire course.

**Request**:
```json
{
  "method": "get_course_analytics",
  "params": {
    "course_id": "RUST101"
  }
}
```

**Response**:
```json
{
  "result": {
    "course_id": "RUST101",
    "total_enrollments": 150,
    "active_students": 45,
    "completed_students": 105,
    "completion_rate": 70.0,
    "average_completion_time": 2592000,
    "average_score": 82.5,
    "score_distribution": {
      "a_count": 15,
      "b_count": 35,
      "c_count": 30,
      "d_count": 20,
      "f_count": 5,
      "median": 83.0,
      "standard_deviation": 12.5
    },
    "module_analytics": [
      {
        "module_id": "basics",
        "completed_count": 120,
        "average_completion_time": 432000,
        "average_score": 88.0,
        "difficulty_rating": 4,
        "engagement_level": 85
      },
      {
        "module_id": "ownership",
        "completed_count": 95,
        "average_completion_time": 600000,
        "average_score": 79.5,
        "difficulty_rating": 7,
        "engagement_level": 75
      }
    ],
    "engagement_metrics": {
      "average_session_time": 2100,
      "interaction_rate": 0.78,
      "participation_rate": 0.85,
      "retention_rate": 0.92
    },
    "dropout_analysis": {
      "total_dropouts": 45,
      "average_time_to_dropout": 1209600,
      "common_reasons": ["difficulty", "time_constraints", "lack_of_engagement"],
      "high_risk_periods": ["week_2", "module_3"]
    }
  }
}
```

#### Update Course Analytics

**Endpoint**: `update_course_analytics`

**Description**: Updates course analytics based on new student data.

**Request**:
```json
{
  "method": "update_course_analytics",
  "params": {
    "course_id": "RUST101",
    "student_updates": [
      {
        "student": "GD5...STUDENT1...",
        "session_completed": true,
        "score": 92.5
      },
      {
        "student": "GD5...STUDENT2...",
        "session_completed": false,
        "dropout_reason": "time_constraints"
      }
    ]
  }
}
```

**Response**:
```json
{
  "result": {
    "analytics_updated": true,
    "course_id": "RUST101",
    "updated_metrics": {
      "new_completion_rate": 71.2,
      "new_average_score": 82.8,
      "active_students": 44
    },
    "trend_changes": {
      "completion_trend": "Increasing",
      "score_trend": "Stable",
      "engagement_trend": "Decreasing"
    }
  }
}
```

### Leaderboard Endpoints

#### Generate Leaderboard

**Endpoint**: `generate_leaderboard`

**Description**: Generates a performance leaderboard for a course.

**Request**:
```json
{
  "method": "generate_leaderboard",
  "params": {
    "course_id": "RUST101",
    "metric": "OverallScore",
    "limit": 50
  }
}
```

**Response**:
```json
{
  "result": {
    "course_id": "RUST101",
    "metric": "OverallScore",
    "generated_at": 1640995200,
    "total_participants": 105,
    "entries": [
      {
        "rank": 1,
        "student": "GD5...TOP_STUDENT...",
        "display_name": "Alice Johnson",
        "score": 98.5,
        "badge": "gold_star",
        "rank_change": 0,
        "achievement_count": 12,
        "learning_streak": 15,
        "last_activity": 1640995000
      },
      {
        "rank": 2,
        "student": "GD5...SECOND_STUDENT...",
        "display_name": "Bob Smith",
        "score": 96.2,
        "badge": "silver_star",
        "rank_change": 1,
        "achievement_count": 10,
        "learning_streak": 12,
        "last_activity": 1640994800
      }
    ],
    "user_rank": {
      "student": "GD5...CURRENT_USER...",
      "rank": 15,
      "score": 85.7,
      "percentile": 85.7
    }
  }
}
```

#### Get Leaderboard Position

**Endpoint**: `get_leaderboard_position`

**Description**: Gets a specific student's position on the leaderboard.

**Request**:
```json
{
  "method": "get_leaderboard_position",
  "params": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "metric": "OverallScore"
  }
}
```

**Response**:
```json
{
  "result": {
    "student": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "metric": "OverallScore",
    "rank": 15,
    "score": 85.7,
    "total_participants": 105,
    "percentile": 85.7,
    "rank_change": 3,
    "nearby_rankings": [
      {
        "rank": 13,
        "student": "GD5...ABOVE_1...",
        "score": 87.2
      },
      {
        "rank": 14,
        "student": "GD5...ABOVE_2...",
        "score": 86.5
      },
      {
        "rank": 16,
        "student": "GD5...BELOW_1...",
        "score": 84.9
      }
    ]
  }
}
```

## Token Contract API

### Token Management Endpoints

#### Initialize Token

**Endpoint**: `initialize`

**Description**: Initializes the token contract with administrative settings.

**Request**:
```json
{
  "method": "initialize",
  "params": {
    "admin": "GD5...ADMIN_ADDRESS...",
    "token_config": {
      "name": "StrellerMinds Token",
      "symbol": "SMT",
      "decimals": 7,
      "initial_supply": 1000000000
    }
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "token_initialized": {
      "name": "StrellerMinds Token",
      "symbol": "SMT",
      "decimals": 7,
      "admin": "GD5...ADMIN_ADDRESS...",
      "initial_supply": 1000000000
    }
  }
}
```

#### Mint Tokens

**Endpoint**: `mint`

**Description**: Mints new tokens and assigns them to a recipient.

**Request**:
```json
{
  "method": "mint",
  "params": {
    "to": "GD5...RECIPIENT_ADDRESS...",
    "amount": 1000
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "minted": {
      "recipient": "GD5...RECIPIENT_ADDRESS...",
      "amount": 1000,
      "new_total_supply": 1000001000,
      "transaction_id": "tx123..."
    }
  }
}
```

**Error Response**:
```json
{
  "error": {
    "code": 6001,
    "message": "Insufficient minting permissions",
    "type": "TokenError"
  }
}
```

#### Transfer Tokens

**Endpoint**: `transfer`

**Description**: Transfers tokens from one address to another.

**Request**:
```json
{
  "method": "transfer",
  "params": {
    "from": "GD5...SENDER_ADDRESS...",
    "to": "GD5...RECIPIENT_ADDRESS...",
    "amount": 500
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "transferred": {
      "from": "GD5...SENDER_ADDRESS...",
      "to": "GD5...RECIPIENT_ADDRESS...",
      "amount": 500,
      "transaction_id": "tx456..."
    }
  }
}
```

#### Get Balance

**Endpoint**: `balance`

**Description**: Retrieves the token balance for a specified address.

**Request**:
```json
{
  "method": "balance",
  "params": {
    "account": "GD5...ACCOUNT_ADDRESS..."
  }
}
```

**Response**:
```json
{
  "result": {
    "account": "GD5...ACCOUNT_ADDRESS...",
    "balance": 2500,
    "last_updated": 1640995200
  }
}
```

### Reward System Endpoints

#### Reward Course Completion

**Endpoint**: `reward_course_completion`

**Description**: Awards tokens to a student for completing a course.

**Request**:
```json
{
  "method": "reward_course_completion",
  "params": {
    "user": "GD5...STUDENT_ADDRESS...",
    "course_id": "RUST101",
    "completion_percentage": 100,
    "final_score": 92.5,
    "time_spent": 2592000
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "reward_awarded": {
      "student": "GD5...STUDENT_ADDRESS...",
      "course_id": "RUST101",
      "base_reward": 200,
      "performance_bonus": 46,
      "speed_bonus": 50,
      "total_reward": 296,
      "new_balance": 2796
    }
  }
}
```

#### Create Achievement

**Endpoint**: `create_achievement`

**Description**: Creates a new achievement with associated token reward.

**Request**:
```json
{
  "method": "create_achievement",
  "params": {
    "admin": "GD5...ADMIN_ADDRESS...",
    "achievement": {
      "id": "rust_master",
      "name": "Rust Programming Master",
      "description": "Complete all Rust programming courses with distinction",
      "category": "Mastery",
      "achievement_type": "Tiered",
      "criteria": {
        "type": "CourseCompletion",
        "course_count": 5,
        "min_score": 90
      },
      "reward_amount": 500,
      "rarity": "Legendary",
      "prerequisites": ["rust_basics", "rust_advanced"]
    }
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "achievement_created": {
      "id": "rust_master",
      "name": "Rust Programming Master",
      "reward_amount": 500,
      "created_at": 1640995200,
      "created_by": "GD5...ADMIN_ADDRESS..."
    }
  }
}
```

#### Award Achievement

**Endpoint**: "award_achievement"

**Description**: Awards an achievement to a student and grants the token reward.

**Request**:
```json
{
  "method": "award_achievement",
  "params": {
    "student": "GD5...STUDENT_ADDRESS...",
    "achievement_id": "rust_master"
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "achievement_awarded": {
      "student": "GD5...STUDENT_ADDRESS...",
      "achievement_id": "rust_master",
      "reward_amount": 500,
      "awarded_at": 1640995200,
      "new_balance": 3296,
      "total_achievements": 8
    }
  }
}
```

### Staking System Endpoints

#### Create Staking Pool

**Endpoint**: `create_staking_pool`

**Description**: Creates a new staking pool with specified parameters.

**Request**:
```json
{
  "method": "create_staking_pool",
  "params": {
    "admin": "GD5...ADMIN_ADDRESS...",
    "pool": {
      "id": "rust_course_staking",
      "name": "Rust Course Learning Staking",
      "apy_percentage": 15.0,
      "min_stake_amount": 100,
      "lock_period_seconds": 2592000,
      "max_total_stake": 50000
    }
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "pool_created": {
      "pool_id": "rust_course_staking",
      "name": "Rust Course Learning Staking",
      "apy_percentage": 15.0,
      "created_at": 1640995200,
      "total_staked": 0
    }
  }
}
```

#### Stake Tokens

**Endpoint**: `stake_tokens`

**Description**: Stakes tokens in a specified staking pool.

**Request**:
```json
{
  "method": "stake_tokens",
  "params": {
    "user": "GD5...STUDENT_ADDRESS...",
    "pool_id": "rust_course_staking",
    "amount": 1000
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "tokens_staked": {
      "user": "GD5...STUDENT_ADDRESS...",
      "pool_id": "rust_course_staking",
      "amount": 1000,
      "stake_id": "stake123...",
      "lock_until": 1643587200,
      "expected_rewards": 150
    }
  }
}
```

#### Get Staking Positions

**Endpoint**: `get_staking_positions`

**Description**: Retrieves all staking positions for a user.

**Request**:
```json
{
  "method": "get_staking_positions",
  "params": {
    "user": "GD5...STUDENT_ADDRESS..."
  }
}
```

**Response**:
```json
{
  "result": {
    "user": "GD5...STUDENT_ADDRESS...",
    "positions": [
      {
        "stake_id": "stake123...",
        "pool_id": "rust_course_staking",
        "amount": 1000,
        "staked_at": 1640995200,
        "lock_until": 1643587200,
        "current_rewards": 25,
        "apy_percentage": 15.0,
        "status": "Active"
      },
      {
        "stake_id": "stake456...",
        "pool_id": "learning_commitment",
        "amount": 500,
        "staked_at": 1640908800,
        "lock_until": 1643500800,
        "current_rewards": 12,
        "apy_percentage": 20.0,
        "status": "Active"
      }
    ],
    "total_staked": 1500,
    "total_pending_rewards": 37
  }
}
```

### Certificate Upgrade Endpoints

#### Burn for Upgrade

**Endpoint**: `burn_for_upgrade`

**Description**: Burns tokens to upgrade a certificate.

**Request**:
```json
{
  "method": "burn_for_upgrade",
  "params": {
    "user": "GD5...STUDENT_ADDRESS...",
    "amount": 200,
    "certificate_id": "cert_rust_101",
    "upgrade_type": "premium"
  }
}
```

**Response**:
```json
{
  "result": {
    "success": true,
    "upgrade_processed": {
      "user": "GD5...STUDENT_ADDRESS...",
      "certificate_id": "cert_rust_101",
      "upgrade_type": "premium",
      "tokens_burned": 200,
      "upgrade_id": "upgrade789...",
      "processed_at": 1640995200,
      "new_certificate_level": "Premium"
    }
  }
}
```

## Error Handling

### Standard Error Format

All API endpoints return errors in a consistent format:

```json
{
  "error": {
    "code": 4001,
    "message": "Human-readable error description",
    "type": "ErrorType",
    "details": {
      "field": "additional_error_context",
      "value": "error_specific_data"
    }
  }
}
```

### Error Codes

| Code | Type | Description |
|------|------|-------------|
| 1001 | AccessControlError | Unauthorized access |
| 1002 | AccessControlError | Invalid role |
| 1003 | AccessControlError | Insufficient privilege |
| 2001 | ValidationError | Invalid address |
| 2002 | ValidationError | Invalid amount |
| 2003 | ValidationError | Invalid string length |
| 3001 | ReentrancyError | Reentrant call detected |
| 3002 | ReentrancyError | Lock acquisition failed |
| 4001 | AnalyticsError | Session not found |
| 4002 | AnalyticsError | Invalid course ID |
| 4003 | AnalyticsError | Insufficient data |
| 5001 | TokenError | Insufficient balance |
| 5002 | TokenError | Insufficient permissions |
| 5003 | TokenError | Transfer failed |
| 6001 | SystemError | Contract not initialized |
| 6002 | SystemError | Storage error |
| 6003 | SystemError | Network error |

### Error Handling Examples

```javascript
try {
  const result = await contract.call("some_method", params);
  console.log("Success:", result);
} catch (error) {
  if (error.code === 1001) {
    console.error("Access denied. Check permissions.");
  } else if (error.code === 5001) {
    console.error("Insufficient balance.");
  } else {
    console.error("Unexpected error:", error.message);
  }
}
```

## Rate Limiting and Gas Optimization

### Rate Limits

```javascript
const rateLimits = {
  // Per user limits
  session_updates_per_hour: 100,
  transfers_per_hour: 50,
  achievement_checks_per_hour: 200,
  
  // Global limits
  total_transactions_per_second: 1000,
  analytics_queries_per_second: 500,
};
```

### Gas Optimization Tips

```javascript
// Batch operations when possible
const batchOperations = async (operations) => {
  const batch = operations.map(op => ({
    contract: op.contract,
    method: op.method,
    params: op.params
  }));
  
  return await client.sendBatchTransaction(batch);
};

// Use efficient data structures
const optimizedSessionData = {
  // Use symbols instead of strings where possible
  course_id: Symbol.new("RUST101"),
  session_type: SessionType.Lecture,
  
  // Batch events instead of individual calls
  events: events.slice(0, 10), // Limit event count
};
```

## Webhook Integration

### Event Webhooks

Configure webhooks to receive real-time notifications:

```javascript
const webhookConfig = {
  url: "https://your-app.com/webhooks/strellerminds",
  events: [
    "session_completed",
    "achievement_awarded",
    "tokens_transferred",
    "course_completed"
  ],
  secret: "webhook_secret_key"
};

// Webhook payload example
{
  "event": "session_completed",
  "timestamp": 1640995200,
  "data": {
    "session_id": "abc123...",
    "student": "GD5...STUDENT...",
    "course_id": "RUST101",
    "final_score": 92.5
  },
  "signature": "sha256=..."
}
```

## SDK Integration

### JavaScript/TypeScript SDK

```typescript
import { StrellerMindsSDK } from '@stellarminds/sdk';

const sdk = new StrellerMindsSDK({
  network: 'testnet',
  contracts: {
    shared: 'shared_contract_address',
    analytics: 'analytics_contract_address',
    token: 'token_contract_address'
  }
});

// Initialize user
await sdk.initializeUser('student_address', 'student_role');

// Start learning session
const session = await sdk.analytics.startSession({
  student: 'student_address',
  courseId: 'RUST101',
  sessionType: 'Lecture'
});

// Update progress
await sdk.analytics.updateSession(session.sessionId, {
  timeSpent: 1800,
  pagesViewed: 5,
  exercisesCompleted: 3
});
```

### Python SDK

```python
from stellarminds_sdk import StrellerMindsClient

client = StrellerMindsClient(
    network='testnet',
    contracts={
        'shared': 'shared_contract_address',
        'analytics': 'analytics_contract_address',
        'token': 'token_contract_address'
    }
)

# Initialize user
client.initialize_user('student_address', 'student_role')

# Get progress analytics
progress = client.analytics.get_progress_analytics(
    student='student_address',
    course_id='RUST101'
)

print(f"Completion: {progress.completion_percentage}%")
print(f"Average Score: {progress.average_score}")
```

## Testing and Development

### Test Environment Setup

```javascript
const testConfig = {
  network: 'futurenet',
  contracts: {
    shared: 'test_shared_address',
    analytics: 'test_analytics_address',
    token: 'test_token_address'
  },
  faucet: 'https://friendbot.stellar.org'
};

// Fund test accounts
await fundTestAccount('test_address', 10000);
```

### Mock Responses for Testing

```javascript
const mockResponses = {
  'record_session': {
    session_id: 'test_session_123',
    start_time: Date.now(),
    status: 'Active'
  },
  'get_progress_analytics': {
    completion_percentage: 75,
    average_score: 85.5,
    learning_streak: 5
  }
};
```

This comprehensive API documentation provides all the necessary information for integrating with the StrellerMinds smart contracts, including detailed examples, error handling, and best practices for production deployment.
