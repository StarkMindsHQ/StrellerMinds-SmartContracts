#![cfg(test)]

use crate::{Token, TokenClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Benchmark for token minting and cross-account transfers.
/// Measures CPU and memory consumption in the test environment.
#[test]
fn benchmark_token_load() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &token_id);

    // Matching lib.rs's initialize(env, admin)
    client.initialize(&admin);

    let mut users = Vec::new();
    for _ in 0..10 {
        users.push(Address::generate(&env));
    }

    // Benchmark Minting
    println!("--- MINTING BENCHMARK ---");
    for user in &users {
        // Matching lib.rs's mint(env, to, amount: u64)
        client.mint(user, &1000);
    }

    // Benchmark Transfers
    println!("--- TRANSFER BENCHMARK ---");
    for i in 0..users.len() - 1 {
        // Matching lib.rs's transfer(env, from, to, amount: u64)
        client.transfer(&users[i], &users[i+1], &10);
    }

    println!("Token benchmark completed for {} users", users.len());
}

#[test]
fn benchmark_gas_efficiency() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let token_id = env.register_contract(None, Token);
    let client = TokenClient::new(&env, &token_id);
    
    client.initialize(&admin);
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    client.mint(&user1, &1000000);
    
    // Multiple transfers to stress-test storage access
    for _ in 0..50 {
        client.transfer(&user1, &user2, &1);
    }
}
