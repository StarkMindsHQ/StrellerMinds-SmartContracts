# Learning Path Templates Contract

## Overview

The Learning Path Templates contract provides pre-built learning path templates for common careers in the StrellerMinds education platform. This contract enables educators and institutions to quickly deploy structured learning paths that can be customized to meet specific needs while maintaining consistency and quality.

## Features

### Predefined Career Templates
- **Web Development**: Complete full-stack development path
- **Data Science**: Comprehensive data science and machine learning program
- **Cloud Architecture**: Enterprise cloud computing and architecture
- **Blockchain Development**: Complete blockchain and Web3 development
- **Cybersecurity**: Professional security and ethical hacking program

### Core Functionality
- Template creation and management
- Template customization for individual needs
- Course generation from templates
- Progress tracking integration
- Statistics and analytics
- Role-based access control

## Interface

### Core Functions

```rust
// Initialize the contract with admin
fn initialize(env: Env, admin: Address) -> Result<(), Error>

// Create a new learning path template
fn create_template(
    env: Env,
    admin: Address,
    template_id: String,
    title: String,
    description: String,
    category: String,
    difficulty_level: String,
    estimated_hours: u32,
    modules: Vec<String>,
    prerequisites: Vec<String>,
    skills_gained: Vec<String>,
) -> Result<(), Error>

// Get template by ID
fn get_template(env: Env, template_id: String) -> Result<TemplateData, Error>

// List all available templates
fn list_templates(env: Env) -> Result<Vec<TemplateData>, Error>

// Get templates by category
fn get_templates_by_category(env: Env, category: String) -> Result<Vec<TemplateData>, Error>

// Customize an existing template
fn customize_template(
    env: Env,
    user: Address,
    template_id: String,
    custom_title: Option<String>,
    custom_description: Option<String>,
    custom_modules: Option<Vec<String>>,
    custom_skills: Option<Vec<String>>,
) -> Result<CustomizedTemplate, Error>

// Apply template to create a course
fn apply_template(
    env: Env,
    admin: Address,
    template_id: String,
    course_id: Symbol,
    instructor: Address,
) -> Result<(), Error>

// Get template statistics
fn get_template_stats(env: Env) -> Result<TemplateStats, Error>
```

## Predefined Templates

### Web Development Template
- **Duration**: 320 hours
- **Difficulty**: Intermediate
- **Modules**: 10 comprehensive modules
- **Skills**: Frontend, Backend, Database, API Design, Version Control
- **Prerequisites**: Basic computer literacy, Problem-solving skills

### Data Science Template
- **Duration**: 400 hours
- **Difficulty**: Advanced
- **Modules**: 10 specialized modules
- **Skills**: Data Analysis, Machine Learning, Statistical Modeling, Data Visualization
- **Prerequisites**: Programming knowledge, Mathematics, Statistics basics

### Cloud Architecture Template
- **Duration**: 350 hours
- **Difficulty**: Advanced
- **Modules**: 11 comprehensive modules
- **Skills**: Cloud Management, Infrastructure as Code, Security, Cost Optimization
- **Prerequisites**: Networking, System Administration, Security basics

### Blockchain Development Template
- **Duration**: 380 hours
- **Difficulty**: Advanced
- **Modules**: 11 specialized modules
- **Skills**: Smart Contracts, Blockchain Architecture, DeFi, Web3 Integration
- **Prerequisites**: JavaScript, Distributed Systems, Cryptography concepts

### Cybersecurity Template
- **Duration**: 420 hours
- **Difficulty**: Advanced
- **Modules**: 11 comprehensive modules
- **Skills**: Security Assessment, Penetration Testing, Incident Response, Compliance
- **Prerequisites**: Networking, Operating Systems, Scripting fundamentals

## Data Structures

### TemplateData
```rust
struct TemplateData {
    id: String,
    title: String,
    description: String,
    category: String,
    difficulty_level: String,
    estimated_hours: u32,
    modules: Vec<String>,
    prerequisites: Vec<String>,
    skills_gained: Vec<String>,
    created_at: u64,
    is_active: bool,
}
```

### CustomizedTemplate
```rust
struct CustomizedTemplate {
    original_template_id: String,
    customized_by: Address,
    custom_title: String,
    custom_description: String,
    custom_modules: Vec<String>,
    custom_skills: Vec<String>,
    customized_at: u64,
}
```

### CourseFromTemplate
```rust
struct CourseFromTemplate {
    course_id: Symbol,
    template_id: String,
    title: String,
    description: String,
    instructor: Address,
    modules: Vec<String>,
    total_modules: u32,
    created_at: u64,
    is_active: bool,
}
```

## Usage Examples

### Initialize Contract with Predefined Templates
```rust
// Initialize contract
let admin = Address::generate(&env);
learning_path_templates.initialize(env.clone(), admin.clone())?;

// Load all predefined templates
templates::PredefinedTemplates::initialize_all_templates(&env, admin)?;
```

### Create Custom Template
```rust
let template_id = String::from_str(&env, "mobile_dev_101");
let title = String::from_str(&env, "Mobile App Development");
let description = String::from_str(&env, "Complete mobile development course");

learning_path_templates.create_template(
    env.clone(),
    admin,
    template_id,
    title,
    description,
    String::from_str(&env, "Mobile Development"),
    String::from_str(&env, "Intermediate"),
    280,
    modules,
    prerequisites,
    skills_gained,
)?;
```

### Customize Template for Student
```rust
let customized = learning_path_templates.customize_template(
    env.clone(),
    student_address,
    String::from_str(&env, "web_dev_101"),
    Some(String::from_str(&env, "Web Development for Beginners")),
    None, // Keep original description
    Some(custom_modules), // Add extra modules
    Some(custom_skills),  // Focus on specific skills
)?;
```

### Apply Template to Create Course
```rust
let course_id = Symbol::short(&env, "WD2024");
let instructor = Address::generate(&env);

learning_path_templates.apply_template(
    env.clone(),
    admin,
    String::from_str(&env, "web_dev_101"),
    course_id,
    instructor,
)?;
```

## Integration with Progress Tracking

The Learning Path Templates contract integrates seamlessly with the Progress contract:

1. **Course Registration**: When a template is applied, the course is automatically registered with the progress contract
2. **Module Mapping**: Template modules are mapped to progress tracking modules
3. **Progress Updates**: Student progress is tracked through the standard progress contract interface
4. **Completion Tracking**: Template completion is monitored through the progress system

## Error Codes

| Code | Description |
|------|-------------|
| 1 | Already initialized |
| 2 | Template already exists |
| 3 | Template not found |
| 4 | Not initialized |
| 5 | Unauthorized |

## Testing

### Running Tests
```bash
# Run all tests for learning path templates contract
cargo test --package learning-path-templates

# Run specific test modules
cargo test --package learning-path-templates test::test_initialization
cargo test --package learning-path-templates test::test_template_creation
cargo test --package learning-path-templates test::test_template_customization
cargo test --package learning-path-templates test::test_predefined_templates
```

### Test Coverage
- **Initialization Tests**: Contract setup and admin configuration
- **Template Creation Tests**: Template creation and validation
- **Template Retrieval Tests**: Template listing and filtering
- **Customization Tests**: Template customization functionality
- **Integration Tests**: Progress tracking integration
- **Predefined Templates Tests**: All five career path templates

## Security Considerations

### Access Control
- Admin-only functions for template creation and course application
- User-specific customization tracking
- Proper authorization checks for all operations

### Data Validation
- Template ID uniqueness validation
- Module count validation
- Prerequisite and skills validation
- Input sanitization

### Storage Optimization
- Efficient storage of template data
- Separation of active and inactive templates
- User-specific customized template storage

## Deployment

### Prerequisites
- Admin address for contract initialization
- Progress contract deployed and accessible
- Proper network configuration

### Deployment Steps
1. Deploy the learning path templates contract
2. Initialize with admin address
3. Load predefined templates using `initialize_all_templates`
4. Configure integration with progress contract
5. Begin template and course management

## Future Enhancements

### Planned Features
- Template versioning and updates
- Template sharing between institutions
- Advanced customization options
- Template performance analytics
- AI-powered template recommendations
- Multi-language template support

### Scalability Improvements
- Batch template operations
- Template marketplace
- Community-contributed templates
- Template rating and review system

## Related Documentation

- [Progress Contract](../progress/README.md)
- [Analytics Contract](../analytics/README.md)
- [Shared Utilities](../shared/README.md)
- [Development Guide](../../docs/development.md)

## License

This contract is part of the StrellerMinds Smart Contracts suite and is licensed under the same terms as the main project.
