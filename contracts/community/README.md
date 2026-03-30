# Community Contract

## Purpose

The Community contract is the social infrastructure layer of the StrellerMinds platform. It brings learners together through threaded forum discussions, a peer mentorship matching system, a moderated knowledge base, community events with attendance tracking, content moderation with report queues, and on-chain governance through proposals and token-weighted voting. XP rewards are integrated throughout — posting, mentoring, attending events, and contributing to the knowledge base all generate on-chain incentives.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — wires together all domain managers and exposes the full public API |
| `forum.rs` | `ForumManager` — threaded posts, nested replies, upvote/downvote, solution marking |
| `mentorship.rs` | `MentorshipManager` — mentor registration, request lifecycle, session completion and rating |
| `knowledge.rs` | `KnowledgeManager` — contribution submission, moderator review, voting on published resources |
| `community_events.rs` | `EventManager` — event creation, registration, attendance marking, XP distribution on completion |
| `moderation.rs` | `ModerationManager` — moderator role assignment, content reporting, report resolution |
| `governance.rs` | `GovernanceManager` — proposal creation, voting, and outcome execution |
| `analytics.rs` | `AnalyticsManager` — aggregates community activity metrics |
| `storage.rs` | `CommunityStorage` — all persistent state via a typed `CommunityKey` enum |
| `types.rs` | `contracttype`-derived structs: `ForumPost`, `ForumReply`, `MentorProfile`, `MentorshipRequest`, `KnowledgeContribution`, `CommunityEvent`, `ModerationReport`, `GovernanceProposal`, `CommunityConfig` |
| `events.rs` | `CommunityEvents` — standardized event emission |
| `errors.rs` | `CommunityError` — 30+ typed error variants across 6 domains |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; seeds default config and counters | Admin |
| **Forum** | | |
| `create_post(author, category, title, content, tags, course_id)` | Creates a new forum post; awards XP to author | User |
| `create_reply(author, post_id, content, parent_reply_id)` | Adds a reply to a post (nested threading supported) | User |
| `mark_solution(post_author, post_id, reply_id)` | Marks a reply as the accepted solution; awards XP | Post Author |
| `vote_post(voter, post_id, upvote)` | Casts an upvote or downvote on a post (once per user) | User |
| `get_post(post_id)` | Returns a post by ID | None |
| `get_post_replies(post_id)` | Returns all replies for a post | None |
| `get_category_posts(category, limit)` | Returns up to `limit` posts from a category | None |
| **Mentorship** | | |
| `register_mentor(mentor, expertise_areas, expertise_level, max_mentees, bio)` | Registers a user as an available mentor | User |
| `request_mentorship(mentee, mentor, topic, message)` | Submits a mentorship request to a specific mentor | User |
| `accept_mentorship(mentor, request_id)` | Accepts a pending mentorship request | Mentor |
| `complete_session(mentor, request_id, duration, notes)` | Records a completed session; awards XP | Mentor |
| `rate_session(mentee, session_id, rating)` | Submits a quality rating for a completed session | Mentee |
| `get_mentor_profile(mentor)` | Returns a mentor's profile | None |
| **Knowledge Base** | | |
| `submit_contribution(contributor, contribution_type, title, content, category, tags)` | Submits a knowledge article for review | User |
| `review_contribution(moderator, contribution_id, approve)` | Approves or rejects a pending contribution | Moderator |
| `vote_contribution(voter, contribution_id, upvote)` | Votes on a published contribution | User |
| `get_contribution(contribution_id)` | Returns a contribution by ID | None |
| `get_user_contributions(user)` | Lists all contributions by a user | None |
| **Events** | | |
| `create_event(organizer, event_type, title, description, start_time, end_time, max_participants, is_public, xp_reward)` | Creates a community event | User |
| `register_for_event(user, event_id)` | Registers a user for an event | User |
| `mark_attendance(organizer, event_id, user)` | Marks an attendee as confirmed | Organizer |
| `complete_event(organizer, event_id)` | Closes an event and distributes XP to attendees | Organizer |
| `submit_event_feedback(user, event_id, rating)` | Submits a rating for a completed event | User |
| `get_event(event_id)` | Returns an event by ID | None |
| **Moderation** | | |
| `add_moderator(admin, moderator, role)` | Grants a moderation role to an address | Admin |
| `report_content(reporter, content_type, content_id, reason, description)` | Files a moderation report against content | User |
| `resolve_report(moderator, report_id, action)` | Resolves a pending report with a moderation action | Moderator |
| **Governance** | | |
| `create_proposal(proposer, title, description, voting_period)` | Creates a governance proposal for community voting | User |
| `vote_on_proposal(voter, proposal_id, in_favor, voting_power)` | Casts a weighted vote on a proposal | User |
| `execute_proposal(executor, proposal_id)` | Executes an approved proposal after voting closes | User |

## Usage Example

```
# 1. Admin initializes the contract
community.initialize(admin)

# 2. Expert registers as a mentor
community.register_mentor(expert, ["Rust", "Soroban"], ExpertLevel, 5, "10 years Rust experience")

# 3. Student requests mentorship and gets accepted
request_id = community.request_mentorship(student, expert, "Smart contracts", "Hi!")
community.accept_mentorship(expert, request_id)

# 4. After a session, mentor records completion and student rates it
session_id = community.complete_session(expert, request_id, 3600, "Covered storage patterns")
community.rate_session(student, session_id, 5)

# 5. Student posts a forum question and another user answers
post_id = community.create_post(student, ForumCategory::Technical, "Soroban storage?", "...", [], "")
reply_id = community.create_reply(other_user, post_id, "Use persistent storage for...", 0)
community.mark_solution(student, post_id, reply_id)

# 6. Community event is created, attendees register, organizer confirms attendance
event_id = community.create_event(organizer, EventType::Workshop, "Soroban Bootcamp", "...", start, end, 50, true, 100)
community.register_for_event(student, event_id)
community.mark_attendance(organizer, event_id, student)
community.complete_event(organizer, event_id)
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Contract has already been initialized |
| `NotInitialized` | 2 | Contract has not been initialized |
| `Unauthorized` | 3 | Caller lacks required privileges |
| `NotFound` | 4 | Requested resource was not found |
| `InvalidInput` | 5 | One or more input fields fail validation |
| `PostNotFound` | 10 | Specified forum post does not exist |
| `ReplyNotFound` | 11 | Specified forum reply does not exist |
| `AlreadyVoted` | 12 | User has already voted on this content |
| `CannotEditPost` | 13 | Post cannot be edited in its current state |
| `PostClosed` | 14 | Post is closed and not accepting replies |
| `MentorNotAvailable` | 20 | Mentor is at capacity or not available |
| `MentorshipNotFound` | 21 | Specified mentorship request does not exist |
| `AlreadyMentor` | 22 | User is already registered as a mentor |
| `MaxMenteesReached` | 23 | Mentor has reached maximum concurrent mentees |
| `InvalidMentorshipStatus` | 24 | Mentorship request cannot be transitioned in this state |
| `ContributionNotFound` | 30 | Specified knowledge contribution does not exist |
| `InvalidContributionStatus` | 31 | Contribution state does not allow this operation |
| `InsufficientReputation` | 32 | User does not have enough reputation for this action |
| `EventNotFound` | 40 | Specified community event does not exist |
| `EventFull` | 41 | Event has reached its maximum participant count |
| `AlreadyRegistered` | 42 | User is already registered for this event |
| `EventNotActive` | 43 | Event is not in an active state |
| `NotModerator` | 50 | Caller does not hold a moderator role |
| `ReportNotFound` | 51 | Specified content report does not exist |
| `ReportLimitReached` | 52 | Reporter has hit their daily report limit |
| `AlreadyReported` | 53 | Reporter already filed a report against this content |
| `ProposalNotFound` | 60 | Specified governance proposal does not exist |
| `VotingClosed` | 61 | Voting window has expired or not yet opened |
| `AlreadyVotedOnProposal` | 62 | User has already voted on this proposal |
| `InsufficientVotingPower` | 63 | User does not have enough voting power |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `gamification` | Community XP rewards (posts, solutions, mentoring) feed gamification profiles |
| `token` | Voting power for governance is derived from token holdings |
| `analytics` | Forum activity, event participation, and knowledge contributions feed analytics |
| `shared` | Uses shared event schema and validation utilities |
