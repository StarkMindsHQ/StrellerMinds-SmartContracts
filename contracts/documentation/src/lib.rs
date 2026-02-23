#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

pub mod analytics;
pub mod api_docs;
pub mod contributions;
pub mod documents;
pub mod knowledge;
pub mod storage;
pub mod translations;
pub mod tutorials;
pub mod types;
pub mod versions;

#[cfg(test)]
mod tests;

use analytics::AnalyticsManager;
use api_docs::ApiDocManager;
use contributions::ContributionManager;
use documents::DocumentManager;
use knowledge::KnowledgeManager;
use storage::Storage;
use translations::TranslationManager;
use tutorials::TutorialManager;
use types::*;
use versions::VersionManager;

#[contract]
pub struct DocumentationContract;

#[contractimpl]
impl DocumentationContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        let config = DocumentationConfig {
            admin: admin.clone(),
            moderators: Vec::new(&env),
            supported_languages: Vec::new(&env),
            max_doc_size: 100000,
            require_review: true,
            enable_contributions: true,
            enable_analytics: true,
        };

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Config, &config);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage()
            .persistent()
            .set(&DataKey::TotalDocuments, &0u64);
        env.storage().persistent().set(&DataKey::TotalViews, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::TotalContributions, &0u64);

        Ok(())
    }

    pub fn get_config(env: Env) -> Result<DocumentationConfig, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    // ========================================================================
    // Document Management
    // ========================================================================

    pub fn create_document(
        env: Env,
        author: Address,
        doc_id: String,
        title: String,
        content: String,
        doc_type: DocumentType,
        category: String,
        tags: Vec<String>,
        language: String,
    ) -> Result<Document, Error> {
        author.require_auth();
        DocumentManager::create_document(
            &env, doc_id, title, content, doc_type, category, &author, tags, language,
        )
    }

    pub fn update_document(
        env: Env,
        author: Address,
        doc_id: String,
        title: Option<String>,
        content: Option<String>,
        status: Option<DocumentStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Document, Error> {
        author.require_auth();
        DocumentManager::update_document(&env, doc_id, title, content, status, tags)
    }

    pub fn publish_document(env: Env, author: Address, doc_id: String) -> Result<(), Error> {
        author.require_auth();
        DocumentManager::publish_document(&env, doc_id)
    }

    pub fn get_document(env: Env, doc_id: String) -> Option<Document> {
        DocumentManager::get_document(&env, &doc_id)
    }

    pub fn view_document(env: Env, doc_id: String) -> Result<(), Error> {
        DocumentManager::increment_view_count(&env, doc_id)
    }

    pub fn mark_helpful(env: Env, user: Address, doc_id: String) -> Result<(), Error> {
        user.require_auth();
        DocumentManager::mark_helpful(&env, doc_id)
    }

    pub fn get_documents_by_category(env: Env, category: String) -> Vec<String> {
        DocumentManager::get_documents_by_category(&env, &category)
    }

    pub fn get_documents_by_author(env: Env, author: Address) -> Vec<String> {
        DocumentManager::get_documents_by_author(&env, &author)
    }

    // ========================================================================
    // Version Management
    // ========================================================================

    pub fn create_version(
        env: Env,
        author: Address,
        doc_id: String,
        version_number: u32,
        content: String,
        changelog: String,
    ) -> Result<DocumentVersion, Error> {
        author.require_auth();
        VersionManager::create_version(&env, doc_id, version_number, content, &author, changelog)
    }

    pub fn get_version(env: Env, doc_id: String, version_number: u32) -> Option<DocumentVersion> {
        VersionManager::get_version(&env, doc_id, version_number)
    }

    pub fn get_current_version(env: Env, doc_id: String) -> Option<DocumentVersion> {
        VersionManager::get_current_version(&env, doc_id)
    }

    // ========================================================================
    // Knowledge Base
    // ========================================================================

    pub fn create_article(
        env: Env,
        author: Address,
        article_id: String,
        title: String,
        content: String,
        category: String,
        tags: Vec<String>,
    ) -> Result<KnowledgeArticle, Error> {
        author.require_auth();
        KnowledgeManager::create_article(&env, article_id, title, content, category, &author, tags)
    }

    pub fn create_faq(
        env: Env,
        author: Address,
        faq_id: String,
        question: String,
        answer: String,
        category: String,
        order_index: u32,
    ) -> Result<FAQ, Error> {
        author.require_auth();
        KnowledgeManager::create_faq(
            &env,
            faq_id,
            question,
            answer,
            category,
            &author,
            order_index,
        )
    }

    pub fn vote_article(
        env: Env,
        user: Address,
        article_id: String,
        is_helpful: bool,
    ) -> Result<(), Error> {
        user.require_auth();
        KnowledgeManager::vote_article(&env, article_id, is_helpful)
    }

    pub fn get_article(env: Env, article_id: String) -> Option<KnowledgeArticle> {
        KnowledgeManager::get_article(&env, &article_id)
    }

    pub fn get_faq(env: Env, faq_id: String) -> Option<FAQ> {
        KnowledgeManager::get_faq(&env, &faq_id)
    }

    // ========================================================================
    // API Documentation
    // ========================================================================

    pub fn create_api_endpoint(
        env: Env,
        admin: Address,
        endpoint_id: String,
        name: String,
        description: String,
        method: String,
        path: String,
        parameters: Vec<ApiParameter>,
        response_schema: String,
        version: String,
    ) -> Result<ApiEndpoint, Error> {
        admin.require_auth();
        ApiDocManager::create_endpoint(
            &env,
            endpoint_id,
            name,
            description,
            method,
            path,
            parameters,
            response_schema,
            version,
        )
    }

    pub fn add_code_example(
        env: Env,
        admin: Address,
        endpoint_id: String,
        example: CodeExample,
    ) -> Result<(), Error> {
        admin.require_auth();
        ApiDocManager::add_code_example(&env, endpoint_id, example)
    }

    pub fn get_api_endpoint(env: Env, endpoint_id: String) -> Option<ApiEndpoint> {
        ApiDocManager::get_endpoint(&env, &endpoint_id)
    }

    // ========================================================================
    // Tutorials
    // ========================================================================

    pub fn create_tutorial(
        env: Env,
        author: Address,
        tutorial_id: String,
        title: String,
        description: String,
        difficulty: DifficultyLevel,
        estimated_time: u32,
        steps: Vec<TutorialStep>,
        prerequisites: Vec<String>,
    ) -> Result<Tutorial, Error> {
        author.require_auth();
        TutorialManager::create_tutorial(
            &env,
            tutorial_id,
            title,
            description,
            difficulty,
            estimated_time,
            &author,
            steps,
            prerequisites,
        )
    }

    pub fn complete_tutorial(env: Env, user: Address, tutorial_id: String) -> Result<(), Error> {
        user.require_auth();
        TutorialManager::complete_tutorial(&env, tutorial_id)
    }

    pub fn get_tutorial(env: Env, tutorial_id: String) -> Option<Tutorial> {
        TutorialManager::get_tutorial(&env, &tutorial_id)
    }

    // ========================================================================
    // Community Contributions
    // ========================================================================

    pub fn submit_contribution(
        env: Env,
        contributor: Address,
        contribution_id: String,
        doc_id: String,
        contribution_type: ContributionType,
        content: String,
    ) -> Result<Contribution, Error> {
        contributor.require_auth();
        ContributionManager::submit_contribution(
            &env,
            contribution_id,
            &contributor,
            doc_id,
            contribution_type,
            content,
        )
    }

    pub fn review_contribution(
        env: Env,
        reviewer: Address,
        contribution_id: String,
        status: ContributionStatus,
        notes: Option<String>,
    ) -> Result<(), Error> {
        reviewer.require_auth();
        ContributionManager::review_contribution(&env, contribution_id, &reviewer, status, notes)
    }

    pub fn get_contribution(env: Env, contribution_id: String) -> Option<Contribution> {
        ContributionManager::get_contribution(&env, &contribution_id)
    }

    // ========================================================================
    // Translations
    // ========================================================================

    pub fn create_translation(
        env: Env,
        translator: Address,
        translation_id: String,
        original_doc_id: String,
        language: String,
        title: String,
        content: String,
    ) -> Result<Translation, Error> {
        translator.require_auth();
        TranslationManager::create_translation(
            &env,
            translation_id,
            original_doc_id,
            language,
            title,
            content,
            &translator,
        )
    }

    pub fn update_translation_status(
        env: Env,
        admin: Address,
        translation_id: String,
        status: TranslationStatus,
    ) -> Result<(), Error> {
        admin.require_auth();
        TranslationManager::update_translation_status(&env, translation_id, status)
    }

    pub fn get_translation(env: Env, translation_id: String) -> Option<Translation> {
        TranslationManager::get_translation(&env, &translation_id)
    }

    // ========================================================================
    // Analytics
    // ========================================================================

    pub fn track_search(
        env: Env,
        user: Address,
        query_id: String,
        query_text: String,
        results_count: u32,
    ) -> Result<SearchQuery, Error> {
        user.require_auth();
        AnalyticsManager::track_search(&env, query_id, query_text, &user, results_count)
    }

    pub fn get_document_analytics(env: Env, doc_id: String) -> Option<DocumentAnalytics> {
        AnalyticsManager::get_document_analytics(&env, &doc_id)
    }

    pub fn get_total_documents(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalDocuments)
    }

    pub fn get_total_views(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalViews)
    }

    pub fn get_total_contributions(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalContributions)
    }
}
