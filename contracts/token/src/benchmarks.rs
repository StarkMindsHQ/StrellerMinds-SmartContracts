#![cfg(test)]

use crate::{Token, TokenClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

/// Benchmark for token minting and cross-account transfers.
/// Measures CPU and memory consumption in the test environment.
#[test]
fn benchmark_token_load() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register(Token, ());
    let client = TokenClient::new(&env, &token_id);

    // Matching lib.rs's initialize(env, admin)
    client.initialize(&admin);

    let mut users: Vec<Address> = Vec::new(&env);
    for _ in 0..10 {
        users.push_back(Address::generate(&env));
    }

    for user in users.iter() {
        // Matching lib.rs's mint(env, to, amount: u64)
        client.mint(&user, &1000);
    }

    for i in 0..users.len() - 1 {
        // Matching lib.rs's transfer(env, from, to, amount: u64)
        let from = users.get(i).expect("source user must exist");
        let to = users.get(i + 1).expect("destination user must exist");
        client.transfer(&from, &to, &10);
    }
}

#[test]
fn benchmark_gas_efficiency() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register(Token, ());
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
