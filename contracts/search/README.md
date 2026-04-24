# Search Contract

## Purpose

The Search contract (`AdvancedSearchContract`) is the intelligent content discovery engine for the StrellerMinds platform. It combines semantic NLP search, collaborative filtering, visual similarity, multilingual support, voice query processing, and personalized learning path optimization into a single on-chain contract. Because heavy compute (ML embeddings, NLP parsing, image processing) cannot run inside a WASM contract, the contract integrates with a set of authorized off-chain oracles that supply pre-computed metadata; the contract stores and indexes that data and enforces access control on all oracle write operations.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — orchestrates all sub-engines and oracle management |
| `src/semantic_search.rs` | NLP-enhanced query understanding and semantic content retrieval |
| `src/recommendation_engine.rs` | Personalized recommendation generation and user profile management |
| `src/content_analyzer.rs` | Tag- and skill-based content indexing and look-up |
| `src/collaborative_filter.rs` | User similarity scoring and interaction-based recommendations |
| `src/visual_search.rs` | Visual metadata storage and image-similarity content retrieval |
| `src/learning_path_optimizer.rs` | Personalized learning path storage, step completion, and next-step retrieval |
| `src/ranking_engine.rs` | Multi-signal result ranking with configurable weights |
| `src/multilingual_search.rs` | Multilingual content storage and language-preference-aware retrieval |
| `src/voice_search.rs` | Voice query storage and conversation session management |
| `src/search_analytics.rs` | Search event recording, CTR tracking, and quality scoring |
| `src/types.rs` | All shared data types (`SearchResultItem`, `Recommendation`, `RankingConfig`, `LearningPath`, etc.) |
| `src/errors.rs` | `SearchError` enum (aliased as `Error`) |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; sets admin and default ranking config | Yes — `admin` |
| `semantic_search(query, user, filters)` | Executes an NLP-enhanced search; returns ranked `SearchResultItem` list | No |
| `store_semantic_metadata(oracle, content_id, metadata)` | Stores pre-computed NLP metadata from an authorized oracle | Yes — oracle |
| `get_recommendations(user, limit)` | Returns personalized content recommendations for `user` | Yes — `user` |
| `store_recommendations(oracle, user, recommendations)` | Stores ML-generated recommendations from an authorized oracle | Yes — oracle |
| `update_user_profile(user, completed_course, completed)` | Updates `user`'s profile with a completed (or uncompleted) course | Yes — `user` |
| `store_content_analysis(oracle, content_id, analysis)` | Stores content analysis data from an authorized oracle | Yes — oracle |
| `get_content_analysis(content_id)` | Returns the stored content analysis for `content_id` | No |
| `find_by_tag(tag)` | Returns content IDs associated with `tag` | No |
| `find_by_skill(skill_name)` | Returns content IDs associated with `skill_name` | No |
| `store_user_similarity(oracle, user_a, user_b, score)` | Stores a collaborative-filter similarity score between two users | Yes — oracle |
| `record_interaction(user, interaction)` | Records a user interaction event for collaborative filtering | Yes — `user` |
| `get_collab_recommendations(user, limit)` | Returns collaborative-filter-based recommendations for `user` | Yes — `user` |
| `store_visual_metadata(oracle, content_id, metadata)` | Stores image-processing metadata from an authorized oracle | Yes — oracle |
| `find_visually_similar(content_id, min_score, limit)` | Returns content IDs visually similar to `content_id` | No |
| `find_by_color(color_hex)` | Returns content IDs matching a dominant color | No |
| `find_by_object(object_type)` | Returns content IDs containing a detected object type | No |
| `store_learning_path(oracle, user, path)` | Stores an optimized learning path for `user` from an authorized oracle | Yes — oracle |
| `get_learning_path(user)` | Returns `user`'s current learning path | Yes — `user` |
| `complete_path_step(user, step_id, completion_score)` | Records completion of a learning path step | Yes — `user` |
| `get_next_step(user)` | Returns the next recommended step in `user`'s learning path | Yes — `user` |
| `rank_results(results, user)` | Ranks a list of content IDs using the multi-signal ranking engine | No |
| `update_ranking_config(admin, config)` | Updates signal weights for the ranking engine | Yes — admin |
| `store_multilingual_content(oracle, content_id, multilingual)` | Stores multilingual translations from an authorized oracle | Yes — oracle |
| `set_language_preferences(user, preferences)` | Stores `user`'s language preferences | Yes — `user` |
| `search_by_language(language, query)` | Returns content IDs matching `query` in the specified language | No |
| `record_search(user, query, results_count)` | Records a search event for analytics purposes | No |
| `record_click(user, query, content_id, rank_position)` | Records a click event for CTR tracking | No |
| `get_ctr(query, content_id)` | Returns the click-through rate for a query/content pair | No |
| `get_search_quality_score(query)` | Returns the computed quality score for a query | No |
| `store_voice_query(oracle, user, processed_query)` | Stores a processed voice query from an authorized oracle | Yes — oracle |
| `create_conversation_session(user)` | Creates a new voice conversation session for `user`; returns session ID | Yes — `user` |
| `get_conversation_session(session_id)` | Returns the conversation session record | No |
| `end_conversation_session(user)` | Ends `user`'s active voice conversation session | Yes — `user` |
| `authorize_oracle(admin, oracle)` | Adds `oracle` to the authorized oracle set | Yes — admin |
| `revoke_oracle(admin, oracle)` | Removes `oracle` from the authorized oracle set | Yes — admin |

## Usage Example

```text
# Initialize contract
search.initialize(admin_address)

# Admin authorizes an off-chain NLP oracle
search.authorize_oracle(admin_address, nlp_oracle_address)

# Oracle stores pre-computed semantic metadata for a course
search.store_semantic_metadata(nlp_oracle_address, "course-rust-101", semantic_metadata)

# Student searches for Rust content
results = search.semantic_search(processed_query, student_address, search_filters)

# Record the search event and a click for analytics
search.record_search(student_address, "learn rust", results.len())
search.record_click(student_address, "learn rust", "course-rust-101", 0)

# Student requests personalized recommendations
recs = search.get_recommendations(student_address, 5)

# Student works through their learning path
step = search.get_next_step(student_address)
search.complete_path_step(student_address, step.id, 95)
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 3 | `Unauthorized` | Caller is not the admin |
| 4 | `InvalidQuery` | Search query is malformed or empty |
| 5 | `ContentNotFound` | Requested content ID does not exist |
| 6 | `InvalidMetadata` | Supplied semantic or visual metadata is invalid |
| 7 | `InvalidScore` | Similarity or quality score is out of valid range |
| 8 | `SessionExpired` | Conversation session has expired or does not exist |
| 9 | `InvalidLanguage` | Language code is not recognized or supported |
| 10 | `OracleNotAuthorized` | Oracle address is not in the authorized oracle list |

## Integration

| Contract | Relationship |
|---|---|
| `documentation` | Search indexes documentation articles and knowledge base entries |
| `analytics` | Search analytics events (CTR, quality scores) feed platform-wide engagement metrics |
| `student-progress-tracker` | Learning path steps map to course modules tracked by the progress tracker |
| `shared` | RBAC and storage conventions shared across all contracts |
