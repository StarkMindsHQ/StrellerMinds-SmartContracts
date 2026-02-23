# Community & Support System Contract

A comprehensive community platform smart contract that enables user support, peer assistance, knowledge sharing, and community engagement while integrating with the learning ecosystem and gamification systems.

## Features

### 1. Forum & Discussion System
- Multi-category forum (General, Course-Specific, Technical Help, Career Advice, etc.)
- Post creation with tags and course association
- Threaded replies with parent-child relationships
- Solution marking for technical questions
- Upvote/downvote system
- Post status management (Active, Resolved, Closed, Pinned, Archived)
- View tracking

### 2. Mentorship Program
- Mentor registration with expertise areas and levels
- Mentorship request system
- Session management and tracking
- Rating system for mentors
- Availability management
- Session notes and duration tracking

### 3. Knowledge Base & Contributions
- Multiple contribution types (Article, Tutorial, Code Snippet, Resource, FAQ)
- Submission and review workflow
- Moderation approval process
- Reward system (XP and tokens)
- Category organization
- View and upvote tracking

### 4. Community Events
- Event creation (Workshop, Webinar, Study Group, Hackathon, Competition, Meetup, AMA)
- Registration system with capacity limits
- Attendance tracking
- Feedback and rating system
- Event status management
- XP rewards for attendance

### 5. Moderation System
- Role-based moderation (Moderator, Senior Moderator, Admin)
- Content reporting with multiple reason categories
- Report review and resolution workflow
- Moderator actions (warn, mute, ban, delete)
- Action history tracking
- Rate limiting for reports

### 6. Community Governance
- Proposal system (Feature Request, Policy Change, Community Rule, Event Proposal)
- Reputation-weighted voting
- Voting duration and quorum requirements
- Proposal lifecycle (Draft, Active, Passed, Rejected, Implemented)
- Vote tracking and finalization

### 7. Analytics & Engagement Tracking
- Community-wide metrics
- User activity statistics
- Reputation calculation
- Engagement tracking (posts, replies, solutions, contributions, events, mentorship)
- Helpful votes tracking

## Integration Points

### Gamification Contract
- XP rewards for community activities
- Achievement triggers for milestones
- Reputation system integration
- Leaderboard contributions

### Analytics Contract
- Activity tracking
- Engagement metrics
- Performance analytics
- Trend analysis

### Token Contract
- Token rewards for contributions
- Incentive distribution
- Reward claiming

## Key Types

### Forum Types
- `ForumPost`: Discussion post with metadata
- `ForumReply`: Reply to a post
- `ForumCategory`: Post categorization
- `PostStatus`: Post lifecycle states

### Mentorship Types
- `MentorProfile`: Mentor information and stats
- `MentorshipRequest`: Mentorship request details
- `MentorshipSession`: Completed session record
- `MentorshipStatus`: Request lifecycle states

### Knowledge Types
- `KnowledgeContribution`: User-submitted content
- `ContributionType`: Content categorization
- `ContributionStatus`: Review workflow states

### Event Types
- `CommunityEvent`: Event details
- `EventParticipant`: Participant record
- `EventType`: Event categorization
- `EventStatus`: Event lifecycle states

### Moderation Types
- `ContentReport`: Report details
- `ModeratorAction`: Moderation action record
- `ReportReason`: Report categorization
- `ModeratorRole`: Permission levels

### Governance Types
- `CommunityProposal`: Proposal details
- `ProposalType`: Proposal categorization
- `ProposalStatus`: Proposal lifecycle states

### Analytics Types
- `CommunityMetrics`: Platform-wide statistics
- `UserCommunityStats`: Individual user statistics

## Usage Examples

### Initialize Contract
```rust
community.initialize(&admin);
```

### Create Forum Post
```rust
let post_id = community.create_post(
    &user,
    &ForumCategory::TechnicalHelp,
    &String::from_str(&env, "How to deploy a contract?"),
    &String::from_str(&env, "I need help deploying my first Soroban contract..."),
    &tags,
    &course_id,
);
```

### Register as Mentor
```rust
community.register_mentor(
    &mentor,
    &expertise_areas,
    &MentorExpertise::Expert,
    &5, // max mentees
    &bio,
);
```

### Submit Knowledge Contribution
```rust
let contrib_id = community.submit_contribution(
    &contributor,
    &ContributionType::Tutorial,
    &title,
    &content,
    &ForumCategory::General,
    &tags,
);
```

### Create Community Event
```rust
let event_id = community.create_event(
    &organizer,
    &EventType::Workshop,
    &title,
    &description,
    &start_time,
    &end_time,
    &max_participants,
    &true, // is_public
    &25, // xp_reward
);
```

### Create Governance Proposal
```rust
let proposal_id = community.create_proposal(
    &proposer,
    &ProposalType::FeatureRequest,
    &title,
    &description,
    &86400, // voting_duration (1 day)
    &100, // min_votes_required
);
```

## Configuration

The contract uses a `CommunityConfig` structure for customizable parameters:

- `post_xp_reward`: XP for creating a post (default: 10)
- `reply_xp_reward`: XP for creating a reply (default: 5)
- `solution_xp_reward`: XP for providing a solution (default: 50)
- `contribution_base_xp`: Base XP for contributions (default: 100)
- `contribution_base_tokens`: Base tokens for contributions (default: 1000)
- `mentor_session_xp`: XP for completing a mentorship session (default: 75)
- `event_attendance_xp`: XP for attending an event (default: 25)
- `min_reputation_to_moderate`: Minimum reputation to moderate (default: 500)
- `max_reports_per_day`: Maximum reports per user per day (default: 10)
- `vote_weight_threshold`: Minimum reputation to vote (default: 100)

## Reputation System

User reputation is calculated based on weighted activities:
- Posts created: 10 points each
- Replies given: 5 points each
- Solutions provided: 50 points each
- Contributions made: 100 points each
- Events attended: 25 points each
- Mentorship sessions: 75 points each
- Helpful votes received: 15 points each

## Security Features

- Authentication required for all state-changing operations
- Role-based access control for moderation
- Rate limiting for reports and endorsements
- Reputation-based voting weights
- Admin-only configuration updates
- Moderator approval for contributions

## Testing

Run tests with:
```bash
cargo test
```

The test suite covers:
- Initialization and configuration
- Forum post and reply creation
- Solution marking and voting
- Mentorship workflow
- Knowledge contribution submission and review
- Event creation and registration
- Governance proposals and voting
- Analytics and metrics

## Future Enhancements

- Direct messaging between users
- Notification system
- Advanced search and filtering
- Content recommendation engine
- Reputation decay over time
- Seasonal challenges and competitions
- Integration with external platforms
- Mobile app support
- Real-time chat functionality
- Video conferencing integration for mentorship

## License

See LICENSE file in the repository root.
