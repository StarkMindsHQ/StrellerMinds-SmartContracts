use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct ForumManager;

impl ForumManager {
    pub fn create_post(
        env: &Env,
        author: &Address,
        category: ForumCategory,
        title: String,
        content: String,
        tags: Vec<String>,
        course_id: String,
    ) -> Result<u64, Error> {
        let post_id = CommunityStorage::increment_counter(env, CommunityKey::PostCounter);
        let now = env.ledger().timestamp();

        let post = ForumPost {
            id: post_id,
            author: author.clone(),
            category: category.clone(),
            title,
            content,
            status: PostStatus::Active,
            created_at: now,
            updated_at: now,
            views: 0,
            replies_count: 0,
            upvotes: 0,
            downvotes: 0,
            is_pinned: false,
            tags,
            course_id,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Post(post_id), &post);

        // Add to category index
        let mut category_posts: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::CategoryPosts(category))
            .unwrap_or_else(|| Vec::new(env));
        category_posts.push_back(post_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::CategoryPosts(post.category), &category_posts);

        // Add to user posts
        let mut user_posts: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserPosts(author.clone()))
            .unwrap_or_else(|| Vec::new(env));
        user_posts.push_back(post_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::UserPosts(author.clone()), &user_posts);

        // Update user stats
        Self::update_user_stats(env, author, 1, 0, 0);

        // Award XP
        let config = CommunityStorage::get_config(env);
        Self::award_xp(env, author, config.post_xp_reward);

        CommunityEvents::emit_post_created(env, author, post_id);
        Ok(post_id)
    }

    pub fn create_reply(
        env: &Env,
        author: &Address,
        post_id: u64,
        content: String,
        parent_reply_id: u64,
    ) -> Result<u64, Error> {
        let mut post: ForumPost = env
            .storage()
            .persistent()
            .get(&CommunityKey::Post(post_id))
            .ok_or(Error::PostNotFound)?;

        if post.status == PostStatus::Closed {
            return Err(Error::PostClosed);
        }

        let reply_id = CommunityStorage::increment_counter(env, CommunityKey::ReplyCounter);
        let now = env.ledger().timestamp();

        let reply = ForumReply {
            id: reply_id,
            post_id,
            author: author.clone(),
            content,
            created_at: now,
            updated_at: now,
            upvotes: 0,
            downvotes: 0,
            is_solution: false,
            parent_reply_id,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Reply(reply_id), &reply);

        // Add to post replies
        let mut replies: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::PostReplies(post_id))
            .unwrap_or_else(|| Vec::new(env));
        replies.push_back(reply_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::PostReplies(post_id), &replies);

        // Update post
        post.replies_count += 1;
        post.updated_at = now;
        env.storage()
            .persistent()
            .set(&CommunityKey::Post(post_id), &post);

        // Update user stats
        Self::update_user_stats(env, author, 0, 1, 0);

        // Award XP
        let config = CommunityStorage::get_config(env);
        Self::award_xp(env, author, config.reply_xp_reward);

        CommunityEvents::emit_reply_created(env, author, post_id, reply_id);
        Ok(reply_id)
    }

    pub fn mark_solution(
        env: &Env,
        post_author: &Address,
        post_id: u64,
        reply_id: u64,
    ) -> Result<(), Error> {
        let post: ForumPost = env
            .storage()
            .persistent()
            .get(&CommunityKey::Post(post_id))
            .ok_or(Error::PostNotFound)?;

        if post.author != *post_author {
            return Err(Error::Unauthorized);
        }

        let mut reply: ForumReply = env
            .storage()
            .persistent()
            .get(&CommunityKey::Reply(reply_id))
            .ok_or(Error::ReplyNotFound)?;

        if reply.post_id != post_id {
            return Err(Error::InvalidInput);
        }

        reply.is_solution = true;
        env.storage()
            .persistent()
            .set(&CommunityKey::Reply(reply_id), &reply);

        // Update post status
        let mut updated_post = post;
        updated_post.status = PostStatus::Resolved;
        env.storage()
            .persistent()
            .set(&CommunityKey::Post(post_id), &updated_post);

        // Update reply author stats
        Self::update_user_stats(env, &reply.author, 0, 0, 1);

        // Award solution XP
        let config = CommunityStorage::get_config(env);
        Self::award_xp(env, &reply.author, config.solution_xp_reward);

        CommunityEvents::emit_solution_marked(env, post_id, reply_id);
        Ok(())
    }

    pub fn vote_post(env: &Env, voter: &Address, post_id: u64, upvote: bool) -> Result<(), Error> {
        let vote_key = CommunityKey::PostVote(voter.clone(), post_id);
        if env.storage().persistent().has(&vote_key) {
            return Err(Error::AlreadyVoted);
        }

        let mut post: ForumPost = env
            .storage()
            .persistent()
            .get(&CommunityKey::Post(post_id))
            .ok_or(Error::PostNotFound)?;

        if upvote {
            post.upvotes += 1;
        } else {
            post.downvotes += 1;
        }

        env.storage()
            .persistent()
            .set(&CommunityKey::Post(post_id), &post);
        env.storage().persistent().set(&vote_key, &upvote);

        // Update author's helpful votes
        if upvote {
            let mut stats = Self::get_user_stats(env, &post.author);
            stats.helpful_votes_received += 1;
            env.storage()
                .persistent()
                .set(&CommunityKey::UserStats(post.author), &stats);
        }

        Ok(())
    }

    pub fn get_post(env: &Env, post_id: u64) -> Option<ForumPost> {
        let mut post: Option<ForumPost> =
            env.storage().persistent().get(&CommunityKey::Post(post_id));

        if let Some(ref mut p) = post {
            p.views += 1;
            env.storage()
                .persistent()
                .set(&CommunityKey::Post(post_id), p);
        }

        post
    }

    pub fn get_post_replies(env: &Env, post_id: u64) -> Vec<ForumReply> {
        let reply_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::PostReplies(post_id))
            .unwrap_or_else(|| Vec::new(env));

        let mut replies = Vec::new(env);
        for id in reply_ids.iter() {
            if let Some(reply) = env.storage().persistent().get(&CommunityKey::Reply(id)) {
                replies.push_back(reply);
            }
        }
        replies
    }

    pub fn get_category_posts(env: &Env, category: ForumCategory, limit: u32) -> Vec<ForumPost> {
        let post_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::CategoryPosts(category))
            .unwrap_or_else(|| Vec::new(env));

        let mut posts = Vec::new(env);
        let max = limit.min(post_ids.len());
        for i in 0..max {
            if let Some(id) = post_ids.get(i) {
                if let Some(post) = env.storage().persistent().get(&CommunityKey::Post(id)) {
                    posts.push_back(post);
                }
            }
        }
        posts
    }

    // Helper functions
    fn update_user_stats(env: &Env, user: &Address, posts: u32, replies: u32, solutions: u32) {
        let mut stats = Self::get_user_stats(env, user);
        stats.posts_created += posts;
        stats.replies_given += replies;
        stats.solutions_provided += solutions;
        env.storage()
            .persistent()
            .set(&CommunityKey::UserStats(user.clone()), &stats);
    }

    fn get_user_stats(env: &Env, user: &Address) -> UserCommunityStats {
        env.storage()
            .persistent()
            .get(&CommunityKey::UserStats(user.clone()))
            .unwrap_or(UserCommunityStats {
                user: user.clone(),
                posts_created: 0,
                replies_given: 0,
                solutions_provided: 0,
                contributions_made: 0,
                events_attended: 0,
                mentorship_sessions: 0,
                helpful_votes_received: 0,
                reputation_score: 0,
                joined_at: env.ledger().timestamp(),
            })
    }

    fn award_xp(_env: &Env, _user: &Address, _xp: u32) {
        // Integration point with gamification contract
        // This would call the gamification contract to award XP
    }
}
