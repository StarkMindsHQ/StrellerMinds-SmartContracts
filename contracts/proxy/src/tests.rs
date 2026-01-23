#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, Symbol,
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

// ============================================
// ENHANCED UPGRADE SYSTEM TESTS
// ============================================

#[test]
fn test_propose_upgrade() {
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

    // Test proposing upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_upgrade",
            args: (
                admin.clone(),
                impl2.clone(),
                1u32,
                1u32,
                0u32,
                "Test upgrade".to_string(),
                2u32,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let proposal_id = client.propose_upgrade(
        &admin,
        &impl2,
        &1,
        &1,
        &0,
        &"Test upgrade".to_string(),
        &2,
    );

    assert!(proposal_id.to_string().starts_with("upgrade_"));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_propose_upgrade_unauthorized() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let non_admin = Address::generate(&env);

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

    // Test proposing upgrade without permission
    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_upgrade",
            args: (
                non_admin.clone(),
                impl2.clone(),
                1u32,
                1u32,
                0u32,
                "Test upgrade".to_string(),
                2u32,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.propose_upgrade(
        &non_admin,
        &impl2,
        &1,
        &1,
        &0,
        &"Test upgrade".to_string(),
        &2,
    );
}

#[test]
fn test_vote_on_upgrade() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

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

    // Propose upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_upgrade",
            args: (
                admin.clone(),
                impl2.clone(),
                1u32,
                1u32,
                0u32,
                "Test upgrade".to_string(),
                2u32,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let proposal_id = client.propose_upgrade(
        &admin,
        &impl2,
        &1,
        &1,
        &0,
        &"Test upgrade".to_string(),
        &2,
    );

    // First vote
    env.mock_auths(&[MockAuth {
        address: &voter1,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "vote_on_upgrade",
            args: (voter1.clone(), proposal_id.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let vote_count = client.vote_on_upgrade(&voter1, &proposal_id);
    assert_eq!(vote_count, 1);

    // Second vote
    env.mock_auths(&[MockAuth {
        address: &voter2,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "vote_on_upgrade",
            args: (voter2.clone(), proposal_id.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let vote_count = client.vote_on_upgrade(&voter2, &proposal_id);
    assert_eq!(vote_count, 2);
}

#[test]
fn test_execute_upgrade_with_timelock() {
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

    // Set timelock of 100 seconds
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_upgrade_timelock",
            args: (100u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_upgrade_timelock(&100);

    // Propose upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_upgrade",
            args: (
                admin.clone(),
                impl2.clone(),
                1u32,
                1u32,
                0u32,
                "Test upgrade".to_string(),
                1u32,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let proposal_id = client.propose_upgrade(
        &admin,
        &impl2,
        &1,
        &1,
        &0,
        &"Test upgrade".to_string(),
        &1,
    );

    // Vote on upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "vote_on_upgrade",
            args: (admin.clone(), proposal_id.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.vote_on_upgrade(&admin, &proposal_id);

    // Try to execute before timelock expires (should fail)
    let result = std::panic::catch_unwind(|| {
        client.execute_upgrade();
    });
    assert!(result.is_err());

    // Advance time past timelock
    env.ledger().with_mut(|li| {
        li.timestamp += 150;
    });

    // Execute upgrade
    client.execute_upgrade();

    // Verify implementation was updated
    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl2);
}

#[test]
fn test_emergency_pause() {
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

    // Set emergency pause
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_emergency_pause",
            args: (true,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_emergency_pause(&true);

    // Verify pause is active
    assert!(client.is_emergency_paused());

    // Try standard upgrade while paused (should panic)
    let result = std::panic::catch_unwind(|| {
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
    });
    assert!(result.is_err());

    // Remove emergency pause
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_emergency_pause",
            args: (false,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_emergency_pause(&false);

    // Verify pause is inactive
    assert!(!client.is_emergency_paused());

    // Standard upgrade should now work
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

    let current_impl = client.get_implementation();
    assert_eq!(current_impl, impl2);
}

#[test]
fn test_get_current_version() {
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

    // Get current version
    let version = client.get_current_version();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_get_upgrade_timelock() {
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

    // Set timelock
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_upgrade_timelock",
            args: (300u32,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_upgrade_timelock(&300);

    // Get timelock
    let timelock = client.get_upgrade_timelock();
    assert!(timelock > 0);
}

#[test]
fn test_get_pending_upgrade() {
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

    // Initially no pending upgrade
    let pending = client.get_pending_upgrade();
    assert!(pending.is_none());

    // Propose and vote for upgrade
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "propose_upgrade",
            args: (
                admin.clone(),
                impl2.clone(),
                1u32,
                1u32,
                0u32,
                "Test upgrade".to_string(),
                1u32,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let proposal_id = client.propose_upgrade(
        &admin,
        &impl2,
        &1,
        &1,
        &0,
        &"Test upgrade".to_string(),
        &1,
    );

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "vote_on_upgrade",
            args: (admin.clone(), proposal_id.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.vote_on_upgrade(&admin, &proposal_id);

    // Still no pending upgrade until executed
    let pending = client.get_pending_upgrade();
    assert!(pending.is_none());

    // Execute upgrade to set pending
    env.ledger().with_mut(|li| {
        li.timestamp += 1000;
    });
    
    client.execute_upgrade();

    // Now there should be a pending upgrade
    let pending = client.get_pending_upgrade();
    assert!(pending.is_some());
}

// ============================================
// BASIC INITIALIZATION TESTS
// ============================================

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
    let (_env, client, _admin, _impl1, _impl2) = setup_test_env();

    // Test getting admin before initialization (should panic)
    client.get_admin();
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn test_get_implementation_not_initialized() {
    let (_env, client, _admin, _impl1, _impl2) = setup_test_env();

    // Test getting implementation before initialization (should panic)
    client.get_implementation();
}

// ============================================
// UPGRADE FUNCTIONALITY TESTS
// ============================================

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

// ============================================
// ROLLBACK FUNCTIONALITY TESTS
// ============================================

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

// ============================================
// MULTIPLE UPGRADES AND ROLLBACKS
// ============================================

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

// ============================================
// RE-INITIALIZATION PROTECTION TESTS
// ============================================

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_reinitialize() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let admin2 = Address::generate(&env);

    // First initialization
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

    // Attempt re-initialization (should panic)
    env.mock_auths(&[MockAuth {
        address: &admin2,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "initialize",
            args: (admin2.clone(), impl2.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.initialize(&admin2, &impl2);
}

#[test]
fn test_initialization_sets_all_fields_correctly() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();

    // Initialize
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

    // Verify all fields are set correctly
    assert_eq!(client.get_admin(), admin, "Admin should be set correctly");
    assert_eq!(
        client.get_implementation(),
        impl1,
        "Implementation should be set correctly"
    );
}

// ============================================
// STORAGE LAYOUT INVARIANT TESTS
// ============================================

#[test]
fn test_storage_isolation_after_upgrade() {
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

    let admin_before = client.get_admin();

    // Perform upgrade
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

    // Admin should remain unchanged after upgrade
    let admin_after = client.get_admin();
    assert_eq!(
        admin_before, admin_after,
        "Admin should not change during upgrade"
    );

    // Implementation should be updated
    assert_eq!(
        client.get_implementation(),
        impl2,
        "Implementation should be updated"
    );
}

#[test]
fn test_rollback_stack_integrity() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let impl3 = Address::generate(&env);

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

    // Multiple upgrades
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

    // Verify current implementation
    assert_eq!(client.get_implementation(), impl3, "Should be on impl3");

    // Rollback twice
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
    assert_eq!(
        client.get_implementation(),
        impl2,
        "Should be on impl2 after first rollback"
    );

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
    assert_eq!(
        client.get_implementation(),
        impl1,
        "Should be on impl1 after second rollback"
    );
}

#[test]
fn test_storage_keys_dont_collide() {
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

    // Verify all storage keys are accessible and distinct
    let stored_admin = client.get_admin();
    let stored_impl = client.get_implementation();

    assert_eq!(stored_admin, admin, "Admin should be retrievable");
    assert_eq!(stored_impl, impl1, "Implementation should be retrievable");
    assert_ne!(
        stored_admin, stored_impl,
        "Admin and implementation should be different"
    );
}

// ============================================
// DELEGATE CALL VALIDATION TESTS
// ============================================

#[test]
fn test_implementation_address_validation() {
    let (env, client, admin, impl1, _impl2) = setup_test_env();

    // Initialize with valid implementation
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

    // Verify implementation is stored correctly
    assert_eq!(
        client.get_implementation(),
        impl1,
        "Implementation should be stored correctly"
    );
}

#[test]
fn test_delegate_call_forwards_to_correct_implementation() {
    let (env, client, admin, impl1, impl2) = setup_test_env();

    // Initialize
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

    // Verify proxy points to impl1
    assert_eq!(client.get_implementation(), impl1);

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

    // Verify proxy now points to impl2
    assert_eq!(client.get_implementation(), impl2);
}

// ============================================
// UNAUTHORIZED OPERATION TESTS
// ============================================

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_admin_cannot_upgrade() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let non_admin = Address::generate(&env);

    // Initialize with admin
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

    // Attempt upgrade from non-admin (should panic)
    env.mock_auths(&[MockAuth {
        address: &non_admin,
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
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_non_admin_cannot_rollback() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let non_admin = Address::generate(&env);

    // Initialize
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

    // Upgrade
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

    // Attempt rollback from non-admin (should panic)
    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "rollback",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.rollback();
}

// ============================================
// UPGRADE CHAIN AND EDGE CASE TESTS
// ============================================

#[test]
fn test_upgrade_after_rollback() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let impl3 = Address::generate(&env);

    // Initialize
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

    // Can upgrade again after rollback
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
    assert_eq!(
        client.get_implementation(),
        impl3,
        "Should be able to upgrade after rollback"
    );
}

#[test]
fn test_admin_remains_consistent_across_operations() {
    let (env, client, admin, impl1, impl2) = setup_test_env();
    let impl3 = Address::generate(&env);

    // Initialize
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

    assert_eq!(client.get_admin(), admin, "Admin should be set after init");

    // After upgrade
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
    assert_eq!(
        client.get_admin(),
        admin,
        "Admin should remain same after upgrade"
    );

    // After another upgrade
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
    assert_eq!(
        client.get_admin(),
        admin,
        "Admin should remain same after second upgrade"
    );

    // After rollback
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
    assert_eq!(
        client.get_admin(),
        admin,
        "Admin should remain same after rollback"
    );
}
