use soroban_sdk::{Address, Env, String, Vec, Map};
use crate::types::*;

/// Search indexing and optimization functionality
pub struct SearchIndexer;

impl SearchIndexer {
    /// Build search index for courses
    pub fn index_courses(env: &Env, courses: Vec<CourseData>) -> Result<(), IndexError> {
        for course in courses {
            Self::index_course_content(env, &course)?;
            Self::update_course_facets(env, &course)?;
        }
        
        Self::optimize_course_index(env)?;
        Ok(())
    }

    /// Build search index for certificates
    pub fn index_certificates(env: &Env, certificates: Vec<CertificateData>) -> Result<(), IndexError> {
        for certificate in certificates {
            Self::index_certificate_content(env, &certificate)?;
            Self::update_certificate_facets(env, &certificate)?;
        }
        
        Self::optimize_certificate_index(env)?;
        Ok(())
    }

    /// Build search index for user progress
    pub fn index_user_progress(env: &Env, progress_data: Vec<UserProgressData>) -> Result<(), IndexError> {
        for progress in progress_data {
            Self::index_progress_content(env, &progress)?;
        }
        
        Self::optimize_progress_index(env)?;
        Ok(())
    }

    /// Index individual course content
    fn index_course_content(env: &Env, course: &CourseData) -> Result<(), IndexError> {
        let search_document = SearchDocument {
            id: course.course_id.clone(),
            content_type: SearchResultType::Course,
            title: course.title.clone(),
            description: course.description.clone(),
            content: course.content.clone(),
            keywords: Self::extract_keywords(&course.title, &course.description),
            metadata: Self::build_course_metadata(course),
            indexed_at: env.ledger().timestamp(),
        };

        Self::store_search_document(env, search_document)?;
        Ok(())
    }

    /// Index individual certificate content
    fn index_certificate_content(env: &Env, certificate: &CertificateData) -> Result<(), IndexError> {
        let search_document = SearchDocument {
            id: certificate.certificate_id.to_string(),
            content_type: SearchResultType::Certificate,
            title: certificate.title.clone(),
            description: certificate.description.clone(),
            content: String::from_str(env, ""),
            keywords: Self::extract_keywords(&certificate.title, &certificate.description),
            metadata: Self::build_certificate_metadata(certificate),
            indexed_at: env.ledger().timestamp(),
        };

        Self::store_search_document(env, search_document)?;
        Ok(())
    }

    /// Index user progress content
    fn index_progress_content(env: &Env, progress: &UserProgressData) -> Result<(), IndexError> {
        let search_document = SearchDocument {
            id: format!("{}_{}", progress.student_id.to_string(), progress.course_id),
            content_type: SearchResultType::UserProgress,
            title: format!("Progress: {}", progress.course_title),
            description: format!("{}% complete", progress.completion_percentage),
            content: String::from_str(env, ""),
            keywords: Vec::new(env),
            metadata: Self::build_progress_metadata(progress),
            indexed_at: env.ledger().timestamp(),
        };

        Self::store_search_document(env, search_document)?;
        Ok(())
    }

    /// Extract keywords from text content
    fn extract_keywords(title: &String, description: &String) -> Vec<String> {
        let mut keywords = Vec::new(&title.env());
        
        // Simple keyword extraction - split by spaces and common delimiters
        let combined_text = format!("{} {}", title, description);
        let words: Vec<&str> = combined_text.split_whitespace().collect();
        
        for word in words {
            if word.len() > 3 { // Only include words longer than 3 characters
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if !clean_word.is_empty() {
                    keywords.push_back(String::from_str(&title.env(), clean_word));
                }
            }
        }

        keywords
    }

    /// Build course metadata for indexing
    fn build_course_metadata(course: &CourseData) -> Map<String, String> {
        let mut metadata = Map::new(&course.course_id.env());
        
        metadata.set(
            String::from_str(&course.course_id.env(), "category"),
            course.category.clone()
        );
        metadata.set(
            String::from_str(&course.course_id.env(), "difficulty"),
            Self::difficulty_to_string(&course.difficulty)
        );
        metadata.set(
            String::from_str(&course.course_id.env(), "duration"),
            course.duration_hours.to_string()
        );
        metadata.set(
            String::from_str(&course.course_id.env(), "instructor"),
            course.instructor_name.clone()
        );
        metadata.set(
            String::from_str(&course.course_id.env(), "price"),
            course.price.to_string()
        );
        metadata.set(
            String::from_str(&course.course_id.env(), "rating"),
            course.rating.to_string()
        );

        metadata
    }

    /// Build certificate metadata for indexing
    fn build_certificate_metadata(certificate: &CertificateData) -> Map<String, String> {
        let mut metadata = Map::new(&certificate.certificate_id.env());
        
        metadata.set(
            String::from_str(&certificate.certificate_id.env(), "course_id"),
            certificate.course_id.clone()
        );
        metadata.set(
            String::from_str(&certificate.certificate_id.env(), "status"),
            Self::status_to_string(&certificate.status)
        );
        metadata.set(
            String::from_str(&certificate.certificate_id.env(), "issue_date"),
            certificate.issue_date.to_string()
        );
        metadata.set(
            String::from_str(&certificate.certificate_id.env(), "expiry_date"),
            certificate.expiry_date.to_string()
        );

        metadata
    }

    /// Build progress metadata for indexing
    fn build_progress_metadata(progress: &UserProgressData) -> Map<String, String> {
        let mut metadata = Map::new(&progress.course_id.env());
        
        metadata.set(
            String::from_str(&progress.course_id.env(), "completion_percentage"),
            progress.completion_percentage.to_string()
        );
        metadata.set(
            String::from_str(&progress.course_id.env(), "enrollment_date"),
            progress.enrollment_date.to_string()
        );
        metadata.set(
            String::from_str(&progress.course_id.env(), "last_activity"),
            progress.last_activity_date.to_string()
        );

        metadata
    }

    /// Store search document in index
    fn store_search_document(env: &Env, document: SearchDocument) -> Result<(), IndexError> {
        let index_key = DataKey::IndexMetadata(document.id.clone());
        env.storage().persistent().set(&index_key, &document);
        
        // Update search terms index
        Self::update_search_terms_index(env, &document)?;
        
        Ok(())
    }

    /// Update search terms index for fast text search
    fn update_search_terms_index(env: &Env, document: &SearchDocument) -> Result<(), IndexError> {
        // Index title words
        Self::index_text_terms(env, &document.title, &document.id)?;
        
        // Index description words
        Self::index_text_terms(env, &document.description, &document.id)?;
        
        // Index keywords
        for keyword in &document.keywords {
            Self::add_term_to_index(env, keyword, &document.id)?;
        }

        Ok(())
    }

    /// Index individual text terms
    fn index_text_terms(env: &Env, text: &String, document_id: &String) -> Result<(), IndexError> {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if clean_word.len() > 2 {
                let term = String::from_str(env, &clean_word);
                Self::add_term_to_index(env, &term, document_id)?;
            }
        }

        Ok(())
    }

    /// Add term to inverted index
    fn add_term_to_index(env: &Env, term: &String, document_id: &String) -> Result<(), IndexError> {
        let term_key = DataKey::AutoCompleteData(term.clone());
        let mut document_ids = env.storage().persistent()
            .get(&term_key)
            .unwrap_or_else(|| Vec::new(env));
        
        // Add document ID if not already present
        let mut found = false;
        for existing_id in &document_ids {
            if existing_id == document_id {
                found = true;
                break;
            }
        }
        
        if !found {
            document_ids.push_back(document_id.clone());
            env.storage().persistent().set(&term_key, &document_ids);
        }

        Ok(())
    }

    /// Update facets for course filtering
    fn update_course_facets(env: &Env, course: &CourseData) -> Result<(), IndexError> {
        // Update category facet
        Self::update_facet_count(env, "categories", &course.category)?;
        
        // Update difficulty facet
        let difficulty_str = Self::difficulty_to_string(&course.difficulty);
        Self::update_facet_count(env, "difficulty", &difficulty_str)?;
        
        // Update instructor facet
        Self::update_facet_count(env, "instructors", &course.instructor_name)?;

        Ok(())
    }

    /// Update facets for certificate filtering
    fn update_certificate_facets(env: &Env, certificate: &CertificateData) -> Result<(), IndexError> {
        // Update status facet
        let status_str = Self::status_to_string(&certificate.status);
        Self::update_facet_count(env, "certificate_status", &status_str)?;
        
        // Update type facet
        let type_str = Self::cert_type_to_string(&certificate.certificate_type);
        Self::update_facet_count(env, "certificate_type", &type_str)?;

        Ok(())
    }

    /// Update facet counts
    fn update_facet_count(env: &Env, facet_name: &str, facet_value: &String) -> Result<(), IndexError> {
        let facet_key = DataKey::SearchSuggestions(String::from_str(env, facet_name));
        let mut facet_values = env.storage().persistent()
            .get(&facet_key)
            .unwrap_or_else(|| Vec::new(env));

        // Find existing facet value or create new one
        let mut found = false;
        for i in 0..facet_values.len() {
            if let Some(mut facet_val) = facet_values.get(i) {
                if let SearchSuggestion { suggestion_text, .. } = facet_val {
                    if suggestion_text == *facet_value {
                        facet_val.popularity_score += 1;
                        facet_values.set(i, facet_val);
                        found = true;
                        break;
                    }
                }
            }
        }

        if !found {
            let new_facet = SearchSuggestion {
                suggestion_text: facet_value.clone(),
                suggestion_type: SuggestionType::Category,
                popularity_score: 1,
                category: Some(String::from_str(env, facet_name)),
                metadata: None,
            };
            facet_values.push_back(new_facet);
        }

        env.storage().persistent().set(&facet_key, &facet_values);
        Ok(())
    }

    /// Optimize course index for better search performance
    fn optimize_course_index(env: &Env) -> Result<(), IndexError> {
        // Implementation would include index compaction, term frequency calculation, etc.
        Ok(())
    }

    /// Optimize certificate index
    fn optimize_certificate_index(env: &Env) -> Result<(), IndexError> {
        // Implementation would include index optimization specific to certificates
        Ok(())
    }

    /// Optimize progress index
    fn optimize_progress_index(env: &Env) -> Result<(), IndexError> {
        // Implementation would include progress-specific optimizations
        Ok(())
    }

    /// Convert difficulty enum to string
    fn difficulty_to_string(difficulty: &DifficultyLevel) -> String {
        match difficulty {
            DifficultyLevel::Beginner => String::from_str(&difficulty.env(), "Beginner"),
            DifficultyLevel::Intermediate => String::from_str(&difficulty.env(), "Intermediate"),
            DifficultyLevel::Advanced => String::from_str(&difficulty.env(), "Advanced"),
            DifficultyLevel::Expert => String::from_str(&difficulty.env(), "Expert"),
        }
    }

    /// Convert certificate status to string
    fn status_to_string(status: &CertificateStatus) -> String {
        match status {
            CertificateStatus::Active => String::from_str(&status.env(), "Active"),
            CertificateStatus::Revoked => String::from_str(&status.env(), "Revoked"),
            CertificateStatus::Expired => String::from_str(&status.env(), "Expired"),
            CertificateStatus::PendingRenewal => String::from_str(&status.env(), "PendingRenewal"),
            CertificateStatus::Renewed => String::from_str(&status.env(), "Renewed"),
        }
    }

    /// Convert certificate type to string
    fn cert_type_to_string(cert_type: &CertificateType) -> String {
        match cert_type {
            CertificateType::Completion => String::from_str(&cert_type.env(), "Completion"),
            CertificateType::Achievement => String::from_str(&cert_type.env(), "Achievement"),
            CertificateType::Professional => String::from_str(&cert_type.env(), "Professional"),
            CertificateType::Accredited => String::from_str(&cert_type.env(), "Accredited"),
            CertificateType::Micro => String::from_str(&cert_type.env(), "Micro"),
        }
    }

    /// Rebuild entire search index
    pub fn rebuild_index(env: &Env) -> Result<(), IndexError> {
        // Clear existing index
        Self::clear_index(env)?;
        
        // Rebuild from source data
        // This would typically fetch data from other contracts
        // For now, we'll just mark the index as rebuilt
        
        let metadata = SearchMetadata {
            query_timestamp: env.ledger().timestamp(),
            index_version: String::from_str(env, "1.0.0"),
            search_engine_version: String::from_str(env, "1.0.0"),
            cache_hit: false,
            total_indexed_items: 0,
            search_suggestions_enabled: true,
        };

        env.storage().persistent().set(&DataKey::IndexMetadata(String::from_str(env, "main")), &metadata);
        Ok(())
    }

    /// Clear search index
    fn clear_index(env: &Env) -> Result<(), IndexError> {
        // Implementation would clear all indexed data
        // This is a simplified version
        Ok(())
    }
}

/// Search document structure for indexing
#[derive(Clone, Debug)]
pub struct SearchDocument {
    pub id: String,
    pub content_type: SearchResultType,
    pub title: String,
    pub description: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub metadata: Map<String, String>,
    pub indexed_at: u64,
}

/// Course data for indexing
#[derive(Clone, Debug)]
pub struct CourseData {
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub category: String,
    pub difficulty: DifficultyLevel,
    pub duration_hours: u32,
    pub instructor_name: String,
    pub price: i64,
    pub rating: u32,
}

/// Certificate data for indexing
#[derive(Clone, Debug)]
pub struct CertificateData {
    pub certificate_id: String,
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub status: CertificateStatus,
    pub certificate_type: CertificateType,
    pub issue_date: u64,
    pub expiry_date: u64,
}

/// User progress data for indexing
#[derive(Clone, Debug)]
pub struct UserProgressData {
    pub student_id: Address,
    pub course_id: String,
    pub course_title: String,
    pub completion_percentage: u32,
    pub enrollment_date: u64,
    pub last_activity_date: u64,
}

/// Indexing error types
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexError {
    IndexCorrupted,
    InsufficientStorage,
    InvalidDocument,
    IndexLocked,
    OptimizationFailed,
}
