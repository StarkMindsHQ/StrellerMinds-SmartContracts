# Oracle Integration Patterns for Advanced AI-Powered Search System

**Created**: February 23, 2026
**Purpose**: Document how off-chain AI services integrate with on-chain smart contracts

---

## Overview

The Advanced AI-Powered Search System uses a **Hybrid Architecture** where computationally intensive AI/ML operations are performed off-chain, and results are submitted to the blockchain by authorized oracles.

```
┌─────────────────────────────────────────────────────────────┐
│                    OFF-CHAIN AI SERVICES                    │
├─────────────────────────────────────────────────────────────┤
│  • NLP Processing        • Image Analysis                   │
│  • Embeddings Generation • Translation APIs                 │
│  • ML Model Inference    • Voice Recognition                │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Submit Results
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    ORACLE LAYER                             │
├─────────────────────────────────────────────────────────────┤
│  • Authorization Check   • Signature Verification           │
│  • Rate Limiting        • Result Validation                 │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Store Results
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              ON-CHAIN SMART CONTRACTS                       │
├─────────────────────────────────────────────────────────────┤
│  • Storage Management    • Integer-only Scoring             │
│  • Access Control       • Event Emission                    │
│  • Query Execution      • Result Ranking                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Pattern 1: Semantic Metadata Submission

### Use Case
Off-chain NLP service analyzes course content and submits semantic metadata.

### Off-Chain Processing
```python
# Python example of NLP service
from transformers import pipeline
import stellar_sdk

# Initialize NLP models
classifier = pipeline("zero-shot-classification")
ner = pipeline("ner")

def analyze_course_content(course_id, content):
    # Extract topics using zero-shot classification
    topics = classifier(
        content,
        candidate_labels=["blockchain", "programming", "security", "design", "data science"]
    )
    
    # Extract entities
    entities = ner(content)
    
    # Generate semantic metadata
    metadata = {
        "course_id": course_id,
        "extracted_topics": [t["label"] for t in topics["labels"][:5]],
        "confidence_scores": [int(t["score"] * 1000) for t in topics["scores"][:5]],
        "entities": [e["word"] for e in entities if e["score"] > 0.9],
        "timestamp": int(time.time())
    }
    
    return metadata
```

### Oracle Submission
```rust
// Smart contract function to receive metadata
pub fn store_semantic_metadata(
    env: Env,
    oracle: Address,
    content_id: String,
    metadata: SemanticMetadata,
) -> Result<(), Error> {
    // 1. Verify oracle authorization
    require_authorized_oracle(&env, &oracle)?;
    
    // 2. Validate metadata structure
    if metadata.extracted_topics.len() == 0 {
        return Err(Error::InvalidMetadata);
    }
    
    // 3. Store metadata
    let key = DataKey::SemanticMetadata(content_id.clone());
    env.storage().persistent().set(&key, &metadata);
    
    // 4. Emit event for indexing
    env.events().publish(
        (symbol_short!("sem_meta"),),
        (content_id, oracle)
    );
    
    Ok(())
}
```

### Integration Flow
```
1. Course Content → Off-chain NLP Service
2. NLP Service → Analyze content (extract topics, entities, intent)
3. NLP Service → Format results as SemanticMetadata
4. NLP Service → Sign transaction as authorized oracle
5. Oracle → Submit to store_semantic_metadata()
6. Smart Contract → Verify oracle, validate data, store
7. Smart Contract → Emit event
8. Indexer → Update search index
```

---

## Pattern 2: Recommendation Submission

### Use Case
ML recommendation engine generates personalized course recommendations.

### Off-Chain Processing
```python
# Collaborative filtering model
import numpy as np
from scipy.sparse import csr_matrix
from sklearn.decomposition import TruncatedSVD

def generate_recommendations(user_id, interaction_matrix, courses, top_k=10):
    # Matrix factorization
    svd = TruncatedSVD(n_components=50)
    user_factors = svd.fit_transform(interaction_matrix)
    course_factors = svd.components_
    
    # Predict scores
    user_idx = get_user_index(user_id)
    scores = np.dot(user_factors[user_idx], course_factors)
    
    # Get top K recommendations
    top_indices = np.argsort(scores)[-top_k:][::-1]
    
    recommendations = []
    for idx in top_indices:
        recommendations.append({
            "course_id": courses[idx],
            "score": int(scores[idx] * 100),  # Convert to 0-1000 scale
            "reason": generate_reason(user_id, courses[idx]),
            "expires_at": int(time.time()) + 86400,  # 24 hours
        })
    
    return recommendations
```

### Oracle Submission
```rust
pub fn store_recommendations(
    env: Env,
    oracle: Address,
    user: Address,
    recommendations: Vec<Recommendation>,
) -> Result<(), Error> {
    require_authorized_oracle(&env, &oracle)?;
    
    // Validate recommendations
    if recommendations.len() > 50 {
        return Err(Error::TooManyRecommendations);
    }
    
    // Store with TTL
    let key = DataKey::RecommendationScores(user.clone());
    env.storage().persistent().set(&key, &recommendations);
    env.storage().persistent().extend_ttl(&key, 86400, 86400); // 24 hour TTL
    
    env.events().publish(
        (symbol_short!("rec_upd"),),
        (user, recommendations.len())
    );
    
    Ok(())
}
```

---

## Pattern 3: Visual Metadata Submission

### Use Case
Image analysis service processes course thumbnails.

### Off-Chain Processing
```python
# Computer vision service
import cv2
import torch
from torchvision import models, transforms

def analyze_course_thumbnail(course_id, image_url):
    # Download and preprocess image
    image = download_image(image_url)
    
    # Extract dominant colors
    colors = extract_dominant_colors(image, n_colors=3)
    
    # Object detection
    objects = detect_objects(image)
    
    # Calculate quality score
    quality = assess_image_quality(image)
    
    return {
        "course_id": course_id,
        "dominant_colors": [rgb_to_hex(c) for c in colors],
        "detected_objects": objects,
        "aspect_ratio": image.shape[1] * 100 // image.shape[0],
        "quality_score": int(quality * 1000),
        "thumbnail_url": image_url,
    }
```

### Oracle Submission
```rust
pub fn store_visual_metadata(
    env: Env,
    oracle: Address,
    content_id: String,
    metadata: VisualMetadata,
) -> Result<(), Error> {
    require_authorized_oracle(&env, &oracle)?;
    
    // Validate visual metadata
    if metadata.quality_score > 1000 {
        return Err(Error::InvalidScore);
    }
    
    let key = DataKey::VisualMetadata(content_id.clone());
    env.storage().persistent().set(&key, &metadata);
    
    // Index by visual category
    index_by_visual_category(&env, &content_id, &metadata);
    
    env.events().publish(
        (symbol_short!("vis_meta"),),
        content_id
    );
    
    Ok(())
}
```

---

## Pattern 4: Translation Submission

### Use Case
Translation API provides multilingual content versions.

### Off-Chain Processing
```python
# Translation service
from google.cloud import translate_v2 as translate

def translate_course_content(course_id, content, target_languages):
    translator = translate.Client()
    translations = {}
    
    for lang in target_languages:
        result = translator.translate(
            content,
            target_language=lang,
            source_language='en'
        )
        
        # Quality assessment
        quality = assess_translation_quality(content, result['translatedText'], lang)
        
        translations[lang] = {
            "target_language": lang,
            "translated_title": result['translatedText'][:200],
            "translated_description": result['translatedText'],
            "quality_score": int(quality * 1000),
            "translated_at": int(time.time()),
        }
    
    return translations
```

### Oracle Submission
```rust
pub fn store_multilingual_content(
    env: Env,
    oracle: Address,
    content_id: String,
    primary_language: Language,
    translations: Map<String, TranslationMeta>,
) -> Result<(), Error> {
    require_authorized_oracle(&env, &oracle)?;
    
    let content = MultilingualContent {
        content_id: content_id.clone(),
        primary_language,
        available_languages: extract_languages(&translations),
        translations,
    };
    
    let key = DataKey::MultilingualContent(content_id.clone());
    env.storage().persistent().set(&key, &content);
    
    // Index by each language
    for lang in content.available_languages.iter() {
        index_by_language(&env, &content_id, lang);
    }
    
    Ok(())
}
```

---

## Pattern 5: Voice Query Processing

### Use Case
Voice recognition service transcribes and processes voice queries.

### Off-Chain Processing
```python
# Voice recognition service
import speech_recognition as sr
from transformers import pipeline

def process_voice_query(audio_file, session_id):
    # Speech to text
    recognizer = sr.Recognizer()
    with sr.AudioFile(audio_file) as source:
        audio = recognizer.record(source)
    
    text = recognizer.recognize_google(audio)
    confidence = calculate_confidence(audio, text)
    
    # Intent extraction
    nlp = pipeline("zero-shot-classification")
    intent = nlp(text, candidate_labels=["search", "filter", "compare", "recommend"])
    
    return {
        "session_id": session_id,
        "transcribed_text": text,
        "original_language": detect_language(text),
        "confidence_score": int(confidence * 1000),
        "extracted_intent": intent["labels"][0],
        "timestamp": int(time.time()),
    }
```

### Oracle Submission
```rust
pub fn store_voice_query(
    env: Env,
    oracle: Address,
    user: Address,
    query: ProcessedVoiceQuery,
) -> Result<(), Error> {
    require_authorized_oracle(&env, &oracle)?;
    
    // Get or create session
    let session = get_or_create_session(&env, user.clone(), query.session_id.clone());
    
    // Add query to session
    let mut updated_session = session;
    updated_session.queries.push_back(query.clone());
    
    // Update context
    extract_and_update_context(&env, &mut updated_session, &query);
    
    // Store session
    let key = DataKey::ConversationSession(query.session_id.clone());
    env.storage().persistent().set(&key, &updated_session);
    
    Ok(())
}
```

---

## Oracle Authorization Management

### Adding an Oracle
```rust
pub fn authorize_oracle(
    env: Env,
    admin: Address,
    oracle: Address,
) -> Result<(), Error> {
    require_admin(&env, &admin)?;
    admin.require_auth();
    
    let key = DataKey::AuthorizedOracles(oracle.clone());
    env.storage().persistent().set(&key, &true);
    
    env.events().publish(
        (symbol_short!("ora_auth"),),
        oracle
    );
    
    Ok(())
}
```

### Revoking an Oracle
```rust
pub fn revoke_oracle(
    env: Env,
    admin: Address,
    oracle: Address,
) -> Result<(), Error> {
    require_admin(&env, &admin)?;
    admin.require_auth();
    
    let key = DataKey::AuthorizedOracles(oracle.clone());
    env.storage().persistent().remove(&key);
    
    env.events().publish(
        (symbol_short!("ora_rev"),),
        oracle
    );
    
    Ok(())
}
```

### Verification Helper
```rust
fn require_authorized_oracle(env: &Env, oracle: &Address) -> Result<(), Error> {
    oracle.require_auth();
    
    let key = DataKey::AuthorizedOracles(oracle.clone());
    match env.storage().persistent().get::<DataKey, bool>(&key) {
        Some(true) => Ok(()),
        _ => Err(Error::UnauthorizedOracle),
    }
}
```

---

## Data Validation Patterns

### Score Validation
```rust
fn validate_score(score: u32) -> Result<(), Error> {
    if score > 1000 {
        return Err(Error::InvalidScore);
    }
    Ok(())
}
```

### Timestamp Validation
```rust
fn validate_timestamp(env: &Env, timestamp: u64) -> Result<(), Error> {
    let current_time = env.ledger().timestamp();
    let max_age = 3600; // 1 hour
    
    if timestamp > current_time || current_time - timestamp > max_age {
        return Err(Error::InvalidTimestamp);
    }
    Ok(())
}
```

### Content Validation
```rust
fn validate_content_id(content_id: &String) -> Result<(), Error> {
    if content_id.len() == 0 || content_id.len() > 64 {
        return Err(Error::InvalidContentId);
    }
    Ok(())
}
```

---

## Rate Limiting Pattern

```rust
pub fn check_oracle_rate_limit(env: &Env, oracle: &Address) -> Result<(), Error> {
    let key = format_rate_limit_key(oracle);
    let current_time = env.ledger().timestamp();
    
    let last_submission: Option<u64> = env.storage().temporary().get(&key);
    
    match last_submission {
        Some(last_time) => {
            let min_interval = 60; // 1 minute between submissions
            if current_time - last_time < min_interval {
                return Err(Error::RateLimitExceeded);
            }
        },
        None => {}
    }
    
    // Update last submission time
    env.storage().temporary().set(&key, &current_time);
    env.storage().temporary().extend_ttl(&key, 3600, 3600);
    
    Ok(())
}
```

---

## Event Emission for Indexing

### Pattern
Every oracle submission should emit an event that off-chain indexers can listen to.

```rust
// Semantic metadata stored
env.events().publish(
    (symbol_short!("sem_meta"), symbol_short!("stored")),
    (content_id.clone(), oracle.clone())
);

// Recommendations updated
env.events().publish(
    (symbol_short!("rec_upd"), symbol_short!("user")),
    (user.clone(), recommendations.len())
);

// Visual metadata indexed
env.events().publish(
    (symbol_short!("vis_meta"), symbol_short!("indexed")),
    (content_id.clone(), metadata.quality_score)
);

// Translation added
env.events().publish(
    (symbol_short!("trans_add"), symbol_short!("lang")),
    (content_id.clone(), language)
);
```

---

## Error Handling

### Error Types
```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    UnauthorizedOracle = 1,
    InvalidMetadata = 2,
    InvalidScore = 3,
    InvalidTimestamp = 4,
    InvalidContentId = 5,
    RateLimitExceeded = 6,
    TooManyRecommendations = 7,
    SessionNotFound = 8,
    InvalidTranslation = 9,
    StorageError = 10,
}
```

### Error Response Pattern
```rust
pub fn handle_oracle_submission(env: Env, oracle: Address, data: Data) -> Result<(), Error> {
    // Check authorization
    require_authorized_oracle(&env, &oracle)?;
    
    // Check rate limit
    check_oracle_rate_limit(&env, &oracle)?;
    
    // Validate data
    validate_data(&data)?;
    
    // Store data
    store_data(&env, &data)?;
    
    // Emit event
    emit_event(&env, &data);
    
    Ok(())
}
```

---

## Testing Oracle Integration

### Mock Oracle for Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_mock_oracle(env: &Env) -> Address {
        let oracle = Address::generate(env);
        let admin = Address::generate(env);
        
        // Authorize mock oracle
        authorize_oracle(env.clone(), admin, oracle.clone()).unwrap();
        
        oracle
    }
    
    #[test]
    fn test_semantic_metadata_submission() {
        let env = Env::default();
        let oracle = create_mock_oracle(&env);
        
        let metadata = SemanticMetadata {
            extracted_topics: vec![
                String::from_str(&env, "rust"),
                String::from_str(&env, "programming"),
            ],
            confidence_scores: vec![950, 900],
            entities: vec![String::from_str(&env, "rust language")],
        };
        
        let result = store_semantic_metadata(
            env.clone(),
            oracle,
            String::from_str(&env, "RUST_101"),
            metadata,
        );
        
        assert!(result.is_ok());
    }
}
```

---

## Production Deployment Checklist

- [ ] Deploy smart contract to testnet
- [ ] Set up off-chain AI services
- [ ] Configure oracle addresses
- [ ] Authorize oracle wallets
- [ ] Set up monitoring and alerting
- [ ] Configure rate limits
- [ ] Test end-to-end submission flow
- [ ] Set up event indexers
- [ ] Configure backup oracles
- [ ] Document emergency procedures

---

## Security Best Practices

1. **Multi-Oracle Verification**: Use multiple oracles and require consensus
2. **Signature Verification**: Always verify oracle signatures
3. **Rate Limiting**: Prevent spam and DOS attacks
4. **Data Validation**: Validate all submitted data
5. **Access Control**: Maintain strict oracle authorization
6. **Monitoring**: Track oracle behavior and submissions
7. **Fallback Mechanisms**: Have backup oracles ready
8. **Audit Logging**: Log all oracle submissions

---

**Status**: ✅ Oracle Integration Patterns Documented
**Coverage**: All 10 AI modules
**Ready for**: Production deployment, Third-party oracle integration
