use crate::types::*;
use soroban_sdk::{Address, Env, String};

pub struct AnalyticsManager;

impl AnalyticsManager {
    pub fn track_search(
        env: &Env,
        query_id: String,
        query_text: String,
        user: &Address,
        results_count: u32,
    ) -> Result<SearchQuery, Error> {
        let search = SearchQuery {
            query_id: query_id.clone(),
            query_text,
            user: user.clone(),
            timestamp: env.ledger().timestamp(),
            results_count,
            clicked_result: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::SearchQuery(query_id), &search);

        Ok(search)
    }

    pub fn track_click(env: &Env, query_id: String, doc_id: String) -> Result<(), Error> {
        let mut search: SearchQuery = env
            .storage()
            .persistent()
            .get(&DataKey::SearchQuery(query_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        search.clicked_result = Some(doc_id);
        env.storage()
            .persistent()
            .set(&DataKey::SearchQuery(query_id), &search);

        Ok(())
    }

    pub fn get_document_analytics(env: &Env, doc_id: &String) -> Option<DocumentAnalytics> {
        env.storage()
            .persistent()
            .get(&DataKey::Analytics(doc_id.clone()))
    }

    pub fn update_analytics(
        env: &Env,
        doc_id: String,
        views: u32,
        helpful: u32,
        not_helpful: u32,
    ) -> Result<(), Error> {
        let mut analytics: DocumentAnalytics = env
            .storage()
            .persistent()
            .get(&DataKey::Analytics(doc_id.clone()))
            .unwrap_or(DocumentAnalytics {
                doc_id: doc_id.clone(),
                total_views: 0,
                unique_viewers: 0,
                avg_time_spent: 0,
                helpful_votes: 0,
                not_helpful_votes: 0,
                completion_rate: 0,
                search_appearances: 0,
            });

        analytics.total_views += views;
        analytics.helpful_votes += helpful;
        analytics.not_helpful_votes += not_helpful;

        env.storage()
            .persistent()
            .set(&DataKey::Analytics(doc_id), &analytics);

        Ok(())
    }
}
