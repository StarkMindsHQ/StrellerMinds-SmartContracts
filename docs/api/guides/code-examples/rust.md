# Rust Code Examples

Code examples for interacting with StrellerMinds smart contracts using Rust and the Soroban SDK.

## Prerequisites

```toml
# Cargo.toml
[dependencies]
soroban-sdk = "22.0.0"
```

## Token Contract

### Initialize and Mint

```rust
use soroban_sdk::{Env, Address};

pub fn initialize_and_mint(
    env: &Env,
    admin: &Address,
    recipient: &Address,
    amount: u64,
) -> Result<(), TokenError> {
    let client = token::TokenClient::new(env, &env.current_contract_address());

    // Initialize contract
    client.initialize(admin)?;

    // Mint tokens (admin only)
    client.mint(recipient, amount)?;

    Ok(())
}
```

### Transfer Tokens

```rust
pub fn transfer_tokens(
    env: &Env,
    sender: &Address,
    recipient: &Address,
    amount: u64,
) -> Result<(), TokenError> {
    let client = token::TokenClient::new(env, &token_contract_id);

    // Transfer requires sender authorization
    client.transfer(sender, recipient, amount)?;

    Ok(())
}
```

### Check Balance

```rust
pub fn check_balance(env: &Env, account: &Address) -> Result<u64, TokenError> {
    let client = token::TokenClient::new(env, &token_contract_id);
    client.balance(account)
}
```

## Certificate Contract

### Initialize and Issue Certificate

```rust
use soroban_sdk::{Env, Address, BytesN, String, Vec};

pub fn issue_certificate(
    env: &Env,
    admin: &Address,
    student: &Address,
    course_id: String,
    certificate_id: BytesN<32>,
) -> Result<BytesN<32>, CertificateError> {
    let client = certificate::CertificateClient::new(env, &cert_contract_id);

    // Initialize (if not already)
    client.initialize(admin)?;

    // Create mint params
    let params = MintCertificateParams {
        certificate_id,
        course_id: course_id.clone(),
        student: student.clone(),
        title: String::from_str(env, "Course Completion"),
        description: String::from_str(env, "Completed course"),
        metadata_uri: BytesN::from_array(env, &[0u8; 32]),
        expiry_date: 0, // No expiry
    };

    // Create multi-sig request
    let request_id = client.create_multisig_request(student, &params, &String::from_str(env, "Issuance request"))?;

    // Approve (would need multiple approvers in real scenario)
    client.process_multisig_approval(approver, &request_id, true, &String::from_str(env, "Approved"), None)?;

    // Execute
    client.execute_multisig_request(admin, &request_id)?;

    Ok(certificate_id)
}
```

### Verify Certificate

```rust
pub fn verify_certificate(
    env: &Env,
    certificate_id: BytesN<32>,
) -> Result<bool, CertificateError> {
    let client = certificate::CertificateClient::new(env, &cert_contract_id);
    client.verify_certificate(&certificate_id)
}
```

### Batch Issue Certificates

```rust
pub fn batch_issue(
    env: &Env,
    admin: &Address,
    params_list: Vec<MintCertificateParams>,
) -> Result<BatchResult, CertificateError> {
    let client = certificate::CertificateClient::new(env, &cert_contract_id);
    client.batch_issue_certificates(admin, &params_list)
}
```

## Progress Contract

### Record Progress

```rust
use soroban_sdk::{Env, Address, Symbol};

pub fn record_student_progress(
    env: &Env,
    student: &Address,
    course_id: &Symbol,
    progress: u32,
) -> Result<(), ProgressError> {
    let client = progress::ProgressClient::new(env, &progress_contract_id);

    // Requires student authorization
    client.record_progress(student, course_id, progress)?;

    Ok(())
}
```

### Get Progress

```rust
pub fn get_student_progress(
    env: &Env,
    student: &Address,
    course_id: &Symbol,
) -> Result<u32, ProgressError> {
    let client = progress::ProgressClient::new(env, &progress_contract_id);
    client.get_progress(student, course_id)
}
```

## Assessment Contract

### Create Assessment with Questions

```rust
use soroban_sdk::{Env, Address, String, Vec};

pub fn create_assessment(
    env: &Env,
    admin: &Address,
    title: String,
    course_id: String,
) -> Result<u64, AssessmentError> {
    let client = assessment::AssessmentClient::new(env, &assessment_contract_id);

    // Initialize
    client.initialize(admin)?;

    // Create metadata
    let metadata = AssessmentMetadata {
        title,
        description: String::from_str(env, "Course assessment"),
        course_id,
    };

    // Create config
    let config = AssessmentConfig {
        time_limit_seconds: 3600,
        max_score: 100,
        passing_score: 70,
        randomization_seed: 0,
    };

    // Create assessment
    let assessment_id = client.create_assessment(admin, &metadata, &config, &Vec::new(env))?;

    // Add questions
    let questions = vec![
        Question {
            question_id: 1,
            question_type: "multiple_choice".into(),
            prompt: String::from_str(env, "What is 2+2?"),
            options: vec![
                String::from_str(env, "3"),
                String::from_str(env, "4"),
                String::from_str(env, "5"),
            ],
            correct_answer: String::from_str(env, "4"),
            points: 10,
        },
    ];

    client.add_questions(admin, assessment_id, &questions)?;

    Ok(assessment_id)
}
```

### Submit Answers

```rust
pub fn submit_answers(
    env: &Env,
    student: &Address,
    submission_id: BytesN<32>,
    answers: Vec<Answer>,
) -> Result<(), AssessmentError> {
    let client = assessment::AssessmentClient::new(env, &assessment_contract_id);

    // Submit requires student authorization
    client.submit_answers(student, &submission_id, &answers)?;

    Ok(())
}
```

## Gamification Contract

### Record Activity and Earn Achievements

```rust
pub fn record_learning_activity(
    env: &Env,
    user: &Address,
    activity_type: &Symbol,
    course_id: &Symbol,
) -> Result<Vec<u64>, GamificationError> {
    let client = gamification::GamificationClient::new(env, &gamification_contract_id);

    let activity = ActivityRecord {
        activity_type: activity_type.clone(),
        course_id: course_id.clone(),
        module_id: symbol_short!("mod1"),
        timestamp: env.ledger().timestamp(),
    };

    // Returns achievement IDs earned from this activity
    let achievements = client.record_activity(user, &activity)?;

    Ok(achievements)
}
```

### Create and Join Guild

```rust
pub fn guild_workflow(
    env: &Env,
    creator: &Address,
    guild_name: String,
) -> Result<u64, GamificationError> {
    let client = gamification::GamificationClient::new(env, &gamification_contract_id);

    // Create guild
    let guild_id = client.create_guild(
        creator,
        &guild_name,
        &String::from_str(env, "A learning guild"),
        &50,  // max members
        &true, // is_public
    )?;

    // Other users can join
    client.join_guild(&user2, guild_id)?;
    client.join_guild(&user3, guild_id)?;

    Ok(guild_id)
}
```

### Endorse Peer

```rust
pub fn endorse_skill(
    env: &Env,
    endorser: &Address,
    endorsee: &Address,
    skill: String,
) -> Result<(), GamificationError> {
    let client = gamification::GamificationClient::new(env, &gamification_contract_id);
    client.endorse_peer(endorser, endorsee, &skill)
}
```

## Health Check

```rust
pub fn check_contract_health(env: &Env) -> ContractHealthReport {
    let token_client = token::TokenClient::new(env, &token_contract_id);
    token_client.health_check()
}
```

## Error Handling

```rust
match client.verify_certificate(&certificate_id) {
    Ok(true) => println!("Certificate is valid"),
    Ok(false) => println!("Certificate is invalid or revoked"),
    Err(CertificateError::CertificateNotFound) => println!("Certificate not found"),
    Err(e) => println!("Error: {:?}", e),
}
```