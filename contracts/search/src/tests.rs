use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_test_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(AdvancedSearchContract, ());

    (env, admin, contract_id)
}

#[test]
fn test_initialize() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);

    // Initialize contract
    let result = client.try_initialize(&admin);
    assert!(result.is_ok());

    // Try to initialize again - should fail
    let result2 = client.try_initialize(&admin);
    assert!(result2.is_err());
}

#[test]
fn test_save_search() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let user = Address::generate(&env);

    let filters = SearchFilters {
        categories: Vec::new(&env),
        difficulty_levels: Vec::new(&env),
        duration_range: MaybeDurationRange::None,
        instructor_ids: Vec::new(&env),
        languages: Vec::new(&env),
        price_range: MaybePriceRange::None,
        rating_range: MaybeRatingRange::None,
        tags: Vec::new(&env),
        certificate_status: Vec::new(&env),
        issue_date_range: MaybeDateRange::None,
        expiry_date_range: MaybeDateRange::None,
        certificate_types: Vec::new(&env),
        completion_range: MaybeCompletionRange::None,
        enrollment_date_range: MaybeDateRange::None,
        last_activity_range: MaybeDateRange::None,
        has_prerequisites: MaybeBool::None,
        has_certificate: MaybeBool::None,
        is_premium: MaybeBool::None,
        is_featured: MaybeBool::None,
    };

    let query = SearchQuery {
        query_text: String::from_str(&env, "rust programming"),
        filters,
        sort_options: SortOptions {
            primary_sort: SortField::Relevance,
            secondary_sort: MaybeSortField::None,
            sort_order: SortOrder::Descending,
        },
        pagination: PaginationOptions { page: 1, page_size: 10, max_results: 100 },
        search_scope: SearchScope::All,
    };

    let name = String::from_str(&env, "My Rust Search");
    let search_id = client.save_search(&user, &name, &query);
    assert!(!search_id.is_empty());

    let saved_searches = client.get_saved_searches(&user);
    assert_eq!(saved_searches.len(), 1);
    assert_eq!(saved_searches.get(0).unwrap().name, name);
}

#[test]
fn test_semantic_search_with_filters() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let oracle = Address::generate(&env);
    client.authorize_oracle(&admin, &oracle);

    let content_id = String::from_str(&env, "course_1");
    let metadata = SemanticMetadata {
        content_id: content_id.clone(),
        topics: Vec::from_array(&env, [String::from_str(&env, "rust")]),
        intent_scores: Map::new(&env),
        semantic_tags: Vec::from_array(&env, [String::from_str(&env, "programming")]),
        entity_types: Vec::new(&env),
        complexity_score: 50,
        last_updated: 1000,
        category: String::from_str(&env, "tech"),
        instructor_id: Address::generate(&env),
        language: String::from_str(&env, "en"),
        price: 100,
        rating: 45,
        duration_hours: 10,
        has_prerequisites: false,
        has_certificate: true,
        is_premium: false,
        is_featured: true,
        certificate_types: Vec::new(&env),
        certificate_status: Vec::new(&env),
    };

    client.store_semantic_metadata(&oracle, &content_id, &metadata);

    let query = ProcessedQuery {
        original_text: String::from_str(&env, "rust"),
        normalized_text: String::from_str(&env, "rust"),
        original_query: String::from_str(&env, "rust"),
        extracted_intent: String::from_str(&env, "search"),
        intent: String::from_str(&env, "search"),
        entities: Vec::new(&env),
        expanded_terms: Vec::new(&env),
        semantic_tags: Vec::from_array(&env, [String::from_str(&env, "rust")]),
        suggested_filters: Vec::new(&env),
        query_type: String::from_str(&env, "informational"),
        confidence: 1000,
    };

    // Test filter that matches
    let mut filters = SearchFilters {
        categories: Vec::from_array(&env, [String::from_str(&env, "tech")]),
        difficulty_levels: Vec::new(&env),
        duration_range: MaybeDurationRange::None,
        instructor_ids: Vec::new(&env),
        languages: Vec::new(&env),
        price_range: MaybePriceRange::None,
        rating_range: MaybeRatingRange::None,
        tags: Vec::new(&env),
        certificate_status: Vec::new(&env),
        issue_date_range: MaybeDateRange::None,
        expiry_date_range: MaybeDateRange::None,
        certificate_types: Vec::new(&env),
        completion_range: MaybeCompletionRange::None,
        enrollment_date_range: MaybeDateRange::None,
        last_activity_range: MaybeDateRange::None,
        has_prerequisites: MaybeBool::None,
        has_certificate: MaybeBool::None,
        is_premium: MaybeBool::None,
        is_featured: MaybeBool::Some(true),
    };

    let results = client.semantic_search(&query, &None, &filters);
    assert_eq!(results.len(), 1);

    // Test filter that doesn't match
    filters.is_featured = MaybeBool::Some(false);
    let results2 = client.semantic_search(&query, &None, &filters);
    assert_eq!(results2.len(), 0);
}
