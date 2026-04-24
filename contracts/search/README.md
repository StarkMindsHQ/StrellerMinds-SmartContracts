# Search Contract

## Overview
A comprehensive search system for educational content on the Stellar blockchain. This contract provides advanced search capabilities including saved searches, search preferences, search history, suggestions, and analytics for educational platforms.

## Interface

### Core Search Functions
```rust
// Initialize the search contract
fn initialize(env: Env, admin: Address) -> Result<(), Error>

// Execute a comprehensive search query
fn search(env: Env, query: SearchQuery, user: Option<Address>) -> Result<SearchResults, Error>

// Get search suggestions based on query prefix
fn get_search_suggestions(env: Env, query_prefix: String, limit: Option<u32>) -> Result<Vec<SearchSuggestion>, Error>

// Get popular search queries
fn get_popular_queries(env: Env, limit: Option<u32>, period: Option<u64>) -> Result<Vec<PopularQuery>, Error>
```

### Saved Search Management
```rust
// Save a search query for future use
fn save_search(env: Env, user: Address, name: String, description: String, query: SearchQuery, notification_enabled: bool) -> Result<String, Error>

// Get saved searches for a user
fn get_saved_searches(env: Env, user: Address) -> Result<Vec<SavedSearch>, Error>

// Execute a saved search
fn execute_saved_search(env: Env, user: Address, search_id: String) -> Result<SearchResults, Error>

// Delete a saved search
fn delete_saved_search(env: Env, user: Address, search_id: String) -> Result<(), Error>

// Mark a saved search as favorite
fn toggle_favorite_search(env: Env, user: Address, search_id: String) -> Result<bool, Error>
```

### Search Preferences and History
```rust
// Set user search preferences
fn set_search_preferences(env: Env, user: Address, preferences: SearchPreferences) -> Result<(), Error>

// Get user search preferences
fn get_search_preferences(env: Env, user: Address) -> Result<SearchPreferences, Error>

// Get search history for a user
fn get_search_history(env: Env, user: Address, limit: Option<u32>) -> Result<Vec<SearchHistoryEntry>, Error>

// Clear search history for a user
fn clear_search_history(env: Env, user: Address) -> Result<(), Error>
```

### Administrative Functions
```rust
// Get search analytics (admin only)
fn get_search_analytics(env: Env, admin: Address, period_start: u64, period_end: u64) -> Result<SearchAnalytics, Error>

// Update search index configuration (admin only)
fn update_index_config(env: Env, admin: Address, index_name: String, config: SearchIndexConfig) -> Result<(), Error>

// Update search weights (admin only)
fn update_search_weights(env: Env, admin: Address, weights: SearchWeights) -> Result<(), Error>

// Add search suggestions (admin only)
fn add_search_suggestions(env: Env, admin: Address, category: String, suggestions: Vec<SearchSuggestion>) -> Result<(), Error>

// Rebuild search index (admin only)
fn rebuild_search_index(env: Env, admin: Address) -> Result<(), Error>
```

## Events

### Search Events
- `search_executed`: Emitted when a search query is executed
- `search_saved`: Emitted when a search is saved for future use
- `search_deleted`: Emitted when a saved search is deleted
- `search_favorited`: Emitted when a search is marked as favorite

### Preference Events
- `preferences_updated`: Emitted when user search preferences are updated
- `history_cleared`: Emitted when search history is cleared

### Administrative Events
- `index_rebuilt`: Emitted when search index is rebuilt
- `weights_updated`: Emitted when search weights are updated
- `suggestions_added`: Emitted when new search suggestions are added

## Configuration

### Search Weights Configuration
```rust
pub struct SearchWeights {
    pub title_weight: u32,
    pub description_weight: u32,
    pub content_weight: u32,
    pub tags_weight: u32,
    pub category_weight: u32,
    pub instructor_weight: u32,
    pub metadata_weight: u32,
}
```

### Search Index Configuration
```rust
pub struct SearchIndexConfig {
    pub index_name: String,
    pub fields: Vec<String>,
    pub weights: SearchWeights,
    pub filters: Vec<SearchFilter>,
    pub sorting_options: Vec<SortOption>,
}
```

### Search Preferences
```rust
pub struct SearchPreferences {
    pub default_sort: SortOption,
    pub results_per_page: u32,
    pub auto_suggestions: bool,
    pub save_history: bool,
    pub notification_enabled: bool,
}
```

## Testing

### Running Tests
```bash
# Run all tests for search contract
cargo test --package search

# Run specific test modules
cargo test --package search tests::test_search_functionality
cargo test --package search tests::test_saved_searches
cargo test --package search tests::test_search_preferences
cargo test --package search tests::test_search_analytics
```

### Test Coverage
- **Search Functionality Tests**: Core search query execution
- **Saved Search Tests**: Save, retrieve, and manage saved searches
- **Preference Tests**: User preference management
- **History Tests**: Search history tracking and management
- **Suggestion Tests**: Search suggestion functionality
- **Analytics Tests**: Search analytics and reporting
- **Administrative Tests**: Admin-only functions and configurations

## Deployment

### Prerequisites
- Admin address for contract initialization
- Search index configuration
- Initial search weights and suggestions

### Deployment Steps
1. Deploy the search contract
2. Initialize with admin address
3. Configure search weights and index settings
4. Add initial search suggestions
5. Set up search analytics collection
6. Begin search operations

### Environment Setup
- Configure search weights for different content types
- Set up search index with appropriate fields
- Add popular search suggestions
- Configure analytics collection parameters
- Set up search result pagination limits

## Usage Examples

### Basic Search
```rust
let query = SearchQuery {
    query_text: "blockchain fundamentals".to_string(),
    filters: vec![SearchFilter::Category("Technology".to_string())],
    sort_by: SortOption::Relevance,
    limit: Some(20),
};

let results = client.search(&query, Some(&user))?;
```

### Saving a Search
```rust
let search_id = client.save_search(
    &user,
    "Blockchain Courses".to_string(),
    "Find all blockchain-related courses".to_string(),
    query,
    true, // notification_enabled
)?;
```

### Setting Search Preferences
```rust
let preferences = SearchPreferences {
    default_sort: SortOption::Relevance,
    results_per_page: 20,
    auto_suggestions: true,
    save_history: true,
    notification_enabled: true,
};

client.set_search_preferences(&user, &preferences)?;
```

### Getting Search Suggestions
```rust
let suggestions = client.get_search_suggestions(
    "blockchain".to_string(),
    Some(10)
)?;
```

## Data Structures

### Search Query
```rust
pub struct SearchQuery {
    pub query_text: String,
    pub filters: Vec<SearchFilter>,
    pub sort_by: SortOption,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
```

### Search Results
```rust
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total_count: u32,
    pub page: u32,
    pub has_more: bool,
    pub execution_time_ms: u64,
}
```

### Saved Search
```rust
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
    pub notification_enabled: bool,
}
```

## Related Docs
- [Advanced Search System](../docs/ADVANCED_SEARCH_SYSTEM.md)
- [Development Guide](../docs/development.md)