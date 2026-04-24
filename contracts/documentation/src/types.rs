#![allow(clippy::enum_variant_names)]
use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

// ============================================================================
// Core Documentation Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub doc_type: DocumentType,
    pub category: String,
    pub author: Address,
    pub version: u32,
    pub status: DocumentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
    pub language: String,
    pub parent_id: Option<String>,
    pub view_count: u32,
    pub helpful_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentType {
    Guide,
    Tutorial,
    ApiDoc,
    Reference,
    FAQ,
    Article,
    Video,
    CodeExample,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentStatus {
    Draft,
    Review,
    Published,
    Archived,
    Deprecated,
}

// ============================================================================
// Version Management
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentVersion {
    pub version_id: String,
    pub doc_id: String,
    pub version_number: u32,
    pub content: String,
    pub author: Address,
    pub created_at: u64,
    pub changelog: String,
    pub is_current: bool,
}

// ============================================================================
// Knowledge Base Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgeArticle {
    pub article_id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author: Address,
    pub created_at: u64,
    pub updated_at: u64,
    pub view_count: u32,
    pub helpful_votes: u32,
    pub not_helpful_votes: u32,
    pub related_articles: Vec<String>,
    pub tags: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FAQ {
    pub faq_id: String,
    pub question: String,
    pub answer: String,
    pub category: String,
    pub author: Address,
    pub created_at: u64,
    pub view_count: u32,
    pub helpful_count: u32,
    pub order_index: u32,
}

// ============================================================================
// API Documentation Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiEndpoint {
    pub endpoint_id: String,
    pub name: String,
    pub description: String,
    pub method: String,
    pub path: String,
    pub parameters: Vec<ApiParameter>,
    pub response_schema: String,
    pub code_examples: Vec<CodeExample>,
    pub version: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeExample {
    pub example_id: String,
    pub title: String,
    pub code: String,
    pub language: String,
    pub description: String,
}

// ============================================================================
// Tutorial Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tutorial {
    pub tutorial_id: String,
    pub title: String,
    pub description: String,
    pub difficulty: DifficultyLevel,
    pub estimated_time: u32,
    pub author: Address,
    pub steps: Vec<TutorialStep>,
    pub prerequisites: Vec<String>,
    pub created_at: u64,
    pub completion_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TutorialStep {
    pub step_number: u32,
    pub title: String,
    pub content: String,
    pub code_snippet: Option<String>,
    pub validation_criteria: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

// ============================================================================
// Community Contribution Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub contribution_id: String,
    pub contributor: Address,
    pub doc_id: String,
    pub contribution_type: ContributionType,
    pub content: String,
    pub status: ContributionStatus,
    pub created_at: u64,
    pub reviewed_by: Option<Address>,
    pub review_notes: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionType {
    NewDocument,
    Edit,
    Translation,
    CodeExample,
    Correction,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionStatus {
    Pending,
    Approved,
    Rejected,
    NeedsRevision,
}

// ============================================================================
// Analytics Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentAnalytics {
    pub doc_id: String,
    pub total_views: u32,
    pub unique_viewers: u32,
    pub avg_time_spent: u32,
    pub helpful_votes: u32,
    pub not_helpful_votes: u32,
    pub completion_rate: u32,
    pub search_appearances: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    pub query_id: String,
    pub query_text: String,
    pub user: Address,
    pub timestamp: u64,
    pub results_count: u32,
    pub clicked_result: Option<String>,
}

// ============================================================================
// Translation Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Translation {
    pub translation_id: String,
    pub original_doc_id: String,
    pub language: String,
    pub title: String,
    pub content: String,
    pub translator: Address,
    pub status: TranslationStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslationStatus {
    InProgress,
    Review,
    Published,
    Outdated,
}

// ============================================================================
// Configuration
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentationConfig {
    pub admin: Address,
    pub moderators: Vec<Address>,
    pub supported_languages: Vec<String>,
    pub max_doc_size: u32,
    pub require_review: bool,
    pub enable_contributions: bool,
    pub enable_analytics: bool,
}

// ============================================================================
// Storage Keys
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Config,
    Initialized,
    Document(String),
    DocumentVersion(String, u32),
    KnowledgeArticle(String),
    FAQ(String),
    ApiEndpoint(String),
    Tutorial(String),
    Contribution(String),
    Translation(String),
    Analytics(String),
    SearchQuery(String),
    CategoryDocs(String),
    UserContributions(Address),
    DocumentsByAuthor(Address),
    TotalDocuments,
    TotalViews,
    TotalContributions,
}

// ============================================================================
// Errors
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    DocumentNotFound = 4,
    InvalidDocument = 5,
    VersionNotFound = 6,
    ContributionNotFound = 7,
    InvalidContribution = 8,
    TranslationNotFound = 9,
    InvalidLanguage = 10,
    DocumentTooLarge = 11,
    InvalidStatus = 12,
    AlreadyExists = 13,
}
