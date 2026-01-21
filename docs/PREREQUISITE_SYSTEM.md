# Course Prerequisites System

## Overview

The Course Prerequisites System is a comprehensive solution for managing and enforcing course dependencies within the StrellerMinds smart contract ecosystem. It provides flexible prerequisite definitions, intelligent checking logic, admin override mechanisms, and learning path generation to ensure proper educational progression.

## Architecture

### Core Components

1. **PrerequisiteManager**: Central management system for all prerequisite operations
2. **Data Structures**: Comprehensive types for prerequisites, overrides, and learning paths
3. **Validation Engine**: Logic for checking prerequisite compliance
4. **Override System**: Admin mechanisms for exceptional cases
5. **Learning Path Generator**: Intelligent course sequencing for students

### Integration Points

- **Certificate Contract**: Main contract interface for prerequisite operations
- **Progress Tracking**: Integration with student progress and completion data
- **RBAC System**: Role-based access control for administrative functions
- **Event System**: Comprehensive logging and audit trail

## Data Structures

### CoursePrerequisite

Defines prerequisite requirements for a course:

```rust
pub struct CoursePrerequisite {
    pub course_id: String,
    pub prerequisite_courses: Vec<PrerequisiteCourse>,
    pub minimum_completion_percentage: u32,
    pub policy: PrerequisitePolicy,
    pub created_by: Address,
    pub created_at: u64,
    pub updated_at: u64,
}
```

### PrerequisiteCourse

Individual prerequisite course requirements:

```rust
pub struct PrerequisiteCourse {
    pub course_id: String,
    pub minimum_percentage: u32,
    pub weight: u32,
    pub required_certificate: bool,
}
```

### PrerequisitePolicy

Enforcement policies for prerequisites:

```rust
pub enum PrerequisitePolicy {
    Strict,      // All prerequisites must be met
    Flexible,    // Some prerequisites can be waived
    Weighted,    // Prerequisites weighted by importance
    Progressive, // Prerequisites can be completed concurrently
}
```

### PrerequisiteCheckResult

Result of prerequisite validation:

```rust
pub struct PrerequisiteCheckResult {
    pub student: Address,
    pub course_id: String,
    pub eligible: bool,
    pub missing_prerequisites: Vec<MissingPrerequisite>,
    pub completed_prerequisites: Vec<CompletedPrerequisite>,
    pub check_timestamp: u64,
    pub override_applied: Option<PrerequisiteOverride>,
}
```

### PrerequisiteOverride

Admin override for exceptional cases:

```rust
pub struct PrerequisiteOverride {
    pub student: Address,
    pub course_id: String,
    pub overridden_prerequisites: Vec<String>,
    pub override_reason: String,
    pub granted_by: Address,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}
```

### LearningPath

Generated learning sequence for students:

```rust
pub struct LearningPath {
    pub student: Address,
    pub target_course: String,
    pub recommended_sequence: Vec<String>,
    pub estimated_total_time: u64,
    pub current_position: u32,
    pub generated_at: u64,
    pub last_updated: u64,
}
```

## API Reference

### Administrative Functions

#### define_prerequisites

Define prerequisites for a course.

```rust
fn define_prerequisites(
    env: Env,
    admin: Address,
    course_prerequisite: CoursePrerequisite,
) -> Result<(), CertificateError>
```

**Parameters:**
- `admin`: Administrator address with appropriate permissions
- `course_prerequisite`: Complete prerequisite definition

**Returns:**
- `Ok(())` on success
- `CertificateError` on validation failure or unauthorized access

**Events Emitted:**
- `prerequisite_defined`

#### grant_prerequisite_override

Grant a prerequisite override for a student.

```rust
fn grant_prerequisite_override(
    env: Env,
    admin: Address,
    override_data: PrerequisiteOverride,
) -> Result<(), CertificateError>
```

**Parameters:**
- `admin`: Administrator address
- `override_data`: Override details including reason and expiration

**Returns:**
- `Ok(())` on success
- `CertificateError` on validation failure

**Events Emitted:**
- `prerequisite_override_granted`

#### revoke_prerequisite_override

Revoke an existing prerequisite override.

```rust
fn revoke_prerequisite_override(
    env: Env,
    admin: Address,
    student: Address,
    course_id: String,
    reason: String,
) -> Result<(), CertificateError>
```

**Parameters:**
- `admin`: Administrator address
- `student`: Student address
- `course_id`: Course identifier
- `reason`: Reason for revocation

**Events Emitted:**
- `prerequisite_override_revoked`

### Student Functions

#### check_prerequisites

Check if a student meets prerequisites for a course.

```rust
fn check_prerequisites(
    env: Env,
    student: Address,
    course_id: String,
    progress_contract: Address,
) -> Result<PrerequisiteCheckResult, CertificateError>
```

**Parameters:**
- `student`: Student address
- `course_id`: Target course identifier
- `progress_contract`: Progress tracking contract address

**Returns:**
- `PrerequisiteCheckResult` with detailed eligibility information

**Events Emitted:**
- `prerequisite_checked`

#### generate_learning_path

Generate an optimal learning path for a student.

```rust
fn generate_learning_path(
    env: Env,
    student: Address,
    target_course: String,
    progress_contract: Address,
) -> Result<LearningPath, CertificateError>
```

**Parameters:**
- `student`: Student address
- `target_course`: Desired target course
- `progress_contract`: Progress tracking contract address

**Returns:**
- `LearningPath` with recommended course sequence

**Events Emitted:**
- `learning_path_generated`

#### validate_enrollment

Validate course enrollment against prerequisites.

```rust
fn validate_enrollment(
    env: Env,
    student: Address,
    course_id: String,
    enrolled_by: Address,
    progress_contract: Address,
) -> Result<(), CertificateError>
```

**Parameters:**
- `student`: Student address
- `course_id`: Course to enroll in
- `enrolled_by`: Address performing enrollment
- `progress_contract`: Progress tracking contract address

**Returns:**
- `Ok(())` if enrollment is valid
- `CertificateError::PrerequisiteNotMet` if prerequisites not satisfied

**Events Emitted:**
- `enrollment_validated`
- `prerequisite_violation` (on failure)

### Query Functions

#### get_course_prerequisites

Get prerequisite definition for a course.

```rust
fn get_course_prerequisites(
    env: Env,
    course_id: String,
) -> Option<CoursePrerequisite>
```

#### get_prerequisite_override

Get active override for a student and course.

```rust
fn get_prerequisite_override(
    env: Env,
    student: Address,
    course_id: String,
) -> Option<PrerequisiteOverride>
```

#### get_dependency_graph

Get dependency graph for a course.

```rust
fn get_dependency_graph(
    env: Env,
    course_id: String,
) -> Option<CourseDependencyNode>
```

#### get_learning_path

Get stored learning path for a student.

```rust
fn get_learning_path(
    env: Env,
    student: Address,
    target_course: String,
) -> Option<LearningPath>
```

#### get_prerequisite_violations

Get prerequisite violations for a student.

```rust
fn get_prerequisite_violations(
    env: Env,
    student: Address,
) -> Vec<PrerequisiteViolation>
```

## Workflow Examples

### Basic Prerequisite Setup

1. **Define Prerequisites**:
```rust
let prerequisite = CoursePrerequisite {
    course_id: String::from_str(&env, "advanced_rust"),
    prerequisite_courses: vec![
        PrerequisiteCourse {
            course_id: String::from_str(&env, "basic_rust"),
            minimum_percentage: 80,
            weight: 1,
            required_certificate: false,
        }
    ],
    minimum_completion_percentage: 80,
    policy: PrerequisitePolicy::Strict,
    created_by: admin.clone(),
    created_at: env.ledger().timestamp(),
    updated_at: env.ledger().timestamp(),
};

contract.define_prerequisites(admin, prerequisite)?;
```

2. **Check Student Eligibility**:
```rust
let result = contract.check_prerequisites(
    student,
    String::from_str(&env, "advanced_rust"),
    progress_contract,
)?;

if result.eligible {
    // Student can enroll
    println!("Student eligible for advanced_rust");
} else {
    // Show missing prerequisites
    for missing in result.missing_prerequisites {
        println!("Missing: {} ({}% required)", 
                missing.course_id, missing.required_percentage);
    }
}
```

### Learning Path Generation

```rust
let learning_path = contract.generate_learning_path(
    student,
    String::from_str(&env, "machine_learning"),
    progress_contract,
)?;

println!("Recommended sequence:");
for (i, course) in learning_path.recommended_sequence.iter().enumerate() {
    println!("{}. {}", i + 1, course);
}

println!("Estimated time: {} hours", 
         learning_path.estimated_total_time / 3600);
```

### Admin Override Management

```rust
// Grant override
let override_data = PrerequisiteOverride {
    student: student.clone(),
    course_id: String::from_str(&env, "advanced_rust"),
    overridden_prerequisites: vec![
        String::from_str(&env, "basic_rust")
    ],
    override_reason: String::from_str(&env, "Industry experience equivalent"),
    granted_by: admin.clone(),
    granted_at: env.ledger().timestamp(),
    expires_at: Some(env.ledger().timestamp() + 86400 * 30), // 30 days
};

contract.grant_prerequisite_override(admin, override_data)?;

// Later revoke if needed
contract.revoke_prerequisite_override(
    admin,
    student,
    String::from_str(&env, "advanced_rust"),
    String::from_str(&env, "Override no longer needed"),
)?;
```

## Events

### prerequisite_defined
Emitted when prerequisites are defined for a course.

**Topics:** `("prerequisite_defined", course_id)`
**Data:** `(admin, prerequisite_count, timestamp)`

### prerequisite_checked
Emitted when prerequisite check is performed.

**Topics:** `("prerequisite_checked", student, course_id)`
**Data:** `(eligible, missing_count, timestamp)`

### prerequisite_override_granted
Emitted when admin grants prerequisite override.

**Topics:** `("prerequisite_override_granted", student, course_id)`
**Data:** `(admin, reason, timestamp)`

### prerequisite_override_revoked
Emitted when admin revokes prerequisite override.

**Topics:** `("prerequisite_override_revoked", student, course_id)`
**Data:** `(admin, reason, timestamp)`

### prerequisite_violation
Emitted when prerequisite violation occurs.

**Topics:** `("prerequisite_violation", student, course_id)`
**Data:** `(attempted_by, missing_count, timestamp)`

### learning_path_generated
Emitted when learning path is generated.

**Topics:** `("learning_path_generated", student, target_course)`
**Data:** `(path_length, estimated_time, timestamp)`

### enrollment_validated
Emitted when enrollment validation occurs.

**Topics:** `("enrollment_validated", student, course_id)`
**Data:** `(enrolled_by, validation_result, timestamp)`

## Error Handling

### Common Errors

- `PrerequisiteNotMet`: Student doesn't meet required prerequisites
- `PrerequisiteNotFound`: No prerequisites defined for course
- `PrerequisiteAlreadyExists`: Prerequisites already defined for course
- `CircularDependency`: Circular dependency detected in prerequisites
- `InvalidPrerequisiteConfig`: Invalid prerequisite configuration
- `PrerequisiteOverrideNotFound`: Override not found for revocation
- `PrerequisiteOverrideExpired`: Override has expired
- `InsufficientProgress`: Student progress insufficient for requirements
- `CertificateRequired`: Certificate required but not obtained
- `InvalidLearningPath`: Learning path generation failed

### Error Recovery

1. **Prerequisite Not Met**: Generate learning path to show required courses
2. **Circular Dependencies**: Review and restructure prerequisite definitions
3. **Override Expired**: Grant new override or require prerequisite completion
4. **Invalid Configuration**: Validate prerequisite parameters before storage

## Security Considerations

### Access Control

- **Administrative Functions**: Require `UpdateCertificateMetadata` permission
- **Override Management**: Restricted to authorized administrators
- **Audit Trail**: All actions logged with timestamps and actor addresses

### Data Validation

- **Prerequisite Configuration**: Validate percentages, weights, and course IDs
- **Override Requests**: Require valid reasons and expiration dates
- **Circular Dependencies**: Prevent infinite loops in prerequisite chains

### Performance Optimization

- **Caching**: Prerequisite check results cached for performance
- **Batch Operations**: Support for bulk prerequisite operations
- **Efficient Storage**: Optimized data structures for gas efficiency

## Integration Examples

### With Progress Tracking

```rust
// Check student progress before prerequisite validation
let progress = progress_contract.get_completion_percentage(
    env.clone(),
    student.clone(),
    course_id.clone(),
)?;

if progress >= prerequisite.minimum_percentage {
    // Student meets progress requirement
    mark_prerequisite_completed(&prerequisite);
}
```

### With Certificate System

```rust
// Validate certificate requirements
if prerequisite_course.required_certificate {
    let has_certificate = certificate_contract.has_valid_certificate(
        env.clone(),
        student.clone(),
        prerequisite_course.course_id.clone(),
    )?;
    
    if !has_certificate {
        return Err(CertificateError::CertificateRequired);
    }
}
```

### With Enrollment System

```rust
// Validate prerequisites before enrollment
contract.validate_enrollment(
    student.clone(),
    course_id.clone(),
    instructor.clone(),
    progress_contract.clone(),
)?;

// Proceed with enrollment if validation passes
enrollment_contract.enroll_student(student, course_id)?;
```

## Testing

### Unit Tests

- Prerequisite definition validation
- Circular dependency detection
- Override management
- Learning path generation
- Configuration validation

### Integration Tests

- End-to-end prerequisite workflows
- Complex dependency resolution
- Override expiration handling
- Violation tracking
- Multi-policy scenarios

### Performance Tests

- Large dependency graphs
- Bulk prerequisite operations
- Concurrent access patterns
- Storage optimization

## Future Enhancements

### Planned Features

1. **Dynamic Prerequisites**: Prerequisites that change based on student performance
2. **Prerequisite Templates**: Reusable prerequisite patterns
3. **Advanced Analytics**: Prerequisite effectiveness tracking
4. **Machine Learning**: Intelligent prerequisite recommendations
5. **Mobile Integration**: Mobile-friendly prerequisite checking

### Scalability Improvements

1. **Batch Processing**: Bulk prerequisite operations
2. **Caching Layer**: Enhanced caching for frequent operations
3. **Indexing**: Improved query performance for large datasets
4. **Compression**: Optimized storage for prerequisite data

## Conclusion

The Course Prerequisites System provides a robust foundation for managing educational progression within the StrellerMinds ecosystem. With comprehensive validation, flexible policies, and intelligent path generation, it ensures students follow optimal learning sequences while providing administrators the tools needed for exceptional case management.

The system's integration with existing certificate and progress tracking systems creates a seamless educational experience that maintains academic rigor while supporting diverse learning paths and institutional requirements.
