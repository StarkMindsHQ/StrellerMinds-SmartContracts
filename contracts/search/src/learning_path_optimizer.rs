use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

/// Learning Path Optimizer
/// Manages personalized learning paths using AI-optimized sequences
/// Tracks skill dependencies and learning progression
pub struct LearningPathOptimizer;

impl LearningPathOptimizer {
    /// Store optimized learning path from oracle
    pub fn store_learning_path(env: &Env, user: Address, path: LearningPath) {
        let key = DataKey::LearningPath(user.clone(), path.path_id.clone());
        env.storage().persistent().set(&key, &path);

        // Store individual skill nodes for quick lookup
        for i in 0..path.skill_nodes.len() {
            if let Some(node) = path.skill_nodes.get(i) {
                Self::store_skill_node(env, &node.skill_id, &node);
            }
        }

        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("path_stor"),),
            (user, path.steps.len()),
        );
    }

    /// Get learning path for user
    pub fn get_learning_path(env: &Env, user: Address) -> Option<LearningPath> {
        let key = DataKey::LearningPath(user, String::from_str(env, "default"));
        env.storage().persistent().get(&key)
    }

    /// Store skill node metadata
    fn store_skill_node(env: &Env, skill_id: &String, node: &SkillNode) {
        let key = DataKey::SkillNode(skill_id.clone());
        env.storage().persistent().set(&key, node);
    }

    /// Get skill node metadata
    pub fn get_skill_node(env: &Env, skill_id: String) -> Option<SkillNode> {
        let key = DataKey::SkillNode(skill_id);
        env.storage().persistent().get(&key)
    }

    /// Update path step completion
    pub fn complete_step(
        env: &Env,
        user: Address,
        step_id: String,
        completion_score: u32, // 0-100
    ) {
        let key = DataKey::LearningPath(user.clone(), String::from_str(env, "default"));

        if let Some(mut path) = env
            .storage()
            .persistent()
            .get::<DataKey, LearningPath>(&key)
        {
            // Update step status
            for i in 0..path.steps.len() {
                if let Some(mut step) = path.steps.get(i) {
                    if step.content_id == step_id {
                        step.completed = true;
                        // Update the step in the vector
                        path.steps.set(i, step);
                        break;
                    }
                }
            }

            // Update path progress
            path.progress_percentage = Self::calculate_path_progress(&path);

            // Save updated path
            env.storage().persistent().set(&key, &path);

            // Emit event for off-chain re-optimization
            env.events().publish(
                (soroban_sdk::symbol_short!("step_comp"),),
                (user, step_id, completion_score),
            );
        }
    }

    /// Calculate overall path progress
    fn calculate_path_progress(path: &LearningPath) -> u32 {
        if path.steps.is_empty() {
            return 0;
        }

        let mut completed = 0u32;

        for i in 0..path.steps.len() {
            if let Some(step) = path.steps.get(i) {
                if step.completed {
                    completed += 1;
                }
            }
        }

        (completed * 100) / path.steps.len()
    }

    /// Get next recommended step in path
    pub fn get_next_step(env: &Env, user: Address) -> Option<PathStep> {
        if let Some(path) = Self::get_learning_path(env, user) {
            // Find first incomplete step
            for i in 0..path.steps.len() {
                if let Some(step) = path.steps.get(i) {
                    if !step.completed {
                        return Some(step);
                    }
                }
            }
        }
        None
    }

    /// Get remaining steps in path
    pub fn get_remaining_steps(env: &Env, user: Address) -> Vec<PathStep> {
        let mut remaining = Vec::new(env);

        if let Some(path) = Self::get_learning_path(env, user) {
            for i in 0..path.steps.len() {
                if let Some(step) = path.steps.get(i) {
                    if !step.completed {
                        remaining.push_back(step);
                    }
                }
            }
        }

        remaining
    }

    /// Check if prerequisites are met for content
    pub fn check_prerequisites(env: &Env, user: Address, content_id: String) -> bool {
        // Get content's required skills
        let required_skills = Self::get_content_required_skills(env, content_id);

        // Get user's learning path to check progress
        if let Some(path) = Self::get_learning_path(env, user) {
            // Check if all required skills are in completed steps
            for i in 0..required_skills.len() {
                if let Some(required) = required_skills.get(i) {
                    let mut has_skill = false;

                    // Check completed steps
                    for j in 0..path.steps.len() {
                        if let Some(step) = path.steps.get(j) {
                            if step.completed && step.skill_id == required {
                                has_skill = true;
                                break;
                            }
                        }
                    }

                    if !has_skill {
                        return false; // Missing prerequisite
                    }
                }
            }
        }

        true // All prerequisites met
    }

    /// Get required skills for content (from skill graph)
    fn get_content_required_skills(env: &Env, content_id: String) -> Vec<String> {
        // This would be populated by oracle based on skill graph
        // Return empty for now
        Vec::new(env)
    }

    /// Estimate time to complete path
    pub fn estimate_completion_time(env: &Env, user: Address) -> u32 {
        if let Some(path) = Self::get_learning_path(env, user) {
            return path.estimated_duration_days;
        }
        0
    }

    /// Calculate skill mastery level
    pub fn get_skill_mastery(env: &Env, user: Address, skill_id: String) -> u32 {
        if let Some(path) = Self::get_learning_path(env, user) {
            // Find steps related to this skill
            let mut total_completion = 0u32;
            let mut count = 0u32;

            for i in 0..path.steps.len() {
                if let Some(step) = path.steps.get(i) {
                    if step.skill_id == skill_id && step.completed {
                        total_completion += step.estimated_effort;
                        count += 1;
                    }
                }
            }

            if count > 0 {
                total_completion / count
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Identify skill gaps for user
    pub fn identify_skill_gaps(
        env: &Env,
        user: Address,
        target_skills: Vec<String>,
    ) -> Vec<String> {
        let mut gaps = Vec::new(env);

        if let Some(path) = Self::get_learning_path(env, user) {
            // Check which target skills are not in completed steps
            for i in 0..target_skills.len() {
                if let Some(target) = target_skills.get(i) {
                    let mut has_skill = false;

                    for j in 0..path.steps.len() {
                        if let Some(step) = path.steps.get(j) {
                            if step.skill_id == target && step.completed {
                                has_skill = true;
                                break;
                            }
                        }
                    }

                    if !has_skill {
                        gaps.push_back(target);
                    }
                }
            }
        } else {
            // No path = all skills are gaps
            return target_skills;
        }

        gaps
    }

    /// Recommend path adjustment based on performance
    pub fn recommend_path_adjustment(env: &Env, user: Address) {
        // Emit event for off-chain path re-optimization
        env.events()
            .publish((soroban_sdk::symbol_short!("adj_path"),), user);
    }

    /// Get path difficulty score
    pub fn get_path_difficulty(env: &Env, user: Address) -> u32 {
        if let Some(path) = Self::get_learning_path(env, user) {
            let mut total_difficulty = 0u32;

            for i in 0..path.steps.len() {
                if let Some(step) = path.steps.get(i) {
                    total_difficulty += step.estimated_effort;
                }
            }

            if path.steps.len() > 0 {
                total_difficulty / path.steps.len()
            } else {
                50 // Default medium
            }
        } else {
            50
        }
    }

    /// Get adaptive difficulty recommendation
    pub fn get_adaptive_difficulty(env: &Env, user: Address, current_step: String) -> u32 {
        // Get user's recent completion scores
        // This would be tracked by oracle and adjusted dynamically
        // Return medium difficulty for now
        50
    }

    /// Find alternative paths to same goal
    pub fn find_alternative_paths(
        env: &Env,
        target_skill: String,
        user_level: u32,
    ) -> Vec<LearningPath> {
        // This would be computed off-chain by AI
        // Return empty for now - oracle would populate
        Vec::new(env)
    }
}
