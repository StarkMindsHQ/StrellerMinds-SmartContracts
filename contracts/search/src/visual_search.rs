use soroban_sdk::{Env, String, Vec};
use crate::types::*;

/// Visual Search
/// Manages visual content metadata from off-chain image processing
/// Enables search by visual features and thumbnails
pub struct VisualSearch;

impl VisualSearch {
    /// Store visual metadata from oracle (image processing service)
    pub fn store_visual_metadata(
        env: &Env,
        content_id: String,
        metadata: VisualMetadata,
    ) {
        let key = DataKey::VisualMetadata(content_id.clone());
        env.storage().persistent().set(&key, &metadata);
        
        // Index by dominant colors
        Self::index_by_colors(env, &content_id, &metadata.dominant_colors);
        
        // Index by detected objects
        Self::index_by_objects(env, &content_id, &metadata.detected_objects);
        
        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("vis_meta"),),
            (content_id, metadata.detected_objects.len())
        );
    }
    
    /// Get visual metadata for content
    pub fn get_visual_metadata(
        env: &Env,
        content_id: String,
    ) -> Option<VisualMetadata> {
        let key = DataKey::VisualMetadata(content_id);
        env.storage().persistent().get(&key)
    }
    
    /// Store visual similarity scores from oracle
    pub fn store_visual_similarity(
        env: &Env,
        content_a: String,
        content_b: String,
        score: SimilarityScore,
    ) {
        // Store bidirectionally
        let key_ab = Self::visual_similarity_key(env, &content_a, &content_b);
        let key_ba = Self::visual_similarity_key(env, &content_b, &content_a);
        
        env.storage().persistent().set(&key_ab, &score);
        env.storage().persistent().set(&key_ba, &score);
    }
    
    /// Get visual similarity between two content items
    pub fn get_visual_similarity(
        env: &Env,
        content_a: String,
        content_b: String,
    ) -> Option<SimilarityScore> {
        let key = Self::visual_similarity_key(env, &content_a, &content_b);
        env.storage().persistent().get(&key)
    }
    
    /// Find visually similar content
    pub fn find_visually_similar(
        env: &Env,
        content_id: String,
        min_score: u32,
        limit: u32,
    ) -> Vec<String> {
        // Get cached similar content from oracle
        let cache_key = Self::similar_cache_key(env, &content_id);
        env.storage().persistent()
            .get(&cache_key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Store list of visually similar content from oracle
    pub fn store_similar_content(
        env: &Env,
        content_id: String,
        similar_ids: Vec<String>,
    ) {
        let cache_key = Self::similar_cache_key(env, &content_id);
        env.storage().persistent().set(&cache_key, &similar_ids);
    }
    
    /// Search by color scheme
    pub fn find_by_color(
        env: &Env,
        color_hex: String,
        tolerance: u32,  // Color distance tolerance
    ) -> Vec<String> {
        let color_key = Self::color_index_key(env, &color_hex);
        env.storage().persistent()
            .get::<String, Vec<String>>(&color_key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Search by detected object
    pub fn find_by_object(
        env: &Env,
        object_type: String,
    ) -> Vec<String> {
        let object_key = Self::object_index_key(env, &object_type);
        env.storage().persistent()
            .get::<String, Vec<String>>(&object_key)
            .unwrap_or_else(|| Vec::new(env))
    }
    
    /// Calculate visual match score for search
    pub fn calculate_visual_score(
        env: &Env,
        content_id: String,
        query_colors: Vec<String>,
        query_objects: Vec<String>,
    ) -> u32 {
        let metadata = match Self::get_visual_metadata(env, content_id) {
            Some(meta) => meta,
            None => return 0,
        };
        
        let mut score = 0u32;
        
        // Color matching (0-400 points)
        let color_score = Self::calculate_color_match_score(
            &metadata.dominant_colors,
            &query_colors,
        );
        score += color_score;
        
        // Object matching (0-600 points)
        let object_score = Self::calculate_object_match_score(
            &metadata.detected_objects,
            &query_objects,
        );
        score += object_score;
        
        score
    }
    
    /// Calculate color matching score
    fn calculate_color_match_score(
        content_colors: &Vec<String>,
        query_colors: &Vec<String>,
    ) -> u32 {
        if query_colors.is_empty() {
            return 0;
        }
        
        let mut matches = 0u32;
        
        for i in 0..query_colors.len() {
            if let Some(query_color) = query_colors.get(i) {
                if content_colors.contains(&query_color) {
                    matches += 1;
                }
            }
        }
        
        // Normalize to 0-400 scale
        (matches * 400) / query_colors.len()
    }
    
    /// Calculate object matching score
    fn calculate_object_match_score(
        content_objects: &Vec<String>,
        query_objects: &Vec<String>,
    ) -> u32 {
        if query_objects.is_empty() {
            return 0;
        }
        
        let mut matches = 0u32;
        
        for i in 0..query_objects.len() {
            if let Some(query_obj) = query_objects.get(i) {
                if content_objects.contains(&query_obj) {
                    matches += 1;
                }
            }
        }
        
        // Normalize to 0-600 scale
        (matches * 600) / query_objects.len()
    }
    
    /// Index content by colors
    fn index_by_colors(env: &Env, content_id: &String, colors: &Vec<String>) {
        for i in 0..colors.len() {
            if let Some(color) = colors.get(i) {
                let color_key = Self::color_index_key(env, &color);
                let mut content_ids = env.storage().persistent()
                    .get::<String, Vec<String>>(&color_key)
                    .unwrap_or_else(|| Vec::new(env));
                
                if !content_ids.contains(content_id) {
                    content_ids.push_back(content_id.clone());
                    env.storage().persistent().set(&color_key, &content_ids);
                }
            }
        }
    }
    
    /// Index content by detected objects
    fn index_by_objects(env: &Env, content_id: &String, objects: &Vec<String>) {
        for i in 0..objects.len() {
            if let Some(object) = objects.get(i) {
                let object_key = Self::object_index_key(env, &object);
                let mut content_ids = env.storage().persistent()
                    .get::<String, Vec<String>>(&object_key)
                    .unwrap_or_else(|| Vec::new(env));
                
                if !content_ids.contains(content_id) {
                    content_ids.push_back(content_id.clone());
                    env.storage().persistent().set(&object_key, &content_ids);
                }
            }
        }
    }
    
    /// Get thumbnail URL for content
    pub fn get_thumbnail(
        env: &Env,
        content_id: String,
    ) -> Option<String> {
        if let Some(metadata) = Self::get_visual_metadata(env, content_id) {
            Some(metadata.thumbnail_url)
        } else {
            None
        }
    }
    
    /// Get aspect ratio for content
    pub fn get_aspect_ratio(
        env: &Env,
        content_id: String,
    ) -> u32 {
        if let Some(metadata) = Self::get_visual_metadata(env, content_id) {
            metadata.aspect_ratio
        } else {
            100  // Default 1:1 (100%)
        }
    }
    
    /// Check if content has high-quality visuals
    pub fn is_high_quality(
        env: &Env,
        content_id: String,
        threshold: u32,
    ) -> bool {
        if let Some(metadata) = Self::get_visual_metadata(env, content_id) {
            metadata.quality_score >= threshold
        } else {
            false
        }
    }
    
    /// Get dominant color palette
    pub fn get_color_palette(
        env: &Env,
        content_id: String,
    ) -> Vec<String> {
        if let Some(metadata) = Self::get_visual_metadata(env, content_id) {
            metadata.dominant_colors
        } else {
            Vec::new(env)
        }
    }
    
    /// Get detected objects
    pub fn get_detected_objects(
        env: &Env,
        content_id: String,
    ) -> Vec<String> {
        if let Some(metadata) = Self::get_visual_metadata(env, content_id) {
            metadata.detected_objects
        } else {
            Vec::new(env)
        }
    }
    
    /// Generate visual similarity key
    fn visual_similarity_key(env: &Env, content_a: &String, content_b: &String) -> DataKey {
        DataKey::VisualMetadata(content_a.clone())
    }
    
    /// Generate similar content cache key
    fn similar_cache_key(env: &Env, content_id: &String) -> DataKey {
        DataKey::VisualMetadata(content_id.clone())
    }
    
    /// Generate color index key
    fn color_index_key(env: &Env, color: &String) -> String {
        String::from_str(env, "color_idx")
    }
    
    /// Generate object index key
    fn object_index_key(env: &Env, object: &String) -> String {
        String::from_str(env, "obj_idx")
    }
}
