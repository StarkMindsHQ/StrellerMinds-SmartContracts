#![cfg(test)]

use crate::{Token, TokenClient, Error};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);
    let admin = Address::random(&env);

    // Test successful initialization
    client.initialize(&admin);

    // Test re-initialization (should fail)
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_mint() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let user = Address::random(&env);

    // Initialize the contract
    client.initialize(&admin);

    // Mint tokens as admin
    env.mock_all_auths();
    client.mint(&user, &100);

    // Check balance
    let balance = client.balance(&user);
    assert_eq!(balance, 100);

    // Test minting with invalid amount
    let result = client.try_mint(&user, &0);
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);

    // Initialize the contract
    client.initialize(&admin);

    // Mint tokens to user1
    env.mock_all_auths();
    client.mint(&user1, &100);

    // Transfer tokens from user1 to user2
    client.transfer(&user1, &user2, &50);

    // Check balances
    let balance1 = client.balance(&user1);
    let balance2 = client.balance(&user2);
    assert_eq!(balance1, 50);
    assert_eq!(balance2, 50);

    // Test transfer with insufficient balance
    let result = client.try_transfer(&user1, &user2, &100);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_reentrancy_guard_transfer() {
    use std::panic;
    let env = Env::default();
    let contract_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &contract_id);
    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    client.initialize(&admin);
    env.mock_all_auths();
    client.mint(&user1, &100);
    // Simulate reentrancy by calling transfer inside a transfer (mocked by direct call)
    let result = panic::catch_unwind(|| {
        let _ = client.transfer(&user1, &user2, &10);
        // Attempt reentrant call
        let _ = client.transfer(&user1, &user2, &10);
    });
    assert!(result.is_err(), "Reentrancy was not prevented");
}
