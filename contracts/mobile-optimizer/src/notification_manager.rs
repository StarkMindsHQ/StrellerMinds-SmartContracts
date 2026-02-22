use soroban_sdk::{Address, Env, Map, String, Vec};

use crate::types::*;

pub struct NotificationManager;

impl NotificationManager {
    pub fn initialize_notifications(env: &Env, user: &Address) -> NotificationConfig {
        let mut channel_prefs = Map::new(env);
        channel_prefs.set(String::from_str(env, "push"), true);
        channel_prefs.set(String::from_str(env, "email"), false);
        channel_prefs.set(String::from_str(env, "in_app"), true);

        let config = NotificationConfig {
            user: user.clone(),
            enabled: true,
            quiet_hours_start: 22,
            quiet_hours_end: 8,
            max_daily_notifications: 10,
            channel_preferences: channel_prefs,
            priority_threshold: NotificationPriorityLevel::All,
        };

        env.storage()
            .persistent()
            .set(&DataKey::NotifConfig(user.clone()), &config);
        config
    }

    pub fn create_learning_reminder(
        env: &Env,
        user: &Address,
        reminder_type: ReminderType,
        title: String,
        message: String,
        scheduled_at: u64,
        repeat_interval: RepeatInterval,
        course_id: String,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let reminder_id = Self::generate_reminder_id(env, now);

        let reminder = LearningReminder {
            reminder_id: reminder_id.clone(),
            user: user.clone(),
            reminder_type,
            title,
            message,
            scheduled_at,
            repeat_interval,
            is_active: true,
            last_sent: 0,
            course_id,
        };

        let mut reminders: Vec<LearningReminder> = env
            .storage()
            .persistent()
            .get(&DataKey::Reminders(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        reminders.push_back(reminder.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Reminders(user.clone()), &reminders);

        Ok(reminder)
    }

    pub fn get_pending_notifications(
        env: &Env,
        user: &Address,
    ) -> Result<Vec<LearningReminder>, MobileOptimizerError> {
        let config: NotificationConfig = env
            .storage()
            .persistent()
            .get(&DataKey::NotifConfig(user.clone()))
            .ok_or(MobileOptimizerError::NotificationError)?;

        if !config.enabled {
            return Ok(Vec::new(env));
        }

        let reminders: Vec<LearningReminder> = env
            .storage()
            .persistent()
            .get(&DataKey::Reminders(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let now = env.ledger().timestamp();
        let mut pending = Vec::new(env);

        for reminder in reminders.iter() {
            if reminder.is_active && reminder.scheduled_at <= now {
                let should_send = match reminder.repeat_interval {
                    RepeatInterval::Once => reminder.last_sent == 0,
                    RepeatInterval::Daily => now.saturating_sub(reminder.last_sent) >= 86400,
                    RepeatInterval::Weekly => now.saturating_sub(reminder.last_sent) >= 604800,
                    RepeatInterval::Custom => now.saturating_sub(reminder.last_sent) >= 3600,
                    RepeatInterval::OnEvent => true,
                };

                if should_send && !Self::is_quiet_hours(now, &config) {
                    pending.push_back(reminder.clone());
                }
            }
        }

        Ok(pending)
    }

    pub fn mark_notification_sent(
        env: &Env,
        user: &Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let reminders: Vec<LearningReminder> = env
            .storage()
            .persistent()
            .get(&DataKey::Reminders(user.clone()))
            .ok_or(MobileOptimizerError::NotificationError)?;

        let now = env.ledger().timestamp();
        let mut updated = Vec::new(env);

        for reminder in reminders.iter() {
            let mut r = reminder.clone();
            if r.reminder_id == reminder_id {
                r.last_sent = now;
                if r.repeat_interval == RepeatInterval::Once {
                    r.is_active = false;
                }
            }
            updated.push_back(r);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Reminders(user.clone()), &updated);

        let record = NotificationRecord {
            notification_id: reminder_id,
            user: user.clone(),
            notification_type: ReminderType::DailyStudy,
            sent_at: now,
            read_at: 0,
            action_taken: false,
            delivery_status: DeliveryStatus::Sent,
        };

        let mut history: Vec<NotificationRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::NotifHistory(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        history.push_back(record);
        env.storage()
            .persistent()
            .set(&DataKey::NotifHistory(user.clone()), &history);

        Ok(())
    }

    pub fn cancel_reminder(
        env: &Env,
        user: &Address,
        reminder_id: String,
    ) -> Result<(), MobileOptimizerError> {
        let reminders: Vec<LearningReminder> = env
            .storage()
            .persistent()
            .get(&DataKey::Reminders(user.clone()))
            .ok_or(MobileOptimizerError::NotificationError)?;

        let mut updated = Vec::new(env);
        for reminder in reminders.iter() {
            let mut r = reminder.clone();
            if r.reminder_id == reminder_id {
                r.is_active = false;
            }
            updated.push_back(r);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Reminders(user.clone()), &updated);
        Ok(())
    }

    pub fn update_notification_config(
        env: &Env,
        user: &Address,
        config: NotificationConfig,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .set(&DataKey::NotifConfig(user.clone()), &config);
        Ok(())
    }

    pub fn get_notification_config(
        env: &Env,
        user: &Address,
    ) -> Result<NotificationConfig, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::NotifConfig(user.clone()))
            .ok_or(MobileOptimizerError::NotificationError)
    }

    pub fn create_streak_reminder(
        env: &Env,
        user: &Address,
        _streak_days: u32,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let scheduled = now + 72000; // ~20 hours from now

        let title = String::from_str(env, "Keep your streak alive!");
        let message = String::from_str(env, "Continue learning to maintain your streak");
        let course_id = String::from_str(env, "streak");

        Self::create_learning_reminder(
            env,
            user,
            ReminderType::StreakMaintenance,
            title,
            message,
            scheduled,
            RepeatInterval::Daily,
            course_id,
        )
    }

    pub fn create_inactivity_nudge(
        env: &Env,
        user: &Address,
        _inactive_hours: u64,
    ) -> Result<LearningReminder, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let title = String::from_str(env, "We miss you!");
        let message = String::from_str(env, "Come back and continue your learning journey");
        let course_id = String::from_str(env, "inactivity");

        Self::create_learning_reminder(
            env,
            user,
            ReminderType::InactivityNudge,
            title,
            message,
            now,
            RepeatInterval::Once,
            course_id,
        )
    }

    pub fn get_notification_history(env: &Env, user: &Address) -> Vec<NotificationRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::NotifHistory(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    fn is_quiet_hours(timestamp: u64, config: &NotificationConfig) -> bool {
        let hour_of_day = ((timestamp % 86400) / 3600) as u32;
        if config.quiet_hours_start > config.quiet_hours_end {
            hour_of_day >= config.quiet_hours_start || hour_of_day < config.quiet_hours_end
        } else {
            hour_of_day >= config.quiet_hours_start && hour_of_day < config.quiet_hours_end
        }
    }

    fn generate_reminder_id(env: &Env, _timestamp: u64) -> String {
        String::from_str(env, "reminder_")
    }
}
