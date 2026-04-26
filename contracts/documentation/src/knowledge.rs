use crate::types::*;
use shared::validation::{CoreValidator, ValidationConfig};
use soroban_sdk::{Address, Env, String, Vec};

pub struct KnowledgeManager;

impl KnowledgeManager {
    pub fn create_article(
        env: &Env,
        article_id: String,
        title: String,
        content: String,
        category: String,
        author: &Address,
        tags: Vec<String>,
    ) -> Result<KnowledgeArticle, Error> {
        // Validate inputs
        CoreValidator::validate_soroban_string_length(
            &title,
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        )
        .map_err(|_| Error::InvalidDocument)?;
        CoreValidator::validate_soroban_string_length(
            &content,
            "content",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_CONTENT_LENGTH,
        )
        .map_err(|_| Error::InvalidDocument)?;
        CoreValidator::validate_vec_size(tags.len(), "tags", ValidationConfig::MAX_TAGS)
            .map_err(|_| Error::InvalidDocument)?;

        if env.storage().persistent().has(&DataKey::KnowledgeArticle(article_id.clone())) {
            return Err(Error::AlreadyExists);
        }

        let article = KnowledgeArticle {
            article_id: article_id.clone(),
            title,
            content,
            category,
            author: author.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            view_count: 0,
            helpful_votes: 0,
            not_helpful_votes: 0,
            related_articles: Vec::new(env),
            tags,
        };

        env.storage().persistent().set(&DataKey::KnowledgeArticle(article_id), &article);

        Ok(article)
    }

    pub fn create_faq(
        env: &Env,
        faq_id: String,
        question: String,
        answer: String,
        category: String,
        author: &Address,
        order_index: u32,
    ) -> Result<FAQ, Error> {
        // Validate inputs
        CoreValidator::validate_soroban_string_length(
            &question,
            "question",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        )
        .map_err(|_| Error::InvalidDocument)?;
        CoreValidator::validate_soroban_string_length(
            &answer,
            "answer",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_CONTENT_LENGTH,
        )
        .map_err(|_| Error::InvalidDocument)?;

        if env.storage().persistent().has(&DataKey::FAQ(faq_id.clone())) {
            return Err(Error::AlreadyExists);
        }

        let faq = FAQ {
            faq_id: faq_id.clone(),
            question,
            answer,
            category,
            author: author.clone(),
            created_at: env.ledger().timestamp(),
            view_count: 0,
            helpful_count: 0,
            order_index,
        };

        env.storage().persistent().set(&DataKey::FAQ(faq_id), &faq);

        Ok(faq)
    }

    pub fn vote_article(env: &Env, article_id: String, is_helpful: bool) -> Result<(), Error> {
        let mut article: KnowledgeArticle = env
            .storage()
            .persistent()
            .get(&DataKey::KnowledgeArticle(article_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        if is_helpful {
            article.helpful_votes += 1;
        } else {
            article.not_helpful_votes += 1;
        }

        env.storage().persistent().set(&DataKey::KnowledgeArticle(article_id), &article);

        Ok(())
    }

    pub fn get_article(env: &Env, article_id: &String) -> Option<KnowledgeArticle> {
        env.storage().persistent().get(&DataKey::KnowledgeArticle(article_id.clone()))
    }

    pub fn get_faq(env: &Env, faq_id: &String) -> Option<FAQ> {
        env.storage().persistent().get(&DataKey::FAQ(faq_id.clone()))
    }
}
