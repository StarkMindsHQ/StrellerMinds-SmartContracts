use soroban_sdk::{Env, String, Vec, Address};
use crate::{LearningPathTemplates, TemplateData};

/// Predefined learning path templates for common careers
pub struct PredefinedTemplates;

impl PredefinedTemplates {
    /// Web Development Learning Path Template
    pub fn web_development_template(env: &Env) -> TemplateData {
        let mut modules = Vec::<String>::new(env);
        modules.push_back(String::from_str(env, "HTML & CSS Fundamentals"));
        modules.push_back(String::from_str(env, "JavaScript Basics"));
        modules.push_back(String::from_str(env, "Advanced JavaScript"));
        modules.push_back(String::from_str(env, "React.js Fundamentals"));
        modules.push_back(String::from_str(env, "Node.js & Express"));
        modules.push_back(String::from_str(env, "Database Design"));
        modules.push_back(String::from_str(env, "API Development"));
        modules.push_back(String::from_str(env, "Frontend Frameworks"));
        modules.push_back(String::from_str(env, "DevOps Basics"));
        modules.push_back(String::from_str(env, "Full Stack Project"));
        
        let mut prerequisites = Vec::<String>::new(env);
        prerequisites.push_back(String::from_str(env, "Basic computer literacy"));
        prerequisites.push_back(String::from_str(env, "Problem-solving skills"));
        
        let mut skills_gained = Vec::<String>::new(env);
        skills_gained.push_back(String::from_str(env, "Frontend development"));
        skills_gained.push_back(String::from_str(env, "Backend development"));
        skills_gained.push_back(String::from_str(env, "Database management"));
        skills_gained.push_back(String::from_str(env, "API design"));
        skills_gained.push_back(String::from_str(env, "Version control"));
        
        TemplateData {
            id: String::from_str(env, "web_dev_101"),
            title: String::from_str(env, "Complete Web Development"),
            description: String::from_str(env, "Comprehensive web development course covering frontend, backend, and full-stack technologies"),
            category: String::from_str(env, "Web Development"),
            difficulty_level: String::from_str(env, "Intermediate"),
            estimated_hours: 320,
            modules,
            prerequisites,
            skills_gained,
            created_at: 0, // Will be set during creation
            is_active: true,
        }
    }

    /// Data Science Learning Path Template
    pub fn data_science_template(env: &Env) -> TemplateData {
        let mut modules = Vec::<String>::new(env);
        modules.push_back(String::from_str(env, "Python for Data Science"));
        modules.push_back(String::from_str(env, "Statistics & Probability"));
        modules.push_back(String::from_str(env, "Data Visualization"));
        modules.push_back(String::from_str(env, "Machine Learning Basics"));
        modules.push_back(String::from_str(env, "Deep Learning Fundamentals"));
        modules.push_back(String::from_str(env, "Natural Language Processing"));
        modules.push_back(String::from_str(env, "Computer Vision"));
        modules.push_back(String::from_str(env, "Big Data Technologies"));
        modules.push_back(String::from_str(env, "Model Deployment"));
        modules.push_back(String::from_str(env, "Capstone Project"));
        
        let mut prerequisites = Vec::<String>::new(env);
        prerequisites.push_back(String::from_str(env, "Basic programming knowledge"));
        prerequisites.push_back(String::from_str(env, "Mathematics fundamentals"));
        prerequisites.push_back(String::from_str(env, "Statistics basics"));
        
        let mut skills_gained = Vec::<String>::new(env);
        skills_gained.push_back(String::from_str(env, "Data analysis"));
        skills_gained.push_back(String::from_str(env, "Machine learning"));
        skills_gained.push_back(String::from_str(env, "Statistical modeling"));
        skills_gained.push_back(String::from_str(env, "Data visualization"));
        skills_gained.push_back(String::from_str(env, "Big data processing"));
        
        TemplateData {
            id: String::from_str(env, "data_science_101"),
            title: String::from_str(env, "Data Science Comprehensive"),
            description: String::from_str(env, "Complete data science program from fundamentals to advanced machine learning"),
            category: String::from_str(env, "Data Science"),
            difficulty_level: String::from_str(env, "Advanced"),
            estimated_hours: 400,
            modules,
            prerequisites,
            skills_gained,
            created_at: 0,
            is_active: true,
        }
    }

    /// Cloud Architecture Learning Path Template
    pub fn cloud_architecture_template(env: &Env) -> TemplateData {
        let mut modules = Vec::<String>::new(env);
        modules.push_back(String::from_str(env, "Cloud Computing Fundamentals"));
        modules.push_back(String::from_str(env, "AWS/Azure/GCP Basics"));
        modules.push_back(String::from_str(env, "Cloud Security"));
        modules.push_back(String::from_str(env, "Network Architecture"));
        modules.push_back(String::from_str(env, "Storage Solutions"));
        modules.push_back(String::from_str(env, "Database Services"));
        modules.push_back(String::from_str(env, "Serverless Computing"));
        modules.push_back(String::from_str(env, "Container Orchestration"));
        modules.push_back(String::from_str(env, "DevOps & CI/CD"));
        modules.push_back(String::from_str(env, "Cost Optimization"));
        modules.push_back(String::from_str(env, "Enterprise Architecture"));
        
        let mut prerequisites = Vec::<String>::new(env);
        prerequisites.push_back(String::from_str(env, "Networking fundamentals"));
        prerequisites.push_back(String::from_str(env, "System administration"));
        prerequisites.push_back(String::from_str(env, "Security basics"));
        
        let mut skills_gained = Vec::<String>::new(env);
        skills_gained.push_back(String::from_str(env, "Cloud platform management"));
        skills_gained.push_back(String::from_str(env, "Infrastructure as code"));
        skills_gained.push_back(String::from_str(env, "Cloud security"));
        skills_gained.push_back(String::from_str(env, "Cost optimization"));
        skills_gained.push_back(String::from_str(env, "Enterprise architecture"));
        
        TemplateData {
            id: String::from_str(env, "cloud_arch_101"),
            title: String::from_str(env, "Cloud Architecture Mastery"),
            description: String::from_str(env, "Complete cloud architecture program covering major cloud providers and enterprise solutions"),
            category: String::from_str(env, "Cloud Architecture"),
            difficulty_level: String::from_str(env, "Advanced"),
            estimated_hours: 350,
            modules,
            prerequisites,
            skills_gained,
            created_at: 0,
            is_active: true,
        }
    }

    /// Blockchain Development Learning Path Template
    pub fn blockchain_development_template(env: &Env) -> TemplateData {
        let mut modules = Vec::<String>::new(env);
        modules.push_back(String::from_str(env, "Blockchain Fundamentals"));
        modules.push_back(String::from_str(env, "Cryptography Basics"));
        modules.push_back(String::from_str(env, "Smart Contract Development"));
        modules.push_back(String::from_str(env, "Solidity Programming"));
        modules.push_back(String::from_str(env, "Ethereum Development"));
        modules.push_back(String::from_str(env, "DeFi Protocols"));
        modules.push_back(String::from_str(env, "Web3.js & Frontend Integration"));
        modules.push_back(String::from_str(env, "Security & Auditing"));
        modules.push_back(String::from_str(env, "Layer 2 Solutions"));
        modules.push_back(String::from_str(env, "DApp Development"));
        modules.push_back(String::from_str(env, "Token Economics"));
        
        let mut prerequisites = Vec::<String>::new(env);
        prerequisites.push_back(String::from_str(env, "JavaScript knowledge"));
        prerequisites.push_back(String::from_str(env, "Understanding of distributed systems"));
        prerequisites.push_back(String::from_str(env, "Basic cryptography concepts"));
        
        let mut skills_gained = Vec::<String>::new(env);
        skills_gained.push_back(String::from_str(env, "Smart contract development"));
        skills_gained.push_back(String::from_str(env, "Blockchain architecture"));
        skills_gained.push_back(String::from_str(env, "DeFi protocol development"));
        skills_gained.push_back(String::from_str(env, "Web3 integration"));
        skills_gained.push_back(String::from_str(env, "Security auditing"));
        
        TemplateData {
            id: String::from_str(env, "blockchain_dev_101"),
            title: String::from_str(env, "Blockchain Development Complete"),
            description: String::from_str(env, "Comprehensive blockchain development course from fundamentals to advanced DApp development"),
            category: String::from_str(env, "Blockchain Development"),
            difficulty_level: String::from_str(env, "Advanced"),
            estimated_hours: 380,
            modules,
            prerequisites,
            skills_gained,
            created_at: 0,
            is_active: true,
        }
    }

    /// Cybersecurity Learning Path Template
    pub fn cybersecurity_template(env: &Env) -> TemplateData {
        let mut modules = Vec::<String>::new(env);
        modules.push_back(String::from_str(env, "Cybersecurity Fundamentals"));
        modules.push_back(String::from_str(env, "Network Security"));
        modules.push_back(String::from_str(env, "Ethical Hacking"));
        modules.push_back(String::from_str(env, "Security Tools & Techniques"));
        modules.push_back(String::from_str(env, "Web Application Security"));
        modules.push_back(String::from_str(env, "Cloud Security"));
        modules.push_back(String::from_str(env, "Incident Response"));
        modules.push_back(String::from_str(env, "Security Compliance"));
        modules.push_back(String::from_str(env, "Penetration Testing"));
        modules.push_back(String::from_str(env, "Security Operations"));
        modules.push_back(String::from_str(env, "Advanced Threat Detection"));
        
        let mut prerequisites = Vec::<String>::new(env);
        prerequisites.push_back(String::from_str(env, "Networking knowledge"));
        prerequisites.push_back(String::from_str(env, "Operating systems basics"));
        prerequisites.push_back(String::from_str(env, "Scripting fundamentals"));
        
        let mut skills_gained = Vec::<String>::new(env);
        skills_gained.push_back(String::from_str(env, "Security assessment"));
        skills_gained.push_back(String::from_str(env, "Penetration testing"));
        skills_gained.push_back(String::from_str(env, "Incident response"));
        skills_gained.push_back(String::from_str(env, "Security operations"));
        skills_gained.push_back(String::from_str(env, "Compliance management"));
        
        TemplateData {
            id: String::from_str(env, "cybersecurity_101"),
            title: String::from_str(env, "Cybersecurity Professional"),
            description: String::from_str(env, "Complete cybersecurity program covering ethical hacking, security operations, and compliance"),
            category: String::from_str(env, "Cybersecurity"),
            difficulty_level: String::from_str(env, "Advanced"),
            estimated_hours: 420,
            modules,
            prerequisites,
            skills_gained,
            created_at: 0,
            is_active: true,
        }
    }

    /// Initialize all predefined templates
    pub fn initialize_all_templates(env: &Env, admin: Address) -> Result<(), soroban_sdk::Error> {
        let mut templates = Vec::<TemplateData>::new(env);
        templates.push_back(Self::web_development_template(env));
        templates.push_back(Self::data_science_template(env));
        templates.push_back(Self::cloud_architecture_template(env));
        templates.push_back(Self::blockchain_development_template(env));
        templates.push_back(Self::cybersecurity_template(env));

        for template in templates.iter() {
            LearningPathTemplates::create_template(
                env.clone(),
                admin.clone(),
                template.id.clone(),
                template.title.clone(),
                template.description.clone(),
                template.category.clone(),
                template.difficulty_level.clone(),
                template.estimated_hours,
                template.modules.clone(),
                template.prerequisites.clone(),
                template.skills_gained.clone(),
            )?;
        }

        Ok(())
    }
}
