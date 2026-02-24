use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

/// Multilingual Search
/// Manages cross-language search with translation metadata from off-chain services
/// Supports language preferences and automatic translation routing
pub struct MultilingualSearch;

impl MultilingualSearch {
    /// Store multilingual content from oracle
    pub fn store_multilingual_content(
        env: &Env,
        content_id: String,
        multilingual: MultilingualContent,
    ) {
        let key = DataKey::MultilingualContent(content_id.clone());
        env.storage().persistent().set(&key, &multilingual);

        // Index by each available language
        for i in 0..multilingual.available_languages.len() {
            if let Some(lang) = multilingual.available_languages.get(i) {
                Self::index_by_language(env, &content_id, &lang);
            }
        }

        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("ml_cont"),),
            (content_id, multilingual.available_languages.len()),
        );
    }

    /// Get multilingual content
    pub fn get_multilingual_content(env: &Env, content_id: String) -> Option<MultilingualContent> {
        let key = DataKey::MultilingualContent(content_id);
        env.storage().persistent().get(&key)
    }

    /// Store user's language preferences
    pub fn store_language_preferences(env: &Env, user: Address, preferences: LanguagePreferences) {
        let key = DataKey::LanguagePreferences(user.clone());
        env.storage().persistent().set(&key, &preferences);

        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("lang_pref"),),
            (user, preferences.preferred_language),
        );
    }

    /// Get user's language preferences
    pub fn get_language_preferences(env: &Env, user: Address) -> Option<LanguagePreferences> {
        let key = DataKey::LanguagePreferences(user);
        env.storage().persistent().get(&key)
    }

    /// Search content in specific language
    pub fn search_by_language(env: &Env, language: Language, query: String) -> Vec<String> {
        let lang_key = Self::language_index_key(env, &language);

        let content_ids = env
            .storage()
            .persistent()
            .get::<String, Vec<String>>(&lang_key)
            .unwrap_or_else(|| Vec::new(env));

        // Further filtering by query would happen in semantic search
        content_ids
    }

    /// Find content available in user's preferred language
    pub fn find_in_preferred_language(
        env: &Env,
        user: Address,
        content_ids: Vec<String>,
    ) -> Vec<String> {
        let mut matching = Vec::new(env);

        if let Some(prefs) = Self::get_language_preferences(env, user) {
            for i in 0..content_ids.len() {
                if let Some(content_id) = content_ids.get(i) {
                    if Self::is_available_in_language(
                        env,
                        content_id.clone(),
                        prefs.preferred_language.clone(),
                    ) {
                        matching.push_back(content_id);
                    }
                }
            }
        }

        matching
    }

    /// Check if content is available in specific language
    pub fn is_available_in_language(env: &Env, content_id: String, language: Language) -> bool {
        if let Some(content) = Self::get_multilingual_content(env, content_id) {
            content.available_languages.contains(&language)
        } else {
            false
        }
    }

    /// Get translation metadata
    pub fn get_translation_metadata(
        env: &Env,
        content_id: String,
        target_language: Language,
    ) -> Option<TranslationMeta> {
        if let Some(content) = Self::get_multilingual_content(env, content_id.clone()) {
            // Get translation metadata by language key
            let lang_key = Self::language_to_string(env, &target_language);
            content.translations.get(lang_key)
        } else {
            None
        }
    }

    /// Get translation quality score
    pub fn get_translation_quality(
        env: &Env,
        content_id: String,
        target_language: Language,
    ) -> u32 {
        if let Some(trans_meta) = Self::get_translation_metadata(env, content_id, target_language) {
            trans_meta.quality_score
        } else {
            0
        }
    }

    /// Recommend content with high-quality translations
    pub fn filter_by_translation_quality(
        env: &Env,
        content_ids: Vec<String>,
        language: Language,
        min_quality: u32,
    ) -> Vec<String> {
        let mut high_quality = Vec::new(env);

        for i in 0..content_ids.len() {
            if let Some(content_id) = content_ids.get(i) {
                let quality =
                    Self::get_translation_quality(env, content_id.clone(), language.clone());
                if quality >= min_quality {
                    high_quality.push_back(content_id);
                }
            }
        }

        high_quality
    }

    /// Translate query to multiple languages (off-chain)
    pub fn translate_query(env: &Env, query: String, target_languages: Vec<Language>) {
        // Emit event for off-chain translation service
        env.events().publish(
            (soroban_sdk::symbol_short!("trans_req"),),
            (query, target_languages.len()),
        );
    }

    /// Store translated query from oracle
    pub fn store_translated_query(
        env: &Env,
        original_query: String,
        target_language: Language,
        translated_query: String,
    ) {
        let key = Self::translation_cache_key(env, &original_query, &target_language);
        env.storage().persistent().set(&key, &translated_query);
    }

    /// Get translated query from cache
    pub fn get_translated_query(
        env: &Env,
        original_query: String,
        target_language: Language,
    ) -> Option<String> {
        let key = Self::translation_cache_key(env, &original_query, &target_language);
        env.storage().persistent().get(&key)
    }

    /// Calculate cross-language similarity score
    pub fn calculate_cross_language_score(
        env: &Env,
        query_language: Language,
        content_language: Language,
        base_score: u32,
    ) -> u32 {
        if query_language == content_language {
            // Same language - no penalty
            base_score
        } else {
            // Different language - apply penalty based on translation quality
            // For now, apply 20% penalty
            (base_score * 80) / 100
        }
    }

    /// Get all supported languages
    pub fn get_supported_languages(env: &Env) -> Vec<Language> {
        // Return all available languages
        let mut languages = Vec::new(env);
        languages.push_back(Language::English);
        languages.push_back(Language::Spanish);
        languages.push_back(Language::French);
        languages.push_back(Language::German);
        languages.push_back(Language::Mandarin);
        languages.push_back(Language::Arabic);
        languages.push_back(Language::Hindi);
        languages.push_back(Language::Portuguese);
        languages.push_back(Language::Russian);
        languages.push_back(Language::Japanese);
        languages
    }

    /// Get content count by language
    pub fn get_language_content_count(env: &Env, language: Language) -> u32 {
        let lang_key = Self::language_index_key(env, &language);

        if let Some(content_ids) = env
            .storage()
            .persistent()
            .get::<String, Vec<String>>(&lang_key)
        {
            content_ids.len()
        } else {
            0
        }
    }

    /// Detect query language (would be done off-chain)
    pub fn detect_query_language(env: &Env, query: String) -> Language {
        // Emit event for off-chain language detection
        env.events()
            .publish((soroban_sdk::symbol_short!("detect_ln"),), query);

        // Default to English for now
        Language::English
    }

    /// Get fallback languages for user
    pub fn get_fallback_languages(env: &Env, user: Address) -> Vec<Language> {
        if let Some(prefs) = Self::get_language_preferences(env, user) {
            prefs.fallback_languages
        } else {
            // Default to English
            let mut fallback = Vec::new(env);
            fallback.push_back(Language::English);
            fallback
        }
    }

    /// Expand search to fallback languages
    pub fn expand_to_fallback_languages(
        env: &Env,
        user: Address,
        content_ids: Vec<String>,
    ) -> Vec<String> {
        let mut expanded = content_ids.clone();
        let fallback_langs = Self::get_fallback_languages(env, user);

        // Add content from fallback languages
        for i in 0..fallback_langs.len() {
            if let Some(lang) = fallback_langs.get(i) {
                let lang_content = Self::search_by_language(env, lang, String::from_str(env, ""));

                for j in 0..lang_content.len() {
                    if let Some(content_id) = lang_content.get(j) {
                        if !expanded.contains(&content_id) {
                            expanded.push_back(content_id);
                        }
                    }
                }
            }
        }

        expanded
    }

    /// Index content by language
    fn index_by_language(env: &Env, content_id: &String, language: &Language) {
        let lang_key = Self::language_index_key(env, language);
        let mut content_ids = env
            .storage()
            .persistent()
            .get::<String, Vec<String>>(&lang_key)
            .unwrap_or_else(|| Vec::new(env));

        if !content_ids.contains(content_id) {
            content_ids.push_back(content_id.clone());
            env.storage().persistent().set(&lang_key, &content_ids);
        }
    }

    /// Generate language index key
    fn language_index_key(env: &Env, language: &Language) -> String {
        Self::language_to_string(env, language)
    }

    /// Convert Language enum to string
    fn language_to_string(env: &Env, language: &Language) -> String {
        let lang_str = match language {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Mandarin => "zh",
            Language::Arabic => "ar",
            Language::Hindi => "hi",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Japanese => "ja",
            Language::Chinese => "zh",
            Language::Korean => "ko",
        };
        String::from_str(env, lang_str)
    }

    /// Generate translation cache key
    fn translation_cache_key(env: &Env, query: &String, language: &Language) -> String {
        String::from_str(env, "trans_cache")
    }
}
