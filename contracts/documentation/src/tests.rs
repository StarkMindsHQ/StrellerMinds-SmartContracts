use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

use crate::types::*;
use crate::{DocumentationContract, DocumentationContractClient};

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    (env, admin, user1, user2)
}

fn setup_contract<'a>(env: &Env, admin: &Address) -> DocumentationContractClient<'a> {
    let contract_id = env.register(DocumentationContract, ());
    let client = DocumentationContractClient::new(env, &contract_id);
    client.initialize(admin);
    client
}

// ============================================================================
// Initialization Tests
// ============================================================================

#[test]
fn test_initialize() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.max_doc_size, 100_000);
    assert!(config.require_review);
    assert!(config.enable_contributions);
    assert!(config.enable_analytics);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_initialize() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);
    client.initialize(&admin);
}

#[test]
fn test_initial_counters_are_zero() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    assert_eq!(client.get_total_documents(), 0);
    assert_eq!(client.get_total_views(), 0);
    assert_eq!(client.get_total_contributions(), 0);
}

// ============================================================================
// Document Management Tests
// ============================================================================

#[test]
fn test_create_document() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "soroban"));

    let doc = client.create_document(
        &user1,
        &String::from_str(&env, "doc-1"),
        &String::from_str(&env, "Getting Started"),
        &String::from_str(&env, "Welcome to the guide"),
        &DocumentType::Guide,
        &String::from_str(&env, "tutorials"),
        &tags,
        &String::from_str(&env, "en"),
    );

    assert_eq!(doc.doc_id, String::from_str(&env, "doc-1"));
    assert_eq!(doc.title, String::from_str(&env, "Getting Started"));
    assert_eq!(doc.author, user1);
    assert_eq!(doc.version, 1);
    assert_eq!(doc.status, DocumentStatus::Draft);
    assert_eq!(doc.view_count, 0);
    assert_eq!(doc.helpful_count, 0);
    assert_eq!(client.get_total_documents(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_document() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-dup");
    let title = String::from_str(&env, "Title");
    let content = String::from_str(&env, "Content");
    let category = String::from_str(&env, "cat");
    let lang = String::from_str(&env, "en");

    client.create_document(
        &user1,
        &doc_id,
        &title,
        &content,
        &DocumentType::Guide,
        &category,
        &tags,
        &lang,
    );
    // Should panic with AlreadyExists
    client.create_document(
        &user1,
        &doc_id,
        &title,
        &content,
        &DocumentType::Guide,
        &category,
        &tags,
        &lang,
    );
}

#[test]
fn test_get_document() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-get");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Article,
        &String::from_str(&env, "general"),
        &tags,
        &String::from_str(&env, "en"),
    );

    let fetched = client.get_document(&doc_id);
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();
    assert_eq!(fetched.doc_id, doc_id);
}

#[test]
fn test_get_nonexistent_document() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let result = client.get_document(&String::from_str(&env, "no-exist"));
    assert!(result.is_none());
}

#[test]
fn test_update_document_title() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-upd");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Old Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    let updated = client.update_document(
        &user1,
        &doc_id,
        &Some(String::from_str(&env, "New Title")),
        &None,
        &None,
        &None,
    );

    assert_eq!(updated.title, String::from_str(&env, "New Title"));
    assert_eq!(updated.version, 1); // Title change doesn't bump version
}

#[test]
fn test_update_document_content_bumps_version() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-ver");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Old content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    let updated = client.update_document(
        &user1,
        &doc_id,
        &None,
        &Some(String::from_str(&env, "New content")),
        &None,
        &None,
    );

    assert_eq!(updated.version, 2);
    assert_eq!(updated.content, String::from_str(&env, "New content"));
}

#[test]
fn test_publish_document() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-pub");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.publish_document(&user1, &doc_id);

    let doc = client.get_document(&doc_id).unwrap();
    assert_eq!(doc.status, DocumentStatus::Published);
}

#[test]
fn test_view_document_increments_count() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-view");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.view_document(&doc_id);
    client.view_document(&doc_id);
    client.view_document(&doc_id);

    let doc = client.get_document(&doc_id).unwrap();
    assert_eq!(doc.view_count, 3);
    assert_eq!(client.get_total_views(), 3);
}

#[test]
fn test_mark_helpful() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-help");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.mark_helpful(&user2, &doc_id);

    let doc = client.get_document(&doc_id).unwrap();
    assert_eq!(doc.helpful_count, 1);
}

#[test]
fn test_get_documents_by_category() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let cat = String::from_str(&env, "blockchain");

    client.create_document(
        &user1,
        &String::from_str(&env, "d1"),
        &String::from_str(&env, "T1"),
        &String::from_str(&env, "C1"),
        &DocumentType::Guide,
        &cat,
        &tags,
        &String::from_str(&env, "en"),
    );
    client.create_document(
        &user1,
        &String::from_str(&env, "d2"),
        &String::from_str(&env, "T2"),
        &String::from_str(&env, "C2"),
        &DocumentType::Article,
        &cat,
        &tags,
        &String::from_str(&env, "en"),
    );

    let docs = client.get_documents_by_category(&cat);
    assert_eq!(docs.len(), 2);
}

#[test]
fn test_get_documents_by_author() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let cat = String::from_str(&env, "cat");

    client.create_document(
        &user1,
        &String::from_str(&env, "a1"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &DocumentType::Guide,
        &cat,
        &tags,
        &String::from_str(&env, "en"),
    );
    client.create_document(
        &user2,
        &String::from_str(&env, "a2"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &DocumentType::Guide,
        &cat,
        &tags,
        &String::from_str(&env, "en"),
    );

    let user1_docs = client.get_documents_by_author(&user1);
    let user2_docs = client.get_documents_by_author(&user2);
    assert_eq!(user1_docs.len(), 1);
    assert_eq!(user2_docs.len(), 1);
}

// ============================================================================
// Version Management Tests
// ============================================================================

#[test]
fn test_create_version() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-v");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "V1 content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    let version = client.create_version(
        &user1,
        &doc_id,
        &1,
        &String::from_str(&env, "Version 1 content"),
        &String::from_str(&env, "Initial release"),
    );

    assert_eq!(version.version_number, 1);
    assert!(version.is_current);
    assert_eq!(version.changelog, String::from_str(&env, "Initial release"));
}

#[test]
fn test_create_multiple_versions() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-mv");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.create_version(
        &user1,
        &doc_id,
        &1,
        &String::from_str(&env, "V1"),
        &String::from_str(&env, "First"),
    );

    let v2 = client.create_version(
        &user1,
        &doc_id,
        &2,
        &String::from_str(&env, "V2"),
        &String::from_str(&env, "Second"),
    );

    assert!(v2.is_current);

    // Previous version should no longer be current
    let v1 = client.get_version(&doc_id, &1).unwrap();
    assert!(!v1.is_current);
}

#[test]
fn test_get_version() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-gv");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.create_version(
        &user1,
        &doc_id,
        &1,
        &String::from_str(&env, "V1 content"),
        &String::from_str(&env, "changelog"),
    );

    let version = client.get_version(&doc_id, &1);
    assert!(version.is_some());

    let missing = client.get_version(&doc_id, &99);
    assert!(missing.is_none());
}

#[test]
fn test_get_current_version() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "doc-cv");

    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &DocumentType::Guide,
        &String::from_str(&env, "cat"),
        &tags,
        &String::from_str(&env, "en"),
    );

    client.create_version(
        &user1,
        &doc_id,
        &1,
        &String::from_str(&env, "V1"),
        &String::from_str(&env, "Log"),
    );

    let current = client.get_current_version(&doc_id);
    assert!(current.is_some());
    assert_eq!(current.unwrap().version_number, 1);
}

// ============================================================================
// Knowledge Base & FAQ Tests
// ============================================================================

#[test]
fn test_create_article() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "stellar"));

    let article = client.create_article(
        &user1,
        &String::from_str(&env, "art-1"),
        &String::from_str(&env, "Stellar Basics"),
        &String::from_str(&env, "Learn about Stellar"),
        &String::from_str(&env, "blockchain"),
        &tags,
    );

    assert_eq!(article.article_id, String::from_str(&env, "art-1"));
    assert_eq!(article.author, user1);
    assert_eq!(article.helpful_votes, 0);
    assert_eq!(article.not_helpful_votes, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_article() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let id = String::from_str(&env, "art-dup");

    client.create_article(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "cat"),
        &tags,
    );
    client.create_article(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "cat"),
        &tags,
    );
}

#[test]
fn test_create_faq() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let faq = client.create_faq(
        &user1,
        &String::from_str(&env, "faq-1"),
        &String::from_str(&env, "What is Soroban?"),
        &String::from_str(&env, "Soroban is a smart contract platform"),
        &String::from_str(&env, "general"),
        &1,
    );

    assert_eq!(faq.faq_id, String::from_str(&env, "faq-1"));
    assert_eq!(faq.order_index, 1);
    assert_eq!(faq.view_count, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_faq() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "faq-dup");

    client.create_faq(
        &user1,
        &id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &String::from_str(&env, "cat"),
        &1,
    );
    client.create_faq(
        &user1,
        &id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &String::from_str(&env, "cat"),
        &2,
    );
}

#[test]
fn test_vote_article_helpful() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let id = String::from_str(&env, "art-vote");

    client.create_article(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "cat"),
        &tags,
    );

    client.vote_article(&user2, &id, &true);

    let article = client.get_article(&id).unwrap();
    assert_eq!(article.helpful_votes, 1);
    assert_eq!(article.not_helpful_votes, 0);
}

#[test]
fn test_vote_article_not_helpful() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let id = String::from_str(&env, "art-down");

    client.create_article(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
        &String::from_str(&env, "cat"),
        &tags,
    );

    client.vote_article(&user2, &id, &false);

    let article = client.get_article(&id).unwrap();
    assert_eq!(article.helpful_votes, 0);
    assert_eq!(article.not_helpful_votes, 1);
}

#[test]
fn test_get_article() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let id = String::from_str(&env, "art-get");

    client.create_article(
        &user1,
        &id,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
        &String::from_str(&env, "cat"),
        &tags,
    );

    assert!(client.get_article(&id).is_some());
    assert!(client
        .get_article(&String::from_str(&env, "nope"))
        .is_none());
}

#[test]
fn test_get_faq() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "faq-get");

    client.create_faq(
        &user1,
        &id,
        &String::from_str(&env, "Q?"),
        &String::from_str(&env, "A."),
        &String::from_str(&env, "cat"),
        &0,
    );

    assert!(client.get_faq(&id).is_some());
    assert!(client.get_faq(&String::from_str(&env, "nope")).is_none());
}

// ============================================================================
// API Documentation Tests
// ============================================================================

#[test]
fn test_create_api_endpoint() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let mut params = Vec::new(&env);
    params.push_back(ApiParameter {
        name: String::from_str(&env, "course_id"),
        param_type: String::from_str(&env, "string"),
        required: true,
        description: String::from_str(&env, "The course identifier"),
        default_value: None,
    });

    let endpoint = client.create_api_endpoint(
        &admin,
        &String::from_str(&env, "ep-1"),
        &String::from_str(&env, "Get Course"),
        &String::from_str(&env, "Retrieve a course by ID"),
        &String::from_str(&env, "GET"),
        &String::from_str(&env, "/api/v1/courses/{id}"),
        &params,
        &String::from_str(&env, "{\"id\":\"string\",\"title\":\"string\"}"),
        &String::from_str(&env, "1.0.0"),
    );

    assert_eq!(endpoint.endpoint_id, String::from_str(&env, "ep-1"));
    assert_eq!(endpoint.code_examples.len(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_api_endpoint() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let params = Vec::new(&env);
    let id = String::from_str(&env, "ep-dup");

    client.create_api_endpoint(
        &admin,
        &id,
        &String::from_str(&env, "N"),
        &String::from_str(&env, "D"),
        &String::from_str(&env, "GET"),
        &String::from_str(&env, "/path"),
        &params,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "1.0"),
    );
    client.create_api_endpoint(
        &admin,
        &id,
        &String::from_str(&env, "N"),
        &String::from_str(&env, "D"),
        &String::from_str(&env, "GET"),
        &String::from_str(&env, "/path"),
        &params,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "1.0"),
    );
}

#[test]
fn test_add_code_example_to_endpoint() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let params = Vec::new(&env);
    let ep_id = String::from_str(&env, "ep-ex");

    client.create_api_endpoint(
        &admin,
        &ep_id,
        &String::from_str(&env, "Endpoint"),
        &String::from_str(&env, "Desc"),
        &String::from_str(&env, "POST"),
        &String::from_str(&env, "/api/data"),
        &params,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "1.0"),
    );

    let example = CodeExample {
        example_id: String::from_str(&env, "ex-1"),
        title: String::from_str(&env, "cURL Example"),
        code: String::from_str(&env, "curl -X POST /api/data"),
        language: String::from_str(&env, "bash"),
        description: String::from_str(&env, "Using cURL"),
    };

    client.add_code_example(&admin, &ep_id, &example);

    let ep = client.get_api_endpoint(&ep_id).unwrap();
    assert_eq!(ep.code_examples.len(), 1);
}

#[test]
fn test_get_api_endpoint() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let params = Vec::new(&env);
    let id = String::from_str(&env, "ep-get");

    client.create_api_endpoint(
        &admin,
        &id,
        &String::from_str(&env, "N"),
        &String::from_str(&env, "D"),
        &String::from_str(&env, "GET"),
        &String::from_str(&env, "/p"),
        &params,
        &String::from_str(&env, "{}"),
        &String::from_str(&env, "1.0"),
    );

    assert!(client.get_api_endpoint(&id).is_some());
    assert!(client
        .get_api_endpoint(&String::from_str(&env, "nope"))
        .is_none());
}

// ============================================================================
// Tutorial Tests
// ============================================================================

#[test]
fn test_create_tutorial() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let mut steps = Vec::new(&env);
    steps.push_back(TutorialStep {
        step_number: 1,
        title: String::from_str(&env, "Setup"),
        content: String::from_str(&env, "Install Soroban CLI"),
        code_snippet: Some(String::from_str(&env, "cargo install soroban-cli")),
        validation_criteria: None,
    });
    steps.push_back(TutorialStep {
        step_number: 2,
        title: String::from_str(&env, "Create Project"),
        content: String::from_str(&env, "Initialize a new project"),
        code_snippet: Some(String::from_str(&env, "soroban init my-project")),
        validation_criteria: Some(String::from_str(&env, "Project directory created")),
    });

    let mut prereqs = Vec::new(&env);
    prereqs.push_back(String::from_str(&env, "Rust basics"));

    let tutorial = client.create_tutorial(
        &user1,
        &String::from_str(&env, "tut-1"),
        &String::from_str(&env, "Build Your First Contract"),
        &String::from_str(&env, "Learn to build Soroban contracts"),
        &DifficultyLevel::Beginner,
        &30,
        &steps,
        &prereqs,
    );

    assert_eq!(tutorial.tutorial_id, String::from_str(&env, "tut-1"));
    assert_eq!(tutorial.difficulty, DifficultyLevel::Beginner);
    assert_eq!(tutorial.estimated_time, 30);
    assert_eq!(tutorial.steps.len(), 2);
    assert_eq!(tutorial.prerequisites.len(), 1);
    assert_eq!(tutorial.completion_count, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_tutorial() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let steps = Vec::new(&env);
    let prereqs = Vec::new(&env);
    let id = String::from_str(&env, "tut-dup");

    client.create_tutorial(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "D"),
        &DifficultyLevel::Beginner,
        &10,
        &steps,
        &prereqs,
    );
    client.create_tutorial(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "D"),
        &DifficultyLevel::Beginner,
        &10,
        &steps,
        &prereqs,
    );
}

#[test]
fn test_complete_tutorial() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let steps = Vec::new(&env);
    let prereqs = Vec::new(&env);
    let id = String::from_str(&env, "tut-comp");

    client.create_tutorial(
        &user1,
        &id,
        &String::from_str(&env, "Tutorial"),
        &String::from_str(&env, "Desc"),
        &DifficultyLevel::Intermediate,
        &60,
        &steps,
        &prereqs,
    );

    client.complete_tutorial(&user2, &id);
    client.complete_tutorial(&user1, &id);

    let tut = client.get_tutorial(&id).unwrap();
    assert_eq!(tut.completion_count, 2);
}

#[test]
fn test_get_tutorial() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let steps = Vec::new(&env);
    let prereqs = Vec::new(&env);
    let id = String::from_str(&env, "tut-get");

    client.create_tutorial(
        &user1,
        &id,
        &String::from_str(&env, "T"),
        &String::from_str(&env, "D"),
        &DifficultyLevel::Advanced,
        &120,
        &steps,
        &prereqs,
    );

    assert!(client.get_tutorial(&id).is_some());
    assert!(client
        .get_tutorial(&String::from_str(&env, "nope"))
        .is_none());
}

// ============================================================================
// Community Contribution Tests
// ============================================================================

#[test]
fn test_submit_contribution() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let contribution = client.submit_contribution(
        &user1,
        &String::from_str(&env, "contrib-1"),
        &String::from_str(&env, "doc-1"),
        &ContributionType::NewDocument,
        &String::from_str(&env, "New guide for beginners"),
    );

    assert_eq!(
        contribution.contribution_id,
        String::from_str(&env, "contrib-1")
    );
    assert_eq!(contribution.contributor, user1);
    assert_eq!(contribution.status, ContributionStatus::Pending);
    assert!(contribution.reviewed_by.is_none());
    assert_eq!(client.get_total_contributions(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_submit_duplicate_contribution() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-dup");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::Edit,
        &String::from_str(&env, "content"),
    );
    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::Edit,
        &String::from_str(&env, "content"),
    );
}

#[test]
fn test_review_contribution_approve() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-rev");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::Correction,
        &String::from_str(&env, "Fix typo"),
    );

    client.review_contribution(
        &admin,
        &id,
        &ContributionStatus::Approved,
        &Some(String::from_str(&env, "Looks good!")),
    );

    let c = client.get_contribution(&id).unwrap();
    assert_eq!(c.status, ContributionStatus::Approved);
    assert_eq!(c.reviewed_by, Some(admin));
    assert!(c.review_notes.is_some());
}

#[test]
fn test_review_contribution_reject() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-rej");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::Translation,
        &String::from_str(&env, "Bad translation"),
    );

    client.review_contribution(
        &admin,
        &id,
        &ContributionStatus::Rejected,
        &Some(String::from_str(&env, "Inaccurate")),
    );

    let c = client.get_contribution(&id).unwrap();
    assert_eq!(c.status, ContributionStatus::Rejected);
}

#[test]
fn test_review_contribution_needs_revision() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-nr");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::CodeExample,
        &String::from_str(&env, "Sample code"),
    );

    client.review_contribution(
        &admin,
        &id,
        &ContributionStatus::NeedsRevision,
        &Some(String::from_str(&env, "Add error handling")),
    );

    let c = client.get_contribution(&id).unwrap();
    assert_eq!(c.status, ContributionStatus::NeedsRevision);
}

#[test]
fn test_review_with_no_notes() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-nn");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::Edit,
        &String::from_str(&env, "content"),
    );

    client.review_contribution(&admin, &id, &ContributionStatus::Approved, &None);

    let c = client.get_contribution(&id).unwrap();
    assert_eq!(c.status, ContributionStatus::Approved);
    assert!(c.review_notes.is_none());
}

#[test]
fn test_get_contribution() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "contrib-g");

    client.submit_contribution(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &ContributionType::NewDocument,
        &String::from_str(&env, "c"),
    );

    assert!(client.get_contribution(&id).is_some());
    assert!(client
        .get_contribution(&String::from_str(&env, "nope"))
        .is_none());
}

// ============================================================================
// Translation Tests
// ============================================================================

#[test]
fn test_create_translation() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let translation = client.create_translation(
        &user1,
        &String::from_str(&env, "trans-1"),
        &String::from_str(&env, "doc-1"),
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Primeros Pasos"),
        &String::from_str(&env, "Bienvenido a la guia"),
    );

    assert_eq!(
        translation.translation_id,
        String::from_str(&env, "trans-1")
    );
    assert_eq!(translation.language, String::from_str(&env, "es"));
    assert_eq!(translation.status, TranslationStatus::InProgress);
    assert_eq!(translation.translator, user1);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_create_duplicate_translation() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "trans-dup");

    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "fr"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
    );
    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "fr"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
    );
}

#[test]
fn test_update_translation_status_to_review() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "trans-rev");

    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "de"),
        &String::from_str(&env, "Titel"),
        &String::from_str(&env, "Inhalt"),
    );

    client.update_translation_status(&admin, &id, &TranslationStatus::Review);

    let t = client.get_translation(&id).unwrap();
    assert_eq!(t.status, TranslationStatus::Review);
}

#[test]
fn test_update_translation_status_to_published() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "trans-pub");

    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "ja"),
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Content"),
    );

    client.update_translation_status(&admin, &id, &TranslationStatus::Published);

    let t = client.get_translation(&id).unwrap();
    assert_eq!(t.status, TranslationStatus::Published);
}

#[test]
fn test_update_translation_status_to_outdated() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "trans-out");

    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "pt"),
        &String::from_str(&env, "Titulo"),
        &String::from_str(&env, "Conteudo"),
    );

    client.update_translation_status(&admin, &id, &TranslationStatus::Outdated);

    let t = client.get_translation(&id).unwrap();
    assert_eq!(t.status, TranslationStatus::Outdated);
}

#[test]
fn test_get_translation() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let id = String::from_str(&env, "trans-g");

    client.create_translation(
        &user1,
        &id,
        &String::from_str(&env, "doc"),
        &String::from_str(&env, "zh"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "C"),
    );

    assert!(client.get_translation(&id).is_some());
    assert!(client
        .get_translation(&String::from_str(&env, "nope"))
        .is_none());
}

// ============================================================================
// Analytics Tests
// ============================================================================

#[test]
fn test_track_search() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let search = client.track_search(
        &user1,
        &String::from_str(&env, "sq-1"),
        &String::from_str(&env, "how to deploy"),
        &5,
    );

    assert_eq!(search.query_id, String::from_str(&env, "sq-1"));
    assert_eq!(search.query_text, String::from_str(&env, "how to deploy"));
    assert_eq!(search.results_count, 5);
    assert!(search.clicked_result.is_none());
}

#[test]
fn test_get_document_analytics() {
    let (env, admin, _, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    // No analytics yet for a random doc
    let result = client.get_document_analytics(&String::from_str(&env, "doc-x"));
    assert!(result.is_none());
}

#[test]
fn test_total_counters_increment() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);

    // Create 3 documents
    for i in 0..3 {
        let id_str = match i {
            0 => "ctr-0",
            1 => "ctr-1",
            _ => "ctr-2",
        };
        client.create_document(
            &user1,
            &String::from_str(&env, id_str),
            &String::from_str(&env, "T"),
            &String::from_str(&env, "C"),
            &DocumentType::Article,
            &String::from_str(&env, "cat"),
            &tags,
            &String::from_str(&env, "en"),
        );
    }

    assert_eq!(client.get_total_documents(), 3);

    // Submit 2 contributions
    for i in 0..2 {
        let id_str = match i {
            0 => "cnt-0",
            _ => "cnt-1",
        };
        client.submit_contribution(
            &user1,
            &String::from_str(&env, id_str),
            &String::from_str(&env, "doc"),
            &ContributionType::Edit,
            &String::from_str(&env, "c"),
        );
    }

    assert_eq!(client.get_total_contributions(), 2);
}

// ============================================================================
// All Document Types Test
// ============================================================================

#[test]
fn test_all_document_types() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let cat = String::from_str(&env, "cat");
    let lang = String::from_str(&env, "en");

    let types = [
        ("dt-guide", DocumentType::Guide),
        ("dt-tut", DocumentType::Tutorial),
        ("dt-api", DocumentType::ApiDoc),
        ("dt-ref", DocumentType::Reference),
        ("dt-faq", DocumentType::FAQ),
        ("dt-art", DocumentType::Article),
        ("dt-vid", DocumentType::Video),
        ("dt-code", DocumentType::CodeExample),
    ];

    for (id, doc_type) in types.iter() {
        let doc = client.create_document(
            &user1,
            &String::from_str(&env, id),
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Content"),
            doc_type,
            &cat,
            &tags,
            &lang,
        );
        assert_eq!(doc.doc_type, *doc_type);
    }

    assert_eq!(client.get_total_documents(), 8);
}

// ============================================================================
// Difficulty Level Tests
// ============================================================================

#[test]
fn test_all_difficulty_levels() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let steps = Vec::new(&env);
    let prereqs = Vec::new(&env);

    let levels = [
        ("dl-beg", DifficultyLevel::Beginner),
        ("dl-int", DifficultyLevel::Intermediate),
        ("dl-adv", DifficultyLevel::Advanced),
        ("dl-exp", DifficultyLevel::Expert),
    ];

    for (id, level) in levels.iter() {
        let tut = client.create_tutorial(
            &user1,
            &String::from_str(&env, id),
            &String::from_str(&env, "T"),
            &String::from_str(&env, "D"),
            level,
            &60,
            &steps,
            &prereqs,
        );
        assert_eq!(tut.difficulty, *level);
    }
}

// ============================================================================
// End-to-End Integration Tests
// ============================================================================

#[test]
fn test_full_document_lifecycle() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "lifecycle");

    // 1. Create document
    let doc = client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Draft Guide"),
        &String::from_str(&env, "Initial content"),
        &DocumentType::Guide,
        &String::from_str(&env, "guides"),
        &tags,
        &String::from_str(&env, "en"),
    );
    assert_eq!(doc.status, DocumentStatus::Draft);

    // 2. Update content
    let updated = client.update_document(
        &user1,
        &doc_id,
        &Some(String::from_str(&env, "Polished Guide")),
        &Some(String::from_str(&env, "Better content")),
        &None,
        &None,
    );
    assert_eq!(updated.version, 2);

    // 3. Create version snapshot
    client.create_version(
        &user1,
        &doc_id,
        &1,
        &String::from_str(&env, "V1"),
        &String::from_str(&env, "Initial"),
    );

    // 4. Publish
    client.publish_document(&user1, &doc_id);
    let published = client.get_document(&doc_id).unwrap();
    assert_eq!(published.status, DocumentStatus::Published);

    // 5. Users view and rate
    client.view_document(&doc_id);
    client.view_document(&doc_id);
    client.mark_helpful(&user2, &doc_id);

    let final_doc = client.get_document(&doc_id).unwrap();
    assert_eq!(final_doc.view_count, 2);
    assert_eq!(final_doc.helpful_count, 1);

    // 6. Submit translation
    client.create_translation(
        &user2,
        &String::from_str(&env, "t-lifecycle"),
        &doc_id,
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Guia Pulida"),
        &String::from_str(&env, "Mejor contenido"),
    );

    // 7. Community contribution
    client.submit_contribution(
        &user2,
        &String::from_str(&env, "c-lifecycle"),
        &doc_id,
        &ContributionType::Correction,
        &String::from_str(&env, "Fixed grammar"),
    );

    client.review_contribution(
        &admin,
        &String::from_str(&env, "c-lifecycle"),
        &ContributionStatus::Approved,
        &None,
    );

    assert_eq!(client.get_total_documents(), 1);
    assert_eq!(client.get_total_views(), 2);
    assert_eq!(client.get_total_contributions(), 1);
}

#[test]
fn test_multi_language_documentation() {
    let (env, admin, user1, user2) = create_test_env();
    let client = setup_contract(&env, &admin);

    let tags = Vec::new(&env);
    let doc_id = String::from_str(&env, "ml-doc");

    // Create original in English
    client.create_document(
        &user1,
        &doc_id,
        &String::from_str(&env, "Getting Started"),
        &String::from_str(&env, "Welcome"),
        &DocumentType::Guide,
        &String::from_str(&env, "onboarding"),
        &tags,
        &String::from_str(&env, "en"),
    );

    // Translate to Spanish
    let es = client.create_translation(
        &user2,
        &String::from_str(&env, "ml-es"),
        &doc_id,
        &String::from_str(&env, "es"),
        &String::from_str(&env, "Primeros Pasos"),
        &String::from_str(&env, "Bienvenido"),
    );
    assert_eq!(es.status, TranslationStatus::InProgress);

    // Translate to French
    let fr = client.create_translation(
        &user2,
        &String::from_str(&env, "ml-fr"),
        &doc_id,
        &String::from_str(&env, "fr"),
        &String::from_str(&env, "Pour Commencer"),
        &String::from_str(&env, "Bienvenue"),
    );
    assert_eq!(fr.language, String::from_str(&env, "fr"));

    // Publish Spanish translation
    client.update_translation_status(
        &admin,
        &String::from_str(&env, "ml-es"),
        &TranslationStatus::Published,
    );

    let published_es = client
        .get_translation(&String::from_str(&env, "ml-es"))
        .unwrap();
    assert_eq!(published_es.status, TranslationStatus::Published);
}

#[test]
fn test_contribution_types_coverage() {
    let (env, admin, user1, _) = create_test_env();
    let client = setup_contract(&env, &admin);

    let types = [
        ("ct-new", ContributionType::NewDocument),
        ("ct-edit", ContributionType::Edit),
        ("ct-trans", ContributionType::Translation),
        ("ct-code", ContributionType::CodeExample),
        ("ct-corr", ContributionType::Correction),
    ];

    for (id, ct) in types.iter() {
        let c = client.submit_contribution(
            &user1,
            &String::from_str(&env, id),
            &String::from_str(&env, "doc"),
            ct,
            &String::from_str(&env, "content"),
        );
        assert_eq!(c.contribution_type, *ct);
        assert_eq!(c.status, ContributionStatus::Pending);
    }

    assert_eq!(client.get_total_contributions(), 5);
}
