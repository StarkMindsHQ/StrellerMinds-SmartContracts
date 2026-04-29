use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Map};

fn create_test_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(AdvancedSearchContract, ());

    (env, admin, contract_id)
}

fn course_analysis(
    env: &Env,
    content_id: &str,
    category: &str,
    skill_name: &str,
    difficulty: u32,
) -> ContentAnalysis {
    ContentAnalysis {
        content_id: String::from_str(env, content_id),
        auto_generated_tags: Vec::from_array(env, [String::from_str(env, skill_name)]),
        extracted_topics: Vec::from_array(env, [Topic {
            name: String::from_str(env, skill_name),
            relevance_score: 900,
            category: String::from_str(env, category),
        }]),
        identified_skills: Vec::from_array(env, [Skill {
            skill_name: String::from_str(env, skill_name),
            required_level: difficulty,
            importance: 90,
        }]),
        difficulty_score: difficulty,
        quality_score: 90,
        readability_score: 85,
        estimated_duration: 120,
        prerequisite_skills: Vec::new(env),
        learning_outcomes: Vec::from_array(
            env,
            [String::from_str(env, "Build production skills")],
        ),
        analysis_timestamp: env.ledger().timestamp(),
    }
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

#[test]
fn test_recommendations_use_learning_history_and_skill_gaps() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let oracle = Address::generate(&env);
    client.authorize_oracle(&admin, &oracle);

    let completed_course = String::from_str(&env, "rust_foundations");
    let next_course = String::from_str(&env, "rust_smart_contracts");
    let unrelated_course = String::from_str(&env, "design_basics");

    client.store_content_analysis(
        &oracle,
        &completed_course,
        &course_analysis(&env, "rust_foundations", "blockchain", "rust", 25),
    );
    client.store_content_analysis(
        &oracle,
        &next_course,
        &course_analysis(&env, "rust_smart_contracts", "blockchain", "soroban", 45),
    );
    client.store_content_analysis(
        &oracle,
        &unrelated_course,
        &course_analysis(&env, "design_basics", "design", "figma", 25),
    );

    let user = Address::generate(&env);
    client.update_user_profile(&user, &completed_course, &true);

    let recommendations = client.get_recommendations(&user, &3);

    assert_eq!(recommendations.len(), 2);
    assert_eq!(recommendations.get(0).unwrap().content_id, next_course);
    assert!(recommendations.get(0).unwrap().score >= recommendations.get(1).unwrap().score);
    assert_ne!(recommendations.get(0).unwrap().reason, String::from_str(&env, ""));
}

#[test]
fn test_success_rate_responds_to_course_fit() {
    let (env, admin, contract_id) = create_test_env();
    let client = AdvancedSearchContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let oracle = Address::generate(&env);
    client.authorize_oracle(&admin, &oracle);

    let foundation = String::from_str(&env, "foundation");
    let matched = String::from_str(&env, "matched");
    let advanced = String::from_str(&env, "advanced");

    client.store_content_analysis(
        &oracle,
        &foundation,
        &course_analysis(&env, "foundation", "blockchain", "rust", 20),
    );
    client.store_content_analysis(
        &oracle,
        &matched,
        &course_analysis(&env, "matched", "blockchain", "soroban", 40),
    );
    client.store_content_analysis(
        &oracle,
        &advanced,
        &course_analysis(&env, "advanced", "blockchain", "zk", 95),
    );

    let user = Address::generate(&env);
    client.update_user_profile(&user, &foundation, &true);

    let matched_rate = client.predict_success_rate(&user, &matched);
    let advanced_rate = client.predict_success_rate(&user, &advanced);

    assert!(matched_rate > advanced_rate);
    assert!(matched_rate <= 100);
}
