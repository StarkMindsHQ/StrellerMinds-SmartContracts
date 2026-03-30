#![allow(clippy::enum_variant_names)]
use shared::config::{ContractConfig, DeploymentEnv};
use soroban_sdk::{contracttype, Address, Env, String, Vec};

// ============================================================================
// Core Documentation Types
// ============================================================================

/// A documentation document stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    /// Unique document identifier.
    pub doc_id: String,
    /// Document title.
    pub title: String,
    /// Document content body.
    pub content: String,
    /// Classification of the document type.
    pub doc_type: DocumentType,
    /// Category the document belongs to.
    pub category: String,
    /// Address of the document author.
    pub author: Address,
    /// Current version number.
    pub version: u32,
    /// Publication lifecycle status.
    pub status: DocumentStatus,
    /// Unix timestamp when the document was created.
    pub created_at: u64,
    /// Unix timestamp when the document was last updated.
    pub updated_at: u64,
    /// Searchable tags associated with the document.
    pub tags: Vec<String>,
    /// Language code of the document content.
    pub language: String,
    /// Optional identifier of the parent document.
    pub parent_id: Option<String>,
    /// Total number of times the document has been viewed.
    pub view_count: u32,
    /// Number of helpful votes received.
    pub helpful_count: u32,
}

/// Classification of a documentation document.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentType {
    /// Step-by-step guide.
    Guide,
    /// Hands-on tutorial.
    Tutorial,
    /// API reference documentation.
    ApiDoc,
    /// Reference material.
    Reference,
    /// Frequently asked questions entry.
    FAQ,
    /// General article.
    Article,
    /// Video content.
    Video,
    /// Runnable code example.
    CodeExample,
}

/// Publication lifecycle status of a document.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentStatus {
    /// Work-in-progress, not yet ready for review.
    Draft,
    /// Under review before publication.
    Review,
    /// Publicly visible and active.
    Published,
    /// Removed from active use but retained for reference.
    Archived,
    /// Outdated and superseded by newer content.
    Deprecated,
}

// ============================================================================
// Version Management
// ============================================================================

/// A specific historical version of a document.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentVersion {
    /// Unique identifier for this version entry.
    pub version_id: String,
    /// Identifier of the parent document.
    pub doc_id: String,
    /// Monotonically increasing version number.
    pub version_number: u32,
    /// Full content of this version.
    pub content: String,
    /// Address of the author who created this version.
    pub author: Address,
    /// Unix timestamp when this version was created.
    pub created_at: u64,
    /// Human-readable description of changes in this version.
    pub changelog: String,
    /// Whether this is the currently active version.
    pub is_current: bool,
}

// ============================================================================
// Knowledge Base Types
// ============================================================================

/// A knowledge base article providing in-depth information on a topic.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeArticle {
    /// Unique article identifier.
    pub article_id: String,
    /// Article title.
    pub title: String,
    /// Full article content.
    pub content: String,
    /// Category the article belongs to.
    pub category: String,
    /// Address of the article author.
    pub author: Address,
    /// Unix timestamp when the article was created.
    pub created_at: u64,
    /// Unix timestamp when the article was last updated.
    pub updated_at: u64,
    /// Total number of views.
    pub view_count: u32,
    /// Number of helpful votes.
    pub helpful_votes: u32,
    /// Number of not-helpful votes.
    pub not_helpful_votes: u32,
    /// IDs of related articles.
    pub related_articles: Vec<String>,
    /// Searchable tags.
    pub tags: Vec<String>,
}

/// A frequently asked question with its answer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FAQ {
    /// Unique FAQ identifier.
    pub faq_id: String,
    /// The question text.
    pub question: String,
    /// The answer text.
    pub answer: String,
    /// Category this FAQ belongs to.
    pub category: String,
    /// Address of the FAQ author.
    pub author: Address,
    /// Unix timestamp when the FAQ was created.
    pub created_at: u64,
    /// Total number of views.
    pub view_count: u32,
    /// Number of helpful votes received.
    pub helpful_count: u32,
    /// Display ordering index within its category.
    pub order_index: u32,
}

// ============================================================================
// API Documentation Types
// ============================================================================

/// Documentation for a single API endpoint.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiEndpoint {
    /// Unique endpoint identifier.
    pub endpoint_id: String,
    /// Short name for the endpoint.
    pub name: String,
    /// Description of what the endpoint does.
    pub description: String,
    /// HTTP method (e.g. GET, POST).
    pub method: String,
    /// URL path of the endpoint.
    pub path: String,
    /// List of accepted parameters.
    pub parameters: Vec<ApiParameter>,
    /// JSON schema of the response.
    pub response_schema: String,
    /// Code examples demonstrating usage.
    pub code_examples: Vec<CodeExample>,
    /// API version this endpoint belongs to.
    pub version: String,
}

/// A single parameter accepted by an API endpoint.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiParameter {
    /// Parameter name.
    pub name: String,
    /// Data type of the parameter.
    pub param_type: String,
    /// Whether the parameter must be supplied.
    pub required: bool,
    /// Description of the parameter's purpose.
    pub description: String,
    /// Optional default value used when the parameter is omitted.
    pub default_value: Option<String>,
}

/// A runnable code example for documentation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeExample {
    /// Unique identifier for this example.
    pub example_id: String,
    /// Short descriptive title.
    pub title: String,
    /// The source code snippet.
    pub code: String,
    /// Programming language of the snippet (e.g. Rust, JavaScript).
    pub language: String,
    /// Explanation of what the example demonstrates.
    pub description: String,
}

// ============================================================================
// Tutorial Types
// ============================================================================

/// A structured tutorial with ordered steps.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tutorial {
    /// Unique tutorial identifier.
    pub tutorial_id: String,
    /// Tutorial title.
    pub title: String,
    /// Short description of what the tutorial covers.
    pub description: String,
    /// Skill level required to follow the tutorial.
    pub difficulty: DifficultyLevel,
    /// Estimated time to complete in minutes.
    pub estimated_time: u32,
    /// Address of the tutorial author.
    pub author: Address,
    /// Ordered list of tutorial steps.
    pub steps: Vec<TutorialStep>,
    /// List of prerequisite topics or tutorial IDs.
    pub prerequisites: Vec<String>,
    /// Unix timestamp when the tutorial was created.
    pub created_at: u64,
    /// Number of times the tutorial has been completed.
    pub completion_count: u32,
}

/// A single step within a tutorial.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialStep {
    /// 1-based position of this step in the tutorial.
    pub step_number: u32,
    /// Step title.
    pub title: String,
    /// Instructional content for this step.
    pub content: String,
    /// Optional code snippet associated with the step.
    pub code_snippet: Option<String>,
    /// Optional criteria used to validate step completion.
    pub validation_criteria: Option<String>,
}

/// Skill difficulty level for tutorials and content.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DifficultyLevel {
    /// Suitable for newcomers with no prior experience.
    Beginner,
    /// Requires some foundational knowledge.
    Intermediate,
    /// Requires solid experience in the domain.
    Advanced,
    /// Requires deep expertise.
    Expert,
}

// ============================================================================
// Community Contribution Types
// ============================================================================

/// A community-submitted contribution to the documentation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    /// Unique contribution identifier.
    pub contribution_id: String,
    /// Address of the contributor.
    pub contributor: Address,
    /// Identifier of the document being contributed to.
    pub doc_id: String,
    /// Nature of the contribution.
    pub contribution_type: ContributionType,
    /// Contributed content.
    pub content: String,
    /// Current review status.
    pub status: ContributionStatus,
    /// Unix timestamp when the contribution was submitted.
    pub created_at: u64,
    /// Address of the reviewer, if already reviewed.
    pub reviewed_by: Option<Address>,
    /// Optional notes left by the reviewer.
    pub review_notes: Option<String>,
}

/// Nature of a community contribution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionType {
    /// Adding a brand new document.
    NewDocument,
    /// Editing existing content.
    Edit,
    /// Providing a translation.
    Translation,
    /// Adding a code example.
    CodeExample,
    /// Fixing an error in existing content.
    Correction,
}

/// Review lifecycle status of a contribution.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionStatus {
    /// Awaiting review.
    Pending,
    /// Accepted and merged.
    Approved,
    /// Declined by reviewers.
    Rejected,
    /// Returned to the contributor for changes.
    NeedsRevision,
}

// ============================================================================
// Analytics Types
// ============================================================================

/// Aggregated engagement analytics for a single document.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentAnalytics {
    /// Identifier of the document being tracked.
    pub doc_id: String,
    /// Total number of views.
    pub total_views: u32,
    /// Number of distinct viewers.
    pub unique_viewers: u32,
    /// Average time spent reading, in seconds.
    pub avg_time_spent: u32,
    /// Number of helpful votes.
    pub helpful_votes: u32,
    /// Number of not-helpful votes.
    pub not_helpful_votes: u32,
    /// Percentage of readers who reached the end.
    pub completion_rate: u32,
    /// Number of times the document appeared in search results.
    pub search_appearances: u32,
}

/// A recorded search query issued by a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    /// Unique query identifier.
    pub query_id: String,
    /// The raw text typed by the user.
    pub query_text: String,
    /// Address of the user who issued the query.
    pub user: Address,
    /// Unix timestamp when the query was executed.
    pub timestamp: u64,
    /// Number of results returned.
    pub results_count: u32,
    /// Optional ID of the result the user clicked.
    pub clicked_result: Option<String>,
}

// ============================================================================
// Translation Types
// ============================================================================

/// A translated version of a document in a target language.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Translation {
    /// Unique translation identifier.
    pub translation_id: String,
    /// Identifier of the source document being translated.
    pub original_doc_id: String,
    /// Language code of this translation (e.g. "es", "fr").
    pub language: String,
    /// Translated title.
    pub title: String,
    /// Translated content body.
    pub content: String,
    /// Address of the translator.
    pub translator: Address,
    /// Current status in the translation lifecycle.
    pub status: TranslationStatus,
    /// Unix timestamp when the translation was created.
    pub created_at: u64,
}

/// Lifecycle status of a document translation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslationStatus {
    /// Translation is being worked on.
    InProgress,
    /// Awaiting review before publication.
    Review,
    /// Publicly visible and active.
    Published,
    /// Source document has changed; translation needs updating.
    Outdated,
}

// ============================================================================
// Configuration
// ============================================================================

/// Contract-level configuration for the documentation system.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationConfig {
    /// Address with administrative privileges.
    pub admin: Address,
    /// Addresses permitted to review and approve contributions.
    pub moderators: Vec<Address>,
    /// Language codes accepted for translations.
    pub supported_languages: Vec<String>,
    /// Maximum allowed document size in bytes.
    pub max_doc_size: u32,
    /// Whether all new documents must go through review before publishing.
    pub require_review: bool,
    /// Whether community contributions are accepted.
    pub enable_contributions: bool,
    /// Whether view and engagement analytics are collected.
    pub enable_analytics: bool,
    // Rate limits (max calls per rate_limit_window)
    pub rate_limit_create_doc: u32,
    pub rate_limit_contribution: u32,
    pub rate_limit_window: u64,
}

impl DocumentationConfig {
    pub fn for_env(env: &Env, admin: Address, profile: DeploymentEnv) -> Self {
        let defaults = ContractConfig::documentation(profile);
        Self {
            admin,
            moderators: Vec::new(env),
            supported_languages: Vec::new(env),
            max_doc_size: defaults.max_doc_size,
            require_review: defaults.require_review,
            enable_contributions: defaults.enable_contributions,
            enable_analytics: defaults.enable_analytics,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.max_doc_size == 0 {
            return Err(Error::InvalidDocument);
        }
        Ok(())
    }
}

// ============================================================================
// Storage Keys
// ============================================================================

/// Storage keys for the documentation contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract administrator address.
    Admin,
    /// Global documentation configuration.
    Config,
    /// Contract initialization flag.
    Initialized,
    /// A document by its ID.
    Document(String),
    /// A specific version of a document by (doc_id, version_number).
    DocumentVersion(String, u32),
    /// A knowledge base article by its ID.
    KnowledgeArticle(String),
    /// A FAQ entry by its ID.
    FAQ(String),
    /// An API endpoint documentation entry by its ID.
    ApiEndpoint(String),
    /// A tutorial by its ID.
    Tutorial(String),
    /// A community contribution by its ID.
    Contribution(String),
    /// A translation by its ID.
    Translation(String),
    /// Analytics data for a document by doc_id.
    Analytics(String),
    /// A recorded search query by its ID.
    SearchQuery(String),
    /// List of document IDs belonging to a category.
    CategoryDocs(String),
    /// List of contribution IDs submitted by a user.
    UserContributions(Address),
    /// List of document IDs authored by a user.
    DocumentsByAuthor(Address),
    /// Running total of documents in the system.
    TotalDocuments,
    /// Running total of document views across the system.
    TotalViews,
    /// Running total of contributions submitted.
    TotalContributions,
    RateLimit(Address, u64), // (user, operation_id) -> RateLimitState
}

// ============================================================================
// Errors
// ============================================================================

pub use crate::errors::{DocumentationError, Error};
