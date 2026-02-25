use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

/// Semantic Search Engine
/// Implements natural language understanding capabilities using pre-processed data from off-chain NLP services
pub struct SemanticSearch;

impl SemanticSearch {
    /// Process semantic search with pre-computed NLP metadata
    pub fn search(
        env: &Env,
        processed_query: ProcessedQuery,
        filters: SearchFilters,
        user: Option<Address>,
    ) -> Vec<SearchResultItem> {
        let mut results = Vec::new(env);

        // Get all content with semantic metadata
        let content_ids = Self::get_indexed_content(env);

        for i in 0..content_ids.len() {
            if let Some(content_id) = content_ids.get(i) {
                // Get semantic metadata for this content
                if let Some(metadata) = Self::get_semantic_metadata(env, &content_id) {
                    // Calculate semantic match score
                    let score =
                        Self::calculate_semantic_score(env, &processed_query, &metadata, &user);

                    // Apply threshold
                    if score > 100 {
                        // Minimum relevance threshold
                        // Apply filters
                        if Self::passes_filters(&metadata, &filters) {
                            let result =
                                Self::create_search_result(env, &content_id, &metadata, score);
                            results.push_back(result);
                        }
                    }
                }
            }
        }

        // Sort by score (descending)
        Self::sort_by_score(env, &mut results);

        results
    }

    /// Store semantic metadata from oracle
    pub fn store_semantic_metadata(env: &Env, content_id: String, metadata: SemanticMetadata) {
        let key = DataKey::SemanticMetadata(content_id);
        env.storage().persistent().set(&key, &metadata);
    }

    /// Retrieve semantic metadata for content
    pub fn get_semantic_metadata(env: &Env, content_id: &String) -> Option<SemanticMetadata> {
        let key = DataKey::SemanticMetadata(content_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Calculate semantic relevance score using integer arithmetic
    fn calculate_semantic_score(
        env: &Env,
        query: &ProcessedQuery,
        metadata: &SemanticMetadata,
        user: &Option<Address>,
    ) -> u32 {
        let mut score = 0u32;

        // Topic matching (0-400 points)
        let topic_score = Self::calculate_topic_score(env, query, metadata);
        score += topic_score;

        // Intent matching (0-300 points)
        let intent_score = Self::calculate_intent_score(env, query, metadata);
        score += intent_score;

        // Entity matching (0-200 points)
        let entity_score = Self::calculate_entity_score(env, query, metadata);
        score += entity_score;

        // Semantic tag overlap (0-100 points)
        let tag_score = Self::calculate_tag_overlap(env, query, metadata);
        score += tag_score;

        // Personalization boost (0-200 points)
        if let Some(user_addr) = user {
            let personalization = Self::calculate_personalization_boost(env, user_addr, metadata);
            score += personalization;
        }

        // Confidence adjustment
        score = (score * query.confidence) / 1000;

        score
    }

    /// Calculate topic matching score
    fn calculate_topic_score(
        _env: &Env,
        query: &ProcessedQuery,
        metadata: &SemanticMetadata,
    ) -> u32 {
        let mut score = 0u32;
        let mut matches = 0u32;

        // Check each query topic against content topics
        for i in 0..query.semantic_tags.len() {
            if let Some(query_topic) = query.semantic_tags.get(i) {
                for j in 0..metadata.topics.len() {
                    if let Some(content_topic) = metadata.topics.get(j) {
                        if Self::topics_match(&query_topic, &content_topic) {
                            matches += 1;
                            score += 100; // Base score per match
                        }
                    }
                }
            }
        }

        // Boost if multiple matches (indicates strong relevance)
        if matches > 2 {
            score += 100;
        }

        score.min(400) // Cap at 400
    }

    /// Calculate intent matching score
    fn calculate_intent_score(
        _env: &Env,
        query: &ProcessedQuery,
        metadata: &SemanticMetadata,
    ) -> u32 {
        // Get intent score from metadata if available
        if let Some(intent_score) = metadata.intent_scores.get(query.intent.clone()) {
            // Scale to 0-300 range
            (intent_score * 300) / 1000
        } else {
            50 // Default minimal score
        }
    }

    /// Calculate entity matching score
    fn calculate_entity_score(
        _env: &Env,
        query: &ProcessedQuery,
        metadata: &SemanticMetadata,
    ) -> u32 {
        let mut score = 0u32;

        for i in 0..query.entities.len() {
            if let Some(entity) = query.entities.get(i) {
                for j in 0..metadata.entity_types.len() {
                    if let Some(entity_type) = metadata.entity_types.get(j) {
                        if entity.entity_type == entity_type {
                            // Score based on entity confidence
                            score += (entity.confidence * 200) / 1000;
                        }
                    }
                }
            }
        }

        score.min(200) // Cap at 200
    }

    /// Calculate semantic tag overlap
    fn calculate_tag_overlap(
        _env: &Env,
        query: &ProcessedQuery,
        metadata: &SemanticMetadata,
    ) -> u32 {
        let mut overlap = 0u32;

        for i in 0..query.semantic_tags.len() {
            if let Some(query_tag) = query.semantic_tags.get(i) {
                for j in 0..metadata.semantic_tags.len() {
                    if let Some(content_tag) = metadata.semantic_tags.get(j) {
                        if query_tag == content_tag {
                            overlap += 1;
                        }
                    }
                }
            }
        }

        // Score based on overlap count
        overlap * 25 // 25 points per matching tag, max 100
    }

    /// Calculate personalization boost based on user profile
    fn calculate_personalization_boost(
        env: &Env,
        user: &Address,
        metadata: &SemanticMetadata,
    ) -> u32 {
        // Get user profile if exists
        let profile_key = DataKey::UserProfile(user.clone());
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<DataKey, UserProfile>(&profile_key)
        {
            let mut boost = 0u32;

            // Check if topics align with user's interests
            for i in 0..metadata.topics.len() {
                if let Some(topic) = metadata.topics.get(i) {
                    if let Some(interest_count) = profile.interaction_counts.get(topic.clone()) {
                        // Higher interaction count = more boost
                        boost += interest_count.min(50); // Cap per topic
                    }
                }
            }

            boost.min(200) // Cap total boost
        } else {
            0
        }
    }

    /// Check if two topics match (exact or similar)
    fn topics_match(topic_a: &String, topic_b: &String) -> bool {
        // For now, exact match
        // In production, this would use similarity threshold
        topic_a == topic_b
    }

    /// Apply search filters to metadata
    fn passes_filters(metadata: &SemanticMetadata, filters: &SearchFilters) -> bool {
        // Check complexity filter if difficulty levels specified
        if !filters.difficulty_levels.is_empty() {
            let complexity = metadata.complexity_score;
            // Map complexity to difficulty (simplified)
            let matches_difficulty = if complexity < 30 {
                filters
                    .difficulty_levels
                    .contains(&DifficultyLevel::Beginner)
            } else if complexity < 60 {
                filters
                    .difficulty_levels
                    .contains(&DifficultyLevel::Intermediate)
            } else if complexity < 85 {
                filters
                    .difficulty_levels
                    .contains(&DifficultyLevel::Advanced)
            } else {
                filters.difficulty_levels.contains(&DifficultyLevel::Expert)
            };

            if !matches_difficulty {
                return false;
            }
        }

        // More filter checks would go here
        true
    }

    /// Create search result from metadata
    fn create_search_result(
        env: &Env,
        content_id: &String,
        _metadata: &SemanticMetadata,
        score: u32,
    ) -> SearchResultItem {
        // This would fetch full item data in production
        // For now, return minimal result
        SearchResultItem {
            item_id: content_id.clone(),
            item_type: SearchResultType::Course,
            title: String::from_str(env, "Course Title"),
            description: String::from_str(env, "Course Description"),
            relevance_score: score,
            metadata: SearchResultMetadata::Course(CourseMetadata {
                course_id: content_id.clone(),
                instructor_id: Address::from_str(
                    env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                ),
                instructor_name: String::from_str(env, "Instructor"),
                category: String::from_str(env, "Category"),
                difficulty: DifficultyLevel::Intermediate,
                duration_hours: 10,
                price: 0,
                rating: 40,
                enrollment_count: 100,
                completion_rate: 75,
                created_date: env.ledger().timestamp(),
                updated_date: env.ledger().timestamp(),
                tags: Vec::new(env),
                language: String::from_str(env, "en"),
                has_certificate: true,
                has_prerequisites: false,
                is_premium: false,
                is_featured: false,
            }),
            highlights: Vec::new(env),
            thumbnail_url: None,
        }
    }

    /// Sort results by score (descending)
    fn sort_by_score(_env: &Env, results: &mut Vec<SearchResultItem>) {
        // Bubble sort (simple for blockchain)
        let len = results.len();
        for i in 0..len {
            for j in 0..(len - i - 1) {
                let score_j = results.get(j).unwrap().relevance_score;
                let score_j_plus_1 = results.get(j + 1).unwrap().relevance_score;

                if score_j < score_j_plus_1 {
                    // Swap
                    let temp_j = results.get(j).unwrap();
                    let temp_j_plus_1 = results.get(j + 1).unwrap();
                    results.set(j, temp_j_plus_1);
                    results.set(j + 1, temp_j);
                }
            }
        }
    }

    /// Get list of indexed content IDs
    fn get_indexed_content(env: &Env) -> Vec<String> {
        // In production, this would query an index
        // For now, return empty vector
        Vec::new(env)
    }

    /// Expand query with synonyms (using pre-computed synonym data)
    pub fn expand_query_with_synonyms(_env: &Env, query_terms: Vec<String>) -> Vec<String> {
        let expanded = query_terms.clone();

        // Add synonyms from pre-computed synonym index
        for i in 0..query_terms.len() {
            if let Some(_term) = query_terms.get(i) {
                // Look up synonyms (would be stored by oracle)
                // Add to expanded terms
            }
        }

        expanded
    }
}
