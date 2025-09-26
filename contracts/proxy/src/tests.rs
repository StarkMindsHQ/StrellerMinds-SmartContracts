#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Env, Address, IntoVal,
};

// Helper function to create a test environment
fn setup_test_env() -> (Env, ProxyClient<'static>, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(Proxy, {});
    let client = ProxyClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let impl1 = Address::generate(&env);
    let impl2 = Address::generate(&env);
    
    (env, client, admin, impl1, impl2)
}

#[test]
fn test_initialize() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Test successful initialization
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.initialize(&admin, &impl1);
    
    // Verify admin and implementation are stored
    let stored_admin = client.get_admin();
    let stored_impl = client.get_implementation();
    assert_eq!(stored_admin, admin);
    assert_eq!(stored_impl, impl1);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_initialize_requires_auth() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Test that initialization requires auth (this should panic without mock_auths)
    client.initialize(&admin, &impl1);
}

#[test]
fn test_upgrade() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Test upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl2.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.upgrade(&impl2);
    
    // Verify implementation was updated
    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl2);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_upgrade_requires_auth() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Test upgrade without auth (should panic)
    client.upgrade(&impl2);
}

#[test]
fn test_rollback() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Upgrade to impl2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl2.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&impl2);
    
    // Verify we're on impl2
    assert_eq!(client.get_implementation(), impl2);
    
    // Test rollback
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.rollback();
    
    // Verify we're back to impl1
    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl1);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_rollback_requires_auth() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Upgrade to impl2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl2.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&impl2);
    
    // Test rollback without auth (should panic)
    client.rollback();
}

#[test]
#[should_panic(expected = "No previous implementation")]
fn test_rollback_no_previous_implementation() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Try to rollback without any upgrades (should panic)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.rollback();
}

#[test]
fn test_multiple_upgrades_and_rollbacks() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let impl3 = Address::generate(&env);
    let impl4 = Address::generate(&env);
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Upgrade to impl2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl2.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&impl2);
    assert_eq!(client.get_implementation(), impl2);
    
    // Upgrade to impl3
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl3.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&impl3);
    assert_eq!(client.get_implementation(), impl3);
    
    // Upgrade to impl4
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl4.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.upgrade(&impl4);
    assert_eq!(client.get_implementation(), impl4);
    
    // Rollback to impl3
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.rollback();
    assert_eq!(client.get_implementation(), impl3);
    
    // Rollback to impl2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.rollback();
    assert_eq!(client.get_implementation(), impl2);
    
    // Rollback to impl1
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.rollback();
    assert_eq!(client.get_implementation(), impl1);
}

#[test]
fn test_get_admin() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Test get_admin
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_get_implementation() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Test get_implementation
    let stored_impl = client.get_implementation();
    assert_eq!(stored_impl, impl1);
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn test_get_admin_not_initialized() {
    let (env, client, _admin, _impl1, _impl2) = setup_test_env();
    
    // Test getting admin before initialization (should panic)
    client.get_admin();
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn test_get_implementation_not_initialized() {
    let (env, client, _admin, _impl1, _impl2) = setup_test_env();
    
    // Test getting implementation before initialization (should panic)
    client.get_implementation();
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn test_upgrade_not_initialized() {
    let (env, client, admin, _impl1, impl2) = setup_test_env();
    
    // Test upgrade before initialization (should panic)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl2.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.upgrade(&impl2);
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn test_rollback_not_initialized() {
    let (env, client, admin, _impl1, _impl2) = setup_test_env();
    
    // Test rollback before initialization (should panic)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.rollback();
}

#[test]
fn test_upgrade_same_implementation() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();
    
    // Initialize contract
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin.clone(), impl1.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin, &impl1);
    
    // Test upgrading to the same implementation
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "upgrade",
            args: (impl1.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.upgrade(&impl1);
    
    // Verify implementation is still the same
    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl1);
    
    // Now we should be able to rollback to the same implementation
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    
    client.rollback();
    
    // Should still be the same implementation
    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl1);
}
