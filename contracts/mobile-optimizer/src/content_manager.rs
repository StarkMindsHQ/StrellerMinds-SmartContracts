use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::types::*;

pub struct ContentManager;

impl ContentManager {
    #[allow(clippy::too_many_arguments)]
    pub fn publish_content(
        env: &Env,
        author: &Address,
        content_id: String,
        content_type: ContentType,
        title: String,
        uri: String,
        access_rule: ContentAccessRule,
        delivery_config: ContentDeliveryConfig,
        content_hash: BytesN<32>,
    ) -> Result<ContentMetadata, MobileOptimizerError> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::ContentItem(content_id.clone()))
        {
            return Err(MobileOptimizerError::InvalidInput);
        }

        let metadata = ContentMetadata {
            content_id: content_id.clone(),
            content_type,
            title,
            uri: uri.clone(),
            current_version: 1,
            author: author.clone(),
            access_rule,
            delivery_config,
            total_views: 0,
            average_rating: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::ContentItem(content_id.clone()), &metadata);

        let version = ContentVersion {
            content_id: content_id.clone(),
            version: 1,
            content_hash,
            uri,
            created_at: env.ledger().timestamp(),
            changelog: String::from_str(env, "Initial release"),
        };

        let mut history = Vec::new(env);
        history.push_back(version);
        env.storage()
            .persistent()
            .set(&DataKey::ContentVersionHistory(content_id), &history);

        Ok(metadata)
    }

    pub fn update_content_version(
        env: &Env,
        author: &Address,
        content_id: String,
        new_uri: String,
        content_hash: BytesN<32>,
        changelog: String,
    ) -> Result<ContentVersion, MobileOptimizerError> {
        let mut metadata: ContentMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::ContentItem(content_id.clone()))
            .ok_or(MobileOptimizerError::ContentError)?;

        if metadata.author != *author {
            return Err(MobileOptimizerError::Unauthorized);
        }

        metadata.current_version += 1;
        metadata.uri = new_uri.clone();

        let version = ContentVersion {
            content_id: content_id.clone(),
            version: metadata.current_version,
            content_hash,
            uri: new_uri,
            created_at: env.ledger().timestamp(),
            changelog,
        };

        let mut history: Vec<ContentVersion> = env
            .storage()
            .persistent()
            .get(&DataKey::ContentVersionHistory(content_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        history.push_back(version.clone());

        env.storage()
            .persistent()
            .set(&DataKey::ContentItem(content_id.clone()), &metadata);
        env.storage()
            .persistent()
            .set(&DataKey::ContentVersionHistory(content_id), &history);

        Ok(version)
    }

    pub fn get_content(
        env: &Env,
        content_id: String,
    ) -> Result<ContentMetadata, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::ContentItem(content_id))
            .ok_or(MobileOptimizerError::ContentError)
    }

    pub fn get_version_history(
        env: &Env,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, MobileOptimizerError> {
        env.storage()
            .persistent()
            .get(&DataKey::ContentVersionHistory(content_id))
            .ok_or(MobileOptimizerError::ContentError)
    }
}
