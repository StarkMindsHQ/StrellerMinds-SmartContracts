use crate::storage::Storage;
use crate::types::*;
use soroban_sdk::{Address, Env, String, Vec};

pub struct DocumentManager;

impl DocumentManager {
    pub fn create_document(
        env: &Env,
        doc_id: String,
        title: String,
        content: String,
        doc_type: DocumentType,
        category: String,
        author: &Address,
        tags: Vec<String>,
        language: String,
    ) -> Result<Document, Error> {
        if env
            .storage()
            .persistent()
            .has(&DataKey::Document(doc_id.clone()))
        {
            return Err(Error::AlreadyExists);
        }

        let now = env.ledger().timestamp();
        let document = Document {
            doc_id: doc_id.clone(),
            title,
            content,
            doc_type,
            category: category.clone(),
            author: author.clone(),
            version: 1,
            status: DocumentStatus::Draft,
            created_at: now,
            updated_at: now,
            tags,
            language,
            parent_id: None,
            view_count: 0,
            helpful_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Document(doc_id.clone()), &document);
        Storage::add_to_category(env, &category, &doc_id);
        Storage::add_to_author_docs(env, author, &doc_id);
        Storage::increment_counter(env, &DataKey::TotalDocuments);

        Ok(document)
    }

    pub fn update_document(
        env: &Env,
        doc_id: String,
        title: Option<String>,
        content: Option<String>,
        status: Option<DocumentStatus>,
        tags: Option<Vec<String>>,
    ) -> Result<Document, Error> {
        let mut doc: Document = env
            .storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        if let Some(t) = title {
            doc.title = t;
        }
        if let Some(c) = content {
            doc.content = c;
            doc.version += 1;
        }
        if let Some(s) = status {
            doc.status = s;
        }
        if let Some(t) = tags {
            doc.tags = t;
        }

        doc.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Document(doc_id), &doc);

        Ok(doc)
    }

    pub fn publish_document(env: &Env, doc_id: String) -> Result<(), Error> {
        let mut doc: Document = env
            .storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        doc.status = DocumentStatus::Published;
        doc.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Document(doc_id), &doc);

        Ok(())
    }

    pub fn get_document(env: &Env, doc_id: &String) -> Option<Document> {
        env.storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))
    }

    pub fn increment_view_count(env: &Env, doc_id: String) -> Result<(), Error> {
        let mut doc: Document = env
            .storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        doc.view_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Document(doc_id), &doc);
        Storage::increment_counter(env, &DataKey::TotalViews);

        Ok(())
    }

    pub fn mark_helpful(env: &Env, doc_id: String) -> Result<(), Error> {
        let mut doc: Document = env
            .storage()
            .persistent()
            .get(&DataKey::Document(doc_id.clone()))
            .ok_or(Error::DocumentNotFound)?;

        doc.helpful_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Document(doc_id), &doc);

        Ok(())
    }

    pub fn get_documents_by_category(env: &Env, category: &String) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::CategoryDocs(category.clone()))
            .unwrap_or(Vec::new(env))
    }

    pub fn get_documents_by_author(env: &Env, author: &Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::DocumentsByAuthor(author.clone()))
            .unwrap_or(Vec::new(env))
    }
}
