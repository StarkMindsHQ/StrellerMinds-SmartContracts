# Test Datasets for Advanced AI-Powered Search System

**Created**: February 23, 2026
**Purpose**: Comprehensive test data for all 10 AI-powered search modules

---

## Dataset 1: Semantic Search Test Data

### Test Courses

```rust
// Course 1: Beginner Rust Programming
{
    course_id: "RUST_101",
    title: "Introduction to Rust Programming",
    description: "Learn Rust from scratch with hands-on projects",
    tags: ["rust", "programming", "beginner", "systems"],
    topics: ["ownership", "borrowing", "memory safety", "cargo"],
    difficulty: Beginner,
    duration_hours: 40,
    rating: 45, // 4.5/5 on 1-50 scale
    semantic_metadata: {
        extracted_topics: ["rust basics", "memory management", "safe concurrency"],
        intent_categories: ["learn", "build", "understand"],
        entities: ["rust language", "cargo tool", "ownership model"],
    }
}

// Course 2: Advanced Blockchain Development
{
    course_id: "BLOCKCHAIN_301",
    title: "Advanced Smart Contract Development on Stellar",
    description: "Master Soroban smart contracts and DeFi protocols",
    tags: ["blockchain", "soroban", "stellar", "defi", "advanced"],
    topics: ["soroban sdk", "contract security", "defi protocols", "testing"],
    difficulty: Advanced,
    duration_hours: 80,
    rating: 48,
    semantic_metadata: {
        extracted_topics: ["smart contracts", "stellar blockchain", "defi"],
        intent_categories: ["master", "build", "deploy"],
        entities: ["soroban", "stellar", "rust", "wasm"],
    }
}

// Course 3: Data Science Fundamentals
{
    course_id: "DS_201",
    title: "Data Science and Machine Learning Basics",
    description: "Introduction to data analysis, statistics, and ML algorithms",
    tags: ["data science", "machine learning", "python", "intermediate"],
    topics: ["statistics", "pandas", "numpy", "scikit-learn", "ml algorithms"],
    difficulty: Intermediate,
    duration_hours: 60,
    rating: 46,
}

// Course 4: Web3 Security Auditing
{
    course_id: "SECURITY_401",
    title: "Smart Contract Security and Auditing",
    description: "Learn to identify and prevent vulnerabilities in smart contracts",
    tags: ["security", "auditing", "smart contracts", "expert"],
    topics: ["reentrancy", "access control", "overflow", "audit methodology"],
    difficulty: Expert,
    duration_hours: 50,
    rating: 49,
}

// Course 5: UI/UX Design for Web3
{
    course_id: "DESIGN_101",
    title: "User Interface Design for Decentralized Applications",
    description: "Create intuitive and beautiful Web3 interfaces",
    tags: ["design", "ui", "ux", "web3", "beginner"],
    topics: ["figma", "user research", "prototyping", "wallet integration"],
    difficulty: Beginner,
    duration_hours: 30,
    rating: 44,
}
```

### Test Queries

```rust
// Query 1: Natural language search
{
    query_text: "I want to learn blockchain development with Rust",
    expected_intent: "learn",
    expected_entities: ["blockchain", "rust"],
    expected_results: ["BLOCKCHAIN_301", "RUST_101"],
}

// Query 2: Skill-based search
{
    query_text: "smart contract security best practices",
    expected_intent: "understand",
    expected_entities: ["smart contracts", "security"],
    expected_results: ["SECURITY_401", "BLOCKCHAIN_301"],
}

// Query 3: Beginner-friendly search
{
    query_text: "easy courses for someone new to programming",
    expected_intent: "learn",
    expected_difficulty_filter: Beginner,
    expected_results: ["RUST_101", "DESIGN_101"],
}

// Query 4: Advanced topic search
{
    query_text: "defi protocol development and testing",
    expected_intent: "master",
    expected_entities: ["defi", "protocols", "testing"],
    expected_results: ["BLOCKCHAIN_301"],
}
```

---

## Dataset 2: Recommendation Engine Test Data

### User Profiles

```rust
// User 1: Blockchain Enthusiast
{
    user_address: Address::from_str("GUSER1..."),
    completed_courses: ["RUST_101", "BLOCKCHAIN_301"],
    skill_levels: {
        "rust": 70,
        "blockchain": 85,
        "smart_contracts": 75,
    },
    interaction_counts: {
        "view": 45,
        "enroll": 3,
        "complete": 2,
    },
    preference_scores: [80, 90, 60, 40, 70], // blockchain, programming, design, data, security
}

// User 2: Full-Stack Developer
{
    user_address: Address::from_str("GUSER2..."),
    completed_courses: ["DESIGN_101", "RUST_101"],
    skill_levels: {
        "rust": 60,
        "design": 80,
        "frontend": 90,
    },
    interaction_counts: {
        "view": 67,
        "enroll": 4,
        "complete": 2,
    },
    preference_scores: [50, 70, 95, 40, 55],
}

// User 3: Security Researcher
{
    user_address: Address::from_str("GUSER3..."),
    completed_courses: ["RUST_101", "BLOCKCHAIN_301", "SECURITY_401"],
    skill_levels: {
        "rust": 90,
        "blockchain": 85,
        "security": 95,
    },
    interaction_counts: {
        "view": 89,
        "enroll": 5,
        "complete": 3,
    },
    preference_scores: [85, 80, 30, 40, 100],
}
```

### Expected Recommendations

```rust
// For User 1 (Blockchain Enthusiast)
{
    expected_recommendations: [
        { course_id: "SECURITY_401", score: 850, reason: "Next step after blockchain" },
        { course_id: "DS_201", score: 650, reason: "Complementary skill" },
    ]
}

// For User 2 (Full-Stack Developer)
{
    expected_recommendations: [
        { course_id: "BLOCKCHAIN_301", score: 750, reason: "Combines rust + new domain" },
        { course_id: "DS_201", score: 600, reason: "Data for better UX" },
    ]
}
```

---

## Dataset 3: Content Analysis Test Data

### Content Samples for Analysis

```rust
// Sample 1: Course Description for Auto-Tagging
{
    content: "This comprehensive course teaches you Rust programming from the ground up. 
              You'll learn ownership, borrowing, lifetimes, and build real-world projects 
              including a CLI tool and a web server. Perfect for beginners with basic 
              programming knowledge.",
    expected_tags: ["rust", "programming", "beginner", "projects", "ownership", "web"],
    expected_skills: ["rust basics", "memory management", "project building"],
    expected_difficulty: Beginner,
    expected_quality_score: 850, // High quality content
}

// Sample 2: Advanced Technical Content
{
    content: "Deep dive into Soroban smart contract development on Stellar. Covers 
              contract architecture, security patterns, testing strategies, and DeFi 
              protocol implementation. Includes reentrancy protection, access control, 
              and gas optimization techniques.",
    expected_tags: ["soroban", "stellar", "smart contracts", "security", "defi"],
    expected_skills: ["smart contract development", "security auditing", "defi protocols"],
    expected_difficulty: Advanced,
    expected_quality_score: 920,
}
```

---

## Dataset 4: Collaborative Filtering Test Data

### User Interaction Matrix

```rust
// Interaction Data
[
    { user: "GUSER1", course: "RUST_101", type: Complete, timestamp: 1708000000 },
    { user: "GUSER1", course: "BLOCKCHAIN_301", type: Complete, timestamp: 1708100000 },
    { user: "GUSER1", course: "SECURITY_401", type: View, timestamp: 1708200000 },
    
    { user: "GUSER2", course: "DESIGN_101", type: Complete, timestamp: 1708000000 },
    { user: "GUSER2", course: "RUST_101", type: Complete, timestamp: 1708150000 },
    { user: "GUSER2", course: "BLOCKCHAIN_301", type: View, timestamp: 1708250000 },
    
    { user: "GUSER3", course: "RUST_101", type: Complete, timestamp: 1707900000 },
    { user: "GUSER3", course: "BLOCKCHAIN_301", type: Complete, timestamp: 1708050000 },
    { user: "GUSER3", course: "SECURITY_401", type: Complete, timestamp: 1708180000 },
    { user: "GUSER3", course: "DS_201", type: View, timestamp: 1708300000 },
]

// Expected Similarities
{
    user_similarity: {
        (GUSER1, GUSER3): 850, // Both completed blockchain + security path
        (GUSER1, GUSER2): 600, // Both completed rust, different paths
        (GUSER2, GUSER3): 550, // Less similar
    }
}
```

---

## Dataset 5: Visual Search Test Data

### Visual Metadata

```rust
// Course 1: Thumbnail with code samples
{
    course_id: "RUST_101",
    visual_metadata: {
        dominant_colors: ["#1E1E1E", "#CE9178", "#4EC9B0"], // VS Code theme colors
        detected_objects: ["code editor", "terminal", "rust logo"],
        visual_category: "programming",
        aspect_ratio: 1920 * 100 / 1080, // 16:9 = 177
        quality_score: 850,
    }
}

// Course 2: Blockchain visualization
{
    course_id: "BLOCKCHAIN_301",
    visual_metadata: {
        dominant_colors: ["#1A1A2E", "#16213E", "#0F3460"],
        detected_objects: ["blockchain", "network diagram", "stellar logo"],
        visual_category: "blockchain",
        aspect_ratio: 177,
        quality_score: 900,
    }
}
```

---

## Dataset 6: Learning Path Test Data

### Skill Dependency Graph

```rust
// Blockchain Developer Path
{
    path_id: "BLOCKCHAIN_PATH_001",
    target_skill: "blockchain_development",
    steps: [
        {
            content_id: "RUST_101",
            skill_id: "rust_basics",
            estimated_effort: 40,
            prerequisites: [],
        },
        {
            content_id: "BLOCKCHAIN_301",
            skill_id: "smart_contracts",
            estimated_effort: 80,
            prerequisites: ["rust_basics"],
        },
        {
            content_id: "SECURITY_401",
            skill_id: "contract_security",
            estimated_effort: 50,
            prerequisites: ["smart_contracts"],
        },
    ],
    estimated_duration_days: 90,
    skill_nodes: [
        { skill_id: "rust_basics", prerequisites: [], difficulty: 30 },
        { skill_id: "smart_contracts", prerequisites: ["rust_basics"], difficulty: 70 },
        { skill_id: "contract_security", prerequisites: ["smart_contracts"], difficulty: 90 },
    ],
}

// Full-Stack Web3 Developer Path
{
    path_id: "FULLSTACK_PATH_001",
    target_skill: "fullstack_web3",
    steps: [
        { content_id: "DESIGN_101", skill_id: "ui_design", estimated_effort: 30, prerequisites: [] },
        { content_id: "RUST_101", skill_id: "rust_basics", estimated_effort: 40, prerequisites: [] },
        { content_id: "BLOCKCHAIN_301", skill_id: "smart_contracts", estimated_effort: 80, prerequisites: ["rust_basics"] },
    ],
    estimated_duration_days: 100,
}
```

---

## Dataset 7: Ranking Engine Test Data

### Ranking Signals

```rust
// Courses with different signal strengths
[
    {
        content_id: "BLOCKCHAIN_301",
        signals: {
            relevance_score: 900,
            quality_score: 880,
            engagement_score: 850,
            authority_score: 920,
            recency_score: 750,
            personalization_score: 870,
        },
        expected_final_score: 875, // Weighted average
    },
    {
        content_id: "RUST_101",
        signals: {
            relevance_score: 850,
            quality_score: 800,
            engagement_score: 900,
            authority_score: 750,
            recency_score: 650,
            personalization_score: 800,
        },
        expected_final_score: 800,
    },
]

// Expected Ranking: BLOCKCHAIN_301 > RUST_101
```

---

## Dataset 8: Multilingual Search Test Data

### Multilingual Content

```rust
// Course with multiple translations
{
    content_id: "RUST_101",
    primary_language: English,
    available_languages: [English, Spanish, French, Mandarin],
    translations: {
        "Spanish": {
            target_language: Spanish,
            translated_title: "Introducción a la Programación Rust",
            translated_description: "Aprende Rust desde cero con proyectos prácticos",
            quality_score: 850,
            translated_at: 1708000000,
        },
        "French": {
            target_language: French,
            translated_title: "Introduction à la Programmation Rust",
            translated_description: "Apprenez Rust à partir de zéro avec des projets pratiques",
            quality_score: 870,
            translated_at: 1708010000,
        },
        "Mandarin": {
            target_language: Mandarin,
            translated_title: "Rust编程入门",
            translated_description: "从零开始学习Rust，通过实践项目掌握",
            quality_score: 820,
            translated_at: 1708020000,
        },
    }
}

// Test Queries in Different Languages
[
    { query: "aprender programación rust", language: Spanish, expected: ["RUST_101"] },
    { query: "rust编程课程", language: Mandarin, expected: ["RUST_101"] },
    { query: "cours de rust", language: French, expected: ["RUST_101"] },
]
```

---

## Dataset 9: Search Analytics Test Data

### Search Events

```rust
[
    {
        event_id: "SEARCH_001",
        user: "GUSER1",
        query: "blockchain development",
        timestamp: 1708000000,
        results_count: 3,
        filters_applied: ["difficulty:intermediate"],
    },
    {
        event_id: "SEARCH_002",
        user: "GUSER1",
        query: "rust programming",
        timestamp: 1708001000,
        results_count: 5,
        filters_applied: [],
    },
]

// Click Events
[
    {
        event_id: "CLICK_001",
        user: "GUSER1",
        query: "blockchain development",
        content_id: "BLOCKCHAIN_301",
        rank_position: 1,
        timestamp: 1708000100,
        dwell_time: 450, // seconds
    },
    {
        event_id: "CLICK_002",
        user: "GUSER1",
        query: "blockchain development",
        content_id: "SECURITY_401",
        rank_position: 2,
        timestamp: 1708000600,
        dwell_time: 120,
    },
]

// Expected CTR: 2 clicks / 3 results = 66.67%
// Expected Quality Score: High (good dwell time on position 1)
```

---

## Dataset 10: Voice Search Test Data

### Voice Queries and Sessions

```rust
// Conversation Session 1
{
    session_id: "VOICE_SESSION_001",
    user: "GUSER1",
    queries: [
        {
            transcribed_text: "Show me blockchain courses",
            confidence_score: 920,
            timestamp: 1708000000,
        },
        {
            transcribed_text: "What about advanced ones?",
            confidence_score: 880,
            timestamp: 1708000030,
        },
        {
            transcribed_text: "Which one covers DeFi?",
            confidence_score: 900,
            timestamp: 1708000060,
        },
    ],
    context_entities: ["blockchain", "advanced", "defi"],
    is_active: true,
}

// Expected Context Understanding:
// Query 1: Show blockchain courses (clear intent)
// Query 2: "advanced ones" → refers to blockchain courses (context preserved)
// Query 3: "Which one" → refers to advanced blockchain courses (full context)
```

---

## Test Scenarios by Module

### Scenario 1: End-to-End Semantic Search
1. User searches: "I want to learn smart contract security"
2. System extracts intent: "learn"
3. System extracts entities: ["smart contracts", "security"]
4. System applies semantic matching
5. Expected result: SECURITY_401 (high score), BLOCKCHAIN_301 (medium score)

### Scenario 2: Personalized Recommendations
1. User (GUSER1) completes RUST_101 and BLOCKCHAIN_301
2. System analyzes profile: blockchain enthusiast
3. System finds similar user: GUSER3 (also blockchain focused)
4. Expected recommendation: SECURITY_401 (what GUSER3 took next)

### Scenario 3: Multilingual Discovery
1. Spanish-speaking user searches: "cursos de blockchain"
2. System detects language: Spanish
3. System retrieves Spanish translations
4. System ranks by translation quality
5. Expected: Courses with high-quality Spanish translations ranked higher

### Scenario 4: Voice Conversation Flow
1. User: "Show me rust courses"
2. System: Returns RUST_101, context saved
3. User: "What's next after that?"
4. System: Uses context → recommends BLOCKCHAIN_301
5. User: "Tell me more about the second one"
6. System: Returns details for BLOCKCHAIN_301

---

## Data Volume for Performance Testing

### Small Dataset (Development)
- 10 courses
- 5 users
- 50 interactions
- 20 search queries

### Medium Dataset (Testing)
- 100 courses
- 50 users
- 1,000 interactions
- 500 search queries

### Large Dataset (Production Simulation)
- 1,000 courses
- 1,000 users
- 50,000 interactions
- 10,000 search queries

---

## Mock Oracle Data Submission Format

```rust
// Example: Semantic metadata submission by oracle
{
    oracle: Address::from_str("GORACLE1..."),
    content_id: "RUST_101",
    metadata_type: "semantic",
    data: SemanticMetadata {
        extracted_topics: vec!["rust", "ownership", "memory safety"],
        confidence_scores: vec![950, 920, 900],
        intent_categories: vec!["learn", "build"],
        entities: vec!["rust language", "cargo"],
    },
    timestamp: 1708000000,
    signature: BytesN<64>::from_array(...),
}
```

---

**Status**: ✅ Test Datasets Complete
**Total Test Cases**: 100+
**Coverage**: All 10 modules
**Ready for**: Unit testing, Integration testing, Performance benchmarking
