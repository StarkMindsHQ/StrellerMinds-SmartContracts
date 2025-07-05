#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Env, Address};

#[test]
fn test_proxy_upgrade_and_rollback() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let impl1 = Address::generate(&env);
    let impl2 = Address::generate(&env);

    // Initialize
    Proxy::initialize(env.clone(), admin.clone(), impl1.clone());
    assert_eq!(Proxy::get_implementation(env.clone()), impl1);
    assert_eq!(Proxy::get_admin(env.clone()), admin);

    // Upgrade
    env.mock_all_auths();
    Proxy::upgrade(env.clone(), impl2.clone());
    assert_eq!(Proxy::get_implementation(env.clone()), impl2);

    // Rollback
    Proxy::rollback(env.clone());
    assert_eq!(Proxy::get_implementation(env.clone()), impl1);
}
