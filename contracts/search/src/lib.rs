#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracterror, Address, Env, String, Vec, Bytes,
};

mod types;
mod semantic_search;
mod recommendation_engine;
mod content_analyzer;
mod collaborative_filter;
mod visual_search;
mod learning_path_optimizer;
mod ranking_engine;
mod multilingual_search;
mod search_analytics;
mod voice_search;

#[cfg(test)]
mod tests;

pub use types::*;
use semantic_search::SemanticSearch;
use recommendation_engine::RecommendationEngine;
use content_analyzer::ContentAnalyzer;
use collaborative_filter::CollaborativeFilter;
use visual_search::VisualSearch;
use learning_path_optimizer::LearningPathOptimizer;
use ranking_engine::RankingEngine;
use multilingual_search::MultilingualSearch;
use search_analytics::SearchAnalytics;
use voice_search::VoiceSearch;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidQuery = 4,
    ContentNotFound = 5,
    InvalidMetadata = 6,
    InvalidScore = 7,
    SessionExpired = 8,
    InvalidLanguage = 9,
    OracleNotAuthorized = 10,
}

#[contract]
pub struct AdvancedSearchContract;

#[contractimpl]
impl AdvancedSearchContract {
    /// Initialize the advanced search contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Initialize default ranking configuration
        let default_config = RankingConfig {
            relevance_weight: 30,
            quality_weight: 20,
            engagement_weight: 15,
            recency_weight: 10,
            personalization_weight: 15,
            authority_weight: 10,
        };
        RankingEngine::store_ranking_config(&env, default_config);

        Ok(())
    }

    // ==================== Semantic Search Functions ====================
    
    /// Execute semantic search with NLP-enhanced query understanding
    pub fn semantic_search(
        env: Env,
        query: ProcessedQuery,
        user: Option<Address>,
        filters: SearchFilters,
    ) -> Result<Vec<SearchResultItem>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(SemanticSearch::search(&env, query, filters, user))
    }
    
    /// Store semantic metadata from oracle (off-chain NLP service)
    pub fn store_semantic_metadata(
        env: Env,
        oracle: Address,
        content_id: String,
        metadata: SemanticMetadata,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        SemanticSearch::store_semantic_metadata(&env, content_id, metadata);
        Ok(())
    }

    // ==================== Recommendation Functions ====================
    
    /// Generate personalized recommendations
    pub fn get_recommendations(
        env: Env,
        user: Address,
        limit: u32,
    ) -> Result<Vec<Recommendation>, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        Ok(RecommendationEngine::generate_recommendations(&env, user, limit))
    }
    
    /// Store recommendations from oracle (off-chain ML service)
    pub fn store_recommendations(
        env: Env,
        oracle: Address,
        user: Address,
        recommendations: Vec<Recommendation>,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        RecommendationEngine::store_recommendations(&env, user, recommendations);
        Ok(())
    }
    
    /// Update user profile with learning activity
    pub fn update_user_profile(
        env: Env,
        user: Address,
        completed_course: String,
        completed: bool,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        RecommendationEngine::update_user_profile(&env, user, completed_course, completed);
        Ok(())
    }

    // ==================== Content Analysis Functions ====================
    
    /// Store content analysis from oracle
    pub fn store_content_analysis(
        env: Env,
        oracle: Address,
        content_id: String,
        analysis: ContentAnalysis,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        ContentAnalyzer::store_analysis(&env, content_id, analysis);
        Ok(())
    }
    
    /// Get content analysis
    pub fn get_content_analysis(
        env: Env,
        content_id: String,
    ) -> Result<ContentAnalysis, Error> {
        Self::require_initialized(&env)?;
        
        ContentAnalyzer::get_analysis(&env, content_id)
            .ok_or(Error::ContentNotFound)
    }
    
    /// Find content by tag
    pub fn find_by_tag(
        env: Env,
        tag: String,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(ContentAnalyzer::find_by_tag(&env, tag))
    }
    
    /// Find content by skill
    pub fn find_by_skill(
        env: Env,
        skill_name: String,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(ContentAnalyzer::find_by_skill(&env, skill_name))
    }

    // ==================== Collaborative Filtering Functions ====================
    
    /// Store user similarity from oracle
    pub fn store_user_similarity(
        env: Env,
        oracle: Address,
        user_a: Address,
        user_b: Address,
        score: SimilarityScore,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        CollaborativeFilter::store_similarity(&env, user_a, user_b, score);
        Ok(())
    }
    
    /// Record user interaction for collaborative filtering
    pub fn record_interaction(
        env: Env,
        user: Address,
        interaction: UserInteraction,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        CollaborativeFilter::record_interaction(&env, user, interaction);
        Ok(())
    }
    
    /// Get collaborative recommendations
    pub fn get_collab_recommendations(
        env: Env,
        user: Address,
        limit: u32,
    ) -> Result<Vec<Recommendation>, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        Ok(CollaborativeFilter::get_collaborative_recommendations(&env, user, limit))
    }

    // ==================== Visual Search Functions ====================
    
    /// Store visual metadata from oracle (image processing service)
    pub fn store_visual_metadata(
        env: Env,
        oracle: Address,
        content_id: String,
        metadata: VisualMetadata,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        VisualSearch::store_visual_metadata(&env, content_id, metadata);
        Ok(())
    }
    
    /// Find visually similar content
    pub fn find_visually_similar(
        env: Env,
        content_id: String,
        min_score: u32,
        limit: u32,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(VisualSearch::find_visually_similar(&env, content_id, min_score, limit))
    }
    
    /// Search by color
    pub fn find_by_color(
        env: Env,
        color_hex: String,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(VisualSearch::find_by_color(&env, color_hex, 20))
    }
    
    /// Search by detected object
    pub fn find_by_object(
        env: Env,
        object_type: String,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(VisualSearch::find_by_object(&env, object_type))
    }

    // ==================== Learning Path Functions ====================
    
    /// Store optimized learning path from oracle
    pub fn store_learning_path(
        env: Env,
        oracle: Address,
        user: Address,
        path: LearningPath,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        LearningPathOptimizer::store_learning_path(&env, user, path);
        Ok(())
    }
    
    /// Get user's learning path
    pub fn get_learning_path(
        env: Env,
        user: Address,
    ) -> Result<LearningPath, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        LearningPathOptimizer::get_learning_path(&env, user)
            .ok_or(Error::ContentNotFound)
    }
    
    /// Complete a learning path step
    pub fn complete_path_step(
        env: Env,
        user: Address,
        step_id: String,
        completion_score: u32,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        LearningPathOptimizer::complete_step(&env, user, step_id, completion_score);
        Ok(())
    }
    
    /// Get next recommended step
    pub fn get_next_step(
        env: Env,
        user: Address,
    ) -> Result<PathStep, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        LearningPathOptimizer::get_next_step(&env, user)
            .ok_or(Error::ContentNotFound)
    }

    // ==================== Ranking Functions ====================
    
    /// Rank search results with multi-signal approach
    pub fn rank_results(
        env: Env,
        results: Vec<String>,
        user: Option<Address>,
    ) -> Result<Vec<RankedResult>, Error> {
        Self::require_initialized(&env)?;
        
        let config = RankingEngine::get_ranking_config(&env);
        Ok(RankingEngine::rank_results(&env, results, user, config))
    }
    
    /// Update ranking configuration (admin only)
    pub fn update_ranking_config(
        env: Env,
        admin: Address,
        config: RankingConfig,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        
        RankingEngine::store_ranking_config(&env, config);
        Ok(())
    }

    // ==================== Multilingual Functions ====================
    
    /// Store multilingual content from oracle
    pub fn store_multilingual_content(
        env: Env,
        oracle: Address,
        content_id: String,
        multilingual: MultilingualContent,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        MultilingualSearch::store_multilingual_content(&env, content_id, multilingual);
        Ok(())
    }
    
    /// Set user language preferences
    pub fn set_language_preferences(
        env: Env,
        user: Address,
        preferences: LanguagePreferences,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        MultilingualSearch::store_language_preferences(&env, user, preferences);
        Ok(())
    }
    
    /// Search by language
    pub fn search_by_language(
        env: Env,
        language: Language,
        query: String,
    ) -> Result<Vec<String>, Error> {
        Self::require_initialized(&env)?;
        
        Ok(MultilingualSearch::search_by_language(&env, language, query))
    }

    // ==================== Analytics Functions ====================
    
    /// Record search event
    pub fn record_search(
        env: Env,
        user: Option<Address>,
        query: String,
        results_count: u32,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        
        SearchAnalytics::record_search(&env, user, query, results_count);
        Ok(())
    }
    
    /// Record click event
    pub fn record_click(
        env: Env,
        user: Option<Address>,
        query: String,
        content_id: String,
        rank_position: u32,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        
        SearchAnalytics::record_click(&env, user, query, content_id, rank_position);
        Ok(())
    }
    
    /// Get click-through rate
    pub fn get_ctr(
        env: Env,
        query: String,
        content_id: String,
    ) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        
        Ok(SearchAnalytics::get_ctr(&env, query, content_id))
    }
    
    /// Get search quality score
    pub fn get_search_quality_score(
        env: Env,
        query: String,
    ) -> Result<u32, Error> {
        Self::require_initialized(&env)?;
        
        Ok(SearchAnalytics::calculate_search_quality_score(&env, query))
    }

    // ==================== Voice Search Functions ====================
    
    /// Store processed voice query from oracle
    pub fn store_voice_query(
        env: Env,
        oracle: Address,
        user: Address,
        processed_query: ProcessedVoiceQuery,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_authorized_oracle(&env, &oracle)?;
        
        VoiceSearch::store_voice_query(&env, user, processed_query);
        Ok(())
    }
    
    /// Create conversation session for voice
    pub fn create_conversation_session(
        env: Env,
        user: Address,
    ) -> Result<String, Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        Ok(VoiceSearch::create_conversation_session(&env, user))
    }
    
    /// Get conversation session
    pub fn get_conversation_session(
        env: Env,
        session_id: String,
    ) -> Result<ConversationSession, Error> {
        Self::require_initialized(&env)?;
        
        VoiceSearch::get_conversation_session(&env, session_id)
            .ok_or(Error::SessionExpired)
    }
    
    /// End conversation session
    pub fn end_conversation_session(
        env: Env,
        user: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        user.require_auth();
        
        VoiceSearch::end_session(&env, user);
        Ok(())
    }

    // ==================== Oracle Management ====================
    
    /// Authorize oracle (admin only)
    pub fn authorize_oracle(
        env: Env,
        admin: Address,
        oracle: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        
        let key = DataKey::AuthorizedOracles(oracle.clone());
        env.storage().persistent().set(&key, &true);
        
        Ok(())
    }
    
    /// Revoke oracle authorization (admin only)
    pub fn revoke_oracle(
        env: Env,
        admin: Address,
        oracle: Address,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &admin)?;
        
        let key = DataKey::AuthorizedOracles(oracle);
        env.storage().persistent().remove(&key);
        
        Ok(())
    }

    // ==================== Helper Functions ====================
    
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
    
    fn require_authorized_oracle(env: &Env, oracle: &Address) -> Result<(), Error> {
        oracle.require_auth();
        
        let key = DataKey::AuthorizedOracles(oracle.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::OracleNotAuthorized);
        }
        
        Ok(())
    }
}

