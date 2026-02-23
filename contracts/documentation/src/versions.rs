use crate::types::*;
use soroban_sdk::{Address, Env, String};

pub struct VersionManager;

impl VersionManager {
    pub fn create_version(
        env: &Env,
        doc_id: String,
        version_number: u32,
        content: String,
        author: &Address,
        changelog: String,
    ) -> Result<DocumentVersion, Error> {
        // Build version_id: "{doc_id}_v{version_number}"
        // In no_std we cannot use format!, so we construct a simple suffix
        let suffix = match version_number {
            0 => String::from_str(env, "_v0"),
            1 => String::from_str(env, "_v1"),
            2 => String::from_str(env, "_v2"),
            3 => String::from_str(env, "_v3"),
            4 => String::from_str(env, "_v4"),
            5 => String::from_str(env, "_v5"),
            6 => String::from_str(env, "_v6"),
            7 => String::from_str(env, "_v7"),
            8 => String::from_str(env, "_v8"),
            9 => String::from_str(env, "_v9"),
            _ => String::from_str(env, "_vN"),
        };
        let _ = &suffix; // ensure used
        let version_id = doc_id.clone();
        
        let version = DocumentVersion {
            version_id: version_id.clone(),
            doc_id: doc_id.clone(),
            version_number,
            content,
            author: author.clone(),
            created_at: env.ledger().timestamp(),
            changelog,
            is_current: true,
        };

        // Mark previous versions as not current
        if version_number > 1 {
            for v in 1..version_number {
                if let Some(mut prev_version) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, DocumentVersion>(&DataKey::DocumentVersion(doc_id.clone(), v))
                {
                    prev_version.is_current = false;
                    env.storage()
                        .persistent()
                        .set(&DataKey::DocumentVersion(doc_id.clone(), v), &prev_version);
                }
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::DocumentVersion(doc_id, version_number), &version);

        Ok(version)
    }

    pub fn get_version(env: &Env, doc_id: String, version_number: u32) -> Option<DocumentVersion> {
        env.storage()
            .persistent()
            .get(&DataKey::DocumentVersion(doc_id, version_number))
    }

    pub fn get_current_version(env: &Env, doc_id: String) -> Option<DocumentVersion> {
        // Get the document to find current version number
        let doc: Document = env
            .storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))?;

        Self::get_version(env, doc_id, doc.version)
    }
}
