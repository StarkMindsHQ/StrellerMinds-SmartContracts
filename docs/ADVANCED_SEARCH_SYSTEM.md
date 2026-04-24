# Advanced Search and Filter System

## Overview

The Advanced Search and Filter System provides sophisticated search capabilities for courses, certificates, and user progress within the StrellerMinds platform. The system features multi-criteria search, advanced filtering, intelligent ranking, saved searches, and comprehensive analytics.

## Architecture

### Core Components

1. **Search Engine**: Multi-criteria search with text matching and relevance scoring
2. **Filter System**: Advanced filtering with faceted search capabilities
3. **Ranking Algorithm**: Intelligent result ranking based on relevance and popularity
4. **Index Management**: Optimized search indexing for fast query execution
5. **User Preferences**: Personalized search settings and saved searches
6. **Analytics Engine**: Search performance tracking and insights

### Search Capabilities

#### Multi-Criteria Search
- **Text Search**: Full-text search across titles, descriptions, and content
- **Faceted Search**: Category-based filtering with dynamic facets
- **Scoped Search**: Search within specific content types (courses, certificates, progress)
- **Advanced Filters**: Complex filtering with multiple criteria combinations

#### Content Types Supported
- **Courses**: Full course catalog with metadata search
- **Certificates**: Certificate records with status and type filtering
- **User Progress**: Learning progress with completion and activity filters
- **Instructors**: Instructor profiles and course associations
- **Categories**: Course categorization and taxonomy

## Search Query Structure

### Basic Search Query

```rust
SearchQuery {
    query_text: "rust programming",
    filters: SearchFilters {
        categories: ["Programming", "Web Development"],
        difficulty_levels: [Intermediate, Advanced],
        duration_range: Some(DurationRange {
            min_hours: Some(10),
            max_hours: Some(50)
        }),
        price_range: Some(PriceRange {
            min_price: Some(0),
            max_price: Some(10000000) // 100 XLM in stroops
        }),
        rating_range: Some(RatingRange {
            min_rating: 40, // 4.0 stars
            max_rating: 50  // 5.0 stars
        }),
        is_premium: Some(false),
        has_certificate: Some(true)
    },
    sort_options: SortOptions {
        primary_sort: Relevance,
        secondary_sort: Some(Rating),
        sort_order: Descending
    },
    pagination: PaginationOptions {
        page: 1,
        page_size: 20,
        max_results: 1000
    },
    search_scope: All
}
```

## Filtering Options

### Course Filters

| Filter Type | Options | Description |
|-------------|---------|-------------|
| **Categories** | Programming, Design, Business, etc. | Course subject categories |
| **Difficulty** | Beginner, Intermediate, Advanced, Expert | Skill level requirements |
| **Duration** | Min/Max hours | Course length filtering |
| **Price Range** | Min/Max in stroops | Cost-based filtering |
| **Rating** | 1-5 stars (scaled 1-50) | User rating filtering |
| **Language** | English, Spanish, etc. | Course language |
| **Instructor** | Instructor addresses | Filter by specific instructors |
| **Premium Status** | True/False | Premium vs free content |
| **Certificate Available** | True/False | Courses offering certificates |
| **Prerequisites** | True/False | Courses with/without prerequisites |
| **Featured** | True/False | Featured course content |

### Certificate Filters

| Filter Type | Options | Description |
|-------------|---------|-------------|
| **Status** | Active, Revoked, Expired, PendingRenewal, Renewed | Certificate validity status |
| **Type** | Completion, Achievement, Professional, Accredited, Micro | Certificate classification |
| **Issue Date** | Date range | When certificate was issued |
| **Expiry Date** | Date range | Certificate expiration filtering |
| **Course** | Course IDs | Certificates for specific courses |
| **Student** | Student addresses | User-specific certificates |

### Progress Filters

| Filter Type | Options | Description |
|-------------|---------|-------------|
| **Completion Range** | 0-100% | Progress completion percentage |
| **Enrollment Date** | Date range | When user enrolled |
| **Last Activity** | Date range | Recent learning activity |
| **Course Status** | In Progress, Completed, Paused | Learning status |
| **Time Spent** | Min/Max minutes | Study time filtering |

## Search Result Ranking

### Relevance Algorithm

The ranking algorithm considers multiple factors:

1. **Text Relevance** (40% weight)
   - Exact title matches: +500 points
   - Title contains query: +100 points per word
   - Description matches: +50 points per word
   - Content matches: +25 points per word

2. **Popularity Metrics** (30% weight)
   - Enrollment count: +1 point per 10 enrollments
   - User rating: +2 points per rating point
   - Completion rate: +1 point per percentage point

3. **Recency** (20% weight)
   - Recently created: +100 points if < 30 days
   - Recently updated: +50 points if < 7 days

4. **User Preferences** (10% weight)
   - Preferred categories: +25 points
   - Preferred difficulty: +15 points
   - Previous interactions: +10 points

### Sorting Options

| Sort Field | Description | Use Case |
|------------|-------------|----------|
| **Relevance** | Algorithm-based relevance score | Default search ranking |
| **Title** | Alphabetical by title | A-Z browsing |
| **Created Date** | Newest/oldest first | Recent content discovery |
| **Updated Date** | Recently modified | Fresh content identification |
| **Rating** | Highest/lowest rated | Quality-based selection |
| **Popularity** | Most/least enrolled | Trending content |
| **Duration** | Shortest/longest courses | Time-based selection |
| **Price** | Cheapest/most expensive | Budget-based filtering |
| **Completion Rate** | Success rate sorting | Effectiveness-based choice |

## Search Features

### Saved Searches

Users can save frequently used searches with:

```rust
SavedSearch {
    search_id: "search_1234567890_42",
    user_id: user_address,
    name: "Advanced Rust Courses",
    description: "High-rated intermediate to advanced Rust programming courses",
    query: SearchQuery { /* saved query */ },
    created_at: 1640995200,
    last_used: 1640995200,
    use_count: 15,
    is_favorite: true,
    notification_enabled: true,
    auto_execute: false,
    execution_frequency: Some(Weekly)
}
```

#### Saved Search Features
- **Favorites**: Mark important searches for quick access
- **Notifications**: Get alerts when new results match saved searches
- **Auto-execution**: Periodic execution for monitoring new content
- **Usage Tracking**: Monitor search popularity and effectiveness

### Search Preferences

Personalized search experience through user preferences:

```rust
SearchPreferences {
    user_id: user_address,
    default_page_size: 20,
    default_sort: Popularity,
    default_sort_order: Descending,
    preferred_categories: ["Programming", "Web Development"],
    preferred_languages: ["English"],
    preferred_difficulty: [Intermediate, Advanced],
    enable_suggestions: true,
    enable_auto_complete: true,
    enable_faceted_search: true,
    search_history_enabled: true,
    max_search_history: 50
}
```

### Search History

Comprehensive search history tracking:

```rust
SearchHistoryEntry {
    search_id: "hist_1234567890",
    user_id: user_address,
    query: SearchQuery { /* executed query */ },
    results_count: 25,
    clicked_results: ["course_123", "course_456"],
    search_timestamp: 1640995200,
    session_id: Some("session_abc123"),
    search_duration_ms: 150
}
```

## API Reference

### Core Search Functions

#### Execute Search

```rust
pub fn search(
    env: Env,
    query: SearchQuery,
    user: Option<Address>,
) -> Result<SearchResults, Error>
```

**Parameters:**
- `query`: Complete search query with filters and options
- `user`: Optional user address for personalization

**Returns:** `SearchResults` with matching items, facets, and metadata

#### Save Search

```rust
pub fn save_search(
    env: Env,
    user: Address,
    name: String,
    description: String,
    query: SearchQuery,
    notification_enabled: bool,
) -> Result<String, Error>
```

**Parameters:**
- `user`: User address (requires authentication)
- `name`: Display name for saved search
- `description`: Search description
- `query`: Search query to save
- `notification_enabled`: Enable new result notifications

**Returns:** Unique search ID

#### Execute Saved Search

```rust
pub fn execute_saved_search(
    env: Env,
    user: Address,
    search_id: String,
) -> Result<SearchResults, Error>
```

**Parameters:**
- `user`: User address (requires authentication)
- `search_id`: ID of saved search to execute

**Returns:** Current search results for saved query

### Search Management

#### Get Saved Searches

```rust
pub fn get_saved_searches(
    env: Env,
    user: Address,
) -> Result<Vec<SavedSearch>, Error>
```

#### Delete Saved Search

```rust
pub fn delete_saved_search(
    env: Env,
    user: Address,
    search_id: String,
) -> Result<(), Error>
```

#### Toggle Favorite Search

```rust
pub fn toggle_favorite_search(
    env: Env,
    user: Address,
    search_id: String,
) -> Result<bool, Error>
```

### User Preferences

#### Set Search Preferences

```rust
pub fn set_search_preferences(
    env: Env,
    user: Address,
    preferences: SearchPreferences,
) -> Result<(), Error>
```

#### Get Search Preferences

```rust
pub fn get_search_preferences(
    env: Env,
    user: Address,
) -> Result<SearchPreferences, Error>
```

### Search History

#### Get Search History

```rust
pub fn get_search_history(
    env: Env,
    user: Address,
    limit: Option<u32>,
) -> Result<Vec<SearchHistoryEntry>, Error>
```

#### Clear Search History

```rust
pub fn clear_search_history(
    env: Env,
    user: Address,
) -> Result<(), Error>
```

### Search Suggestions

#### Get Search Suggestions

```rust
pub fn get_search_suggestions(
    env: Env,
    query_prefix: String,
    limit: Option<u32>,
) -> Result<Vec<SearchSuggestion>, Error>
```

#### Get Popular Queries

```rust
pub fn get_popular_queries(
    env: Env,
    limit: Option<u32>,
    period: Option<u64>,
) -> Result<Vec<PopularQuery>, Error>
```

### Administrative Functions

#### Update Search Weights

```rust
pub fn update_search_weights(
    env: Env,
    admin: Address,
    weights: SearchWeights,
) -> Result<(), Error>
```

#### Update Index Configuration

```rust
pub fn update_index_config(
    env: Env,
    admin: Address,
    index_name: String,
    config: SearchIndexConfig,
) -> Result<(), Error>
```

#### Rebuild Search Index

```rust
pub fn rebuild_search_index(
    env: Env,
    admin: Address,
) -> Result<(), Error>
```

#### Get Search Analytics

```rust
pub fn get_search_analytics(
    env: Env,
    admin: Address,
    period_start: u64,
    period_end: u64,
) -> Result<SearchAnalytics, Error>
```

## Usage Examples

### Basic Course Search

```rust
// Simple text search for programming courses
let query = SearchQuery {
    query_text: String::from_str(&env, "javascript"),
    filters: SearchFilters {
        categories: {
            let mut cats = Vec::new(&env);
            cats.push_back(String::from_str(&env, "Programming"));
            cats
        },
        difficulty_levels: {
            let mut diffs = Vec::new(&env);
            diffs.push_back(DifficultyLevel::Beginner);
            diffs.push_back(DifficultyLevel::Intermediate);
            diffs
        },
        // ... other filters default to empty/None
    },
    sort_options: SortOptions {
        primary_sort: SortField::Rating,
        secondary_sort: Some(SortField::Popularity),
        sort_order: SortOrder::Descending,
    },
    pagination: PaginationOptions {
        page: 1,
        page_size: 15,
        max_results: 150,
    },
    search_scope: SearchScope::Courses,
};

let results = contract.search(query, Some(user_address))?;
```

### Advanced Certificate Search

```rust
// Search for active professional certificates issued in the last year
let query = SearchQuery {
    query_text: String::from_str(&env, ""),
    filters: SearchFilters {
        certificate_status: {
            let mut statuses = Vec::new(&env);
            statuses.push_back(CertificateStatus::Active);
            statuses
        },
        certificate_types: {
            let mut types = Vec::new(&env);
            types.push_back(CertificateType::Professional);
            types.push_back(CertificateType::Accredited);
            types
        },
        issue_date_range: Some(DateRange {
            start_date: Some(env.ledger().timestamp() - 31536000), // 1 year ago
            end_date: Some(env.ledger().timestamp()),
        }),
        // ... other filters
    },
    sort_options: SortOptions {
        primary_sort: SortField::IssueDate,
        secondary_sort: None,
        sort_order: SortOrder::Descending,
    },
    pagination: PaginationOptions {
        page: 1,
        page_size: 25,
        max_results: 500,
    },
    search_scope: SearchScope::Certificates,
};

let results = contract.search(query, Some(user_address))?;
```

### Progress Analytics Search

```rust
// Find users with high completion rates in advanced courses
let query = SearchQuery {
    query_text: String::from_str(&env, ""),
    filters: SearchFilters {
        completion_range: Some(CompletionRange {
            min_percentage: 80,
            max_percentage: 100,
        }),
        difficulty_levels: {
            let mut diffs = Vec::new(&env);
            diffs.push_back(DifficultyLevel::Advanced);
            diffs.push_back(DifficultyLevel::Expert);
            diffs
        },
        last_activity_range: Some(DateRange {
            start_date: Some(env.ledger().timestamp() - 2592000), // 30 days ago
            end_date: Some(env.ledger().timestamp()),
        }),
        // ... other filters
    },
    sort_options: SortOptions {
        primary_sort: SortField::Progress,
        secondary_sort: Some(SortField::CompletionRate),
        sort_order: SortOrder::Descending,
    },
    pagination: PaginationOptions {
        page: 1,
        page_size: 50,
        max_results: 1000,
    },
    search_scope: SearchScope::UserProgress,
};

let results = contract.search(query, Some(admin_address))?;
```

### Saved Search Workflow

```rust
// 1. Create and save a search
let search_query = /* ... create search query ... */;

let search_id = contract.save_search(
    user_address.clone(),
    String::from_str(&env, "My Weekly Tech Search"),
    String::from_str(&env, "Latest technology courses for continuous learning"),
    search_query,
    true, // Enable notifications
)?;

// 2. Mark as favorite
let is_favorite = contract.toggle_favorite_search(
    user_address.clone(),
    search_id.clone(),
)?;

// 3. Execute saved search later
let results = contract.execute_saved_search(
    user_address.clone(),
    search_id,
)?;

// 4. Check search history
let history = contract.get_search_history(
    user_address,
    Some(10), // Last 10 searches
)?;
```

## Search Index Management

### Index Structure

The search system maintains optimized indices for fast query execution:

1. **Full-Text Index**: Inverted index for text search across all searchable fields
2. **Facet Indices**: Specialized indices for categorical filtering
3. **Metadata Index**: Structured data for advanced filtering
4. **User Index**: Personalization and preference data

### Index Configuration

```rust
SearchIndexConfig {
    index_name: String::from_str(&env, "main_search_index"),
    indexed_fields: vec![
        IndexedField {
            field_name: String::from_str(&env, "title"),
            field_type: IndexFieldType::Text,
            weight: 10,
            searchable: true,
            facetable: false,
            sortable: true,
            highlight: true,
        },
        IndexedField {
            field_name: String::from_str(&env, "category"),
            field_type: IndexFieldType::Keyword,
            weight: 6,
            searchable: true,
            facetable: true,
            sortable: true,
            highlight: false,
        },
        // ... more fields
    ],
    search_weights: SearchWeights {
        title_weight: 10,
        description_weight: 5,
        content_weight: 3,
        tags_weight: 7,
        category_weight: 6,
        instructor_weight: 4,
        metadata_weight: 2,
    },
    update_frequency: IndexUpdateFrequency::Hourly,
    max_index_size: 1000000, // 1MB
    enable_fuzzy_search: true,
    enable_synonym_search: true,
    enable_autocomplete: true,
}
```

### Index Optimization

- **Automatic Optimization**: Periodic index optimization for performance
- **Incremental Updates**: Efficient updates without full rebuilds
- **Compression**: Storage optimization for large indices
- **Caching**: Query result caching for frequently accessed data

## Analytics and Insights

### Search Analytics

The system provides comprehensive analytics for search performance:

```rust
SearchAnalytics {
    total_searches: 15420,
    unique_users: 1250,
    average_results_per_search: 23,
    most_popular_queries: vec![
        PopularQuery {
            query_text: String::from_str(&env, "javascript"),
            search_count: 450,
            unique_users: 320,
            average_results: 28,
            click_through_rate: 65, // 65%
        },
        // ... more popular queries
    ],
    most_clicked_results: vec![
        PopularResult {
            item_id: String::from_str(&env, "course_js_fundamentals"),
            item_type: SearchResultType::Course,
            title: String::from_str(&env, "JavaScript Fundamentals"),
            click_count: 1250,
            unique_users: 890,
            average_position: 2, // Usually appears at position 2
        },
        // ... more popular results
    ],
    search_performance_metrics: PerformanceMetrics {
        average_query_time_ms: 45,
        cache_hit_rate: 78, // 78%
        index_size_mb: 125,
        total_indexed_items: 25000,
        search_success_rate: 92, // 92% of searches return results
    },
    period_start: 1640995200,
    period_end: 1643673600,
}
```

### Performance Metrics

- **Query Performance**: Average response times and optimization opportunities
- **Cache Efficiency**: Hit rates and cache optimization
- **Index Health**: Size, fragmentation, and optimization status
- **User Engagement**: Click-through rates and search success rates

## Security and Privacy

### Access Control

- **User Authentication**: All user-specific operations require authentication
- **Admin Functions**: Administrative operations restricted to contract admin
- **Data Isolation**: User searches and preferences are isolated per user
- **Permission Validation**: Comprehensive permission checking

### Privacy Protection

- **Search History**: Optional and user-controlled
- **Personal Data**: Minimal collection of personally identifiable information
- **Data Retention**: Configurable retention periods for search data
- **Anonymization**: Analytics data anonymized for privacy protection

## Performance Optimization

### Query Optimization

1. **Index Usage**: Efficient index utilization for fast queries
2. **Query Planning**: Optimal query execution paths
3. **Result Caching**: Frequently accessed results cached
4. **Pagination**: Efficient large result set handling

### Storage Optimization

1. **Compression**: Search index compression for storage efficiency
2. **Partitioning**: Data partitioning for scalability
3. **Cleanup**: Automatic cleanup of old search data
4. **Archival**: Long-term storage for historical analytics

## Integration Examples

### Course Discovery Integration

```rust
// Integration with course catalog for enhanced discovery
let discovery_query = SearchQuery {
    query_text: String::from_str(&env, ""),
    filters: SearchFilters {
        is_featured: Some(true),
        rating_range: Some(RatingRange {
            min_rating: 40, // 4+ stars
            max_rating: 50,
        }),
        has_certificate: Some(true),
        // ... other filters
    },
    sort_options: SortOptions {
        primary_sort: SortField::Popularity,
        secondary_sort: Some(SortField::Rating),
        sort_order: SortOrder::Descending,
    },
    pagination: PaginationOptions {
        page: 1,
        page_size: 12, // Grid layout
        max_results: 100,
    },
    search_scope: SearchScope::Courses,
};

let featured_courses = search_contract.search(discovery_query, None)?;
```

### Learning Path Integration

```rust
// Integration with prerequisite system for learning path recommendations
let prerequisite_search = SearchQuery {
    query_text: user_interests,
    filters: SearchFilters {
        has_prerequisites: Some(false), // Start with basics
        difficulty_levels: {
            let mut diffs = Vec::new(&env);
            diffs.push_back(DifficultyLevel::Beginner);
            diffs
        },
        preferred_categories: user_preferences.preferred_categories,
        // ... other filters
    },
    sort_options: SortOptions {
        primary_sort: SortField::Difficulty,
        secondary_sort: Some(SortField::Rating),
        sort_order: SortOrder::Ascending,
    },
    // ... pagination and scope
};

let beginner_courses = search_contract.search(prerequisite_search, Some(user))?;
```

### Analytics Dashboard Integration

```rust
// Integration with analytics for instructor insights
let instructor_analytics = SearchQuery {
    query_text: String::from_str(&env, ""),
    filters: SearchFilters {
        instructor_ids: {
            let mut instructors = Vec::new(&env);
            instructors.push_back(instructor_address);
            instructors
        },
        // ... other filters
    },
    sort_options: SortOptions {
        primary_sort: SortField::Popularity,
        secondary_sort: Some(SortField::Rating),
        sort_order: SortOrder::Descending,
    },
    search_scope: SearchScope::Courses,
    // ... pagination
};

let instructor_courses = search_contract.search(instructor_analytics, Some(instructor))?;
```

## Future Enhancements

### Planned Features

1. **Machine Learning Integration**: AI-powered search relevance and recommendations
2. **Semantic Search**: Natural language understanding for better query interpretation
3. **Voice Search**: Audio query processing and voice-activated search
4. **Visual Search**: Image-based course discovery and matching
5. **Collaborative Filtering**: User behavior-based recommendations

### Scalability Improvements

1. **Distributed Indexing**: Multi-node search index distribution
2. **Real-time Updates**: Live index updates for immediate content availability
3. **Advanced Caching**: Multi-level caching for improved performance
4. **Query Optimization**: Advanced query planning and execution optimization

### Analytics Enhancements

1. **Predictive Analytics**: Search trend prediction and content recommendations
2. **A/B Testing**: Search algorithm testing and optimization
3. **User Journey Tracking**: Complete learning path analytics
4. **Performance Monitoring**: Real-time search performance dashboards

## Conclusion

The Advanced Search and Filter System provides a comprehensive, scalable, and user-friendly search experience for the StrellerMinds platform. With sophisticated filtering, intelligent ranking, personalization features, and robust analytics, the system enhances content discoverability and user engagement while maintaining high performance and security standards.

Key benefits:
- **Comprehensive Search**: Multi-criteria search across all content types
- **Advanced Filtering**: Sophisticated filtering with faceted search
- **Intelligent Ranking**: Relevance-based result ordering with personalization
- **User Experience**: Saved searches, preferences, and search history
- **Performance**: Optimized indexing and caching for fast queries
- **Analytics**: Detailed insights for platform optimization
- **Scalability**: Designed for growth and high-volume usage
- **Security**: Robust access control and privacy protection

The system is production-ready and integrates seamlessly with existing StrellerMinds contracts for a unified learning platform experience.
