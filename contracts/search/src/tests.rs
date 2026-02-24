#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn create_test_env() -> (Env, Address, SearchContract) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract = SearchContract::new(&env, contract_id);

    (env, admin, contract)
}

fn create_test_search_query(env: &Env) -> SearchQuery {
    SearchQuery {
        query_text: String::from_str(env, "rust programming"),
        filters: SearchFilters {
            categories: Vec::new(env),
            difficulty_levels: Vec::new(env),
            duration_range: None,
            instructor_ids: Vec::new(env),
            languages: Vec::new(env),
            price_range: None,
            rating_range: None,
            tags: Vec::new(env),
            certificate_status: Vec::new(env),
            issue_date_range: None,
            expiry_date_range: None,
            certificate_types: Vec::new(env),
            completion_range: None,
            enrollment_date_range: None,
            last_activity_range: None,
            has_prerequisites: None,
            has_certificate: None,
            is_premium: None,
            is_featured: None,
        },
        sort_options: SortOptions {
            primary_sort: SortField::Relevance,
            secondary_sort: None,
            sort_order: SortOrder::Descending,
        },
        pagination: PaginationOptions {
            page: 1,
            page_size: 10,
            max_results: 100,
        },
        search_scope: SearchScope::All,
    }
}

#[test]
fn test_initialize() {
    let (env, admin, contract) = create_test_env();

    let result = contract.initialize(admin.clone());
    assert!(result.is_ok());

    // Test double initialization fails
    let result2 = contract.initialize(admin);
    assert_eq!(result2, Err(Error::AlreadyInitialized));
}

#[test]
fn test_search_basic_functionality() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let query = create_test_search_query(&env);

    let result = contract.search(query, Some(user));
    assert!(result.is_ok());

    let search_results = result.unwrap();
    assert_eq!(search_results.page, 1);
    assert_eq!(search_results.page_size, 10);
}

#[test]
fn test_course_filtering() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let mut query = create_test_search_query(&env);

    // Add category filter
    let mut categories = Vec::new(&env);
    categories.push_back(String::from_str(&env, "Programming"));
    query.filters.categories = categories;

    // Add difficulty filter
    let mut difficulties = Vec::new(&env);
    difficulties.push_back(DifficultyLevel::Intermediate);
    query.filters.difficulty_levels = difficulties;

    // Add duration filter
    query.filters.duration_range = Some(DurationRange {
        min_hours: Some(5),
        max_hours: Some(20),
    });

    let result = contract.search(query, Some(user));
    assert!(result.is_ok());
}

#[test]
fn test_certificate_filtering() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let mut query = create_test_search_query(&env);
    query.search_scope = SearchScope::Certificates;

    // Add certificate status filter
    let mut statuses = Vec::new(&env);
    statuses.push_back(CertificateStatus::Active);
    query.filters.certificate_status = statuses;

    // Add certificate type filter
    let mut types = Vec::new(&env);
    types.push_back(CertificateType::Professional);
    query.filters.certificate_types = types;

    let result = contract.search(query, Some(user));
    assert!(result.is_ok());
}

#[test]
fn test_progress_filtering() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let mut query = create_test_search_query(&env);
    query.search_scope = SearchScope::UserProgress;

    // Add completion range filter
    query.filters.completion_range = Some(CompletionRange {
        min_percentage: 50,
        max_percentage: 100,
    });

    let result = contract.search(query, Some(user));
    assert!(result.is_ok());
}

#[test]
fn test_sorting_options() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);

    // Test sorting by title
    let mut query = create_test_search_query(&env);
    query.sort_options.primary_sort = SortField::Title;
    query.sort_options.sort_order = SortOrder::Ascending;

    let result = contract.search(query, Some(user.clone()));
    assert!(result.is_ok());

    // Test sorting by date
    let mut query2 = create_test_search_query(&env);
    query2.sort_options.primary_sort = SortField::CreatedDate;
    query2.sort_options.sort_order = SortOrder::Descending;

    let result2 = contract.search(query2, Some(user));
    assert!(result2.is_ok());
}

#[test]
fn test_pagination() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let mut query = create_test_search_query(&env);

    // Test first page
    query.pagination.page = 1;
    query.pagination.page_size = 5;

    let result = contract.search(query.clone(), Some(user.clone()));
    assert!(result.is_ok());
    let page1 = result.unwrap();
    assert_eq!(page1.page, 1);
    assert_eq!(page1.page_size, 5);

    // Test second page
    query.pagination.page = 2;
    let result2 = contract.search(query, Some(user));
    assert!(result2.is_ok());
    let page2 = result2.unwrap();
    assert_eq!(page2.page, 2);
}

#[test]
fn test_saved_searches() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let query = create_test_search_query(&env);

    // Save a search
    let search_name = String::from_str(&env, "My Rust Search");
    let search_desc = String::from_str(&env, "Search for Rust programming courses");

    let result = contract.save_search(user.clone(), search_name.clone(), search_desc, query, true);
    assert!(result.is_ok());
    let search_id = result.unwrap();

    // Get saved searches
    let saved_searches = contract.get_saved_searches(user.clone());
    assert!(saved_searches.is_ok());
    let searches = saved_searches.unwrap();
    assert_eq!(searches.len(), 1);
    assert_eq!(searches.get(0).unwrap().name, search_name);

    // Execute saved search
    let exec_result = contract.execute_saved_search(user.clone(), search_id.clone());
    assert!(exec_result.is_ok());

    // Delete saved search
    let delete_result = contract.delete_saved_search(user, search_id);
    assert!(delete_result.is_ok());
}

#[test]
fn test_search_preferences() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);

    let preferences = SearchPreferences {
        user_id: user.clone(),
        default_page_size: 20,
        default_sort: SortField::Rating,
        default_sort_order: SortOrder::Descending,
        preferred_categories: {
            let mut cats = Vec::new(&env);
            cats.push_back(String::from_str(&env, "Programming"));
            cats
        },
        preferred_languages: {
            let mut langs = Vec::new(&env);
            langs.push_back(String::from_str(&env, "English"));
            langs
        },
        preferred_difficulty: {
            let mut diffs = Vec::new(&env);
            diffs.push_back(DifficultyLevel::Intermediate);
            diffs
        },
        enable_suggestions: true,
        enable_auto_complete: true,
        enable_faceted_search: true,
        search_history_enabled: true,
        max_search_history: 50,
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    // Set preferences
    let result = contract.set_search_preferences(user.clone(), preferences.clone());
    assert!(result.is_ok());

    // Get preferences
    let get_result = contract.get_search_preferences(user);
    assert!(get_result.is_ok());
    let retrieved_prefs = get_result.unwrap();
    assert_eq!(retrieved_prefs.default_page_size, 20);
    assert_eq!(retrieved_prefs.default_sort, SortField::Rating);
}

#[test]
fn test_search_history() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let query = create_test_search_query(&env);

    // Perform a search (this should add to history)
    let _result = contract.search(query, Some(user.clone()));

    // Get search history
    let history_result = contract.get_search_history(user.clone(), Some(10));
    assert!(history_result.is_ok());

    // Clear search history
    let clear_result = contract.clear_search_history(user);
    assert!(clear_result.is_ok());
}

#[test]
fn test_search_suggestions() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let query_prefix = String::from_str(&env, "rust");
    let suggestions = contract.get_search_suggestions(query_prefix, Some(5));
    assert!(suggestions.is_ok());
}

#[test]
fn test_popular_queries() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let popular = contract.get_popular_queries(Some(10), None);
    assert!(popular.is_ok());
}

#[test]
fn test_favorite_searches() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let query = create_test_search_query(&env);

    // Save a search
    let search_id = contract
        .save_search(
            user.clone(),
            String::from_str(&env, "Test Search"),
            String::from_str(&env, "Description"),
            query,
            false,
        )
        .unwrap();

    // Toggle favorite
    let toggle_result = contract.toggle_favorite_search(user.clone(), search_id.clone());
    assert!(toggle_result.is_ok());
    assert_eq!(toggle_result.unwrap(), true);

    // Toggle again
    let toggle_result2 = contract.toggle_favorite_search(user, search_id);
    assert!(toggle_result2.is_ok());
    assert_eq!(toggle_result2.unwrap(), false);
}

#[test]
fn test_admin_functions() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    // Test update search weights
    let weights = SearchWeights {
        title_weight: 15,
        description_weight: 8,
        content_weight: 5,
        tags_weight: 10,
        category_weight: 7,
        instructor_weight: 6,
        metadata_weight: 3,
    };

    let result = contract.update_search_weights(admin.clone(), weights);
    assert!(result.is_ok());

    // Test add search suggestions
    let category = String::from_str(&env, "programming");
    let mut suggestions = Vec::new(&env);
    suggestions.push_back(SearchSuggestion {
        suggestion_text: String::from_str(&env, "rust programming"),
        suggestion_type: SuggestionType::Query,
        popularity_score: 100,
        category: Some(category.clone()),
        metadata: None,
    });

    let sugg_result = contract.add_search_suggestions(admin, category, suggestions);
    assert!(sugg_result.is_ok());
}

#[test]
fn test_unauthorized_access() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let unauthorized_user = Address::generate(&env);
    let weights = SearchWeights {
        title_weight: 10,
        description_weight: 5,
        content_weight: 3,
        tags_weight: 7,
        category_weight: 6,
        instructor_weight: 4,
        metadata_weight: 2,
    };

    // Should fail for non-admin
    let result = contract.update_search_weights(unauthorized_user, weights);
    assert_eq!(result, Err(Error::Unauthorized));
}

#[test]
fn test_complex_search_scenario() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let mut query = create_test_search_query(&env);

    // Complex filtering scenario
    let mut categories = Vec::new(&env);
    categories.push_back(String::from_str(&env, "Programming"));
    categories.push_back(String::from_str(&env, "Web Development"));
    query.filters.categories = categories;

    let mut difficulties = Vec::new(&env);
    difficulties.push_back(DifficultyLevel::Intermediate);
    difficulties.push_back(DifficultyLevel::Advanced);
    query.filters.difficulty_levels = difficulties;

    query.filters.duration_range = Some(DurationRange {
        min_hours: Some(10),
        max_hours: Some(50),
    });

    query.filters.price_range = Some(PriceRange {
        min_price: Some(0),
        max_price: Some(10000000), // 100 XLM in stroops
    });

    query.filters.rating_range = Some(RatingRange {
        min_rating: 40, // 4.0 stars
        max_rating: 50, // 5.0 stars
    });

    query.filters.is_premium = Some(false);
    query.filters.has_certificate = Some(true);

    // Execute complex search
    let result = contract.search(query, Some(user));
    assert!(result.is_ok());

    let search_results = result.unwrap();
    assert!(search_results.execution_time_ms > 0);
}

#[test]
fn test_search_scope_variations() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);
    let base_query = create_test_search_query(&env);

    // Test different search scopes
    let scopes = vec![
        SearchScope::Courses,
        SearchScope::Certificates,
        SearchScope::UserProgress,
        SearchScope::All,
        SearchScope::Custom({
            let mut targets = Vec::new(&env);
            targets.push_back(SearchTarget::Courses);
            targets.push_back(SearchTarget::Certificates);
            targets
        }),
    ];

    for scope in scopes {
        let mut query = base_query.clone();
        query.search_scope = scope;

        let result = contract.search(query, Some(user.clone()));
        assert!(result.is_ok());
    }
}

#[test]
fn test_edge_cases() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);

    // Test empty query
    let mut empty_query = create_test_search_query(&env);
    empty_query.query_text = String::from_str(&env, "");

    let result = contract.search(empty_query, Some(user.clone()));
    assert!(result.is_ok());

    // Test invalid pagination
    let mut invalid_pagination_query = create_test_search_query(&env);
    invalid_pagination_query.pagination.page = 0;

    let result2 = contract.search(invalid_pagination_query, Some(user.clone()));
    // Should handle gracefully or return error

    // Test very large page size
    let mut large_page_query = create_test_search_query(&env);
    large_page_query.pagination.page_size = 1000;

    let result3 = contract.search(large_page_query, Some(user));
    assert!(result3.is_ok());
}

// Integration tests for search workflows
#[test]
fn test_end_to_end_search_workflow() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user = Address::generate(&env);

    // 1. Set user preferences
    let preferences = SearchPreferences {
        user_id: user.clone(),
        default_page_size: 15,
        default_sort: SortField::Popularity,
        default_sort_order: SortOrder::Descending,
        preferred_categories: {
            let mut cats = Vec::new(&env);
            cats.push_back(String::from_str(&env, "Technology"));
            cats
        },
        preferred_languages: Vec::new(&env),
        preferred_difficulty: Vec::new(&env),
        enable_suggestions: true,
        enable_auto_complete: true,
        enable_faceted_search: true,
        search_history_enabled: true,
        max_search_history: 25,
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    contract
        .set_search_preferences(user.clone(), preferences)
        .unwrap();

    // 2. Perform initial search
    let query = create_test_search_query(&env);
    let search_result = contract.search(query.clone(), Some(user.clone())).unwrap();
    assert!(search_result.results.len() <= 15); // Should use user's preferred page size

    // 3. Save the search
    let search_id = contract
        .save_search(
            user.clone(),
            String::from_str(&env, "My Technology Search"),
            String::from_str(&env, "Searching for technology courses"),
            query,
            true,
        )
        .unwrap();

    // 4. Mark as favorite
    let is_favorite = contract
        .toggle_favorite_search(user.clone(), search_id.clone())
        .unwrap();
    assert!(is_favorite);

    // 5. Execute saved search
    let saved_result = contract
        .execute_saved_search(user.clone(), search_id.clone())
        .unwrap();
    assert_eq!(saved_result.page_size, search_result.page_size);

    // 6. Check search history
    let history = contract.get_search_history(user.clone(), Some(5)).unwrap();
    assert!(history.len() > 0);

    // 7. Get search suggestions
    let suggestions = contract
        .get_search_suggestions(String::from_str(&env, "tech"), Some(3))
        .unwrap();

    // 8. Clean up - delete saved search
    contract
        .delete_saved_search(user.clone(), search_id)
        .unwrap();

    // 9. Clear history
    contract.clear_search_history(user).unwrap();
}

#[test]
fn test_multi_user_search_scenario() {
    let (env, admin, contract) = create_test_env();
    contract.initialize(admin.clone()).unwrap();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Both users perform searches
    let query1 = create_test_search_query(&env);
    let query2 = {
        let mut q = create_test_search_query(&env);
        q.query_text = String::from_str(&env, "javascript web development");
        q
    };

    let result1 = contract.search(query1.clone(), Some(user1.clone()));
    let result2 = contract.search(query2.clone(), Some(user2.clone()));

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Both users save searches
    let search_id1 = contract
        .save_search(
            user1.clone(),
            String::from_str(&env, "User1 Search"),
            String::from_str(&env, "Description 1"),
            query1,
            false,
        )
        .unwrap();

    let search_id2 = contract
        .save_search(
            user2.clone(),
            String::from_str(&env, "User2 Search"),
            String::from_str(&env, "Description 2"),
            query2,
            false,
        )
        .unwrap();

    // Verify users can only access their own saved searches
    let user1_searches = contract.get_saved_searches(user1.clone()).unwrap();
    let user2_searches = contract.get_saved_searches(user2.clone()).unwrap();

    assert_eq!(user1_searches.len(), 1);
    assert_eq!(user2_searches.len(), 1);
    assert_eq!(user1_searches.get(0).unwrap().search_id, search_id1);
    assert_eq!(user2_searches.get(0).unwrap().search_id, search_id2);
}

// Helper function to create contract ID for tests
fn contract_id() -> [u8; 32] {
    [0; 32]
}
