use crate::types::*;
use soroban_sdk::{Address, Env, String};

pub struct TranslationManager;

impl TranslationManager {
    pub fn create_translation(
        env: &Env,
        translation_id: String,
        original_doc_id: String,
        language: String,
        title: String,
        content: String,
        translator: &Address,
    ) -> Result<Translation, Error> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::Translation(translation_id.clone()))
        {
            return Err(Error::AlreadyExists);
        }

        let translation = Translation {
            translation_id: translation_id.clone(),
            original_doc_id,
            language,
            title,
            content,
            translator: translator.clone(),
            status: TranslationStatus::InProgress,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Translation(translation_id), &translation);

        Ok(translation)
    }

    pub fn update_translation_status(
        env: &Env,
        translation_id: String,
        status: TranslationStatus,
    ) -> Result<(), Error> {
        let mut translation: Translation = env
            .storage()
            .persistent()
            .get(&DataKey::Translation(translation_id.clone()))
            .ok_or(Error::TranslationNotFound)?;

        translation.status = status;
        env.storage()
            .persistent()
            .set(&DataKey::Translation(translation_id), &translation);

        Ok(())
    }

    pub fn get_translation(env: &Env, translation_id: &String) -> Option<Translation> {
        env.storage()
            .persistent()
            .get(&DataKey::Translation(translation_id.clone()))
    }
}
