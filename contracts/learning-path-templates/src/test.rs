#[cfg(test)]
use soroban_sdk::{symbol_short, Address, Env};

#[cfg(test)]
use crate::LearningPathTemplates;

#[test]
fn test_contract_compiles() {
    // Basic test to ensure the contract compiles and can be instantiated
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    
    // Just verify the contract was registered successfully
    assert_ne!(contract_id, Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")));
}

#[test]
fn test_initialization_basic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    
    // Create a test admin address
    let admin = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
    
    // Test that we can call initialize (this will panic if there are issues)
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize"),
        admin,
    );
    
    // Verify we can call get_admin
    let retrieved_admin = env.invoke_contract(
        &contract_id,
        &symbol_short!("get_admin"),
        soroban_sdk::Vec::new(&env),
    );
    
    // Convert the result back to Address
    let admin_address: Address = retrieved_admin.try_into_val(&env).unwrap();
    assert_eq!(admin_address, admin);
}

#[test]
fn test_prebuilt_templates_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    
    // Initialize contract
    let admin = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize"),
        admin,
    );
    
    // Initialize pre-built templates
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize_prebuilt_templates"),
        admin,
    );
    
    // Test getting all templates
    let templates = env.invoke_contract(
        &contract_id,
        &symbol_short!("get_all_templates"),
        soroban_sdk::Vec::new(&env),
    );
    
    // Convert result to Vec and check length
    let template_vec: soroban_sdk::Vec<crate::LearningPath> = templates.try_into_val(&env).unwrap();
    assert_eq!(template_vec.len(), 5); // Should have 5 pre-built templates
}

#[test]
fn test_get_template() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    
    // Initialize contract
    let admin = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize"),
        admin,
    );
    
    // Initialize pre-built templates
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize_prebuilt_templates"),
        admin,
    );
    
    // Test getting a specific template
    let web_dev_template = env.invoke_contract(
        &contract_id,
        &symbol_short!("get_template"),
        symbol_short!("web_dev"),
    );
    
    // Should not be void (template exists)
    assert!(!web_dev_template.is_void());
    
    // Convert to LearningPath and verify
    let template: crate::LearningPath = web_dev_template.try_into_val(&env).unwrap();
    assert_eq!(template.id, symbol_short!("web_dev"));
    assert_eq!(template.career_track, String::from_str(&env, "Web Development"));
}

#[test]
fn test_nonexistent_template() {
    let env = Env::default();
    let contract_id = env.register_contract(None, LearningPathTemplates);
    
    // Initialize contract
    let admin = Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"));
    env.invoke_contract(
        &contract_id,
        &symbol_short!("initialize"),
        admin,
    );
    
    // Test getting a non-existent template
    let nonexistent_template = env.invoke_contract(
        &contract_id,
        &symbol_short!("get_template"),
        symbol_short!("nonexistent"),
    );
    
    // Should be void (template doesn't exist)
    assert!(nonexistent_template.is_void());
}
