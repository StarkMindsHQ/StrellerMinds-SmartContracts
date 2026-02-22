use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::*;

pub struct ContentCacheManager;

impl ContentCacheManager {
    pub fn initialize_cache(env: &Env, user: &Address) -> CacheConfig {
        let config = CacheConfig {
            max_cache_size_bytes: 50 * 1024 * 1024, // 50 MB
            default_ttl_seconds: 86400,              // 24 hours
            eviction_policy: EvictionPolicy::LeastRecentlyUsed,
            prefetch_enabled: true,
            compression_enabled: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::UserCacheConfig(user.clone()), &config);

        let stats = CacheStats {
            total_entries: 0,
            total_size_bytes: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            hit_rate_bps: 0,
            avg_access_time_ms: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::CacheStats(user.clone()), &stats);

        config
    }

    pub fn cache_content(
        env: &Env,
        user: &Address,
        cache_key: String,
        content_hash: BytesN<32>,
        content_type: ContentType,
        size_bytes: u64,
        ttl_seconds: u64,
    ) -> Result<(), MobileOptimizerError> {
        let config: CacheConfig = env
            .storage()
            .persistent()
            .get(&DataKey::UserCacheConfig(user.clone()))
            .unwrap_or_else(|| Self::initialize_cache(env, user));

        let mut stats: CacheStats = env
            .storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)?;

        if stats.total_size_bytes + size_bytes > config.max_cache_size_bytes {
            let freed = Self::evict_entries(env, user, size_bytes, &config)?;
            if freed < size_bytes {
                return Err(MobileOptimizerError::CacheFull);
            }
        }

        let now = env.ledger().timestamp();
        let priority = Self::determine_cache_priority(&content_type);

        let entry = CacheEntry {
            cache_key: cache_key.clone(),
            content_hash,
            content_type,
            size_bytes,
            created_at: now,
            expires_at: now + ttl_seconds,
            access_count: 0,
            last_accessed: now,
            priority,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ContentCache(cache_key), &entry);

        stats.total_entries += 1;
        stats.total_size_bytes += size_bytes;
        env.storage()
            .persistent()
            .set(&DataKey::CacheStats(user.clone()), &stats);

        Ok(())
    }

    pub fn get_cached_content(
        env: &Env,
        user: &Address,
        cache_key: String,
    ) -> Result<CacheEntry, MobileOptimizerError> {
        let mut stats: CacheStats = env
            .storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)?;

        match env
            .storage()
            .persistent()
            .get::<DataKey, CacheEntry>(&DataKey::ContentCache(cache_key.clone()))
        {
            Some(mut entry) => {
                let now = env.ledger().timestamp();
                if now > entry.expires_at {
                    Self::remove_entry(env, user, &cache_key, entry.size_bytes)?;
                    stats.miss_count += 1;
                    Self::update_hit_rate(&mut stats);
                    env.storage()
                        .persistent()
                        .set(&DataKey::CacheStats(user.clone()), &stats);
                    return Err(MobileOptimizerError::CacheError);
                }

                entry.access_count += 1;
                entry.last_accessed = now;
                env.storage()
                    .persistent()
                    .set(&DataKey::ContentCache(cache_key), &entry);

                stats.hit_count += 1;
                Self::update_hit_rate(&mut stats);
                env.storage()
                    .persistent()
                    .set(&DataKey::CacheStats(user.clone()), &stats);

                Ok(entry)
            }
            None => {
                stats.miss_count += 1;
                Self::update_hit_rate(&mut stats);
                env.storage()
                    .persistent()
                    .set(&DataKey::CacheStats(user.clone()), &stats);
                Err(MobileOptimizerError::CacheError)
            }
        }
    }

    pub fn setup_prefetch_rules(
        env: &Env,
        user: &Address,
        rules: Vec<PrefetchRule>,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .set(&DataKey::PrefetchRules(user.clone()), &rules);
        Ok(())
    }

    pub fn get_prefetch_recommendations(
        env: &Env,
        user: &Address,
        network_quality: &NetworkQuality,
    ) -> Vec<PrefetchRule> {
        let rules: Vec<PrefetchRule> = env
            .storage()
            .persistent()
            .get(&DataKey::PrefetchRules(user.clone()))
            .unwrap_or_else(|| Self::default_prefetch_rules(env));

        let mut applicable = Vec::new(env);
        for rule in rules.iter() {
            if Self::is_network_sufficient(network_quality, &rule.network_requirement) {
                applicable.push_back(rule.clone());
            }
        }
        applicable
    }

    pub fn execute_prefetch(
        env: &Env,
        user: &Address,
        trigger: PrefetchTrigger,
        network_quality: &NetworkQuality,
    ) -> Result<u32, MobileOptimizerError> {
        let rules = Self::get_prefetch_recommendations(env, user, network_quality);
        let mut prefetched_count = 0u32;

        for rule in rules.iter() {
            if rule.trigger == trigger {
                let content_hash = BytesN::from_array(env, &[0u8; 32]);
                Self::cache_content(
                    env,
                    user,
                    rule.rule_id.clone(),
                    content_hash,
                    rule.content_type.clone(),
                    rule.max_prefetch_size_bytes,
                    86400,
                )?;
                prefetched_count += 1;
            }
        }

        Ok(prefetched_count)
    }

    pub fn invalidate_cache(
        env: &Env,
        user: &Address,
        cache_key: String,
    ) -> Result<(), MobileOptimizerError> {
        if let Some(entry) = env
            .storage()
            .persistent()
            .get::<DataKey, CacheEntry>(&DataKey::ContentCache(cache_key.clone()))
        {
            Self::remove_entry(env, user, &cache_key, entry.size_bytes)?;
        }
        Ok(())
    }

    pub fn get_cache_stats(
        env: &Env,
        user: &Address,
    ) -> Result<CacheStats, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)
    }

    pub fn update_cache_config(
        env: &Env,
        user: &Address,
        config: CacheConfig,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .set(&DataKey::UserCacheConfig(user.clone()), &config);
        Ok(())
    }

    pub fn clear_expired_entries(
        env: &Env,
        user: &Address,
    ) -> Result<u32, MobileOptimizerError> {
        // In a real implementation we'd iterate through all cache entries.
        // Here we record the cleanup action.
        let mut stats: CacheStats = env
            .storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)?;

        let cleaned = stats.eviction_count;
        stats.eviction_count = 0;
        env.storage()
            .persistent()
            .set(&DataKey::CacheStats(user.clone()), &stats);
        Ok(cleaned)
    }

    // Internal helpers

    fn evict_entries(
        env: &Env,
        user: &Address,
        needed_bytes: u64,
        _config: &CacheConfig,
    ) -> Result<u64, MobileOptimizerError> {
        let mut stats: CacheStats = env
            .storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)?;

        let freed = needed_bytes;
        stats.total_size_bytes = stats.total_size_bytes.saturating_sub(freed);
        stats.eviction_count += 1;
        if stats.total_entries > 0 {
            stats.total_entries -= 1;
        }
        env.storage()
            .persistent()
            .set(&DataKey::CacheStats(user.clone()), &stats);
        Ok(freed)
    }

    fn remove_entry(
        env: &Env,
        user: &Address,
        cache_key: &String,
        size_bytes: u64,
    ) -> Result<(), MobileOptimizerError> {
        env.storage()
            .persistent()
            .remove(&DataKey::ContentCache(cache_key.clone()));

        let mut stats: CacheStats = env
            .storage()
            .persistent()
            .get(&DataKey::CacheStats(user.clone()))
            .ok_or(MobileOptimizerError::CacheError)?;

        stats.total_size_bytes = stats.total_size_bytes.saturating_sub(size_bytes);
        if stats.total_entries > 0 {
            stats.total_entries -= 1;
        }
        env.storage()
            .persistent()
            .set(&DataKey::CacheStats(user.clone()), &stats);
        Ok(())
    }

    fn update_hit_rate(stats: &mut CacheStats) {
        let total = stats.hit_count + stats.miss_count;
        if total > 0 {
            stats.hit_rate_bps = ((stats.hit_count * 10000) / total) as u32;
        }
    }

    fn determine_cache_priority(content_type: &ContentType) -> CachePriority {
        match content_type {
            ContentType::CourseMaterial => CachePriority::High,
            ContentType::VideoLesson => CachePriority::Normal,
            ContentType::QuizData => CachePriority::Essential,
            ContentType::Certificate => CachePriority::High,
            ContentType::UserProfile => CachePriority::Essential,
            ContentType::SearchResults => CachePriority::Low,
            ContentType::ProgressData => CachePriority::Essential,
            ContentType::NotificationData => CachePriority::Evictable,
        }
    }

    fn is_network_sufficient(current: &NetworkQuality, required: &NetworkQuality) -> bool {
        let current_level = Self::network_quality_level(current);
        let required_level = Self::network_quality_level(required);
        current_level >= required_level
    }

    fn network_quality_level(quality: &NetworkQuality) -> u32 {
        match quality {
            NetworkQuality::Offline => 0,
            NetworkQuality::Poor => 1,
            NetworkQuality::Fair => 2,
            NetworkQuality::Good => 3,
            NetworkQuality::Excellent => 4,
        }
    }

    fn default_prefetch_rules(env: &Env) -> Vec<PrefetchRule> {
        let mut rules = Vec::new(env);
        rules.push_back(PrefetchRule {
            rule_id: String::from_str(env, "prefetch_course_material"),
            content_type: ContentType::CourseMaterial,
            trigger: PrefetchTrigger::OnCourseEnroll,
            network_requirement: NetworkQuality::Good,
            max_prefetch_size_bytes: 5 * 1024 * 1024,
        });
        rules.push_back(PrefetchRule {
            rule_id: String::from_str(env, "prefetch_quiz"),
            content_type: ContentType::QuizData,
            trigger: PrefetchTrigger::OnModuleComplete,
            network_requirement: NetworkQuality::Fair,
            max_prefetch_size_bytes: 1024 * 1024,
        });
        rules.push_back(PrefetchRule {
            rule_id: String::from_str(env, "prefetch_on_wifi"),
            content_type: ContentType::VideoLesson,
            trigger: PrefetchTrigger::OnWifiConnect,
            network_requirement: NetworkQuality::Excellent,
            max_prefetch_size_bytes: 20 * 1024 * 1024,
        });
        rules
    }
}
