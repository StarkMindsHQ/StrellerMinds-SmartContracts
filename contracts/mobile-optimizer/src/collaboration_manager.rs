use soroban_sdk::{Address, Env, String, Vec};

use crate::types::*;

pub struct CollaborationManager;

impl CollaborationManager {
    pub fn create_study_group(
        env: &Env,
        creator: &Address,
        group_id: String,
        name: String,
        topic: String,
        max_members: u32,
    ) -> Result<StudyGroup, MobileOptimizerError> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::StudyGroup(group_id.clone()))
        {
            return Err(MobileOptimizerError::CollaborationError);
        }

        let mut members = Vec::new(env);
        members.push_back(creator.clone());

        let group = StudyGroup {
            group_id: group_id.clone(),
            name,
            creator: creator.clone(),
            members,
            topic,
            created_at: env.ledger().timestamp(),
            is_active: true,
            max_members,
        };

        env.storage()
            .persistent()
            .set(&DataKey::StudyGroup(group_id), &group);
        Self::update_profile_stats(env, creator, |p| p.groups_joined += 1);

        Ok(group)
    }

    pub fn join_study_group(
        env: &Env,
        user: &Address,
        group_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let mut group: StudyGroup = env
            .storage()
            .persistent()
            .get(&DataKey::StudyGroup(group_id.clone()))
            .ok_or(MobileOptimizerError::CollaborationError)?;

        if !group.is_active || group.members.len() >= group.max_members {
            return Err(MobileOptimizerError::CollaborationError);
        }

        for m in group.members.iter() {
            if m == *user {
                return Ok(());
            }
        }

        group.members.push_back(user.clone());
        env.storage()
            .persistent()
            .set(&DataKey::StudyGroup(group_id), &group);
        Self::update_profile_stats(env, user, |p| p.groups_joined += 1);

        Ok(())
    }

    pub fn create_post(
        env: &Env,
        author: &Address,
        post_id: String,
        group_id: String,
        content: String,
        parent_id: Option<String>,
    ) -> Result<ForumPost, MobileOptimizerError> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::ForumPost(post_id.clone()))
        {
            return Err(MobileOptimizerError::CollaborationError);
        }

        let post = ForumPost {
            post_id: post_id.clone(),
            group_id,
            author: author.clone(),
            content,
            timestamp: env.ledger().timestamp(),
            upvotes: 0,
            parent_id,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ForumPost(post_id), &post);
        Ok(post)
    }

    pub fn submit_review(
        env: &Env,
        reviewer: &Address,
        target_user: &Address,
        review_id: String,
        context_id: String,
        score: u32,
        comments: String,
    ) -> Result<PeerReview, MobileOptimizerError> {
        if score > 100 {
            return Err(MobileOptimizerError::InvalidInput);
        }

        let review = PeerReview {
            review_id: review_id.clone(),
            reviewer: reviewer.clone(),
            target_user: target_user.clone(),
            context_id,
            score,
            comments,
            timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::PeerReview(review_id), &review);
        Self::update_profile_stats(env, reviewer, |p| p.reviews_given += 1);

        Self::update_profile_stats(env, target_user, |p| {
            p.reputation_score = (p.reputation_score + score) / 2;
        });

        Ok(review)
    }

    pub fn request_mentorship(
        env: &Env,
        mentee: &Address,
        mentor: &Address,
        session_id: String,
        topic: String,
        scheduled_at: u64,
        duration_minutes: u32,
    ) -> Result<MentorshipSession, MobileOptimizerError> {
        let session = MentorshipSession {
            session_id: session_id.clone(),
            mentor: mentor.clone(),
            mentee: mentee.clone(),
            topic,
            status: MentorshipStatus::Pending,
            scheduled_at,
            duration_minutes,
        };

        env.storage()
            .persistent()
            .set(&DataKey::MentorshipSession(session_id), &session);
        Ok(session)
    }

    pub fn update_mentorship_status(
        env: &Env,
        caller: &Address,
        session_id: String,
        new_status: MentorshipStatus,
    ) -> Result<(), MobileOptimizerError> {
        let mut session: MentorshipSession = env
            .storage()
            .persistent()
            .get(&DataKey::MentorshipSession(session_id.clone()))
            .ok_or(MobileOptimizerError::CollaborationError)?;

        if *caller != session.mentor && *caller != session.mentee {
            return Err(MobileOptimizerError::Unauthorized);
        }

        session.status = new_status.clone();
        env.storage()
            .persistent()
            .set(&DataKey::MentorshipSession(session_id), &session);

        if matches!(new_status, MentorshipStatus::Completed) {
            Self::update_profile_stats(env, &session.mentor, |p| p.mentorships_completed += 1);
        }

        Ok(())
    }

    pub fn get_profile(env: &Env, user: &Address) -> CollaborationProfile {
        env.storage()
            .persistent()
            .get(&DataKey::CollabProfile(user.clone()))
            .unwrap_or(CollaborationProfile {
                user: user.clone(),
                reputation_score: 50,
                groups_joined: 0,
                reviews_given: 0,
                mentorships_completed: 0,
                badges: Vec::new(env),
            })
    }

    fn update_profile_stats<F>(env: &Env, user: &Address, f: F)
    where
        F: FnOnce(&mut CollaborationProfile),
    {
        let mut profile = Self::get_profile(env, user);
        f(&mut profile);
        env.storage()
            .persistent()
            .set(&DataKey::CollabProfile(user.clone()), &profile);
    }
}