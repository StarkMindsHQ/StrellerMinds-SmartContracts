use soroban_sdk::{contracttype, Address, BytesN, Map, String, Vec};

// ============================================================================
// CORE ENUM TYPES (must be defined before Maybe wrappers)
// ============================================================================

/// Available sorting fields
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    Relevance,
    Title,
    CreatedDate,
    UpdatedDate,
    Rating,
    Popularity,
    Duration,
    Difficulty,
    Price,
    CompletionRate,
    IssueDate,
    ExpiryDate,
    Progress,
}

/// Frequency for auto-executing saved searches
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionFrequency {
    Daily,
    Weekly,
    Monthly,
    Custom(u64),
}

// ============================================================================
// OPTIONAL WRAPPER TYPES (Soroban doesn't support Option<T> in contracttype)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeDurationRange {
    None,
    Some(DurationRange),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybePriceRange {
    None,
    Some(PriceRange),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeRatingRange {
    None,
    Some(RatingRange),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeDateRange {
    None,
    Some(DateRange),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeCompletionRange {
    None,
    Some(CompletionRange),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeBool {
    None,
    Some(bool),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeSortField {
    None,
    Some(SortField),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeExecutionFrequency {
    None,
    Some(ExecutionFrequency),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeU64 {
    None,
    Some(u64),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeString {
    None,
    Some(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybeAddress {
    None,
    Some(Address),
}

// ============================================================================
// CORE TYPES
// ============================================================================

/// Difficulty level enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Duration range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurationRange {
    pub min_hours: u32, // 0 means no minimum
    pub max_hours: u32, // 0 means no maximum
}

/// Price range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceRange {
    pub min_price: i64, // In stroops, i64::MIN means no minimum
    pub max_price: i64, // In stroops, i64::MAX means no maximum
}

/// Rating range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatingRange {
    pub min_rating: u32, // 1-5 stars (scaled to 1-50 for precision)
    pub max_rating: u32, // 1-5 stars (scaled to 1-50 for precision)
}

/// Date range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateRange {
    pub start_date: u64, // Unix timestamp, 0 means no start date
    pub end_date: u64,   // Unix timestamp, 0 means no end date
}

/// Completion range for progress filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionRange {
    pub min_percentage: u32, // 0-100
    pub max_percentage: u32, // 0-100
}

/// Certificate status for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked,
    Expired,
    PendingRenewal,
    Renewed,
}

/// Certificate type classification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateType {
    Completion,
    Achievement,
    Professional,
    Accredited,
    Micro,
}

/// Comprehensive filtering options
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFilters {
    // Course filters
    pub categories: Vec<String>,                 // Course categories
    pub difficulty_levels: Vec<DifficultyLevel>, // Difficulty filtering
    pub duration_range: MaybeDurationRange,      // Duration filtering
    pub instructor_ids: Vec<Address>,            // Filter by instructors
    pub languages: Vec<String>,                  // Course languages
    pub price_range: MaybePriceRange,            // Price filtering
    pub rating_range: MaybeRatingRange,          // Rating filtering
    pub tags: Vec<String>,                       // Course tags

    // Certificate filters
    pub certificate_status: Vec<CertificateStatus>, // Certificate status
    pub issue_date_range: MaybeDateRange,           // Issue date filtering
    pub expiry_date_range: MaybeDateRange,          // Expiry date filtering
    pub certificate_types: Vec<CertificateType>,    // Certificate types

    // Progress filters
    pub completion_range: MaybeCompletionRange, // Progress completion
    pub enrollment_date_range: MaybeDateRange,  // Enrollment filtering
    pub last_activity_range: MaybeDateRange,    // Last activity filtering

    // Advanced filters
    pub has_prerequisites: MaybeBool, // Filter by prerequisite requirements
    pub has_certificate: MaybeBool,   // Filter by certificate availability
    pub is_premium: MaybeBool,        // Premium content filter
    pub is_featured: MaybeBool,       // Featured content filter
}

/// Search query structure for multi-criteria searches
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    pub query_text: String,            // Text search query
    pub filters: SearchFilters,        // Applied filters
    pub sort_options: SortOptions,     // Sorting preferences
    pub pagination: PaginationOptions, // Pagination settings
    pub search_scope: SearchScope,     // What to search (courses, certificates, etc.)
}

/// Sorting options for search results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortOptions {
    pub primary_sort: SortField,
    pub secondary_sort: MaybeSortField,
    pub sort_order: SortOrder,
}

/// Sort order enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Pagination options
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginationOptions {
    pub page: u32,        // Page number (1-based)
    pub page_size: u32,   // Results per page
    pub max_results: u32, // Maximum total results to return
}

/// Search scope definition
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchScope {
    Courses,
    Certificates,
    UserProgress,
    All,
    Custom(Vec<SearchTarget>),
}

/// Individual search targets
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchTarget {
    Courses,
    Certificates,
    UserProgress,
    Instructors,
    Categories,
    Tags,
}

/// Search result container
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResults {
    pub query_id: String,                // Unique query identifier
    pub total_results: u32,              // Total matching results
    pub page: u32,                       // Current page
    pub page_size: u32,                  // Results per page
    pub has_more: bool,                  // Whether more results exist
    pub results: Vec<SearchResultItem>,  // Actual results
    pub facets: Vec<SearchFacet>,        // Faceted search results
    pub suggestions: Vec<String>,        // Search suggestions
    pub execution_time_ms: u32,          // Query execution time
    pub search_metadata: SearchMetadata, // Additional metadata
}

/// Simple search result (for AI functions)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub content_id: String,
    pub score: u32,
    pub rank: u32,
}

/// Simple search filter (for AI functions)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFilter {
    pub field: String,
    pub value: String,
    pub operator: FilterOperator,
}

/// Filter operators
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilterOperator {
    Equal,
    GreaterThan,
    LessThan,
    Contains,
    In,
}

/// Individual search result item
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResultItem {
    pub item_id: String,                  // Unique item identifier
    pub item_type: SearchResultType,      // Type of result
    pub title: String,                    // Item title
    pub description: String,              // Item description
    pub relevance_score: u32,             // Relevance score (0-1000)
    pub metadata: SearchResultMetadata,   // Type-specific metadata
    pub highlights: Vec<SearchHighlight>, // Text highlights
    pub thumbnail_url: Option<String>,    // Optional thumbnail
}

/// Search result types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchResultType {
    Course,
    Certificate,
    UserProgress,
    Instructor,
    Category,
    Tag,
}

/// Type-specific metadata for search results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchResultMetadata {
    Course(CourseMetadata),
    Certificate(CertificateMetadata),
    Progress(ProgressMetadata),
    Instructor(InstructorMetadata),
}

/// Course-specific metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CourseMetadata {
    pub course_id: String,
    pub instructor_id: Address,
    pub instructor_name: String,
    pub category: String,
    pub difficulty: DifficultyLevel,
    pub duration_hours: u32,
    pub price: i64,
    pub rating: u32, // 1-50 scale
    pub enrollment_count: u32,
    pub completion_rate: u32, // Percentage
    pub created_date: u64,
    pub updated_date: u64,
    pub tags: Vec<String>,
    pub language: String,
    pub has_certificate: bool,
    pub has_prerequisites: bool,
    pub is_premium: bool,
    pub is_featured: bool,
}

/// Certificate-specific metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateMetadata {
    pub certificate_id: BytesN<32>,
    pub course_id: String,
    pub student_id: Address,
    pub instructor_id: Address,
    pub certificate_type: CertificateType,
    pub status: CertificateStatus,
    pub issue_date: u64,
    pub expiry_date: u64,
    pub completion_percentage: u32,
    pub grade: Option<String>,
    pub verification_url: Option<String>,
}

/// Progress-specific metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressMetadata {
    pub student_id: Address,
    pub course_id: String,
    pub completion_percentage: u32,
    pub modules_completed: u32,
    pub total_modules: u32,
    pub last_activity_date: u64,
    pub enrollment_date: u64,
    pub estimated_completion_date: Option<u64>,
    pub time_spent_minutes: u32,
    pub current_module: Option<String>,
}

/// Instructor-specific metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructorMetadata {
    pub instructor_id: Address,
    pub name: String,
    pub bio: String,
    pub rating: u32,
    pub course_count: u32,
    pub student_count: u32,
    pub specializations: Vec<String>,
    pub verified: bool,
}

/// Search text highlights
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchHighlight {
    pub field: String,                       // Field name that matched
    pub original_text: String,               // Original text
    pub highlighted_text: String,            // Text with highlights
    pub match_positions: Vec<MatchPosition>, // Position of matches
}

/// Match position in text
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchPosition {
    pub start: u32,
    pub end: u32,
    pub match_type: MatchType,
}

/// Type of text match
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchType {
    Exact,
    Partial,
    Fuzzy,
    Synonym,
}

/// Faceted search results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFacet {
    pub facet_name: String,
    pub facet_values: Vec<FacetValue>,
}

/// Individual facet value with count
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacetValue {
    pub value: String,
    pub count: u32,
    pub selected: bool,
}

/// Search execution metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchMetadata {
    pub query_timestamp: u64,
    pub index_version: String,
    pub search_engine_version: String,
    pub cache_hit: bool,
    pub total_indexed_items: u32,
    pub search_suggestions_enabled: bool,
}

/// Saved search preferences
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedSearch {
    pub search_id: String,
    pub user_id: Address,
    pub name: String,
    pub description: String,
    pub query: SearchQuery,
    pub created_at: u64,
    pub last_used: u64,
    pub use_count: u32,
    pub is_favorite: bool,
    pub notification_enabled: bool, // Notify when new results match
    pub auto_execute: bool,         // Auto-execute periodically
    pub execution_frequency: MaybeExecutionFrequency,
}

/// Search preferences for users
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchPreferences {
    pub user_id: Address,
    pub default_page_size: u32,
    pub default_sort: SortField,
    pub default_sort_order: SortOrder,
    pub preferred_categories: Vec<String>,
    pub preferred_languages: Vec<String>,
    pub preferred_difficulty: Vec<DifficultyLevel>,
    pub enable_suggestions: bool,
    pub enable_auto_complete: bool,
    pub enable_faceted_search: bool,
    pub search_history_enabled: bool,
    pub max_search_history: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Search history entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchHistoryEntry {
    pub search_id: String,
    pub user_id: Address,
    pub query: SearchQuery,
    pub results_count: u32,
    pub clicked_results: Vec<String>, // IDs of results user clicked
    pub search_timestamp: u64,
    pub session_id: Option<String>,
    pub search_duration_ms: u32,
}

/// Search analytics data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchAnalytics {
    pub total_searches: u32,
    pub unique_users: u32,
    pub average_results_per_search: u32,
    pub most_popular_queries: Vec<PopularQuery>,
    pub most_clicked_results: Vec<PopularResult>,
    pub search_performance_metrics: PerformanceMetrics,
    pub period_start: u64,
    pub period_end: u64,
}

/// Popular search query
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopularQuery {
    pub query_text: String,
    pub search_count: u32,
    pub unique_users: u32,
    pub average_results: u32,
    pub click_through_rate: u32, // Percentage
}

/// Popular search result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopularResult {
    pub item_id: String,
    pub item_type: SearchResultType,
    pub title: String,
    pub click_count: u32,
    pub unique_users: u32,
    pub average_position: u32, // Average position in search results
}

/// Search performance metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    pub average_query_time_ms: u32,
    pub cache_hit_rate: u32, // Percentage
    pub index_size_mb: u32,
    pub total_indexed_items: u32,
    pub search_success_rate: u32, // Percentage of searches with results
}

/// Search index configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchIndexConfig {
    pub index_name: String,
    pub indexed_fields: Vec<IndexedField>,
    pub search_weights: SearchWeights,
    pub update_frequency: IndexUpdateFrequency,
    pub max_index_size: u32,
    pub enable_fuzzy_search: bool,
    pub enable_synonym_search: bool,
    pub enable_autocomplete: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Indexed field configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedField {
    pub field_name: String,
    pub field_type: IndexFieldType,
    pub weight: u32, // Search weight (1-10)
    pub searchable: bool,
    pub facetable: bool,
    pub sortable: bool,
    pub highlight: bool,
}

/// Index field types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexFieldType {
    Text,
    Keyword,
    Number,
    Date,
    Boolean,
    Array,
}

/// Search weights for different content types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchWeights {
    pub title_weight: u32,
    pub description_weight: u32,
    pub content_weight: u32,
    pub tags_weight: u32,
    pub category_weight: u32,
    pub instructor_weight: u32,
    pub metadata_weight: u32,
}

/// Index update frequency
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexUpdateFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Manual,
}

/// Search suggestion configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchSuggestion {
    pub suggestion_text: String,
    pub suggestion_type: SuggestionType,
    pub popularity_score: u32,
    pub category: Option<String>,
    pub metadata: Option<String>,
}

/// Types of search suggestions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuggestionType {
    Query,      // Query completion
    Course,     // Course suggestion
    Category,   // Category suggestion
    Instructor, // Instructor suggestion
    Tag,        // Tag suggestion
    Correction, // Spelling correction
}

/// Storage keys for the search contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract admin
    Admin,
    /// Contract initialization flag
    Initialized,
    /// Search index configuration
    IndexConfig(String),
    /// Saved searches by user
    SavedSearches(Address),
    /// User search preferences
    SearchPreferences(Address),
    /// Search history by user
    SearchHistory(Address),
    /// Search analytics data
    SearchAnalytics(u64), // Time period
    /// Search suggestions
    SearchSuggestions(String), // Category
    /// Search cache
    SearchCache(String), // Query hash
    /// Popular queries
    PopularQueries(u64), // Time period
    /// Search performance metrics
    PerformanceMetrics(u64), // Time period
    /// Index metadata
    IndexMetadata(String),
    /// Search weights configuration
    SearchWeights,
    /// Auto-complete data
    AutoCompleteData(String), // Prefix
    // AI-Powered Search Keys
    /// Semantic metadata for content
    SemanticMetadata(String), // Content ID
    /// User learning profile
    UserProfile(Address),
    /// Recommendation scores
    RecommendationScores(Address),
    /// Content analysis cache
    ContentAnalysis(String), // Content ID
    /// Similarity scores
    SimilarityScores(String), // Item ID
    /// Visual metadata
    VisualMetadata(String), // Content ID
    /// Learning paths
    LearningPath(Address, String), // User + Path ID
    /// Skill dependencies
    SkillNode(String), // Skill name
    /// Ranking signals
    RankingSignals(String), // Content ID
    /// Multilingual content
    MultilingualContent(String), // Content ID
    /// Language preferences
    LanguagePreferences(Address),
    /// Conversation sessions
    ConversationSession(String), // Session ID
    /// User interactions
    UserInteractions(Address),
    /// Oracle authorized addresses
    AuthorizedOracles(Address), // Oracle address
}

// ============================================================================
// AI-POWERED SEARCH TYPES
// ============================================================================

/// Semantic search metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticMetadata {
    pub content_id: String,
    pub topics: Vec<String>,
    pub intent_scores: Map<String, u32>, // Intent type -> score (0-1000)
    pub semantic_tags: Vec<String>,
    pub entity_types: Vec<String>,
    pub complexity_score: u32, // 0-100
    pub last_updated: u64,
}

/// Processed query from NLP service
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedQuery {
    pub original_text: String,
    pub normalized_text: String,
    pub original_query: String,
    pub extracted_intent: String,
    pub intent: String,
    pub entities: Vec<Entity>,
    pub expanded_terms: Vec<String>,
    pub semantic_tags: Vec<String>,
    pub suggested_filters: Vec<String>,
    pub query_type: String,
    pub confidence: u32, // 0-1000
}

/// Extracted entity
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entity {
    pub entity_type: String, // "course", "topic", "skill", etc.
    pub value: String,
    pub confidence: u32, // 0-1000
}

/// User learning profile
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub user_address: Address,
    pub completed_courses: Vec<String>,
    pub skill_levels: Map<String, u32>, // Skill -> Level (0-100)
    pub interaction_counts: Map<String, u32>, // Category -> Count
    pub preference_scores: Vec<u32>,    // Simplified preference vector
    pub last_updated: u64,
}

/// Recommendation with score
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recommendation {
    pub content_id: String,
    pub content_type: String,
    pub score: u32, // 0-1000
    pub reason: String,
    pub confidence: u32, // 0-100
    pub computed_at: u64,
    pub expires_at: u64,
}

/// Content analysis result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentAnalysis {
    pub content_id: String,
    pub auto_generated_tags: Vec<String>,
    pub extracted_topics: Vec<Topic>,
    pub identified_skills: Vec<Skill>,
    pub difficulty_score: u32,   // 0-100
    pub quality_score: u32,      // 0-100
    pub readability_score: u32,  // 0-100
    pub estimated_duration: u32, // minutes
    pub prerequisite_skills: Vec<String>,
    pub learning_outcomes: Vec<String>,
    pub analysis_timestamp: u64,
}

/// Topic with relevance
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic {
    pub name: String,
    pub relevance_score: u32, // 0-1000
    pub category: String,
}

/// Skill requirement
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Skill {
    pub skill_name: String,
    pub required_level: u32, // 0-100
    pub importance: u32,     // 0-100
}

/// User interaction tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserInteraction {
    pub user_address: Address,
    pub content_id: String,
    pub interaction_type: InteractionType,
    pub timestamp: u64,
    pub value: u32, // Rating, time spent, etc.
}

/// Interaction types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionType {
    View,
    Click,
    Enroll,
    Complete,
    Rate,
    Share,
    Save,
    Like,
    Bookmark,
}

/// Similarity score between items
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimilarityScore {
    pub item_a: String,
    pub item_b: String,
    pub similarity: u32, // 0-1000
    pub method: String,  // "collaborative", "content-based", etc.
    pub computed_at: u64,
}

/// Visual metadata for images
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualMetadata {
    pub content_id: String,
    pub image_hash: BytesN<32>,
    pub dominant_colors: Vec<String>,  // Hex colors
    pub detected_objects: Vec<String>, // Object names
    pub visual_category: String,
    pub style_tags: Vec<String>,
    pub feature_vector_hash: BytesN<32>,
    pub similarity_cluster: u32,
    pub thumbnail_url: String,
    pub aspect_ratio: u32,  // Width/height percentage
    pub quality_score: u32, // 0-1000
}

/// Learning path
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningPath {
    pub path_id: String,
    pub user_address: Address,
    pub target_skill: String,
    pub courses: Vec<PathStep>,
    pub steps: Vec<PathStep>, // Alias for courses
    pub total_duration: u32,
    pub estimated_completion: u64,
    pub estimated_duration_days: u32,
    pub progress_percentage: u32,
    pub efficiency_score: u32, // 0-100
    pub created_at: u64,
    pub skill_nodes: Vec<SkillNode>, // For dependency tracking
}

/// Step in learning path
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathStep {
    pub content_id: String, // Course/content ID
    pub course_id: String,
    pub skill_id: String,
    pub sequence_number: u32,
    pub prerequisites_met: bool,
    pub estimated_duration: u32,
    pub estimated_effort: u32, // Difficulty score
    pub skills_gained: Vec<String>,
    pub importance_score: u32, // 0-100
    pub completed: bool,
}

/// Skill node in dependency graph
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillNode {
    pub skill_id: String,
    pub skill_name: String,
    pub prerequisites: Vec<String>,
    pub difficulty: u32,     // 0-100
    pub estimated_time: u32, // hours
    pub related_courses: Vec<String>,
}

/// Ranking signals for results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankingSignals {
    pub relevance_score: u32,       // 0-1000
    pub quality_score: u32,         // 0-1000
    pub engagement_score: u32,      // 0-1000
    pub recency_score: u32,         // 0-1000
    pub personalization_score: u32, // 0-1000
    pub authority_score: u32,       // 0-1000
    pub completion_rate: u32,       // 0-1000
    pub user_rating: u32,           // 0-1000
}

/// Ranking configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankingConfig {
    pub relevance_weight: u32,
    pub quality_weight: u32,
    pub engagement_weight: u32,
    pub recency_weight: u32,
    pub personalization_weight: u32,
    pub authority_weight: u32,
}

/// Ranked search result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedResult {
    pub content_id: String, // Content ID
    pub result: SearchResultItem,
    pub score: u32, // Final ranking score
    pub final_score: u32,
    pub signals: RankingSignals,
    pub rank_position: u32,
}

/// Supported languages
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Mandarin, // Chinese Mandarin
    Japanese,
    Arabic,
    Portuguese,
    Russian,
    Korean,
    Hindi, // Add Hindi
}

/// Multilingual content metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultilingualContent {
    pub content_id: String,
    pub primary_language: Language,
    pub available_languages: Vec<Language>,
    pub translations: Map<String, TranslationMeta>, // Lang -> Meta
}

/// Translation metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationMeta {
    pub target_language: Language,
    pub translated_title: String,
    pub translated_description: String,
    pub quality_score: u32,       // 0-1000
    pub translation_quality: u32, // 0-100
    pub last_updated: u64,
}

/// Language preferences
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguagePreferences {
    pub user_address: Address,
    pub preferred_language: Language,
    pub fallback_languages: Vec<Language>,
}

/// Conversation session for voice/chat
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationSession {
    pub session_id: String,
    pub user: Address, // User address
    pub start_time: u64,
    pub last_interaction_time: u64,
    pub queries: Vec<ProcessedVoiceQuery>, // Query history
    pub context_entities: Vec<String>,     // Context tracking
    pub is_active: bool,
}

/// Processed voice query
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessedVoiceQuery {
    pub session_id: String,
    pub transcribed_text: String,
    pub detected_intent: String,
    pub extracted_entities: Vec<Entity>,
    pub entities: Vec<String>, // Simple entity list
    pub confidence_score: u32, // 0-1000
    pub confidence: u32,       // 0-1000
    pub timestamp: u64,
    pub requires_context: bool,
}

/// Oracle submission for verification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSubmission {
    pub data_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub oracle_address: Address,
    pub timestamp: u64,
    pub nonce: u64,
}

/// Search event for analytics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchEvent {
    pub user: MaybeAddress,
    pub query: String,
    pub timestamp: u64,
    pub results_count: u32,
    pub filters_applied: Vec<String>,
}

/// Click tracking event
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClickEvent {
    pub user: MaybeAddress,
    pub query: String,
    pub content_id: String,
    pub rank_position: u32,
    pub timestamp: u64,
}
