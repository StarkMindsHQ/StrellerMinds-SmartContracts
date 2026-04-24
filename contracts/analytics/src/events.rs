#![allow(dead_code)]
use crate::types::LeaderboardMetric;
use shared::event_schema::{
    AnalyticsEventData, EventData, MetricsUpdatedEvent, SessionCompletedEvent,
    SessionRecordedEvent, StandardEvent,
};
use soroban_sdk::{Address, BytesN, Env, Symbol};

/// Analytics contract events
pub struct AnalyticsEvents;

impl AnalyticsEvents {
    pub fn emit_session_recorded(env: &Env, session_id: &BytesN<32>, student: &Address) {
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(AnalyticsEventData::SessionRecorded(SessionRecordedEvent {
                session_id: session_id.clone(),
            })),
        )
        .emit(env);
    }

    pub fn emit_session_completed(env: &Env, session_id: &BytesN<32>, student: &Address) {
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(AnalyticsEventData::SessionCompleted(SessionCompletedEvent {
                session_id: session_id.clone(),
            })),
        )
        .emit(env);
    }

    pub fn emit_data_aggregated(
        env: &Env,
        _course_id: &Symbol,
        _date: u64,
        active_students: u32,
        _total_sessions: u32,
    ) {
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(AnalyticsEventData::MetricsUpdated(MetricsUpdatedEvent {
                metric_id: Symbol::new(env, "data_aggregated"),
                new_value: active_students as u64,
            })),
        )
        .emit(env);
    }

    pub fn emit_leaderboard_updated(
        env: &Env,
        _course_id: &Symbol,
        _metric_type: LeaderboardMetric,
        _top_student: &Address,
        top_score: u32,
        _total_entries: u32,
    ) {
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            env.current_contract_address(),
            EventData::Analytics(AnalyticsEventData::MetricsUpdated(MetricsUpdatedEvent {
                metric_id: Symbol::new(env, "leaderboard_upd"),
                new_value: top_score as u64,
            })),
        )
        .emit(env);
    }

    pub fn emit_report_generated(
        env: &Env,
        student: &Address,
        _course_id: &Symbol,
        _report_period: &str,
        _start_date: u64,
        _end_date: u64,
        sessions_count: u32,
    ) {
        StandardEvent::new(
            env,
            Symbol::new(env, "analytics"),
            student.clone(),
            EventData::Analytics(AnalyticsEventData::MetricsUpdated(MetricsUpdatedEvent {
                metric_id: Symbol::new(env, "report_generated"),
                new_value: sessions_count as u64,
            })),
        )
        .emit(env);
    }
}
