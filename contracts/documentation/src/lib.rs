#![no_std]
#![allow(clippy::too_many_arguments)]

pub mod errors;

use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
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
use shared::config::DeploymentEnv;
use storage::Storage;
use translations::TranslationManager;
use tutorials::TutorialManager;
use types::*;
use versions::VersionManager;

const RL_OP_CREATE_DOC: u64 = 1;
const RL_OP_CONTRIBUTION: u64 = 2;

fn check_rate_limit_doc(
    env: &Env,
    user: &Address,
    operation: u64,
    max_calls: u32,
    window: u64,
) -> Result<(), Error> {
    let admin: Option<Address> = env.storage().persistent().get(&DataKey::Admin);
    if admin.as_ref() == Some(user) {
        return Ok(());
    }
    enforce_rate_limit(
        env,
        &DataKey::RateLimit(user.clone(), operation),
        &RateLimitConfig { max_calls, window_seconds: window },
    )
    .map_err(|_| Error::RateLimitExceeded)
}

#[contract]
pub struct DocumentationContract;

#[contractimpl]
impl DocumentationContract {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the documentation contract and set the admin address.
    ///
    /// Must be called once before any other function. Requires authorization from `admin`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - The address that will hold admin privileges.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyInitialized`] if the contract has already been set up.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        let config = DocumentationConfig::for_env(&env, admin.clone(), DeploymentEnv::Production);
        config.validate()?;

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Config, &config);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::TotalDocuments, &0u64);
        env.storage().persistent().set(&DataKey::TotalViews, &0u64);
        env.storage().persistent().set(&DataKey::TotalContributions, &0u64);

        Ok(())
    }

    /// Retrieve the current documentation contract configuration.
    ///
    /// # Errors
    /// Returns [`DocumentationError::NotInitialized`] if the contract has not been initialized.
    ///
    /// # Example
    /// ```ignore
    /// let config = client.get_config();
    /// ```
    pub fn get_config(env: Env) -> Result<DocumentationConfig, Error> {
        env.storage().persistent().get(&DataKey::Config).ok_or(Error::NotInitialized)
    }

    // ========================================================================
    // Document Management
    // ========================================================================

    /// Create and store a new documentation document authored by the caller.
    ///
    /// Requires authorization from `author`. The document starts in a draft state pending review.
    ///
    /// # Arguments
    /// * `author` - Address of the document author; must sign the transaction.
    /// * `doc_id` - Unique identifier for the new document.
    /// * `title` - Human-readable title of the document.
    /// * `content` - Full document body content.
    /// * `doc_type` - Category type of the document (e.g., guide, reference).
    /// * `category` - Grouping category string for organizational purposes.
    /// * `tags` - List of searchable tag strings.
    /// * `language` - Language code (e.g., `"en"`) for the document content.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if a document with `doc_id` already exists.
    /// Returns [`DocumentationError::DocumentTooLarge`] if `content` exceeds the size limit.
    ///
    /// # Example
    /// ```ignore
    /// client.create_document(&author, &doc_id, &title, &content, &doc_type, &category, &tags, &language);
    /// ```
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
        let doc_cfg: DocumentationConfig =
            env.storage().persistent().get(&DataKey::Config).ok_or(Error::NotInitialized)?;
        check_rate_limit_doc(
            &env,
            &author,
            RL_OP_CREATE_DOC,
            doc_cfg.rate_limit_create_doc,
            doc_cfg.rate_limit_window,
        )?;
        DocumentManager::create_document(
            &env, doc_id, title, content, doc_type, category, &author, tags, language,
        )
    }

    /// Update one or more fields of an existing document.
    ///
    /// Only fields provided as `Some(...)` are updated; `None` fields are left unchanged.
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the original author; must sign the transaction.
    /// * `doc_id` - Identifier of the document to update.
    /// * `title` - Optional new title for the document.
    /// * `content` - Optional new body content for the document.
    /// * `status` - Optional new publication status.
    /// * `tags` - Optional new list of tags.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `doc_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.update_document(&author, &doc_id, &None, &Some(new_content), &None, &None);
    /// ```
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

    /// Publish a draft document, making it publicly visible.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the document author; must sign the transaction.
    /// * `doc_id` - Identifier of the document to publish.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `doc_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.publish_document(&author, &doc_id);
    /// ```
    pub fn publish_document(env: Env, author: Address, doc_id: String) -> Result<(), Error> {
        author.require_auth();
        DocumentManager::publish_document(&env, doc_id)
    }

    /// Retrieve a document by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `doc_id` - Identifier of the document to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let doc = client.get_document(&doc_id);
    /// ```
    pub fn get_document(env: Env, doc_id: String) -> Option<Document> {
        DocumentManager::get_document(&env, &doc_id)
    }

    /// Record a view event for the given document, incrementing its view counter.
    ///
    /// # Arguments
    /// * `doc_id` - Identifier of the document being viewed.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `doc_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.view_document(&doc_id);
    /// ```
    pub fn view_document(env: Env, doc_id: String) -> Result<(), Error> {
        DocumentManager::increment_view_count(&env, doc_id)
    }

    /// Mark a document as helpful, incrementing its helpfulness counter.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user marking the document helpful.
    /// * `doc_id` - Identifier of the document being marked.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `doc_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.mark_helpful(&user, &doc_id);
    /// ```
    pub fn mark_helpful(env: Env, user: Address, doc_id: String) -> Result<(), Error> {
        user.require_auth();
        DocumentManager::mark_helpful(&env, doc_id)
    }

    /// Return the list of document IDs belonging to the given category.
    ///
    /// # Arguments
    /// * `category` - Category string to filter documents by.
    ///
    /// # Example
    /// ```ignore
    /// let ids = client.get_documents_by_category(&category);
    /// ```
    pub fn get_documents_by_category(env: Env, category: String) -> Vec<String> {
        DocumentManager::get_documents_by_category(&env, &category)
    }

    /// Return the list of document IDs authored by the given address.
    ///
    /// # Arguments
    /// * `author` - Address of the document author to filter by.
    ///
    /// # Example
    /// ```ignore
    /// let ids = client.get_documents_by_author(&author);
    /// ```
    pub fn get_documents_by_author(env: Env, author: Address) -> Vec<String> {
        DocumentManager::get_documents_by_author(&env, &author)
    }

    // ========================================================================
    // Version Management
    // ========================================================================

    /// Create a new versioned snapshot of an existing document.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the version author; must sign the transaction.
    /// * `doc_id` - Identifier of the parent document.
    /// * `version_number` - Monotonically increasing version number for this snapshot.
    /// * `content` - Full body content for this version.
    /// * `changelog` - Human-readable description of changes from the previous version.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `doc_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.create_version(&author, &doc_id, &2u32, &content, &changelog);
    /// ```
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

    /// Retrieve a specific version of a document by document ID and version number.
    ///
    /// # Arguments
    /// * `doc_id` - Identifier of the parent document.
    /// * `version_number` - The version number to retrieve.
    ///
    /// # Example
    /// ```ignore
    /// let version = client.get_version(&doc_id, &1u32);
    /// ```
    pub fn get_version(env: Env, doc_id: String, version_number: u32) -> Option<DocumentVersion> {
        VersionManager::get_version(&env, doc_id, version_number)
    }

    /// Retrieve the most recent version of a document.
    ///
    /// # Arguments
    /// * `doc_id` - Identifier of the parent document.
    ///
    /// # Example
    /// ```ignore
    /// let version = client.get_current_version(&doc_id);
    /// ```
    pub fn get_current_version(env: Env, doc_id: String) -> Option<DocumentVersion> {
        VersionManager::get_current_version(&env, doc_id)
    }

    // ========================================================================
    // Knowledge Base
    // ========================================================================

    /// Create a new knowledge base article.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the article author; must sign the transaction.
    /// * `article_id` - Unique identifier for the new article.
    /// * `title` - Human-readable title of the article.
    /// * `content` - Full article body content.
    /// * `category` - Grouping category for the article.
    /// * `tags` - List of searchable tag strings.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if an article with `article_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.create_article(&author, &article_id, &title, &content, &category, &tags);
    /// ```
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

    /// Create a new FAQ entry in the knowledge base.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the FAQ author; must sign the transaction.
    /// * `faq_id` - Unique identifier for the new FAQ entry.
    /// * `question` - The question text.
    /// * `answer` - The answer text.
    /// * `category` - Grouping category for the FAQ.
    /// * `order_index` - Display order position within its category.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if an FAQ with `faq_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.create_faq(&author, &faq_id, &question, &answer, &category, &0u32);
    /// ```
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
        KnowledgeManager::create_faq(&env, faq_id, question, answer, category, &author, order_index)
    }

    /// Submit a helpfulness vote on a knowledge base article.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the voter; must sign the transaction.
    /// * `article_id` - Identifier of the article being voted on.
    /// * `is_helpful` - `true` to vote helpful, `false` to vote not helpful.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `article_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.vote_article(&user, &article_id, &true);
    /// ```
    pub fn vote_article(
        env: Env,
        user: Address,
        article_id: String,
        is_helpful: bool,
    ) -> Result<(), Error> {
        user.require_auth();
        KnowledgeManager::vote_article(&env, article_id, is_helpful)
    }

    /// Retrieve a knowledge base article by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `article_id` - Identifier of the article to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let article = client.get_article(&article_id);
    /// ```
    pub fn get_article(env: Env, article_id: String) -> Option<KnowledgeArticle> {
        KnowledgeManager::get_article(&env, &article_id)
    }

    /// Retrieve an FAQ entry by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `faq_id` - Identifier of the FAQ to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let faq = client.get_faq(&faq_id);
    /// ```
    pub fn get_faq(env: Env, faq_id: String) -> Option<FAQ> {
        KnowledgeManager::get_faq(&env, &faq_id)
    }

    // ========================================================================
    // API Documentation
    // ========================================================================

    /// Register a new API endpoint in the documentation registry.
    ///
    /// Requires authorization from `admin`.
    ///
    /// # Arguments
    /// * `admin` - Admin address; must sign the transaction.
    /// * `endpoint_id` - Unique identifier for the endpoint.
    /// * `name` - Human-readable name of the endpoint.
    /// * `description` - Description of what the endpoint does.
    /// * `method` - HTTP method (e.g., `"GET"`, `"POST"`).
    /// * `path` - URL path of the endpoint.
    /// * `parameters` - List of accepted parameters.
    /// * `response_schema` - JSON schema string describing the response.
    /// * `version` - API version string (e.g., `"v1"`).
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if an endpoint with `endpoint_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.create_api_endpoint(&admin, &endpoint_id, &name, &desc, &"GET", &path, &params, &schema, &"v1");
    /// ```
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

    /// Attach a code example to an existing API endpoint entry.
    ///
    /// Requires authorization from `admin`.
    ///
    /// # Arguments
    /// * `admin` - Admin address; must sign the transaction.
    /// * `endpoint_id` - Identifier of the target API endpoint.
    /// * `example` - The code example to attach.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `endpoint_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.add_code_example(&admin, &endpoint_id, &example);
    /// ```
    pub fn add_code_example(
        env: Env,
        admin: Address,
        endpoint_id: String,
        example: CodeExample,
    ) -> Result<(), Error> {
        admin.require_auth();
        ApiDocManager::add_code_example(&env, endpoint_id, example)
    }

    /// Retrieve an API endpoint entry by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `endpoint_id` - Identifier of the API endpoint to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let endpoint = client.get_api_endpoint(&endpoint_id);
    /// ```
    pub fn get_api_endpoint(env: Env, endpoint_id: String) -> Option<ApiEndpoint> {
        ApiDocManager::get_endpoint(&env, &endpoint_id)
    }

    // ========================================================================
    // Tutorials
    // ========================================================================

    /// Create a new step-by-step tutorial.
    ///
    /// Requires authorization from `author`.
    ///
    /// # Arguments
    /// * `author` - Address of the tutorial author; must sign the transaction.
    /// * `tutorial_id` - Unique identifier for the tutorial.
    /// * `title` - Human-readable title of the tutorial.
    /// * `description` - Short summary of what the tutorial covers.
    /// * `difficulty` - Difficulty level (e.g., beginner, intermediate, advanced).
    /// * `estimated_time` - Estimated completion time in minutes.
    /// * `steps` - Ordered list of tutorial steps.
    /// * `prerequisites` - List of tutorial or document IDs that should be completed first.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if a tutorial with `tutorial_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.create_tutorial(&author, &tutorial_id, &title, &desc, &difficulty, &30u32, &steps, &prereqs);
    /// ```
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

    /// Record the completion of a tutorial by the calling user.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the user completing the tutorial; must sign the transaction.
    /// * `tutorial_id` - Identifier of the completed tutorial.
    ///
    /// # Errors
    /// Returns [`DocumentationError::DocumentNotFound`] if `tutorial_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.complete_tutorial(&user, &tutorial_id);
    /// ```
    pub fn complete_tutorial(env: Env, user: Address, tutorial_id: String) -> Result<(), Error> {
        user.require_auth();
        TutorialManager::complete_tutorial(&env, tutorial_id)
    }

    /// Retrieve a tutorial by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `tutorial_id` - Identifier of the tutorial to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let tutorial = client.get_tutorial(&tutorial_id);
    /// ```
    pub fn get_tutorial(env: Env, tutorial_id: String) -> Option<Tutorial> {
        TutorialManager::get_tutorial(&env, &tutorial_id)
    }

    // ========================================================================
    // Community Contributions
    // ========================================================================

    /// Submit a community contribution (edit, addition, or correction) for a document.
    ///
    /// Requires authorization from `contributor`. Contributions start in a pending review state.
    ///
    /// # Arguments
    /// * `contributor` - Address of the contributor; must sign the transaction.
    /// * `contribution_id` - Unique identifier for the contribution.
    /// * `doc_id` - Identifier of the document being contributed to.
    /// * `contribution_type` - Type of contribution (e.g., edit, correction, addition).
    /// * `content` - The contributed content.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if a contribution with `contribution_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.submit_contribution(&contributor, &contribution_id, &doc_id, &contribution_type, &content);
    /// ```
    pub fn submit_contribution(
        env: Env,
        contributor: Address,
        contribution_id: String,
        doc_id: String,
        contribution_type: ContributionType,
        content: String,
    ) -> Result<Contribution, Error> {
        contributor.require_auth();
        let doc_cfg: DocumentationConfig =
            env.storage().persistent().get(&DataKey::Config).ok_or(Error::NotInitialized)?;
        check_rate_limit_doc(
            &env,
            &contributor,
            RL_OP_CONTRIBUTION,
            doc_cfg.rate_limit_contribution,
            doc_cfg.rate_limit_window,
        )?;
        ContributionManager::submit_contribution(
            &env,
            contribution_id,
            &contributor,
            doc_id,
            contribution_type,
            content,
        )
    }

    /// Review and update the status of a pending community contribution.
    ///
    /// Requires authorization from `reviewer`.
    ///
    /// # Arguments
    /// * `reviewer` - Address of the reviewer; must sign the transaction.
    /// * `contribution_id` - Identifier of the contribution to review.
    /// * `status` - New status to apply (e.g., approved, rejected).
    /// * `notes` - Optional reviewer notes explaining the decision.
    ///
    /// # Errors
    /// Returns [`DocumentationError::ContributionNotFound`] if `contribution_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.review_contribution(&reviewer, &contribution_id, &ContributionStatus::Approved, &None);
    /// ```
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

    /// Retrieve a community contribution by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `contribution_id` - Identifier of the contribution to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let contribution = client.get_contribution(&contribution_id);
    /// ```
    pub fn get_contribution(env: Env, contribution_id: String) -> Option<Contribution> {
        ContributionManager::get_contribution(&env, &contribution_id)
    }

    // ========================================================================
    // Translations
    // ========================================================================

    /// Create a translation of an existing document into another language.
    ///
    /// Requires authorization from `translator`.
    ///
    /// # Arguments
    /// * `translator` - Address of the translator; must sign the transaction.
    /// * `translation_id` - Unique identifier for the translation.
    /// * `original_doc_id` - Identifier of the source document being translated.
    /// * `language` - Target language code (e.g., `"es"`, `"fr"`).
    /// * `title` - Translated title.
    /// * `content` - Translated body content.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if a translation with `translation_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.create_translation(&translator, &translation_id, &doc_id, &"es", &title, &content);
    /// ```
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

    /// Update the review status of a translation (e.g., approve or reject it).
    ///
    /// Requires authorization from `admin`.
    ///
    /// # Arguments
    /// * `admin` - Admin address; must sign the transaction.
    /// * `translation_id` - Identifier of the translation to update.
    /// * `status` - New translation status to apply.
    ///
    /// # Errors
    /// Returns [`DocumentationError::TranslationNotFound`] if `translation_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.update_translation_status(&admin, &translation_id, &TranslationStatus::Approved);
    /// ```
    pub fn update_translation_status(
        env: Env,
        admin: Address,
        translation_id: String,
        status: TranslationStatus,
    ) -> Result<(), Error> {
        admin.require_auth();
        TranslationManager::update_translation_status(&env, translation_id, status)
    }

    /// Retrieve a translation by its identifier, returning `None` if not found.
    ///
    /// # Arguments
    /// * `translation_id` - Identifier of the translation to fetch.
    ///
    /// # Example
    /// ```ignore
    /// let translation = client.get_translation(&translation_id);
    /// ```
    pub fn get_translation(env: Env, translation_id: String) -> Option<Translation> {
        TranslationManager::get_translation(&env, &translation_id)
    }

    // ========================================================================
    // Analytics
    // ========================================================================

    /// Record a search query event for analytics purposes.
    ///
    /// Requires authorization from `user`.
    ///
    /// # Arguments
    /// * `user` - Address of the searching user; must sign the transaction.
    /// * `query_id` - Unique identifier for this search event.
    /// * `query_text` - The search query string entered by the user.
    /// * `results_count` - Number of results returned for this query.
    ///
    /// # Errors
    /// Returns [`DocumentationError::AlreadyExists`] if a query with `query_id` already exists.
    ///
    /// # Example
    /// ```ignore
    /// client.track_search(&user, &query_id, &query_text, &10u32);
    /// ```
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

    /// Retrieve aggregated analytics for a specific document, returning `None` if not found.
    ///
    /// # Arguments
    /// * `doc_id` - Identifier of the document to fetch analytics for.
    ///
    /// # Example
    /// ```ignore
    /// let analytics = client.get_document_analytics(&doc_id);
    /// ```
    pub fn get_document_analytics(env: Env, doc_id: String) -> Option<DocumentAnalytics> {
        AnalyticsManager::get_document_analytics(&env, &doc_id)
    }

    /// Return the total number of documents ever created in this contract.
    ///
    /// # Example
    /// ```ignore
    /// let count = client.get_total_documents();
    /// ```
    pub fn get_total_documents(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalDocuments)
    }

    /// Return the total number of document views recorded across all documents.
    ///
    /// # Example
    /// ```ignore
    /// let views = client.get_total_views();
    /// ```
    pub fn get_total_views(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalViews)
    }

    /// Return the total number of community contributions ever submitted.
    ///
    /// # Example
    /// ```ignore
    /// let count = client.get_total_contributions();
    /// ```
    pub fn get_total_contributions(env: Env) -> u64 {
        Storage::get_counter(&env, &DataKey::TotalContributions)
    }
}
