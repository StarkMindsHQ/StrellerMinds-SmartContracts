use soroban_sdk::{
    contract, contractimpl,
    Address, Env, Error, Symbol, String, Vec, Map,
};

pub mod templates;

#[contract]
pub struct LearningPathTemplates;

#[contractimpl]
impl LearningPathTemplates {
    /// Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&Symbol::short(&env, "admin")) {
            return Err(Error::from_contract_error(1)); // Already initialized
        }
        
        // Set admin
        env.storage().instance().set(&Symbol::short(&env, "admin"), &admin);
        
        // Initialize templates storage
        env.storage().instance().set(&Symbol::short(&env, "templates"), &Vec::<String>::new(&env));
        
        Ok(())
    }

    /// Create a new learning path template
    pub fn create_template(
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
    ) -> Result<(), Error> {
        // Check admin authorization
        Self::require_admin(&env, &admin)?;
        
        // Create template data
        let template_data = TemplateData {
            id: template_id.clone(),
            title,
            description,
            category,
            difficulty_level,
            estimated_hours,
            modules,
            prerequisites,
            skills_gained,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };
        
        // Store template
        let templates_key = Symbol::short(&env, "templates");
        let mut templates: Vec<String> = env.storage().instance().get(&templates_key).unwrap_or(Vec::new(&env));
        
        // Check if template already exists
        if templates.contains(&template_id) {
            return Err(Error::from_contract_error(2)); // Template already exists
        }
        
        templates.push_back(template_id.clone());
        env.storage().instance().set(&templates_key, &templates);
        
        // Store template data
        let template_key = Symbol::short(&env, "template");
        env.storage().instance().set(&(template_key, template_id), &template_data);
        
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(env: Env, template_id: String) -> Result<TemplateData, Error> {
        let template_key = Symbol::short(&env, "template");
        let template: Option<TemplateData> = env.storage().instance().get(&(template_key, template_id));
        
        template.ok_or(Error::from_contract_error(3)) // Template not found
    }

    /// List all available templates
    pub fn list_templates(env: Env) -> Result<Vec<TemplateData>, Error> {
        let templates_key = Symbol::short(&env, "templates");
        let template_ids: Vec<String> = env.storage().instance().get(&templates_key).unwrap_or(Vec::new(&env));
        
        let mut templates = Vec::<TemplateData>::new(&env);
        
        for template_id in template_ids.iter() {
            if let Ok(template) = Self::get_template(env.clone(), template_id.clone()) {
                if template.is_active {
                    templates.push_back(template);
                }
            }
        }
        
        Ok(templates)
    }

    /// Get templates by category
    pub fn get_templates_by_category(env: Env, category: String) -> Result<Vec<TemplateData>, Error> {
        let all_templates = Self::list_templates(env)?;
        let mut filtered_templates = Vec::<TemplateData>::new(&env);
        
        for template in all_templates.iter() {
            if template.category == category {
                filtered_templates.push_back(template.clone());
            }
        }
        
        Ok(filtered_templates)
    }

    /// Customize an existing template
    pub fn customize_template(
        env: Env,
        user: Address,
        template_id: String,
        custom_title: Option<String>,
        custom_description: Option<String>,
        custom_modules: Option<Vec<String>>,
        custom_skills: Option<Vec<String>>,
    ) -> Result<CustomizedTemplate, Error> {
        // Get original template
        let original_template = Self::get_template(env.clone(), template_id)?;
        
        // Create customized version
        let customized = CustomizedTemplate {
            original_template_id: original_template.id.clone(),
            customized_by: user,
            custom_title: custom_title.unwrap_or(original_template.title.clone()),
            custom_description: custom_description.unwrap_or(original_template.description.clone()),
            custom_modules: custom_modules.unwrap_or(original_template.modules.clone()),
            custom_skills: custom_skills.unwrap_or(original_template.skills_gained.clone()),
            customized_at: env.ledger().timestamp(),
        };
        
        // Store customized template
        let custom_key = Symbol::short(&env, "custom_template");
        let custom_id = format!("{}_{}", template_id, user);
        env.storage().persistent().set(&(custom_key, String::from_str(&env, &custom_id)), &customized);
        
        Ok(customized)
    }

    /// Get user's customized templates
    pub fn get_user_customized_templates(env: Env, user: Address) -> Result<Vec<CustomizedTemplate>, Error> {
        let custom_key = Symbol::short(&env, "custom_template");
        let user_custom_key = Symbol::short(&env, "user_custom");
        
        let custom_ids: Vec<String> = env.storage().persistent().get(&(user_custom_key, user)).unwrap_or(Vec::new(&env));
        let mut customized_templates = Vec::<CustomizedTemplate>::new(&env);
        
        for custom_id in custom_ids.iter() {
            if let Some(template) = env.storage().persistent().get::<_, CustomizedTemplate>(&env, &(custom_key, custom_id)) {
                customized_templates.push_back(template);
            }
        }
        
        Ok(customized_templates)
    }

    /// Apply template to create a course
    pub fn apply_template(
        env: Env,
        admin: Address,
        template_id: String,
        course_id: Symbol,
        instructor: Address,
    ) -> Result<(), Error> {
        // Check admin authorization
        Self::require_admin(&env, &admin)?;
        
        // Get template
        let template = Self::get_template(env.clone(), template_id)?;
        
        // Create course from template
        let course_data = CourseFromTemplate {
            course_id,
            template_id: template.id.clone(),
            title: template.title.clone(),
            description: template.description.clone(),
            instructor,
            modules: template.modules.clone(),
            total_modules: template.modules.len() as u32,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };
        
        // Store course
        let course_key = Symbol::short(&env, "course");
        env.storage().instance().set(&(course_key, course_id), &course_data);
        
        // Update progress contract with new course
        // This would interact with the progress contract
        Self::register_course_with_progress(env.clone(), course_id, course_data.total_modules)?;
        
        Ok(())
    }

    /// Get template statistics
    pub fn get_template_stats(env: Env) -> Result<TemplateStats, Error> {
        let templates = Self::list_templates(env.clone())?;
        
        let mut category_counts = Map::<String, u32>::new(&env);
        let mut difficulty_counts = Map::<String, u32>::new(&env);
        
        for template in templates.iter() {
            // Count by category
            let category_count = category_counts.get(template.category.clone()).unwrap_or(0);
            category_counts.set(template.category.clone(), category_count + 1);
            
            // Count by difficulty
            let difficulty_count = difficulty_counts.get(template.difficulty_level.clone()).unwrap_or(0);
            difficulty_counts.set(template.difficulty_level.clone(), difficulty_count + 1);
        }
        
        Ok(TemplateStats {
            total_templates: templates.len() as u32,
            category_distribution: category_counts,
            difficulty_distribution: difficulty_counts,
        })
    }

    /// Helper function to check admin authorization
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin_key = Symbol::short(env, "admin");
        let admin: Address = env.storage().instance().get(&admin_key)
            .ok_or(Error::from_contract_error(4))?; // Not initialized
        
        if &admin != caller {
            return Err(Error::from_contract_error(5)); // Unauthorized
        }
        
        Ok(())
    }

    /// Helper function to register course with progress contract
    fn register_course_with_progress(env: Env, course_id: Symbol, total_modules: u32) -> Result<(), Error> {
        // This would interact with the progress contract
        // For now, we'll store the course info locally
        let progress_courses_key = Symbol::short(&env, "progress_courses");
        env.storage().instance().set(&(progress_courses_key, course_id), &total_modules);
        
        Ok(())
    }
}

// Data structures
#[derive(Clone)]
pub struct TemplateData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub difficulty_level: String,
    pub estimated_hours: u32,
    pub modules: Vec<String>,
    pub prerequisites: Vec<String>,
    pub skills_gained: Vec<String>,
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct CustomizedTemplate {
    pub original_template_id: String,
    pub customized_by: Address,
    pub custom_title: String,
    pub custom_description: String,
    pub custom_modules: Vec<String>,
    pub custom_skills: Vec<String>,
    pub customized_at: u64,
}

#[derive(Clone)]
pub struct CourseFromTemplate {
    pub course_id: Symbol,
    pub template_id: String,
    pub title: String,
    pub description: String,
    pub instructor: Address,
    pub modules: Vec<String>,
    pub total_modules: u32,
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct TemplateStats {
    pub total_templates: u32,
    pub category_distribution: Map<String, u32>,
    pub difficulty_distribution: Map<String, u32>,
}

// Error codes
// 1: Already initialized
// 2: Template already exists
// 3: Template not found
// 4: Not initialized
// 5: Unauthorized
