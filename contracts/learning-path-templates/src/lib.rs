#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, String, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct LearningModule {
    pub id: Symbol,
    pub title: String,
    pub description: String,
    pub duration_hours: u32,
    pub difficulty_level: u32, // 1-5 scale
    pub prerequisites: Vec<Symbol>,
    pub skills_gained: Vec<String>,
    pub resources: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct LearningPath {
    pub id: Symbol,
    pub title: String,
    pub description: String,
    pub career_track: String,
    pub total_duration_hours: u32,
    pub difficulty_level: u32,
    pub modules: Vec<LearningModule>,
    pub is_template: bool,
    pub is_customizable: bool,
    pub created_by: Address,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct CustomizedPath {
    pub template_id: Symbol,
    pub customized_path_id: Symbol,
    pub student_address: Address,
    pub modifications: Map<Symbol, LearningModule>, // module_id -> modified module
    pub completion_progress: Map<Symbol, u32>,      // module_id -> percentage
    pub customized_at: u64,
}

#[contracttype]
pub enum DataKey {
    Template(Symbol),       // template_id -> LearningPath
    CustomizedPath(Symbol), // customized_path_id -> CustomizedPath
    StudentPaths(Address),  // student_address -> Vec<customized_path_id>
    TemplateIndex,          // Vec<Symbol> - all template IDs
    Admin,                  // Address
    PathCounter,            // u64 - for generating unique IDs
}

#[contract]
pub struct LearningPathTemplates;

#[contractimpl]
impl LearningPathTemplates {
    /// Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        if env.storage().instance().get::<DataKey, Address>(&DataKey::Admin).is_some() {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TemplateIndex, &Vec::<Symbol>::new(&env));
        env.storage().instance().set(&DataKey::PathCounter, &0u64);
    }

    /// Create a new learning path template
    pub fn create_template(
        env: Env,
        template_id: Symbol,
        title: String,
        description: String,
        career_track: String,
        modules: Vec<LearningModule>,
        is_customizable: bool,
        creator: Address,
    ) {
        let admin = Self::get_admin(env.clone());
        if creator != admin {
            creator.require_auth();
        }

        // Check if template already exists
        if env
            .storage()
            .instance()
            .get::<DataKey, LearningPath>(&DataKey::Template(template_id.clone()))
            .is_some()
        {
            panic!("template already exists");
        }

        // Calculate total duration and determine difficulty
        let mut total_duration = 0u32;
        let mut max_difficulty = 0u32;

        for module in modules.iter() {
            total_duration += module.duration_hours;
            if module.difficulty_level > max_difficulty {
                max_difficulty = module.difficulty_level;
            }
        }

        let learning_path = LearningPath {
            id: template_id.clone(),
            title,
            description,
            career_track,
            total_duration_hours: total_duration,
            difficulty_level: max_difficulty,
            modules,
            is_template: true,
            is_customizable,
            created_by: creator.clone(),
            created_at: env.ledger().timestamp(),
        };

        // Store the template
        env.storage().instance().set(&DataKey::Template(template_id.clone()), &learning_path);

        // Update template index
        let mut index: Vec<Symbol> =
            env.storage().instance().get(&DataKey::TemplateIndex).unwrap_or(Vec::new(&env));
        index.push_back(template_id.clone());
        env.storage().instance().set(&DataKey::TemplateIndex, &index);

        // Emit event
        env.events()
            .publish((symbol_short!("template"), symbol_short!("created")), (template_id, creator));
    }

    /// Get a learning path template by ID
    pub fn get_template(env: Env, template_id: Symbol) -> Option<LearningPath> {
        env.storage().instance().get(&DataKey::Template(template_id))
    }

    /// Get all available templates
    pub fn get_all_templates(env: Env) -> Vec<LearningPath> {
        let index: Vec<Symbol> =
            env.storage().instance().get(&DataKey::TemplateIndex).unwrap_or(Vec::new(&env));

        let mut templates = Vec::new(&env);
        for template_id in index.iter() {
            if let Some(template) = env.storage().instance().get(&DataKey::Template(template_id)) {
                templates.push_back(template);
            }
        }
        templates
    }

    /// Customize a learning path template for a student
    pub fn customize_template(
        env: Env,
        template_id: Symbol,
        student: Address,
        modifications: Map<Symbol, LearningModule>,
    ) -> Symbol {
        student.require_auth();

        // Get the template
        let template: LearningPath = env
            .storage()
            .instance()
            .get(&DataKey::Template(template_id.clone()))
            .expect("template not found");

        if !template.is_customizable {
            panic!("template is not customizable");
        }

        // Generate unique customized path ID
        let counter: u64 = env.storage().instance().get(&DataKey::PathCounter).unwrap_or(0u64);
        let new_counter = counter + 1;
        env.storage().instance().set(&DataKey::PathCounter, &new_counter);

        let customized_path_id = Symbol::new(&env, "custom_");

        // Create customized path
        let customized_path = CustomizedPath {
            template_id: template_id.clone(),
            customized_path_id: customized_path_id.clone(),
            student_address: student.clone(),
            modifications,
            completion_progress: Map::new(&env),
            customized_at: env.ledger().timestamp(),
        };

        // Store customized path
        env.storage()
            .instance()
            .set(&DataKey::CustomizedPath(customized_path_id.clone()), &customized_path);

        // Update student's paths
        let mut student_paths: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::StudentPaths(student.clone()))
            .unwrap_or(Vec::new(&env));
        student_paths.push_back(customized_path_id.clone());
        env.storage().instance().set(&DataKey::StudentPaths(student.clone()), &student_paths);

        // Emit event
        env.events().publish(
            (symbol_short!("path"), symbol_short!("custom")),
            (customized_path_id.clone(), student.clone(), template_id),
        );

        customized_path_id
    }

    /// Get a customized learning path
    pub fn get_customized_path(env: Env, customized_path_id: Symbol) -> Option<CustomizedPath> {
        env.storage().instance().get(&DataKey::CustomizedPath(customized_path_id))
    }

    /// Get all customized paths for a student
    pub fn get_student_paths(env: Env, student: Address) -> Vec<CustomizedPath> {
        let path_ids: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&DataKey::StudentPaths(student.clone()))
            .unwrap_or(Vec::new(&env));

        let mut paths = Vec::new(&env);
        for path_id in path_ids.iter() {
            if let Some(path) = env.storage().instance().get(&DataKey::CustomizedPath(path_id)) {
                paths.push_back(path);
            }
        }
        paths
    }

    /// Update progress for a customized path
    pub fn update_progress(
        env: Env,
        customized_path_id: Symbol,
        module_id: Symbol,
        progress_percent: u32,
        student: Address,
    ) {
        if progress_percent > 100 {
            panic!("progress cannot exceed 100");
        }

        student.require_auth();

        let mut customized_path: CustomizedPath = env
            .storage()
            .instance()
            .get(&DataKey::CustomizedPath(customized_path_id.clone()))
            .expect("customized path not found");

        if customized_path.student_address != student {
            panic!("not authorized to update this path");
        }

        // Update progress
        customized_path.completion_progress.set(module_id.clone(), progress_percent);

        // Store updated path
        env.storage()
            .instance()
            .set(&DataKey::CustomizedPath(customized_path_id.clone()), &customized_path);

        // Emit event
        env.events().publish(
            (symbol_short!("progress"), symbol_short!("updated")),
            (customized_path_id, module_id, progress_percent, student),
        );
    }

    /// Get templates by career track
    pub fn get_templates_by_career(env: Env, career_track: String) -> Vec<LearningPath> {
        let all_templates = Self::get_all_templates(env.clone());
        let mut filtered_templates = Vec::new(&env);

        for template in all_templates.iter() {
            if template.career_track == career_track {
                filtered_templates.push_back(template);
            }
        }
        filtered_templates
    }

    /// Delete a template (admin only)
    pub fn delete_template(env: Env, template_id: Symbol, admin: Address) {
        let contract_admin = Self::get_admin(env.clone());
        if admin != contract_admin {
            admin.require_auth();
        }

        // Remove template
        env.storage().instance().remove(&DataKey::Template(template_id.clone()));

        // Update index
        let mut index: Vec<Symbol> =
            env.storage().instance().get(&DataKey::TemplateIndex).unwrap_or(Vec::new(&env));

        let new_index: Vec<Symbol> = {
            let mut temp_vec = Vec::new(&env);
            for id in index.iter() {
                if id != template_id {
                    temp_vec.push_back(id.clone());
                }
            }
            temp_vec
        };
        env.storage().instance().set(&DataKey::TemplateIndex, &new_index);

        // Emit event
        env.events()
            .publish((symbol_short!("template"), symbol_short!("deleted")), (template_id, admin));
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("admin not set")
    }
}

// Pre-built template initialization functions
#[contractimpl]
impl LearningPathTemplates {
    /// Initialize all pre-built career templates
    pub fn initialize_prebuilt_templates(env: Env, admin: Address) {
        Self::initialize_web_dev_template(env.clone(), admin.clone());
        Self::initialize_data_science_template(env.clone(), admin.clone());
        Self::initialize_cloud_architecture_template(env.clone(), admin.clone());
        Self::initialize_blockchain_dev_template(env.clone(), admin.clone());
        Self::initialize_cybersecurity_template(env.clone(), admin);
    }

    /// Initialize Web Development template
    fn initialize_web_dev_template(env: Env, admin: Address) {
        let modules = Vec::from_array(
            &env,
            [
                LearningModule {
                    id: symbol_short!("html_css"),
                    title: String::from_str(&env, "HTML & CSS Fundamentals"),
                    description: String::from_str(&env, "Learn the building blocks of web pages"),
                    duration_hours: 40,
                    difficulty_level: 1u32,
                    prerequisites: Vec::new(&env),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "HTML5"),
                            String::from_str(&env, "CSS3"),
                            String::from_str(&env, "Responsive Design"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "MDN Web Docs"),
                            String::from_str(&env, "CSS Tricks"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("js"),
                    title: String::from_str(&env, "JavaScript Programming"),
                    description: String::from_str(&env, "Master JavaScript for web development"),
                    duration_hours: 60,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("html_css")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "ES6+"),
                            String::from_str(&env, "DOM Manipulation"),
                            String::from_str(&env, "Async Programming"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "JavaScript.info"),
                            String::from_str(&env, "Eloquent JavaScript"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("react"),
                    title: String::from_str(&env, "React.js Framework"),
                    description: String::from_str(&env, "Build modern web applications with React"),
                    duration_hours: 50,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("js")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Components"),
                            String::from_str(&env, "State Management"),
                            String::from_str(&env, "Hooks"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "React Documentation"),
                            String::from_str(&env, "React Tutorial"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("nodejs"),
                    title: String::from_str(&env, "Node.js & Express"),
                    description: String::from_str(&env, "Backend development with Node.js"),
                    duration_hours: 45,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("js")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Server-side JavaScript"),
                            String::from_str(&env, "REST APIs"),
                            String::from_str(&env, "Express.js"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Node.js Docs"),
                            String::from_str(&env, "Express.js Guide"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("fullstack"),
                    title: String::from_str(&env, "Full Stack Project"),
                    description: String::from_str(&env, "Build a complete full-stack application"),
                    duration_hours: 80,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(
                        &env,
                        [symbol_short!("react"), symbol_short!("nodejs")],
                    ),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Database Integration"),
                            String::from_str(&env, "Authentication"),
                            String::from_str(&env, "Deployment"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Full Stack Open"),
                            String::from_str(&env, "Dev.to Tutorials"),
                        ],
                    ),
                },
            ],
        );

        Self::create_template(
            env.clone(),
            symbol_short!("web_dev"),
            String::from_str(&env, "Web Development Career Path"),
            String::from_str(&env, "Complete path to become a full-stack web developer"),
            String::from_str(&env, "Web Development"),
            modules,
            true,
            admin,
        );
    }

    /// Initialize Data Science template
    fn initialize_data_science_template(env: Env, admin: Address) {
        let modules = Vec::from_array(
            &env,
            [
                LearningModule {
                    id: symbol_short!("py_basic"),
                    title: String::from_str(&env, "Python for Data Science"),
                    description: String::from_str(
                        &env,
                        "Python programming fundamentals for data analysis",
                    ),
                    duration_hours: 50,
                    difficulty_level: 1u32,
                    prerequisites: Vec::new(&env),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Python"),
                            String::from_str(&env, "NumPy"),
                            String::from_str(&env, "Pandas"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Python.org Tutorial"),
                            String::from_str(&env, "Real Python"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("stats"),
                    title: String::from_str(&env, "Statistics & Probability"),
                    description: String::from_str(
                        &env,
                        "Mathematical foundations for data science",
                    ),
                    duration_hours: 60,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("py_basic")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Descriptive Statistics"),
                            String::from_str(&env, "Probability Theory"),
                            String::from_str(&env, "Hypothesis Testing"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Khan Academy Statistics"),
                            String::from_str(&env, "Statistical Thinking"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("ml_basics"),
                    title: String::from_str(&env, "Machine Learning Fundamentals"),
                    description: String::from_str(
                        &env,
                        "Introduction to machine learning algorithms",
                    ),
                    duration_hours: 70,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(
                        &env,
                        [symbol_short!("py_basic"), symbol_short!("stats")],
                    ),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Supervised Learning"),
                            String::from_str(&env, "Unsupervised Learning"),
                            String::from_str(&env, "Model Evaluation"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Scikit-learn Documentation"),
                            String::from_str(&env, "Andrew Ng's ML Course"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("deep_lrn"),
                    title: String::from_str(&env, "Deep Learning & Neural Networks"),
                    description: String::from_str(
                        &env,
                        "Advanced neural network architectures and applications",
                    ),
                    duration_hours: 80,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("ml_basics")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Neural Networks"),
                            String::from_str(&env, "TensorFlow/PyTorch"),
                            String::from_str(&env, "CNN/RNN"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Deep Learning Book"),
                            String::from_str(&env, "Fast.ai Course"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("data_proj"),
                    title: String::from_str(&env, "Data Science Projects"),
                    description: String::from_str(&env, "Real-world data science projects"),
                    duration_hours: 90,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("ml_basics")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Data Cleaning"),
                            String::from_str(&env, "Feature Engineering"),
                            String::from_str(&env, "Model Deployment"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Kaggle Competitions"),
                            String::from_str(&env, "UCI ML Repository"),
                        ],
                    ),
                },
            ],
        );

        Self::create_template(
            env.clone(),
            symbol_short!("data_sc"),
            String::from_str(&env, "Data Science Career Path"),
            String::from_str(&env, "Complete path to become a data scientist"),
            String::from_str(&env, "Data Science"),
            modules,
            true,
            admin,
        );
    }

    /// Initialize Cloud Architecture template
    fn initialize_cloud_architecture_template(env: Env, admin: Address) {
        let modules = Vec::from_array(
            &env,
            [
                LearningModule {
                    id: symbol_short!("cld_basic"),
                    title: String::from_str(&env, "Cloud Computing Fundamentals"),
                    description: String::from_str(
                        &env,
                        "Understanding cloud concepts and providers",
                    ),
                    duration_hours: 40,
                    difficulty_level: 1u32,
                    prerequisites: Vec::new(&env),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Cloud Concepts"),
                            String::from_str(&env, "AWS/Azure/GCP"),
                            String::from_str(&env, "IaaS/PaaS/SaaS"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Cloud Computing Basics"),
                            String::from_str(&env, "AWS Cloud Practitioner"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("net"),
                    title: String::from_str(&env, "Cloud Networking"),
                    description: String::from_str(&env, "Network architecture in the cloud"),
                    duration_hours: 50,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("cld_basic")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "VPC"),
                            String::from_str(&env, "Load Balancing"),
                            String::from_str(&env, "CDN"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "AWS Networking Guide"),
                            String::from_str(&env, "Azure Networking"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("storage"),
                    title: String::from_str(&env, "Cloud Storage Solutions"),
                    description: String::from_str(&env, "Storage options and best practices"),
                    duration_hours: 45,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("cld_basic")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Object Storage"),
                            String::from_str(&env, "Database Storage"),
                            String::from_str(&env, "Backup Strategies"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "AWS Storage Guide"),
                            String::from_str(&env, "Cloud Storage Patterns"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("security"),
                    title: String::from_str(&env, "Cloud Security"),
                    description: String::from_str(
                        &env,
                        "Security best practices for cloud environments",
                    ),
                    duration_hours: 60,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(
                        &env,
                        [symbol_short!("cld_basic"), symbol_short!("net")],
                    ),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "IAM"),
                            String::from_str(&env, "Encryption"),
                            String::from_str(&env, "Compliance"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Cloud Security Guide"),
                            String::from_str(&env, "AWS Security Hub"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("arch"),
                    title: String::from_str(&env, "Cloud Architecture Design"),
                    description: String::from_str(&env, "Designing scalable cloud solutions"),
                    duration_hours: 70,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(
                        &env,
                        [symbol_short!("net"), symbol_short!("storage"), symbol_short!("security")],
                    ),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "High Availability"),
                            String::from_str(&env, "Auto Scaling"),
                            String::from_str(&env, "Microservices"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "AWS Well-Architected"),
                            String::from_str(&env, "Cloud Architecture Patterns"),
                        ],
                    ),
                },
            ],
        );

        Self::create_template(
            env.clone(),
            symbol_short!("cloud"),
            String::from_str(&env, "Cloud Architecture Career Path"),
            String::from_str(&env, "Complete path to become a cloud architect"),
            String::from_str(&env, "Cloud Architecture"),
            modules,
            true,
            admin,
        );
    }

    /// Initialize Blockchain Development template
    fn initialize_blockchain_dev_template(env: Env, admin: Address) {
        let modules = Vec::from_array(
            &env,
            [
                LearningModule {
                    id: symbol_short!("bc_basic"),
                    title: String::from_str(&env, "Blockchain Fundamentals"),
                    description: String::from_str(
                        &env,
                        "Understanding blockchain technology and concepts",
                    ),
                    duration_hours: 45,
                    difficulty_level: 1u32,
                    prerequisites: Vec::new(&env),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Distributed Ledgers"),
                            String::from_str(&env, "Consensus Mechanisms"),
                            String::from_str(&env, "Cryptography Basics"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Bitcoin Whitepaper"),
                            String::from_str(&env, "Ethereum Whitepaper"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("solidity"),
                    title: String::from_str(&env, "Solidity Programming"),
                    description: String::from_str(&env, "Smart contract development with Solidity"),
                    duration_hours: 60,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("bc_basic")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Smart Contracts"),
                            String::from_str(&env, "Solidity Syntax"),
                            String::from_str(&env, "Contract Patterns"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Solidity Documentation"),
                            String::from_str(&env, "CryptoZombies"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("ethereum"),
                    title: String::from_str(&env, "Ethereum Development"),
                    description: String::from_str(&env, "Building DApps on Ethereum"),
                    duration_hours: 55,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("solidity")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Web3.js"),
                            String::from_str(&env, "EVM"),
                            String::from_str(&env, "Gas Optimization"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Ethereum.org"),
                            String::from_str(&env, "OpenZeppelin"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("defi"),
                    title: String::from_str(&env, "DeFi Development"),
                    description: String::from_str(
                        &env,
                        "Building decentralized finance applications",
                    ),
                    duration_hours: 65,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("ethereum")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "DEX Development"),
                            String::from_str(&env, "Lending Protocols"),
                            String::from_str(&env, "Yield Farming"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "DeFi Pulse"),
                            String::from_str(&env, "Uniswap Docs"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("dapp_proj"),
                    title: String::from_str(&env, "Blockchain Projects"),
                    description: String::from_str(&env, "Complete blockchain application projects"),
                    duration_hours: 85,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("ethereum")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Full-Stack DApps"),
                            String::from_str(&env, "NFT Development"),
                            String::from_str(&env, "DAO Development"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Buildspace"),
                            String::from_str(&env, "Hardhat Framework"),
                        ],
                    ),
                },
            ],
        );

        Self::create_template(
            env.clone(),
            symbol_short!("bc_dev"),
            String::from_str(&env, "Blockchain Development Career Path"),
            String::from_str(&env, "Complete path to become a blockchain developer"),
            String::from_str(&env, "Blockchain Development"),
            modules,
            true,
            admin,
        );
    }

    /// Initialize Cybersecurity template
    fn initialize_cybersecurity_template(env: Env, admin: Address) {
        let modules = Vec::from_array(
            &env,
            [
                LearningModule {
                    id: symbol_short!("sec_basic"),
                    title: String::from_str(&env, "Cybersecurity Fundamentals"),
                    description: String::from_str(
                        &env,
                        "Introduction to cybersecurity concepts and principles",
                    ),
                    duration_hours: 50,
                    difficulty_level: 1u32,
                    prerequisites: Vec::new(&env),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Security Principles"),
                            String::from_str(&env, "Risk Management"),
                            String::from_str(&env, "Security Frameworks"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "NIST Framework"),
                            String::from_str(&env, "CompTIA Security+"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("net_sec"),
                    title: String::from_str(&env, "Network Security"),
                    description: String::from_str(
                        &env,
                        "Securing network infrastructure and communications",
                    ),
                    duration_hours: 60,
                    difficulty_level: 2u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("sec_basic")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Firewalls"),
                            String::from_str(&env, "IDS/IPS"),
                            String::from_str(&env, "VPN"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Cisco Security"),
                            String::from_str(&env, "Wireshark"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("eth_hack"),
                    title: String::from_str(&env, "Ethical Hacking"),
                    description: String::from_str(
                        &env,
                        "Penetration testing and vulnerability assessment",
                    ),
                    duration_hours: 70,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("net_sec")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Penetration Testing"),
                            String::from_str(&env, "Vulnerability Assessment"),
                            String::from_str(&env, "Security Tools"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Metasploit"),
                            String::from_str(&env, "Burp Suite"),
                        ],
                    ),
                },
                LearningModule {
                    id: symbol_short!("sec_ops"),
                    title: String::from_str(&env, "Security Operations"),
                    description: String::from_str(
                        &env,
                        "Security monitoring and incident response",
                    ),
                    duration_hours: 65,
                    difficulty_level: 3u32,
                    prerequisites: Vec::from_array(&env, [symbol_short!("net_sec")]),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "SIEM"),
                            String::from_str(&env, "Incident Response"),
                            String::from_str(&env, "Threat Hunting"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [String::from_str(&env, "Splunk"), String::from_str(&env, "MITRE ATT&CK")],
                    ),
                },
                LearningModule {
                    id: symbol_short!("adv_sec"),
                    title: String::from_str(&env, "Advanced Security Topics"),
                    description: String::from_str(
                        &env,
                        "Advanced cybersecurity concepts and specializations",
                    ),
                    duration_hours: 75,
                    difficulty_level: 4u32,
                    prerequisites: Vec::from_array(
                        &env,
                        [symbol_short!("eth_hack"), symbol_short!("sec_ops")],
                    ),
                    skills_gained: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "Cloud Security"),
                            String::from_str(&env, "Application Security"),
                            String::from_str(&env, "GRC"),
                        ],
                    ),
                    resources: Vec::from_array(
                        &env,
                        [
                            String::from_str(&env, "OWASP Top 10"),
                            String::from_str(&env, "Cloud Security Alliance"),
                        ],
                    ),
                },
            ],
        );

        Self::create_template(
            env.clone(),
            symbol_short!("cyber"),
            String::from_str(&env, "Cybersecurity Career Path"),
            String::from_str(&env, "Complete path to become a cybersecurity professional"),
            String::from_str(&env, "Cybersecurity"),
            modules,
            true,
            admin,
        );
    }
}

pub mod test;
