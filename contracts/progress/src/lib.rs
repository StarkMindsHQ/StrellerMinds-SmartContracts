#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Env, Map, Symbol, Vec,
};
use shared::reentrancy_guard::ReentrancyLock;

// Storage keys
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const USER_PROGRESS: Symbol = symbol_short!("UPROG");
const COURSE_KEY: Symbol = symbol_short!("COURSE");

// Use the contracterror macro to define errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    CourseNotFound = 4,
    InvalidProgress = 5,
    ModuleAlreadyCompleted = 6,
    NonIncreasingProgress = 7,
    InvalidProgressRange = 8,
}

#[contract]
pub struct Progress;

#[contractimpl]
impl Progress {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&ADMIN_KEY) {
            return Err(Error::AlreadyInitialized);
        }

        // Require authorization from the admin
        admin.require_auth();

        // Store admin address
        env.storage().instance().set(&ADMIN_KEY, &admin);

        Ok(())
    }

    // Add a new course
    pub fn add_course(env: Env, course_id: Symbol, total_modules: u32) -> Result<(), Error> {
        // Get admin and check authorization
        let admin = Self::get_admin(&env)?;
        admin.require_auth();

        // Create a storage key for this course
        let key = (COURSE_KEY, course_id);

        // Store course info
        env.storage().instance().set(&key, &total_modules);

        Ok(())
    }

    // Get course total modules
    pub fn get_course_modules(env: Env, course_id: Symbol) -> Result<u32, Error> {
        // Create a storage key for this course
        let key = (COURSE_KEY, course_id);

        // Check if course exists
        if !env.storage().instance().has(&key) {
            return Err(Error::CourseNotFound);
        }

        // Get course total modules
        let total_modules = env.storage().instance().get(&key).unwrap();

        Ok(total_modules)
    }

    // Update user progress for a course
    pub fn update_progress(
        env: Env,
        user: Address,
        course_id: Symbol,
        module: u32,
        completed: bool,
    ) -> Result<(), Error> {
        let _guard = ReentrancyLock::new(&env);
        // Require authorization from the user
        user.require_auth();

        // Check if course exists and get total modules
        let total_modules = Self::get_course_modules(env.clone(), course_id.clone())?;

        // Create a storage key for user progress
        let key = (USER_PROGRESS, user.clone());

        // Get or create user progress map
        let mut user_progress: Map<Symbol, Vec<bool>> = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Map::new(&env)
        };

        // Get or create course progress vector
        let mut course_progress = if user_progress.contains_key(course_id.clone()) {
            user_progress.get(course_id.clone()).unwrap()
        } else {
            // Initialize with false values for each module (1-indexed, so we need total_modules + 1)
            let mut progress = Vec::new(&env);
            for _ in 0..=total_modules {
                progress.push_back(false);
            }
            progress
        };

        // Validate module number
        if module == 0 || module > total_modules {
            // Log the invalid progress attempt
            env.events().publish((Symbol::short("error"), "invalid_module"), 
                &format!("{},{},{},{}", user.clone(), course_id.clone(), module, completed));
            return Err(Error::InvalidProgress);
        }

        // VALIDATION 1: Check if the module is already completed
        let current_status = course_progress.get(module as u32).unwrap_or(false);
        if current_status && completed {
            // Log the attempt to modify a completed module
            env.events().publish((Symbol::short("error"), "already_completed"), 
                &format!("{},{},{}", user.clone(), course_id.clone(), module));
            return Err(Error::ModuleAlreadyCompleted);
        }

        // VALIDATION 2: Ensure progress only increases (can't mark a completed module as incomplete)
        if current_status && !completed {
            // Log the non-increasing progress attempt
            env.events().publish((Symbol::short("error"), "non_increasing"), 
                &format!("{},{},{},{}->{}", user.clone(), course_id.clone(), module, current_status, completed));
            return Err(Error::NonIncreasingProgress);
        }
        
        // Update the module progress (modules are 1-indexed in the API but 0-indexed in storage)
        course_progress.set(module as u32, completed);

        // Update the user progress map
        user_progress.set(course_id, course_progress);

        // Store updated progress
        env.storage().instance().set(&key, &user_progress);

        // Log successful progress update
        env.events().publish((Symbol::short("info"), "progress_update"), 
            &format!("{},{},{},{}", user.clone(), course_id.clone(), module, completed));

        Ok(())
    }

    // Get user progress for a course
    pub fn get_progress(env: Env, user: Address, course_id: Symbol) -> Result<Vec<bool>, Error> {
        // Check if course exists
        Self::get_course_modules(env.clone(), course_id.clone())?;

        // Create a storage key for user progress
        let key = (USER_PROGRESS, user.clone());

        // Check if user has any progress
        if !env.storage().instance().has(&key) {
            return Err(Error::NotInitialized);
        }

        // Get user progress map
        let user_progress: Map<Symbol, Vec<bool>> = env.storage().instance().get(&key).unwrap();

        // Check if user has progress for this course
        if !user_progress.contains_key(course_id.clone()) {
            return Err(Error::NotInitialized);
        }

        // Get course progress
        let course_progress = user_progress.get(course_id).unwrap();

        Ok(course_progress)
    }

    // Get completion percentage for a course
    pub fn get_completion_percentage(
        env: Env,
        user: Address,
        course_id: Symbol,
    ) -> Result<u32, Error> {
        // Get course progress
        let progress = Self::get_progress(env.clone(), user, course_id)?;

        // Count completed modules (skip index 0 since modules are 1-indexed)
        let mut completed = 0;
        for i in 1..progress.len() {
            if progress.get(i as u32).unwrap_or(false) {
                completed += 1;
            }
        }

        // Calculate percentage (modules are 1-indexed, so total is len-1)
        let total = progress.len() - 1;
        if total == 0 {
            return Ok(0);
        }

        let percentage = (completed * 100) / total;

        Ok(percentage as u32)
    }

    // Helper function to get admin
    fn get_admin(env: &Env) -> Result<Address, Error> {
        if !env.storage().instance().has(&ADMIN_KEY) {
            return Err(Error::NotInitialized);
        }

        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        Ok(admin)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Symbol;
    
    // Helper function to set up a test environment with a course and initial progress
    fn setup_test_env() -> (Env, ProgressClient, Address, Address, Symbol) {
        let env = Env::default();
        let contract_id = env.register(Progress, {});
        let client = ProgressClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Initialize the contract
        env.mock_all_auths();
        client.initialize(&admin);
        
        // Add a course
        let course_id = Symbol::short("BLOCKCHAIN101");
        client.add_course(&course_id, &5);
        
        (env, client, admin, user, course_id)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register(Progress, {});
        let client = ProgressClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        // Test successful initialization
        env.mock_all_auths();
        let result = client.try_initialize(&admin).unwrap();
        assert!(result.is_ok());

        // Test re-initialization (should fail)
        let result = client.try_initialize(&admin);
        assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
    }

    #[test]
    fn test_course_management() {
        let env = Env::default();
        let contract_id = env.register(Progress, {});
        let client = ProgressClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        // Initialize
        env.mock_all_auths();
        let initialize_res = client.try_initialize(&admin).unwrap();
        assert!(initialize_res.is_ok());

        // Add a course
        let course_id = Symbol::short("RUST101");
        let total_modules = 10;
        let add_course_res = client.try_add_course(&course_id, &total_modules).unwrap();
        assert!(add_course_res.is_ok());

        // Get course modules
        let modules = client.try_get_course_modules(&course_id).unwrap();
        assert_eq!(modules.unwrap(), total_modules);

        // Try to get non-existent course
        let invalid_course = Symbol::short("INVALID");
        let result = client.try_get_course_modules(&invalid_course);
        assert_eq!(result, Err(Ok(Error::CourseNotFound)));
    }

    #[test]
    fn test_user_progress() {
        let env = Env::default();
        let contract_id = env.register(Progress, {});
        let client = ProgressClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Initialize
        env.mock_all_auths();
        let initialize_res = client.try_initialize(&admin).unwrap();
        assert!(initialize_res.is_ok());

        // Add a course
        let course_id = Symbol::short("RUST101");
        let total_modules = 10;
        let add_course_res = client.try_add_course(&course_id, &total_modules).unwrap();
        assert!(add_course_res.is_ok());

        // Update progress for module 1
        let update_progress_res = client
            .try_update_progress(&user, &course_id, &1, &true)
            .unwrap();
        assert!(update_progress_res.is_ok());

        // Get progress
        let progress = client.try_get_progress(&user, &course_id).unwrap().unwrap();
        assert_eq!(progress.len(), total_modules + 1); // +1 because modules are 1-indexed
        assert_eq!(progress.get(1), Some(true));

        // Update progress for module 2
        let update_progress_res = client
            .try_update_progress(&user, &course_id, &2, &true)
            .unwrap();
        assert!(update_progress_res.is_ok());

        // Get completion percentage
        let percentage = client
            .try_get_completion_percentage(&user, &course_id)
            .unwrap();
        assert_eq!(percentage, Ok(20)); // 2 out of 10 modules = 20%

        // Try to update invalid module number
        let result = client.try_update_progress(&user, &course_id, &0, &true);
        assert_eq!(result, Err(Ok(Error::InvalidProgress)));

        let result = client.try_update_progress(&user, &course_id, &11, &true);
        assert_eq!(result, Err(Ok(Error::InvalidProgress)));
        
        // TEST 1: Try to update an already completed module
        let result = client.try_update_progress(&user, &course_id, &1, &true);
        assert_eq!(result, Err(Ok(Error::ModuleAlreadyCompleted)));
        
        // TEST 2: Try to decrease progress (mark a completed module as incomplete)
        let result = client.try_update_progress(&user, &course_id, &1, &false);
        assert_eq!(result, Err(Ok(Error::NonIncreasingProgress)));
        
        // TEST 3: Successfully complete another module
        let update_progress_res = client
            .try_update_progress(&user, &course_id, &3, &true)
            .unwrap();
        assert!(update_progress_res.is_ok());
        
        // Verify progress was updated correctly
        let progress = client.try_get_progress(&user, &course_id).unwrap().unwrap();
        assert_eq!(progress.get(1), Some(true));
        assert_eq!(progress.get(2), Some(true));
        assert_eq!(progress.get(3), Some(true));
        
        // Verify completion percentage is now 30%
        let percentage = client
            .try_get_completion_percentage(&user, &course_id)
            .unwrap();
        assert_eq!(percentage, Ok(30)); // 3 out of 10 modules = 30%
    }
    
    #[test]
    fn test_progress_validation_rules() {
        // Set up test environment
        let (env, client, _admin, user, course_id) = setup_test_env();
        env.mock_all_auths();
        
        // Test 1: Valid progress update (module 1 to completed)
        let res = client.try_update_progress(&user, &course_id, &1, &true).unwrap();
        assert!(res.is_ok());
        
        // Test 2: Cannot complete an already completed module
        let res = client.try_update_progress(&user, &course_id, &1, &true);
        assert_eq!(res, Err(Ok(Error::ModuleAlreadyCompleted)));
        
        // Test 3: Cannot mark a completed module as incomplete (non-increasing progress)
        let res = client.try_update_progress(&user, &course_id, &1, &false);
        assert_eq!(res, Err(Ok(Error::NonIncreasingProgress)));
        
        // Test 4: Invalid module number (too low)
        let res = client.try_update_progress(&user, &course_id, &0, &true);
        assert_eq!(res, Err(Ok(Error::InvalidProgress)));
        
        // Test 5: Invalid module number (too high)
        let res = client.try_update_progress(&user, &course_id, &6, &true);
        assert_eq!(res, Err(Ok(Error::InvalidProgress)));
        
        // Test 6: Complete remaining modules one by one
        for module in 2..=5 {
            let res = client.try_update_progress(&user, &course_id, &module, &true).unwrap();
            assert!(res.is_ok());
        }
        
        // Test 7: Verify final completion percentage is 100%
        let percentage = client.try_get_completion_percentage(&user, &course_id).unwrap();
        assert_eq!(percentage, Ok(100)); // All 5 modules completed
    }
}
