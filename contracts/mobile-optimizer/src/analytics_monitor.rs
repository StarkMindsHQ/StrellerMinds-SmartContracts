use soroban_sdk::{Address, Env, Map, String, Vec};

use crate::types::*;

pub struct AnalyticsMonitor;

impl AnalyticsMonitor {
    pub fn track_event(
        env: &Env,
        user: &Address,
        event_type: AnalyticsEventType,
        properties: Map<String, String>,
        session_id: String,
        device_type: DeviceType,
    ) -> Result<AnalyticsEvent, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let event = AnalyticsEvent {
            event_id: String::from_str(env, "evt"),
            user: user.clone(),
            event_type,
            timestamp: now,
            properties,
            session_id,
            device_type,
        };

        let mut events: Vec<AnalyticsEvent> = env
            .storage()
            .persistent()
            .get(&DataKey::AnalyticsEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        events.push_back(event.clone());

        if events.len() > 500 {
            let mut trimmed = Vec::new(env);
            for i in (events.len() - 250)..events.len() {
                if let Some(e) = events.get(i) {
                    trimmed.push_back(e);
                }
            }
            events = trimmed;
        }

        env.storage()
            .persistent()
            .set(&DataKey::AnalyticsEvents(user.clone()), &events);

        Ok(event)
    }

    pub fn record_performance_metrics(
        env: &Env,
        _user: &Address,
        metrics: PerformanceMetrics,
    ) -> Result<(), MobileOptimizerError> {
        env.storage().persistent().set(
            &DataKey::PerformanceLog(metrics.session_id.clone()),
            &metrics,
        );
        Ok(())
    }

    pub fn update_user_engagement(
        env: &Env,
        user: &Address,
        session_duration_seconds: u64,
        courses_accessed: u32,
        modules_completed: u32,
    ) -> Result<UserEngagement, MobileOptimizerError> {
        let now = env.ledger().timestamp();
        let mut engagement: UserEngagement = env
            .storage()
            .persistent()
            .get(&DataKey::UserEngagement(user.clone()))
            .unwrap_or(UserEngagement {
                user: user.clone(),
                daily_active_time_seconds: 0,
                sessions_today: 0,
                courses_accessed: 0,
                modules_completed: 0,
                streak_days: 0,
                last_active: 0,
                engagement_score: 0,
            });

        let day_boundary = now - (now % 86400);
        let last_day = engagement.last_active - (engagement.last_active % 86400);

        if day_boundary != last_day {
            engagement.daily_active_time_seconds = 0;
            engagement.sessions_today = 0;

            if day_boundary == last_day + 86400 {
                engagement.streak_days += 1;
            } else if engagement.last_active > 0 {
                engagement.streak_days = 1;
            }
        }

        engagement.daily_active_time_seconds += session_duration_seconds;
        engagement.sessions_today += 1;
        engagement.courses_accessed += courses_accessed;
        engagement.modules_completed += modules_completed;
        engagement.last_active = now;
        engagement.engagement_score = Self::calculate_engagement_score(&engagement);

        env.storage()
            .persistent()
            .set(&DataKey::UserEngagement(user.clone()), &engagement);

        Ok(engagement)
    }

    pub fn get_user_engagement(
        env: &Env,
        user: &Address,
    ) -> Result<UserEngagement, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::UserEngagement(user.clone()))
            .ok_or(MobileOptimizerError::AnalyticsNotAvailable)
    }

    pub fn get_analytics_events(env: &Env, user: &Address) -> Vec<AnalyticsEvent> {
        env.storage()
            .persistent()
            .get(&DataKey::AnalyticsEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn get_mobile_analytics(
        env: &Env,
        user: &Address,
        device_id: String,
        period_start: u64,
        period_end: u64,
    ) -> Result<MobileAnalytics, MobileOptimizerError> {
        let events: Vec<AnalyticsEvent> = env
            .storage()
            .persistent()
            .get(&DataKey::AnalyticsEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut total_ops = 0u32;
        let mut successful = 0u32;
        let mut failed = 0u32;
        let mut network_dist = Map::new(env);

        for event in events.iter() {
            if event.timestamp >= period_start && event.timestamp <= period_end {
                total_ops += 1;
                match event.event_type {
                    AnalyticsEventType::ErrorOccurred => failed += 1,
                    _ => successful += 1,
                }
            }
        }

        network_dist.set(String::from_str(env, "good"), total_ops);

        let analytics = MobileAnalytics {
            user: user.clone(),
            device_id,
            session_count: total_ops / 5 + 1,
            total_operations: total_ops,
            successful_operations: successful,
            failed_operations: failed,
            average_gas_used: if total_ops > 0 { 45000 } else { 0 },
            network_quality_distribution: network_dist,
            common_operation_types: Vec::new(env),
            optimization_impact: OptimizationImpact {
                gas_savings_pct: 15,
                op_success_rate_improvement: 10,
                avg_response_improve_ms: 200,
                battery_reduction_pct: 12,
                data_reduction_pct: 20,
            },
            period_start,
            period_end,
        };

        Ok(analytics)
    }

    pub fn get_analytics_dashboard(env: &Env) -> AnalyticsDashboard {
        env.storage()
            .persistent()
            .get(&DataKey::AnalyticsDashboard)
            .unwrap_or(AnalyticsDashboard {
                total_users: 0,
                active_users_24h: 0,
                active_users_7d: 0,
                total_sessions: 0,
                avg_session_duration_seconds: 0,
                offline_usage_percentage: 0,
                cache_hit_rate_bps: 0,
                avg_sync_time_ms: 0,
                error_rate_bps: 0,
                top_devices: Vec::new(env),
            })
    }

    pub fn update_analytics_dashboard(
        env: &Env,
        dashboard: AnalyticsDashboard,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .set(&DataKey::AnalyticsDashboard, &dashboard);
        Ok(())
    }

    pub fn get_performance_metrics(
        env: &Env,
        session_id: String,
    ) -> Result<PerformanceMetrics, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::PerformanceLog(session_id))
            .ok_or(MobileOptimizerError::AnalyticsNotAvailable)
    }

    fn calculate_engagement_score(engagement: &UserEngagement) -> u32 {
        let mut score = 0u32;

        let daily_minutes = engagement.daily_active_time_seconds / 60;
        score += if daily_minutes > 60 {
            30
        } else {
            (daily_minutes as u32) / 2
        };

        score += if engagement.streak_days > 30 {
            30
        } else {
            engagement.streak_days
        };

        score += if engagement.modules_completed > 20 {
            20
        } else {
            engagement.modules_completed
        };

        score += if engagement.sessions_today > 5 {
            10
        } else {
            engagement.sessions_today * 2
        };

        score += if engagement.courses_accessed > 5 {
            10
        } else {
            engagement.courses_accessed * 2
        };

        if score > 100 {
            100
        } else {
            score
        }
    }
}
