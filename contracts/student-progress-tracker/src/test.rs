#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, symbol_short,
};

// Helper function to create a test environment
fn setup_test_env() -> (Env, ProgressTrackerClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(ProgressTracker, {});
    let client = ProgressTrackerClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    
    (env, client, admin, student)
}

#[test]
fn test_initialize() {
    let (env, client, admin, _student) = setup_test_env();
    
    // Test successful initialization
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.initialize(&admin);
    
    // Verify admin is stored
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_initialize_requires_auth() {
    let (_env, client, admin, _student) = setup_test_env();
    
    // Test that initialization requires auth (this should panic without mock_auths)
    client.initialize(&admin);
}

#[test]
fn test_update_progress_student_auth() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    let percent = 75u32;
    
    // Test student updating their own progress
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), percent).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.update_progress(&student, &course_id, &module_id, &percent);
    
    // Verify progress was stored
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.get(module_id), Some(percent));
}

#[test]
fn test_update_progress_admin_auth() {
    let (env, client, admin, _student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    let percent = 50u32;
    
    // Test admin updating their own progress (admin is also a student)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (admin.clone(), course_id.clone(), module_id.clone(), percent).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.update_progress(&admin, &course_id, &module_id, &percent);
    
    // Verify progress was stored
    let progress_map = client.get_progress(&admin, &course_id);
    assert_eq!(progress_map.get(module_id), Some(percent));
}

#[test]
#[should_panic(expected = "percentage cannot be more than 100")]
fn test_update_progress_invalid_percentage() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    let invalid_percent = 150u32; // > 100
    
    // Test that percentage > 100 panics
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), invalid_percent).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    // Should panic due to invalid percentage
    client.update_progress(&student, &course_id, &module_id, &invalid_percent);
}

#[test]
fn test_update_progress_boundary_values() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    
    // Test boundary values: 0% and 100%
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), 0u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module_id, &0u32);
    
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.get(module_id.clone()), Some(0u32));
    
    // Test 100%
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), 100u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module_id, &100u32);
    
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.get(module_id), Some(100u32));
}

#[test]
fn test_get_progress_empty() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    
    // Test getting progress for student with no progress
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.len(), 0);
}

#[test]
fn test_multiple_modules_same_course() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module1 = symbol_short!("MOD1");
    let module2 = symbol_short!("MOD2");
    let module3 = symbol_short!("MOD3");
    
    // Update progress for multiple modules
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module1.clone(), 25u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module1, &25u32);
    
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module2.clone(), 50u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module2, &50u32);
    
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module3.clone(), 75u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module3, &75u32);
    
    // Verify all modules are stored correctly
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.get(module1), Some(25u32));
    assert_eq!(progress_map.get(module2), Some(50u32));
    assert_eq!(progress_map.get(module3), Some(75u32));
    assert_eq!(progress_map.len(), 3);
}

#[test]
fn test_multiple_courses_same_student() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course1 = symbol_short!("RUST101");
    let course2 = symbol_short!("BLOCK101");
    let module_id = symbol_short!("MOD1");
    
    // Update progress for different courses
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course1.clone(), module_id.clone(), 30u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course1, &module_id, &30u32);
    
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course2.clone(), module_id.clone(), 60u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course2, &module_id, &60u32);
    
    // Verify both courses are stored separately
    let progress1 = client.get_progress(&student, &course1);
    let progress2 = client.get_progress(&student, &course2);
    
    assert_eq!(progress1.get(module_id.clone()), Some(30u32));
    assert_eq!(progress2.get(module_id), Some(60u32));
}

#[test]
fn test_multiple_students_same_course() {
    let (env, client, admin, student1) = setup_test_env();
    let student2 = Address::generate(&env);
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    
    // Update progress for different students
    env.mock_auths(&[MockAuth {
        address: &student1,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student1.clone(), course_id.clone(), module_id.clone(), 40u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student1, &course_id, &module_id, &40u32);
    
    env.mock_auths(&[MockAuth {
        address: &student2,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student2.clone(), course_id.clone(), module_id.clone(), 80u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student2, &course_id, &module_id, &80u32);
    
    // Verify both students have separate progress
    let progress1 = client.get_progress(&student1, &course_id);
    let progress2 = client.get_progress(&student2, &course_id);
    
    assert_eq!(progress1.get(module_id.clone()), Some(40u32));
    assert_eq!(progress2.get(module_id), Some(80u32));
}

#[test]
fn test_update_progress_overwrites_existing() {
    let (env, client, admin, student) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    
    // First update
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), 30u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module_id, &30u32);
    
    // Second update (should overwrite)
    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_progress",
            args: (student.clone(), course_id.clone(), module_id.clone(), 70u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.update_progress(&student, &course_id, &module_id, &70u32);
    
    // Verify the second value overwrote the first
    let progress_map = client.get_progress(&student, &course_id);
    assert_eq!(progress_map.get(module_id), Some(70u32));
    assert_eq!(progress_map.len(), 1); // Still only one module
}

#[test]
#[should_panic(expected = "admin not set")]
fn test_get_admin_not_initialized() {
    let (_env, client, _admin, _student) = setup_test_env();
    
    // Test getting admin before initialization (should panic)
    client.get_admin();
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unauthorized_update_progress() {
    let (env, client, admin, student) = setup_test_env();
    let _unauthorized_user = Address::generate(&env);
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin);
    
    let course_id = symbol_short!("RUST101");
    let module_id = symbol_short!("MOD1");
    let percent = 50u32;
    
    // Test unauthorized user trying to update student's progress (should panic)
    client.update_progress(&student, &course_id, &module_id, &percent);
}
