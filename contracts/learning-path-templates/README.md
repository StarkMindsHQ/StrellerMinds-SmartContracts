# Learning Path Templates Contract

## Overview

The Learning Path Templates contract provides a comprehensive system for creating, managing, and customizing educational learning paths on the blockchain. This contract enables educational institutions and students to create structured career paths with predefined modules, track progress, and customize learning experiences.

## Features

### Core Functionality
- **Template Management**: Create and manage reusable learning path templates
- **Career-Specific Paths**: Pre-built templates for common career tracks
- **Customization**: Students can customize templates to fit their needs
- **Progress Tracking**: Track completion progress for each module
- **Role-Based Access**: Admin controls and student self-management

### Pre-built Career Templates
1. **Web Development** - Full-stack web development path
2. **Data Science** - Complete data science curriculum
3. **Cloud Architecture** - Cloud computing and architecture
4. **Blockchain Development** - Blockchain and smart contract development
5. **Cybersecurity** - Comprehensive security training

## Data Structures

### LearningModule
```rust
pub struct LearningModule {
    pub id: Symbol,              // Unique module identifier
    pub title: String,          // Module title
    pub description: String,    // Module description
    pub duration_hours: u32,    // Estimated completion time
    pub difficulty_level: u8,    // 1-5 difficulty scale
    pub prerequisites: Vec<Symbol>, // Required modules
    pub skills_gained: Vec<String>, // Skills acquired
    pub resources: Vec<String>,     // Learning resources
}
```

### LearningPath
```rust
pub struct LearningPath {
    pub id: Symbol,                    // Unique path identifier
    pub title: String,                 // Path title
    pub description: String,           // Path description
    pub career_track: String,         // Career category
    pub total_duration_hours: u32,     // Total completion time
    pub difficulty_level: u8,         // Overall difficulty
    pub modules: Vec<LearningModule>, // Path modules
    pub is_template: bool,            // Template flag
    pub is_customizable: bool,         // Customization allowed
    pub created_by: Address,           // Creator address
    pub created_at: u64,              // Creation timestamp
}
```

### CustomizedPath
```rust
pub struct CustomizedPath {
    pub template_id: Symbol,                    // Original template
    pub customized_path_id: Symbol,             // Customized path ID
    pub student_address: Address,               // Student owner
    pub modifications: Map<Symbol, LearningModule>, // Module changes
    pub completion_progress: Map<Symbol, u32>,  // Progress tracking
    pub customized_at: u64,                     // Customization timestamp
}
```

## Contract Interface

### Core Functions

#### `initialize(env: Env, admin: Address)`
Initialize the contract with an admin address.
- **Authorization**: Admin only
- **Events**: None

#### `create_template(env: Env, template_id: Symbol, title: String, description: String, career_track: String, modules: Vec<LearningModule>, is_customizable: bool, creator: Address)`
Create a new learning path template.
- **Authorization**: Admin or creator
- **Events**: `template_created`

#### `get_template(env: Env, template_id: Symbol) -> Option<LearningPath>`
Retrieve a learning path template by ID.
- **Authorization**: Public
- **Returns**: Template data or None

#### `get_all_templates(env: Env) -> Vec<LearningPath>`
Get all available templates.
- **Authorization**: Public
- **Returns**: Vector of all templates

#### `customize_template(env: Env, template_id: Symbol, student: Address, modifications: Map<Symbol, LearningModule>) -> Symbol`
Customize a template for a student.
- **Authorization**: Student only
- **Returns**: Customized path ID
- **Events**: `path_customized`

#### `get_customized_path(env: Env, customized_path_id: Symbol) -> Option<CustomizedPath>`
Retrieve a customized learning path.
- **Authorization**: Public
- **Returns**: Customized path data or None

#### `get_student_paths(env: Env, student: Address) -> Vec<CustomizedPath>`
Get all customized paths for a student.
- **Authorization**: Public
- **Returns**: Vector of student's paths

#### `update_progress(env: Env, customized_path_id: Symbol, module_id: Symbol, progress_percent: u32, student: Address)`
Update progress for a specific module.
- **Authorization**: Student only
- **Events**: `progress_updated`

#### `get_templates_by_career(env: Env, career_track: String) -> Vec<LearningPath>`
Get templates filtered by career track.
- **Authorization**: Public
- **Returns**: Filtered template list

#### `delete_template(env: Env, template_id: Symbol, admin: Address)`
Delete a template (admin only).
- **Authorization**: Admin only
- **Events**: `template_deleted`

#### `get_admin(env: Env) -> Address`
Get the contract admin address.
- **Authorization**: Public
- **Returns**: Admin address

### Template Initialization Functions

#### `initialize_prebuilt_templates(env: Env, admin: Address)`
Initialize all pre-built career templates.
- **Authorization**: Admin only
- **Templates Created**: Web Development, Data Science, Cloud Architecture, Blockchain Development, Cybersecurity

## Events

### Template Events
- `template_created`: Emitted when a new template is created
  - Data: `(template_id, creator)`
- `template_deleted`: Emitted when a template is deleted
  - Data: `(template_id, admin)`

### Path Events
- `path_customized`: Emitted when a template is customized
  - Data: `(customized_path_id, student, template_id)`

### Progress Events
- `progress_updated`: Emitted when progress is updated
  - Data: `(customized_path_id, module_id, progress_percent, student)`

## Usage Examples

### Initializing the Contract
```rust
let admin = Address::generate(&env);
client.initialize(&admin);
```

### Creating a Custom Template
```rust
let modules = Vec::from_array(&env, [
    LearningModule {
        id: symbol_short!("intro"),
        title: String::from_str(&env, "Introduction"),
        description: String::from_str(&env, "Basic concepts"),
        duration_hours: 20,
        difficulty_level: 1,
        prerequisites: Vec::new(&env),
        skills_gained: Vec::from_array(&env, [String::from_str(&env, "Basics")]),
        resources: Vec::from_array(&env, [String::from_str(&env, "Resource 1")]),
    }
]);

client.create_template(
    &symbol_short!("my_template"),
    &String::from_str(&env, "My Template"),
    &String::from_str(&env, "Description"),
    &String::from_str(&env, "Career Track"),
    &modules,
    &true,
    &admin,
);
```

### Initializing Pre-built Templates
```rust
client.initialize_prebuilt_templates(&admin);
```

### Customizing a Template
```rust
let student = Address::generate(&env);
let modifications = Map::new(&env);
let customized_id = client.customize_template(
    &symbol_short!("web_dev"),
    &student,
    &modifications,
);
```

### Updating Progress
```rust
client.update_progress(
    &customized_id,
    &symbol_short!("html_css"),
    &75u32,
    &student,
);
```

### Getting Student Progress
```rust
let student_paths = client.get_student_paths(&student);
for path in student_paths.iter() {
    let progress = path.completion_progress;
    // Process progress data
}
```

## Pre-built Templates Details

### Web Development Career Path
**Total Duration**: 275 hours
**Modules**:
1. HTML & CSS Fundamentals (40h, Level 1)
2. JavaScript Programming (60h, Level 2)
3. React.js Framework (50h, Level 3)
4. Node.js & Express (45h, Level 3)
5. Full Stack Project (80h, Level 4)

### Data Science Career Path
**Total Duration**: 350 hours
**Modules**:
1. Python for Data Science (50h, Level 1)
2. Statistics & Probability (60h, Level 2)
3. Machine Learning Fundamentals (70h, Level 3)
4. Deep Learning & Neural Networks (80h, Level 4)
5. Data Science Projects (90h, Level 4)

### Cloud Architecture Career Path
**Total Duration**: 265 hours
**Modules**:
1. Cloud Computing Fundamentals (40h, Level 1)
2. Cloud Networking (50h, Level 2)
3. Cloud Storage Solutions (45h, Level 2)
4. Cloud Security (60h, Level 3)
5. Cloud Architecture Design (70h, Level 4)

### Blockchain Development Career Path
**Total Duration**: 310 hours
**Modules**:
1. Blockchain Fundamentals (45h, Level 1)
2. Solidity Programming (60h, Level 2)
3. Ethereum Development (55h, Level 3)
4. DeFi Development (65h, Level 4)
5. Blockchain Projects (85h, Level 4)

### Cybersecurity Career Path
**Total Duration**: 320 hours
**Modules**:
1. Cybersecurity Fundamentals (50h, Level 1)
2. Network Security (60h, Level 2)
3. Ethical Hacking (70h, Level 3)
4. Security Operations (65h, Level 3)
5. Advanced Security Topics (75h, Level 4)

## Testing

### Running Tests
```bash
# Run all tests
cargo test --package learning-path-templates

# Run specific test
cargo test --package learning-path-templates test_initialization
```

### Test Coverage
- **Initialization Tests**: Contract setup and admin configuration
- **Template Management Tests**: Creation, retrieval, and deletion
- **Customization Tests**: Template customization and restrictions
- **Progress Tracking Tests**: Progress updates and validation
- **Authorization Tests**: Role-based access control
- **Pre-built Templates Tests**: Template structure and content
- **Edge Case Tests**: Error handling and boundary conditions

## Security Considerations

### Access Control
- **Admin Functions**: Only admin can initialize contract and delete templates
- **Student Functions**: Students can only modify their own paths and progress
- **Public Functions**: Template retrieval is publicly accessible

### Input Validation
- **Progress Range**: Progress is validated to be 0-100%
- **Template Existence**: Operations check for template existence
- **Customization Permissions**: Only customizable templates can be modified

### Data Integrity
- **Unique IDs**: Template and path IDs are generated to ensure uniqueness
- **Immutable Templates**: Original templates cannot be modified by students
- **Progress Tracking**: Progress is stored per student per customized path

## Integration Points

### With Student Progress Tracker
- **Complementary Tracking**: This contract manages path structure while progress tracker handles detailed module progress
- **Data Sharing**: Can share progress data between contracts
- **Event Integration**: Progress events can trigger updates in progress tracker

### With Analytics Contract
- **Learning Analytics**: Provides data for learning analytics
- **Performance Metrics**: Enables detailed performance tracking
- **Path Optimization**: Supports data-driven path improvements

### With Token Contract
- **Incentive Integration**: Can reward progress completion
- **Achievement System**: Supports token-based achievement rewards
- **Staking Mechanisms**: Enable staking for path commitments

## Deployment

### Prerequisites
- Admin address for initialization
- Soroban CLI for deployment
- Testnet/Mainnet configuration

### Deployment Steps
1. Deploy the contract
2. Initialize with admin address
3. Initialize pre-built templates (optional)
4. Configure access control
5. Set up event monitoring

### Environment Variables
- `STELLAR_SECRET_KEY`: Admin secret key for deployment
- `SOROBAN_RPC_URL`: RPC endpoint for contract interaction

## Gas Optimization

### Storage Efficiency
- **Symbol Usage**: Uses Soroban symbols for efficient storage
- **Map Organization**: Efficient data organization for quick lookup
- **Persistent Storage**: Progress data stored persistently

### Computation Efficiency
- **Batch Operations**: Supports batch operations where possible
- **Event Optimization**: Minimal event emission for reduced gas costs
- **Lazy Loading**: Data loaded only when needed

## Future Enhancements

### Planned Features
- **Skill Assessment**: Automated skill level assessment
- **AI Recommendations**: AI-powered path recommendations
- **Social Learning**: Collaborative learning features
- **Certification Integration**: Automated certificate issuance
- **Marketplace**: Template marketplace for educators

### Scalability Improvements
- **Sharding Support**: Support for large-scale deployments
- **Caching Layer**: On-chain caching for frequently accessed data
- **Batch Processing**: Support for batch operations
- **Cross-chain Support**: Multi-chain template sharing

## License

This contract is part of the StrellerMinds Smart Contracts suite and is licensed under the same terms as the main project.
