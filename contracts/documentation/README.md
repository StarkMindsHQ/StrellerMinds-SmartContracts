# Documentation Contract

## Overview

The `documentation` contract provides on-chain documentation management for articles, tutorials, API docs, translations, versioning, contributions, and usage analytics.

## Quick Start

```bash
cargo test -p documentation
```

Initialize the contract with an admin, then create documents, tutorials, API docs, and translation records through the public entrypoints in [lib.rs](./src/lib.rs).

## Usage Examples

```rust
// Initialize the documentation contract
DocumentationContract::initialize(env, admin)?;

// Create a document
DocumentationContract::create_document(
    env,
    author,
    doc_id,
    title,
    content,
    doc_type,
    category,
    tags,
    language,
)?;

// Publish and view a document
DocumentationContract::publish_document(env, author, doc_id)?;
DocumentationContract::view_document(env, doc_id)?;
```

## Contribution Guide

- Keep additions scoped to documentation workflows, storage, analytics, or translation management.
- Maintain compatibility with existing `DocumentationError` variants and stored `DataKey` values.
- Add tests in `src/tests.rs` when introducing new document lifecycle or contribution flows.

## Troubleshooting

- `NotInitialized`: initialize the contract before creating or reading configuration-driven records.
- `DocumentNotFound`: verify the document identifier and ensure the item was created in the expected environment.
- `DocumentTooLarge`: reduce payload size or update the configured document limit through the admin flow.

## Related Files

- `src/lib.rs`: main contract implementation
- `src/documents.rs`: document lifecycle logic
- `src/translations.rs`: translation management
- `src/tests.rs`: comprehensive tests
