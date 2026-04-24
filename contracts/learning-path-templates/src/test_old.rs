#[cfg(test)]
use soroban_sdk::{symbol_short, Address, Env, Map, Symbol, Vec, String};
#[cfg(test)]
use soroban_sdk::testutils::{Address as TestAddress};
#[cfg(test)]
use crate::{
    LearningPathTemplates, LearningModule, DataKey, CustomizedPath, LearningPath
};

#[cfg(test)]
pub struct LearningPathTemplatesClient<'a> {
    env: &'a Env,
    contract_id: &'a soroban_sdk::Address,
}

#[cfg(test)]
impl<'a> LearningPathTemplatesClient<'a> {
    pub fn new(env: &'a Env, contract_id: &'a soroban_sdk::Address) -> Self {
        Self { env, contract_id }
    }

    fn invoke<T: soroban_sdk::IntoVal<soroban_sdk::Env, soroban_sdk::Val>>(
        &self,
        fn_name: soroban_sdk::Symbol,
        args: T,
    ) -> soroban_sdk::Val {
        self.env.invoke_contract(self.contract_id, &fn_name, args)
    }

    pub fn initialize(&self, admin: &Address) {
        self.invoke(symbol_short!("initialize"), admin);
    }

    pub fn get_admin(&self) -> Address {
        self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_admin"),
            soroban_sdk::Vec::new(self.env),
        ).try_into_val(self.env).unwrap()
    }

    pub fn create_template(
        &self,
        template_id: &Symbol,
        title: &String,
        description: &String,
        career_track: &String,
        modules: &Vec<LearningModule>,
        is_customizable: &bool,
        creator: &Address,
    ) {
        self.invoke(
            symbol_short!("create_template"),
            (template_id, title, description, career_track, modules, is_customizable, creator),
        );
    }

    pub fn get_template(&self, template_id: &Symbol) -> Option<LearningPath> {
        let result = self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_template"),
            template_id,
        );
        if result.is_void() {
            None
        } else {
            Some(result.try_into_val(self.env).unwrap())
        }
    }

    pub fn get_all_templates(&self) -> Vec<LearningPath> {
        self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_all_templates"),
            soroban_sdk::Vec::new(self.env),
        ).try_into_val(self.env).unwrap()
    }

    pub fn customize_template(
        &self,
        template_id: &Symbol,
        student: &Address,
        modifications: &Map<Symbol, LearningModule>,
    ) -> Symbol {
        self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("customize_template"),
            (template_id, student, modifications),
        ).try_into_val(self.env).unwrap()
    }

    pub fn get_customized_path(&self, customized_path_id: &Symbol) -> Option<CustomizedPath> {
        let result = self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_customized_path"),
            customized_path_id,
        );
        if result.is_void() {
            None
        } else {
            Some(result.try_into_val(self.env).unwrap())
        }
    }

    pub fn get_student_paths(&self, student: &Address) -> Vec<CustomizedPath> {
        self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_student_paths"),
            student,
        ).try_into_val(self.env).unwrap()
    }

    pub fn update_progress(
        &self,
        customized_path_id: &Symbol,
        module_id: &Symbol,
        progress_percent: &u32,
        student: &Address,
    ) {
        self.invoke(
            symbol_short!("update_progress"),
            (customized_path_id, module_id, progress_percent, student),
        );
    }

    pub fn get_templates_by_career(&self, career_track: &String) -> Vec<LearningPath> {
        self.env.invoke_contract(
            self.contract_id,
            &symbol_short!("get_templates_by_career"),
            career_track,
        ).try_into_val(self.env).unwrap()
    }

    pub fn delete_template(&self, template_id: &Symbol, admin: &Address) {
        self.invoke(symbol_short!("delete_template"), (template_id, admin));
    }

    pub fn initialize_prebuilt_templates(&self, admin: &Address) {
        self.invoke(symbol_short!("initialize_prebuilt_templates"), admin);
    }
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    
    // Test successful initialization
    client.initialize(&admin);
    
    // Verify admin is set
    assert_eq!(client.get_admin(), admin);
    
    // Test double initialization fails
    let result = env.try_invoke_contract::<_, _, ()>(
        &contract_id,
        &symbol_short!("initialize"),
        Vec::from_array(&env, [&admin].into())
    );
    assert!(result.is_err());
}

#[test]
fn test_template_creation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    client.initialize(&admin);
    
    // Create test modules
    let modules = Vec::from_array(&env, [
        LearningModule {
            id: symbol_short!("module1"),
            title: String::from_str(&env, "Test Module 1"),
            description: String::from_str(&env, "Description for module 1"),
            duration_hours: 20,
            difficulty_level: 1,
            prerequisites: Vec::new(&env),
            skills_gained: Vec::from_array(&env, [String::from_str(&env, "Test Skill")]),
            resources: Vec::from_array(&env, [String::from_str(&env, "Test Resource")]),
        }
    ]);
    
    // Create template
    client.create_template(
        &symbol_short!("test_template"),
        &String::from_str(&env, "Test Template"),
        &String::from_str(&env, "Test Description"),
        &String::from_str(&env, "Test Career"),
        &modules,
        &true,
        &admin,
    );
    
    // Verify template exists
    let template = client.get_template(&symbol_short!("test_template")).unwrap();
    assert_eq!(template.title, String::from_str(&env, "Test Template"));
    assert_eq!(template.career_track, String::from_str(&env, "Test Career"));
    assert_eq!(template.total_duration_hours, 20);
    assert_eq!(template.difficulty_level, 1);
    assert!(template.is_customizable);
}

#[test]
fn test_template_retrieval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    client.initialize(&admin);
    
    // Initialize pre-built templates
    client.initialize_prebuilt_templates(&admin);
    
    // Test getting all templates
    let all_templates = client.get_all_templates();
    assert_eq!(all_templates.len(), 5); // Should have 5 pre-built templates
    
    // Test getting templates by career track
    let web_dev_templates = client.get_templates_by_career(&String::from_str(&env, "Web Development"));
    assert_eq!(web_dev_templates.len(), 1);
    assert_eq!(web_dev_templates.get(0).unwrap().title, String::from_str(&env, "Web Development Career Path"));
    
    // Test getting specific template
    let web_dev_template = client.get_template(&symbol_short!("web_dev")).unwrap();
    assert_eq!(web_dev_template.id, symbol_short!("web_dev"));
    assert_eq!(web_dev_template.modules.len(), 5); // Should have 5 modules
}

#[test]
fn test_template_customization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    let student = TestAddress::generate(&env);
    
    client.initialize(&admin);
    client.initialize_prebuilt_templates(&admin);
    
    // Customize a template
    let modifications = Map::new(&env);
    let customized_id = client.customize_template(
        &symbol_short!("web_dev"),
        &student,
        &modifications,
    );
    
    // Verify customized path exists
    let customized_path = client.get_customized_path(&customized_id).unwrap();
    assert_eq!(customized_path.template_id, symbol_short!("web_dev"));
    assert_eq!(customized_path.student_address, student);
    
    // Verify it's in student's paths
    let student_paths = client.get_student_paths(&student);
    assert_eq!(student_paths.len(), 1);
    assert_eq!(student_paths.get(0).unwrap().customized_path_id, customized_id);
}

#[test]
fn test_progress_tracking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    let student = TestAddress::generate(&env);
    
    client.initialize(&admin);
    client.initialize_prebuilt_templates(&admin);
    
    // Customize a template
    let modifications = Map::new(&env);
    let customized_id = client.customize_template(
        &symbol_short!("web_dev"),
        &student,
        &modifications,
    );
    
    // Update progress
    client.update_progress(
        &customized_id,
        &symbol_short!("html_css"),
        &50u32,
        &student,
    );
    
    // Verify progress was updated
    let updated_path = client.get_customized_path(&customized_id).unwrap();
    assert_eq!(
        updated_path.completion_progress.get(symbol_short!("html_css")),
        Some(50u32)
    );
    
    // Test invalid progress (> 100)
    let result = env.try_invoke_contract::<_, _, ()>(
        &contract_id,
        &symbol_short!("update_progress"),
        (
            customized_id,
            symbol_short!("html_css"),
            101u32,
            student
        ).into()
    );
    assert!(result.is_err());
}

#[test]
fn test_template_deletion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    let non_admin = Address::generate(&env);
    
    client.initialize(&admin);
    
    // Create a test template
    let modules = Vec::new(&env);
    client.create_template(
        &symbol_short!("temp_template"),
        &String::from_str(&env, "Temp"),
        &String::from_str(&env, "Temp"),
        &String::from_str(&env, "Temp"),
        &modules,
        &true,
        &admin,
    );
    
    // Verify template exists
    assert!(client.get_template(&symbol_short!("temp_template")).is_some());
    
    // Test deletion by admin
    client.delete_template(&symbol_short!("temp_template"), &admin);
    
    // Verify template is deleted
    assert!(client.get_template(&symbol_short!("temp_template")).is_none());
    
    // Test deletion by non-admin fails
    client.create_template(
        &symbol_short!("temp_template2"),
        &String::from_str(&env, "Temp2"),
        &String::from_str(&env, "Temp2"),
        &String::from_str(&env, "Temp2"),
        &modules,
        &true,
        &admin,
    );
    
    let result = env.try_invoke_contract::<_, _, ()>(
        &contract_id,
        &symbol_short!("delete_template"),
        (symbol_short!("temp_template2"), non_admin).into()
    );
    assert!(result.is_err());
}

#[test]
fn test_prebuilt_templates_structure() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    client.initialize(&admin);
    client.initialize_prebuilt_templates(&admin);
    
    // Test Web Development template
    let web_dev = client.get_template(&symbol_short!("web_dev")).unwrap();
    assert_eq!(web_dev.career_track, String::from_str(&env, "Web Development"));
    assert_eq!(web_dev.modules.len(), 5);
    assert!(web_dev.is_customizable);
    
    // Verify module structure
    let first_module = web_dev.modules.get(0).unwrap();
    assert_eq!(first_module.id, symbol_short!("html_css"));
    assert_eq!(first_module.difficulty_level, 1);
    assert_eq!(first_module.duration_hours, 40);
    
    // Test Data Science template
    let data_science = client.get_template(&symbol_short!("data_science")).unwrap();
    assert_eq!(data_science.career_track, String::from_str(&env, "Data Science"));
    assert_eq!(data_science.modules.len(), 5);
    
    // Test Cloud Architecture template
    let cloud_arch = client.get_template(&symbol_short!("cloud_arch")).unwrap();
    assert_eq!(cloud_arch.career_track, String::from_str(&env, "Cloud Architecture"));
    assert_eq!(cloud_arch.modules.len(), 5);
    
    // Test Blockchain Development template
    let blockchain = client.get_template(&symbol_short!("blockchain_dev")).unwrap();
    assert_eq!(blockchain.career_track, String::from_str(&env, "Blockchain Development"));
    assert_eq!(blockchain.modules.len(), 5);
    
    // Test Cybersecurity template
    let cybersecurity = client.get_template(&symbol_short!("cybersecurity")).unwrap();
    assert_eq!(cybersecurity.career_track, String::from_str(&env, "Cybersecurity"));
    assert_eq!(cybersecurity.modules.len(), 5);
}

#[test]
fn test_authorization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    let student = TestAddress::generate(&env);
    let other_student = TestAddress::generate(&env);
    
    client.initialize(&admin);
    client.initialize_prebuilt_templates(&admin);
    
    // Customize template for student
    let modifications = Map::new(&env);
    let customized_id = client.customize_template(
        &symbol_short!("web_dev"),
        &student,
        &modifications,
    );
    
    // Test that other student cannot update progress
    let result = env.try_invoke_contract::<_, _, ()>(
        &contract_id,
        &symbol_short!("update_progress"),
        (
            customized_id,
            symbol_short!("html_css"),
            50u32,
            other_student
        ).into()
    );
    assert!(result.is_err());
    
    // Test that original student can update progress
    client.update_progress(
        &customized_id,
        &symbol_short!("html_css"),
        &50u32,
        &student,
    );
}

#[test]
fn test_template_customization_restrictions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    let student = TestAddress::generate(&env);
    
    client.initialize(&admin);
    
    // Create a non-customizable template
    let modules = Vec::new(&env);
    client.create_template(
        &symbol_short!("fixed_template"),
        &String::from_str(&env, "Fixed Template"),
        &String::from_str(&env, "Cannot be customized"),
        &String::from_str(&env, "Fixed Career"),
        &modules,
        &false, // Not customizable
        &admin,
    );
    
    // Test that customization fails for non-customizable template
    let modifications = Map::new(&env);
    let result = env.try_invoke_contract::<_, _, Symbol>(
        &contract_id,
        &symbol_short!("customize_template"),
        (
            symbol_short!("fixed_template"),
            student,
            modifications
        ).into()
    );
    assert!(result.is_err());
}

#[test]
fn test_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    let client = LearningPathTemplatesClient::new(&env, &contract_id);
    
    let admin = TestAddress::generate(&env);
    client.initialize(&admin);
    
    // Test getting non-existent template
    assert!(client.get_template(&symbol_short!("nonexistent")).is_none());
    
    // Test getting non-existent customized path
    assert!(client.get_customized_path(&symbol_short!("nonexistent")).is_none());
    
    // Test getting paths for non-existent student
    let non_student = TestAddress::generate(&env);
    let student_paths = client.get_student_paths(&non_student);
    assert_eq!(student_paths.len(), 0);
    
    // Test getting templates by non-existent career track
    let empty_templates = client.get_templates_by_career(&String::from_str(&env, "Non-existent Career"));
    assert_eq!(empty_templates.len(), 0);
}
