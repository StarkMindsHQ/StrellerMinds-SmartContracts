use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Vec};

pub struct UserExperienceManager;

impl UserExperienceManager {
    #[allow(clippy::too_many_arguments)]
    pub fn set_ui_preferences(
        env: &Env,
        user: &Address,
        theme_id: String,
        language: String,
        font_scale: u32,
        high_contrast: bool,
        reduce_motion: bool,
        layout_mode: LayoutMode,
        accessibility_settings: Map<String, bool>,
    ) -> Result<UiPreferences, MobileOptimizerError> {
        let prefs = UiPreferences {
            user: user.clone(),
            theme_id,
            language,
            font_scale,
            high_contrast,
            reduce_motion,
            layout_mode,
            accessibility_settings,
        };

        env.storage()
            .persistent()
            .set(&DataKey::UiPreferences(user.clone()), &prefs);
        Ok(prefs)
    }

    pub fn get_ui_preferences(
        env: &Env,
        user: &Address,
    ) -> Result<UiPreferences, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::UiPreferences(user.clone()))
            .ok_or(MobileOptimizerError::ConfigNotFound)
    }

    pub fn update_onboarding_progress(
        env: &Env,
        user: &Address,
        step_id: String,
        is_skipped: bool,
    ) -> Result<OnboardingState, MobileOptimizerError> {
        let mut state = env
            .storage()
            .persistent()
            .get(&DataKey::OnboardingState(user.clone()))
            .unwrap_or(OnboardingState {
                user: user.clone(),
                is_completed: false,
                current_step: 0,
                completed_steps: Vec::new(env),
                skipped_steps: Vec::new(env),
                last_updated: env.ledger().timestamp(),
            });

        if is_skipped {
            state.skipped_steps.push_back(step_id);
        } else {
            state.completed_steps.push_back(step_id);
        }

        state.current_step += 1;
        state.last_updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::OnboardingState(user.clone()), &state);
        Ok(state)
    }

    pub fn complete_onboarding(env: &Env, user: &Address) -> Result<(), MobileOptimizerError> {
        let mut state: OnboardingState = env
            .storage()
            .persistent()
            .get(&DataKey::OnboardingState(user.clone()))
            .ok_or(MobileOptimizerError::InvalidInput)?;
        state.is_completed = true;
        state.last_updated = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::OnboardingState(user.clone()), &state);
        Ok(())
    }

    pub fn submit_feedback(
        env: &Env,
        user: &Address,
        category: String,
        rating: u32,
        comment: String,
        context_data: Map<String, String>,
    ) -> Result<UserFeedback, MobileOptimizerError> {
        // Generate a simple ID based on timestamp and user (in production, use a better ID generation strategy)
        let timestamp = env.ledger().timestamp();
        let mut feedback_id = String::from_str(env, "fb_");
        // Note: Soroban String concatenation is limited, this is a simplified representation

        let feedback = UserFeedback {
            feedback_id,
            user: user.clone(),
            category,
            rating,
            comment,
            context_data,
            timestamp,
        };

        // Store individual feedback (in a real system, we might index this better)
        // For now, we just append to user history
        let mut history: Vec<UserFeedback> = env
            .storage()
            .persistent()
            .get(&DataKey::UserFeedbackHistory(user.clone()))
            .unwrap_or(Vec::new(env));
        history.push_back(feedback.clone());
        env.storage()
            .persistent()
            .set(&DataKey::UserFeedbackHistory(user.clone()), &history);

        Ok(feedback)
    }
}
