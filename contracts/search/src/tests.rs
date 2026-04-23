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
        pagination: PaginationOptions {
            page: 1,
            page_size: 10,
            max_results: 100,
        },
        search_scope: SearchScope::All,
    };

    let name = String::from_str(&env, "My Rust Search");
    let search_id = client.save_search(&user, &name, &query);
    assert!(search_id.len() > 0);

    let saved_searches = client.get_saved_searches(&user);
    assert_eq!(saved_searches.len(), 1);
    assert_eq!(saved_searches.get(0).unwrap().name, name);
}
