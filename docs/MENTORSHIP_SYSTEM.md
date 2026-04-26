# Mentorship System

The StrellerMinds Mentorship System provides a decentralized platform for students (mentees) to connect with experts (mentors) for guidance, skill development, and career advice.

## Core Features

- **Mentor Registration**: Experts can create profiles detailing their expertise, bio, and availability.
- **Mentorship Matching**: Mentees can request mentorship from registered mentors based on their expertise.
- **Session Scheduling**: Mentors and mentees can manage mentorship sessions with status tracking (Pending, Active, Completed).
- **Ratings & Reviews**: Mentees can provide feedback and ratings for mentors after sessions.

## API Reference

### `register_mentor`

```rust
register_mentor(
    mentor: Address,
    name: String,
    expertise: Vec<String>,
    bio: String,
) -> Result<(), MentorshipError>
```

Registers a new mentor profile. The mentor must authorize the transaction.

### `request_mentorship`

```rust
request_mentorship(
    mentee: Address,
    mentor: Address,
) -> Result<u64, MentorshipError>
```

Sends a mentorship request to a mentor. Returns a unique session ID.

### `update_session_status`

```rust
update_session_status(
    user: Address,
    session_id: u64,
    new_status: MentorshipStatus,
) -> Result<(), MentorshipError>
)
```

Updates the status of a mentorship session. Transitions:
- `Pending` -> `Active` (Mentor must approve)
- `Active` -> `Completed`
- `Pending` -> `Rejected` (Mentor)
- `Pending` -> `Cancelled` (Mentee)

### `submit_review`

```rust
submit_review(
    reviewer: Address,
    target: Address,
    rating: u32,
    comment: String,
) -> Result<(), MentorshipError>
```

Submits a rating (1-5) and comment for a mentor or mentee.

## Data Structures

### `MentorProfile`

| Field | Type | Description |
|-------|------|-------------|
| `address` | `Address` | Unique address of the mentor. |
| `name` | `String` | Display name. |
| `expertise` | `Vec<String>` | List of skill tags. |
| `rating_sum` | `u32` | Sum of all ratings received. |
| `rating_count` | `u32` | Total number of ratings. |

### `MentorshipSession`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Unique session ID. |
| `mentor` | `Address` | Address of the mentor. |
| `mentee` | `Address` | Address of the mentee. |
| `status` | `MentorshipStatus` | Current state of the session. |
| `start_time` | `u64` | Ledger timestamp when session started. |

## Error Codes

| Code | Description |
|------|-------------|
| `Unauthorized` | Caller does not have permission for the action. |
| `AlreadyRegistered` | Mentor profile already exists. |
| `InvalidRating` | Rating must be between 1 and 5. |
| `SelfMentorshipNotAllowed` | Mentee cannot be the same as the mentor. |
