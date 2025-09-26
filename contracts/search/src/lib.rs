#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Address, Env, String, Vec, Map, BytesN,
};

mod types;
mod search_manager;
mod indexing;

#[cfg(test)]
mod tests;

pub use types::*;
use search_manager::{SearchManager, SearchError};
use indexing::{SearchIndexer, IndexError};
use shared::reentrancy_guard::ReentrancyLock;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidQuery = 4,
    IndexNotFound = 5,
    TooManyResults = 6,
    InvalidFilters = 7,
    SearchTimeout = 8,
    InvalidPagination = 9,
    PreferencesNotFound = 10,
    HistoryNotFound = 11,
    SavedSearchNotFound = 12,
}

#[contract]
pub struct SearchContract;

#[contractimpl]
impl SearchContract {
    /// Initialize the search contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Initialize default search weights
        let default_weights = SearchWeights {
            title_weight: 10,
            description_weight: 5,
            content_weight: 3,
            tags_weight: 7,
            category_weight: 6,
            instructor_weight: 4,
            metadata_weight: 2,
        };
        env.storage().persistent().set(&DataKey::SearchWeights, &default_weights);

        Ok(())
    }

    /// Execute a comprehensive search query
    pub fn search(
        env: Env,
        query: SearchQuery,
        user: Option<Address>,
    ) -> Result<SearchResults, Error> {
        Self::require_initialized(&env)?;

        if let Some(user_addr) = &user {
            user_addr.require_auth();
        }

        SearchManager::execute_search(&env, query, user)
            .map_err(|e| Self::map_search_error(e))
    }

    /// Save a search query for future use
    pub fn save_search(
        env: Env,
        user: Address,
        name: String,
        description: String,
        query: SearchQuery,
        notification_enabled: bool,
    ) -> Result<String, Error> {
        let _guard = ReentrancyLock::new(&env);
        Self::require_initialized(&env)?;
        user.require_auth();

        let search_id = Self::generate_search_id(&env, &user, &name);
        
        let saved_search = SavedSearch {
            search_id: search_id.clone(),
            user_id: user.clone(),
            name,
            description,
            query,
            created_at: env.ledger().timestamp(),
            last_used: env.ledger().timestamp(),
            use_count: 0,
            is_favorite: false,
            notification_enabled,
            auto_execute: false,
            execution_frequency: None,
        };

        let mut user_searches = env.storage().persistent()
            .get(&DataKey::SavedSearches(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        
        user_searches.push_back(saved_search);
        env.storage().persistent().set(&DataKey::SavedSearches(user), &user_searches);

        Ok(search_id)
    }

    /// Get saved searches for a user
    pub fn get_saved_searches(env: Env, user: Address) -> Result<Vec<SavedSearch>, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        let searches = env.storage().persistent()
            .get(&DataKey::SavedSearches(user))
            .unwrap_or_else(|| Vec::new(&env));

        Ok(searches)
    }

    /// Execute a saved search
    pub fn execute_saved_search(
        env: Env,
        user: Address,
        search_id: String,
    ) -> Result<SearchResults, Error> {
        let _guard = ReentrancyLock::new(&env);
        Self::require_initialized(&env)?;
        user.require_auth();

        let mut user_searches = env.storage().persistent()
            .get(&DataKey::SavedSearches(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        // Find and update the saved search
        let mut found_search = None;
        for i in 0..user_searches.len() {
            if let Some(mut search) = user_searches.get(i) {
                if search.search_id == search_id {
                    search.last_used = env.ledger().timestamp();
                    search.use_count += 1;
                    user_searches.set(i, search.clone());
                    found_search = Some(search);
                    break;
                }
            }
        }

        let saved_search = found_search.ok_or(Error::SavedSearchNotFound)?;
        
        // Update storage
        env.storage().persistent().set(&DataKey::SavedSearches(user.clone()), &user_searches);

        // Execute the search
        SearchManager::execute_search(&env, saved_search.query, Some(user))
            .map_err(|e| Self::map_search_error(e))
    }

    /// Set user search preferences
    pub fn set_search_preferences(
        env: Env,
        user: Address,
        preferences: SearchPreferences,
    ) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        Self::require_initialized(&env)?;
        user.require_auth();

        env.storage().persistent().set(&DataKey::SearchPreferences(user), &preferences);
        Ok(())
    }

    /// Get user search preferences
    pub fn get_search_preferences(env: Env, user: Address) -> Result<SearchPreferences, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        env.storage().persistent()
            .get(&DataKey::SearchPreferences(user.clone()))
            .ok_or(Error::PreferencesNotFound)
    }

    /// Get search history for a user
    pub fn get_search_history(
        env: Env,
        user: Address,
        limit: Option<u32>,
    ) -> Result<Vec<SearchHistoryEntry>, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        let history = env.storage().persistent()
            .get(&DataKey::SearchHistory(user))
            .unwrap_or_else(|| Vec::new(&env));

        if let Some(limit_val) = limit {
            let mut limited_history = Vec::new(&env);
            let end_idx = (limit_val as usize).min(history.len());
            for i in 0..end_idx {
                if let Some(entry) = history.get(i) {
                    limited_history.push_back(entry);
                }
            }
            Ok(limited_history)
        } else {
            Ok(history)
        }
    }

    /// Get search suggestions based on query prefix
    pub fn get_search_suggestions(
        env: Env,
        query_prefix: String,
        limit: Option<u32>,
    ) -> Result<Vec<SearchSuggestion>, Error> {
        Self::require_initialized(&env)?;

        let suggestions = env.storage().persistent()
            .get(&DataKey::SearchSuggestions(query_prefix.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        if let Some(limit_val) = limit {
            let mut limited_suggestions = Vec::new(&env);
            let end_idx = (limit_val as usize).min(suggestions.len());
            for i in 0..end_idx {
                if let Some(suggestion) = suggestions.get(i) {
                    limited_suggestions.push_back(suggestion);
                }
            }
            Ok(limited_suggestions)
        } else {
            Ok(suggestions)
        }
    }

    /// Get search analytics (admin only)
    pub fn get_search_analytics(
        env: Env,
        admin: Address,
        period_start: u64,
        period_end: u64,
    ) -> Result<SearchAnalytics, Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        let analytics_key = DataKey::SearchAnalytics(period_start);
        env.storage().persistent()
            .get(&analytics_key)
            .ok_or(Error::IndexNotFound)
    }

    /// Update search index configuration (admin only)
    pub fn update_index_config(
        env: Env,
        admin: Address,
        index_name: String,
        config: SearchIndexConfig,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        env.storage().persistent().set(&DataKey::IndexConfig(index_name), &config);
        Ok(())
    }

    /// Update search weights (admin only)
    pub fn update_search_weights(
        env: Env,
        admin: Address,
        weights: SearchWeights,
    ) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        env.storage().persistent().set(&DataKey::SearchWeights, &weights);
        Ok(())
    }

    /// Add search suggestions (admin only)
    pub fn add_search_suggestions(
        env: Env,
        admin: Address,
        category: String,
        suggestions: Vec<SearchSuggestion>,
    ) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        env.storage().persistent().set(&DataKey::SearchSuggestions(category), &suggestions);
        Ok(())
    }

    /// Delete a saved search
    pub fn delete_saved_search(
        env: Env,
        user: Address,
        search_id: String,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        let mut user_searches = env.storage().persistent()
            .get(&DataKey::SavedSearches(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        // Find and remove the search
        let mut found_index = None;
        for i in 0..user_searches.len() {
            if let Some(search) = user_searches.get(i) {
                if search.search_id == search_id {
                    found_index = Some(i);
                    break;
                }
            }
        }

        if let Some(index) = found_index {
            user_searches.remove(index);
            env.storage().persistent().set(&DataKey::SavedSearches(user), &user_searches);
            Ok(())
        } else {
            Err(Error::SavedSearchNotFound)
        }
    }

    /// Mark a saved search as favorite
    pub fn toggle_favorite_search(
        env: Env,
        user: Address,
        search_id: String,
    ) -> Result<bool, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        let mut user_searches = env.storage().persistent()
            .get(&DataKey::SavedSearches(user.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        for i in 0..user_searches.len() {
            if let Some(mut search) = user_searches.get(i) {
                if search.search_id == search_id {
                    search.is_favorite = !search.is_favorite;
                    let is_favorite = search.is_favorite;
                    user_searches.set(i, search);
                    env.storage().persistent().set(&DataKey::SavedSearches(user), &user_searches);
                    return Ok(is_favorite);
                }
            }
        }

        Err(Error::SavedSearchNotFound)
    }

    /// Clear search history for a user
    pub fn clear_search_history(env: Env, user: Address) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();

        let empty_history = Vec::new(&env);
        env.storage().persistent().set(&DataKey::SearchHistory(user), &empty_history);
        Ok(())
    }

    /// Get popular search queries
    pub fn get_popular_queries(
        env: Env,
        limit: Option<u32>,
        period: Option<u64>,
    ) -> Result<Vec<PopularQuery>, Error> {
        Self::require_initialized(&env)?;

        let period_key = period.unwrap_or(env.ledger().timestamp());
        let queries = env.storage().persistent()
            .get(&DataKey::PopularQueries(period_key))
            .unwrap_or_else(|| Vec::new(&env));

        if let Some(limit_val) = limit {
            let mut limited_queries = Vec::new(&env);
            let end_idx = (limit_val as usize).min(queries.len());
            for i in 0..end_idx {
                if let Some(query) = queries.get(i) {
                    limited_queries.push_back(query);
                }
            }
            Ok(limited_queries)
        } else {
            Ok(queries)
        }
    }

    // Helper functions
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_admin(env: &Env, user: &Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        
        if admin != *user {
            return Err(Error::Unauthorized);
        }
        
        user.require_auth();
        Ok(())
    }

    fn generate_search_id(env: &Env, user: &Address, name: &String) -> String {
        // Generate unique search ID based on user, name, and timestamp
        let timestamp = env.ledger().timestamp();
        let user_str = user.to_string();
        let combined = format!("search_{}_{}", timestamp, user_str.len());
        String::from_str(env, &combined)
    }

    /// Rebuild search index (admin only)
    pub fn rebuild_search_index(env: Env, admin: Address) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;

        SearchIndexer::rebuild_index(&env)
            .map_err(|_| Error::IndexNotFound)?;

        Ok(())
    }

    fn map_search_error(error: SearchError) -> Error {
        match error {
            SearchError::InvalidQuery => Error::InvalidQuery,
            SearchError::IndexNotFound => Error::IndexNotFound,
            SearchError::PermissionDenied => Error::Unauthorized,
            SearchError::TooManyResults => Error::TooManyResults,
            SearchError::InvalidFilters => Error::InvalidFilters,
            SearchError::SearchTimeout => Error::SearchTimeout,
        }
    }
}
