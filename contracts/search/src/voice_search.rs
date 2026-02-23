use soroban_sdk::{Address, Env, String, Vec};
use crate::types::*;

/// Voice Search
/// Manages voice query processing from off-chain speech-to-text services
/// Handles conversational search and context maintenance
pub struct VoiceSearch;

impl VoiceSearch {
    /// Store processed voice query from oracle
    pub fn store_voice_query(
        env: &Env,
        user: Address,
        processed_query: ProcessedVoiceQuery,
    ) {
        // Store in conversation session
        Self::add_to_conversation(env, user.clone(), &processed_query);
        
        // Store processed query for search
        let key = Self::voice_query_key(env, processed_query.timestamp);
        env.storage().persistent().set(&key, &processed_query);
        
        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("voice_qry"),),
            (user, processed_query.transcribed_text.clone())
        );
    }
    
    /// Get processed voice query
    pub fn get_voice_query(
        env: &Env,
        timestamp: u64,
    ) -> Option<ProcessedVoiceQuery> {
        let key = Self::voice_query_key(env, timestamp);
        env.storage().persistent().get(&key)
    }
    
    /// Create or update conversation session
    pub fn create_conversation_session(
        env: &Env,
        user: Address,
    ) -> String {
        let session_id = Self::generate_session_id(env, &user);
        
        let session = ConversationSession {
            session_id: session_id.clone(),
            user: user.clone(),
            start_time: env.ledger().timestamp(),
            last_interaction_time: env.ledger().timestamp(),
            queries: Vec::new(env),
            context_entities: Vec::new(env),
            is_active: true,
        };
        
        let key = DataKey::ConversationSession(session_id.clone());
        env.storage().persistent().set(&key, &session);
        
        // Store active session for user
        let user_session_key = Self::user_session_key(env, &user);
        env.storage().persistent().set(&user_session_key, &session_id);
        
        session_id
    }
    
    /// Get conversation session
    pub fn get_conversation_session(
        env: &Env,
        session_id: String,
    ) -> Option<ConversationSession> {
        let key = DataKey::ConversationSession(session_id);
        env.storage().persistent().get(&key)
    }
    
    /// Get user's active session
    pub fn get_user_active_session(
        env: &Env,
        user: Address,
    ) -> Option<ConversationSession> {
        let user_session_key = Self::user_session_key(env, &user);
        
        if let Some(session_id) = env.storage().persistent()
            .get::<String, String>(&user_session_key) {
            Self::get_conversation_session(env, session_id)
        } else {
            None
        }
    }
    
    /// Add query to conversation
    fn add_to_conversation(
        env: &Env,
        user: Address,
        query: &ProcessedVoiceQuery,
    ) {
        // Get or create session
        let session_id = if let Some(session) = Self::get_user_active_session(env, user.clone()) {
            session.session_id
        } else {
            Self::create_conversation_session(env, user.clone())
        };
        
        let key = DataKey::ConversationSession(session_id.clone());
        
        if let Some(mut session) = env.storage().persistent()
            .get::<DataKey, ConversationSession>(&key) {
            
            // Add query
            session.queries.push_back(query.clone());
            
            // Update context with new entities
            for i in 0..query.entities.len() {
                if let Some(entity) = query.entities.get(i) {
                    if !session.context_entities.contains(&entity) {
                        session.context_entities.push_back(entity);
                    }
                }
            }
            
            // Update timestamp
            session.last_interaction_time = env.ledger().timestamp();
            
            // Keep only recent queries (last 10)
            if session.queries.len() > 10 {
                session.queries.remove(0);
            }
            
            env.storage().persistent().set(&key, &session);
        }
    }
    
    /// Process conversational query with context
    pub fn process_conversational_query(
        env: &Env,
        user: Address,
        query_text: String,
    ) -> ProcessedQuery {
        // Get conversation context
        let context_strings = if let Some(session) = Self::get_user_active_session(env, user.clone()) {
            session.context_entities
        } else {
            Vec::new(env)
        };
        
        // Convert context strings to Entity objects
        let mut context_entities = Vec::new(env);
        for i in 0..context_strings.len() {
            if let Some(value) = context_strings.get(i) {
                context_entities.push_back(Entity {
                    entity_type: String::from_str(env, "context"),
                    value,
                    confidence: 800,
                });
            }
        }
        
        // Build processed query with context
        // In practice, this would be enriched off-chain
        ProcessedQuery {
            original_text: query_text.clone(),
            normalized_text: query_text.clone(),
            original_query: query_text.clone(),
            extracted_intent: String::from_str(env, ""),
            intent: String::from_str(env, ""),
            entities: context_entities,
            expanded_terms: Vec::new(env),
            semantic_tags: Vec::new(env),
            suggested_filters: Vec::new(env),
            query_type: String::from_str(env, "voice"),
            confidence: 800,
        }
    }
    
    /// Handle follow-up question
    pub fn handle_follow_up(
        env: &Env,
        user: Address,
        follow_up_text: String,
    ) -> ProcessedQuery {
        // Use conversation context to understand follow-up
        let session = Self::get_user_active_session(env, user.clone());
        
        if session.is_none() {
            // No context - treat as new query
            return Self::process_conversational_query(env, user, follow_up_text);
        }
        
        let session_data = session.unwrap();
        
        // Get last query for context
        let last_query = if session_data.queries.len() > 0 {
            session_data.queries.get(session_data.queries.len() - 1)
        } else {
            None
        };
        
        // Convert context strings to Entity objects
        let mut context_entities = Vec::new(env);
        for i in 0..session_data.context_entities.len() {
            if let Some(value) = session_data.context_entities.get(i) {
                context_entities.push_back(Entity {
                    entity_type: String::from_str(env, "context"),
                    value,
                    confidence: 800,
                });
            }
        }
        
        // Build contextual query
        // Off-chain service would resolve pronouns and references
        ProcessedQuery {
            original_text: follow_up_text.clone(),
            normalized_text: follow_up_text.clone(),
            original_query: follow_up_text.clone(),
            extracted_intent: String::from_str(env, "follow_up"),
            intent: String::from_str(env, "follow_up"),
            entities: context_entities,
            expanded_terms: Vec::new(env),
            semantic_tags: Vec::new(env),
            suggested_filters: Vec::new(env),
            query_type: String::from_str(env, "voice_followup"),
            confidence: 800,
        }
    }
    
    /// End conversation session
    pub fn end_session(
        env: &Env,
        user: Address,
    ) {
        if let Some(mut session) = Self::get_user_active_session(env, user.clone()) {
            session.is_active = false;
            
            let key = DataKey::ConversationSession(session.session_id.clone());
            env.storage().persistent().set(&key, &session);
            
            // Clear active session for user
            let user_session_key = Self::user_session_key(env, &user);
            env.storage().persistent().remove(&user_session_key);
            
            // Emit event
            env.events().publish(
                (soroban_sdk::symbol_short!("end_sess"),),
                (user, session.session_id)
            );
        }
    }
    
    /// Check if session is expired
    pub fn is_session_expired(
        env: &Env,
        session: &ConversationSession,
        timeout_seconds: u64,
    ) -> bool {
        let current_time = env.ledger().timestamp();
        let elapsed = current_time - session.last_interaction_time;
        elapsed > timeout_seconds
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(
        env: &Env,
        timeout_seconds: u64,
    ) {
        // This would iterate through active sessions
        // For now, emit event for off-chain cleanup
        env.events().publish(
            (soroban_sdk::symbol_short!("cleanup"),),
            timeout_seconds
        );
    }
    
    /// Get voice search confidence score
    pub fn get_confidence_score(
        env: &Env,
        timestamp: u64,
    ) -> u32 {
        if let Some(query) = Self::get_voice_query(env, timestamp) {
            query.confidence_score
        } else {
            0
        }
    }
    
    /// Request voice query processing from oracle
    pub fn request_voice_processing(
        env: &Env,
        user: Address,
        audio_hash: String,  // Hash/reference to audio file
    ) {
        // Emit event for off-chain speech-to-text processing
        env.events().publish(
            (soroban_sdk::symbol_short!("proc_voic"),),
            (user, audio_hash)
        );
    }
    
    /// Get conversation context summary
    pub fn get_context_summary(
        env: &Env,
        user: Address,
    ) -> Vec<String> {
        if let Some(session) = Self::get_user_active_session(env, user) {
            session.context_entities
        } else {
            Vec::new(env)
        }
    }
    
    /// Clear conversation context
    pub fn clear_context(
        env: &Env,
        user: Address,
    ) {
        if let Some(mut session) = Self::get_user_active_session(env, user.clone()) {
            session.context_entities = Vec::new(env);
            session.queries = Vec::new(env);
            
            let key = DataKey::ConversationSession(session.session_id.clone());
            env.storage().persistent().set(&key, &session);
        }
    }
    
    /// Get voice search history
    pub fn get_voice_search_history(
        env: &Env,
        user: Address,
        limit: u32,
    ) -> Vec<ProcessedVoiceQuery> {
        if let Some(session) = Self::get_user_active_session(env, user) {
            let mut history = Vec::new(env);
            let start = if session.queries.len() > limit {
                session.queries.len() - limit
            } else {
                0
            };
            
            for i in start..session.queries.len() {
                if let Some(query) = session.queries.get(i) {
                    history.push_back(query);
                }
            }
            
            history
        } else {
            Vec::new(env)
        }
    }
    
    /// Suggest voice command shortcuts
    pub fn suggest_voice_commands(
        env: &Env,
        user: Address,
    ) -> Vec<String> {
        // Based on user's search history and preferences
        // Would be computed off-chain
        let mut suggestions = Vec::new(env);
        
        // Default suggestions
        suggestions.push_back(String::from_str(env, "Show my courses"));
        suggestions.push_back(String::from_str(env, "Find beginner tutorials"));
        suggestions.push_back(String::from_str(env, "Continue learning"));
        
        suggestions
    }
    
    /// Calculate voice search quality metrics
    pub fn get_voice_quality_metrics(
        env: &Env,
        timestamp: u64,
    ) -> VoiceQualityMetrics {
        if let Some(query) = Self::get_voice_query(env, timestamp) {
            VoiceQualityMetrics {
                confidence: query.confidence_score,
                clarity: query.confidence_score,  // Approximate
                background_noise: 100 - query.confidence_score,  // Inverse
                success_rate: 800,  // Default 80%
            }
        } else {
            VoiceQualityMetrics {
                confidence: 0,
                clarity: 0,
                background_noise: 1000,
                success_rate: 0,
            }
        }
    }
    
    /// Storage key generators
    fn voice_query_key(env: &Env, timestamp: u64) -> String {
        String::from_str(env, "voice_qry")
    }
    
    fn user_session_key(env: &Env, user: &Address) -> String {
        String::from_str(env, "usr_sess")
    }
    
    fn generate_session_id(env: &Env, user: &Address) -> String {
        let timestamp = env.ledger().timestamp();
        String::from_str(env, "sess_id")
    }
}

/// Voice quality metrics
#[derive(Clone)]
pub struct VoiceQualityMetrics {
    pub confidence: u32,
    pub clarity: u32,
    pub background_noise: u32,
    pub success_rate: u32,
}
