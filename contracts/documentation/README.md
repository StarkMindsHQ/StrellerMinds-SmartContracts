# Documentation Contract

## Purpose

The Documentation contract is the on-chain content management system for the StrellerMinds platform. It stores and versions instructional documents, knowledge base articles, FAQs, step-by-step tutorials, and API endpoint specifications. Community members can submit contributions (edits, corrections, additions) for reviewer approval, translators can create language variants of any document, and an analytics layer tracks views, searches, and helpfulness votes. The result is a fully auditable, community-driven documentation hub with no off-chain dependency for core content operations.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — delegates to domain manager structs; exposes the full public API |
| `src/documents.rs` (`DocumentManager`) | CRUD for `Document` records: create, update, publish, view, mark-helpful, query by category/author |
| `src/versions.rs` (`VersionManager`) | Immutable version snapshots of documents with changelog strings |
| `src/knowledge.rs` (`KnowledgeManager`) | `KnowledgeArticle` and `FAQ` management with helpfulness voting |
| `src/api_docs.rs` (`ApiDocManager`) | `ApiEndpoint` registry with parameter specs, response schemas, and `CodeExample` attachment |
| `src/tutorials.rs` (`TutorialManager`) | `Tutorial` creation with ordered `TutorialStep` lists, difficulty levels, and completion tracking |
| `src/contributions.rs` (`ContributionManager`) | Community contribution submission and reviewer approval workflow |
| `src/translations.rs` (`TranslationManager`) | Document translations with language codes and admin-controlled status updates |
| `src/analytics.rs` (`AnalyticsManager`) | Search query tracking, per-document view/helpfulness aggregation |
| `src/storage.rs` (`Storage`) | Shared storage accessor helpers and counter utilities |
| `src/types.rs` | All data types: `Document`, `DocumentVersion`, `KnowledgeArticle`, `FAQ`, `ApiEndpoint`, `Tutorial`, `Contribution`, `Translation`, `DocumentationConfig`, etc. |
| `src/errors.rs` | `DocumentationError` enum (aliased as `Error`) |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; persists config and counters | Yes — `admin` |
| `get_config()` | Returns the current `DocumentationConfig` | No |
| `create_document(author, doc_id, title, content, doc_type, category, tags, language)` | Creates a draft document | Yes — `author` |
| `update_document(author, doc_id, title?, content?, status?, tags?)` | Updates fields of an existing document | Yes — `author` |
| `publish_document(author, doc_id)` | Transitions a draft document to published | Yes — `author` |
| `get_document(doc_id)` | Returns the `Document`, or `None` | No |
| `view_document(doc_id)` | Increments the document's view counter | No |
| `mark_helpful(user, doc_id)` | Increments the helpfulness counter | Yes — `user` |
| `get_documents_by_category(category)` | Lists document IDs in a category | No |
| `get_documents_by_author(author)` | Lists document IDs authored by `author` | No |
| `create_version(author, doc_id, version_number, content, changelog)` | Snapshots a new document version | Yes — `author` |
| `get_version(doc_id, version_number)` | Returns a specific `DocumentVersion`, or `None` | No |
| `get_current_version(doc_id)` | Returns the latest `DocumentVersion`, or `None` | No |
| `create_article(author, article_id, title, content, category, tags)` | Creates a `KnowledgeArticle` | Yes — `author` |
| `create_faq(author, faq_id, question, answer, category, order_index)` | Creates an `FAQ` entry | Yes — `author` |
| `vote_article(user, article_id, is_helpful)` | Submits a helpfulness vote on an article | Yes — `user` |
| `get_article(article_id)` | Returns the `KnowledgeArticle`, or `None` | No |
| `get_faq(faq_id)` | Returns the `FAQ` entry, or `None` | No |
| `create_api_endpoint(admin, endpoint_id, name, description, method, path, parameters, response_schema, version)` | Registers a new API endpoint entry | Yes — `admin` |
| `add_code_example(admin, endpoint_id, example)` | Attaches a `CodeExample` to an endpoint | Yes — `admin` |
| `get_api_endpoint(endpoint_id)` | Returns the `ApiEndpoint`, or `None` | No |
| `create_tutorial(author, tutorial_id, title, description, difficulty, estimated_time, steps, prerequisites)` | Creates a `Tutorial` | Yes — `author` |
| `complete_tutorial(user, tutorial_id)` | Records tutorial completion for `user` | Yes — `user` |
| `get_tutorial(tutorial_id)` | Returns the `Tutorial`, or `None` | No |
| `submit_contribution(contributor, contribution_id, doc_id, contribution_type, content)` | Submits a community contribution pending review | Yes — `contributor` |
| `review_contribution(reviewer, contribution_id, status, notes?)` | Approves or rejects a contribution | Yes — `reviewer` |
| `get_contribution(contribution_id)` | Returns the `Contribution`, or `None` | No |
| `create_translation(translator, translation_id, original_doc_id, language, title, content)` | Creates a document translation | Yes — `translator` |
| `update_translation_status(admin, translation_id, status)` | Approves or rejects a translation | Yes — `admin` |
| `get_translation(translation_id)` | Returns the `Translation`, or `None` | No |
| `track_search(user, query_id, query_text, results_count)` | Records a search analytics event | Yes — `user` |
| `get_document_analytics(doc_id)` | Returns aggregated `DocumentAnalytics`, or `None` | No |
| `get_total_documents()` | Returns the total document count | No |
| `get_total_views()` | Returns the total view count across all documents | No |
| `get_total_contributions()` | Returns the total contribution count | No |

## Usage Example

```text
# Initialize
documentation.initialize(admin_address)

# Author creates a draft guide
doc = documentation.create_document(
    author_address, "guide-soroban-basics", "Soroban Basics",
    "Full content here...", DocumentType::Guide,
    "smart-contracts", ["soroban", "stellar"], "en"
)

# Author publishes the document
documentation.publish_document(author_address, "guide-soroban-basics")

# Reader views the document; counter increments
documentation.view_document("guide-soroban-basics")

# Reader marks it helpful
documentation.mark_helpful(reader_address, "guide-soroban-basics")

# Translator adds a Spanish version
documentation.create_translation(
    translator_address, "guide-soroban-basics-es",
    "guide-soroban-basics", "es", "Conceptos Básicos de Soroban", "Contenido..."
)

# Admin approves the translation
documentation.update_translation_status(
    admin_address, "guide-soroban-basics-es", TranslationStatus::Approved
)

# Community member submits a correction
documentation.submit_contribution(
    contributor_address, "contrib-001", "guide-soroban-basics",
    ContributionType::Correction, "Fixed typo in section 3..."
)

# Reviewer approves the contribution
documentation.review_contribution(
    reviewer_address, "contrib-001", ContributionStatus::Approved, None
)
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `NotInitialized` | Contract has not been initialized yet |
| 2 | `AlreadyInitialized` | `initialize` has already been called |
| 3 | `Unauthorized` | Caller does not have the required permissions |
| 4 | `DocumentNotFound` | No document exists with the supplied ID |
| 5 | `InvalidDocument` | Document data is malformed or fails validation |
| 6 | `VersionNotFound` | The requested document version does not exist |
| 7 | `ContributionNotFound` | No contribution found with the supplied ID |
| 8 | `InvalidContribution` | Contribution data is malformed or fails validation |
| 9 | `TranslationNotFound` | No translation found with the supplied ID |
| 10 | `InvalidLanguage` | Language code is not recognized or supported |
| 11 | `DocumentTooLarge` | Document content exceeds the configured size limit |
| 12 | `InvalidStatus` | The requested status transition is not valid |
| 13 | `AlreadyExists` | A document, article, FAQ, tutorial, or contribution with this ID already exists |

## Integration

| Contract | Relationship |
|---|---|
| `search` | The Search contract indexes documentation articles, FAQs, and tutorials for platform-wide discovery |
| `analytics` | Document view and search events feed platform engagement metrics |
| `community` | Community contributors who submit documentation improvements may earn gamification rewards |
| `shared` | Storage conventions, RBAC patterns, and `DataKey` conventions shared across the suite |
