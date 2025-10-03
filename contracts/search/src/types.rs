use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Search query structure for multi-criteria searches
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    pub query_text: String,           // Text search query
    pub filters: SearchFilters,       // Applied filters
    pub sort_options: SortOptions,    // Sorting preferences
    pub pagination: PaginationOptions, // Pagination settings
    pub search_scope: SearchScope,    // What to search (courses, certificates, etc.)
}

/// Comprehensive filtering options
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFilters {
    // Course filters
    pub categories: Vec<String>,          // Course categories
    pub difficulty_levels: Vec<DifficultyLevel>, // Difficulty filtering
    pub duration_range: Option<DurationRange>,   // Duration filtering
    pub instructor_ids: Vec<Address>,     // Filter by instructors
    pub languages: Vec<String>,           // Course languages
    pub price_range: Option<PriceRange>,  // Price filtering
    pub rating_range: Option<RatingRange>, // Rating filtering
    pub tags: Vec<String>,                // Course tags
    
    // Certificate filters
    pub certificate_status: Vec<CertificateStatus>, // Certificate status
    pub issue_date_range: Option<DateRange>,        // Issue date filtering
    pub expiry_date_range: Option<DateRange>,       // Expiry date filtering
    pub certificate_types: Vec<CertificateType>,    // Certificate types
    
    // Progress filters
    pub completion_range: Option<CompletionRange>,  // Progress completion
    pub enrollment_date_range: Option<DateRange>,   // Enrollment filtering
    pub last_activity_range: Option<DateRange>,     // Last activity filtering
    
    // Advanced filters
    pub has_prerequisites: Option<bool>,    // Filter by prerequisite requirements
    pub has_certificate: Option<bool>,      // Filter by certificate availability
    pub is_premium: Option<bool>,          // Premium content filter
    pub is_featured: Option<bool>,         // Featured content filter
}

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
    pub min_hours: Option<u32>,
    pub max_hours: Option<u32>,
}

/// Price range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceRange {
    pub min_price: Option<i64>,  // In stroops
    pub max_price: Option<i64>,  // In stroops
}

/// Rating range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatingRange {
    pub min_rating: u32,  // 1-5 stars (scaled to 1-50 for precision)
    pub max_rating: u32,  // 1-5 stars (scaled to 1-50 for precision)
}

/// Date range for filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateRange {
    pub start_date: Option<u64>,  // Unix timestamp
    pub end_date: Option<u64>,    // Unix timestamp
}

/// Completion range for progress filtering
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionRange {
    pub min_percentage: u32,  // 0-100
    pub max_percentage: u32,  // 0-100
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

/// Sorting options for search results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortOptions {
    pub primary_sort: SortField,
    pub secondary_sort: Option<SortField>,
    pub sort_order: SortOrder,
}

/// Available sorting fields
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    Relevance,          // Search relevance score
    Title,              // Alphabetical by title
    CreatedDate,        // Creation date
    UpdatedDate,        // Last update date
    Rating,             // User rating
    Popularity,         // Enrollment count
    Duration,           // Course duration
    Difficulty,         // Difficulty level
    Price,              // Course price
    CompletionRate,     // Course completion rate
    IssueDate,          // Certificate issue date (for certificates)
    ExpiryDate,         // Certificate expiry date (for certificates)
    Progress,           // User progress (for progress searches)
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
    pub page: u32,           // Page number (1-based)
    pub page_size: u32,      // Results per page
    pub max_results: u32,    // Maximum total results to return
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
    pub query_id: String,                    // Unique query identifier
    pub total_results: u32,                  // Total matching results
    pub page: u32,                          // Current page
    pub page_size: u32,                     // Results per page
    pub has_more: bool,                     // Whether more results exist
    pub results: Vec<SearchResultItem>,     // Actual results
    pub facets: Vec<SearchFacet>,          // Faceted search results
    pub suggestions: Vec<String>,           // Search suggestions
    pub execution_time_ms: u32,            // Query execution time
    pub search_metadata: SearchMetadata,    // Additional metadata
}

/// Individual search result item
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResultItem {
    pub item_id: String,                    // Unique item identifier
    pub item_type: SearchResultType,        // Type of result
    pub title: String,                      // Item title
    pub description: String,                // Item description
    pub relevance_score: u32,              // Relevance score (0-1000)
    pub metadata: SearchResultMetadata,     // Type-specific metadata
    pub highlights: Vec<SearchHighlight>,   // Text highlights
    pub thumbnail_url: Option<String>,      // Optional thumbnail
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
    pub rating: u32,                        // 1-50 scale
    pub enrollment_count: u32,
    pub completion_rate: u32,               // Percentage
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
    pub field: String,                      // Field name that matched
    pub original_text: String,              // Original text
    pub highlighted_text: String,           // Text with highlights
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
    pub notification_enabled: bool,      // Notify when new results match
    pub auto_execute: bool,              // Auto-execute periodically
    pub execution_frequency: Option<ExecutionFrequency>,
}

/// Frequency for auto-executing saved searches
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionFrequency {
    Daily,
    Weekly,
    Monthly,
    Custom(u64),  // Custom interval in seconds
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
    pub clicked_results: Vec<String>,    // IDs of results user clicked
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
    pub click_through_rate: u32,  // Percentage
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
    pub average_position: u32,    // Average position in search results
}

/// Search performance metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    pub average_query_time_ms: u32,
    pub cache_hit_rate: u32,      // Percentage
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
    pub weight: u32,              // Search weight (1-10)
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
    Query,           // Query completion
    Course,          // Course suggestion
    Category,        // Category suggestion
    Instructor,      // Instructor suggestion
    Tag,             // Tag suggestion
    Correction,      // Spelling correction
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
}
