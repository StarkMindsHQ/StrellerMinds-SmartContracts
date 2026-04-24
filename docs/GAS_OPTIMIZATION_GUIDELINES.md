# Gas Optimization Guidelines for StrellerMinds Smart Contracts

## Overview

This guide provides comprehensive best practices for developing gas-efficient smart contracts on the StrellerMinds platform. Following these guidelines will ensure optimal performance and cost-effectiveness for all contract operations.

## Core Principles

### 1. Minimize Storage Operations
Storage is the most expensive operation on Soroban. Every storage read/write consumes significant gas.

### 2. Batch Operations When Possible
Group multiple operations into single transactions to reduce overhead.

### 3. Use Efficient Data Structures
Choose data structures that minimize storage and computation requirements.

### 4. Optimize Data Layout
Pack related data together and use compact data types.

### 5. Implement Smart Caching
Cache frequently accessed data in instance storage.

## Storage Optimization Guidelines

### DO ✅

#### Use Batched Storage Operations
```rust
// GOOD: Batched storage writes
let mut batch_data = Map::new(&env);
batch_data.set(OptimizedKey::Admin, admin.to_xdr(&env));
batch_data.set(OptimizedKey::Config, config.to_xdr(&env));
batch_data.set(OptimizedKey::Counter, counter.to_xdr(&env));
StorageOptimizer::batch_write(&env, &batch_data);
```

#### Pack Boolean Flags
```rust
// GOOD: Pack 32 boolean flags into single u32
pub struct UserFlags {
    pub packed: u32, // 32 boolean flags
}

impl UserFlags {
    pub fn set_flag(&mut self, flag_index: usize, value: bool) {
        if value {
            self.packed |= 1 << flag_index;
        } else {
            self.packed &= !(1 << flag_index);
        }
    }
    
    pub fn get_flag(&self, flag_index: usize) -> bool {
        (self.packed & (1 << flag_index)) != 0
    }
}
```

#### Use Compact Data Types
```rust
// GOOD: Use smallest sufficient data types
pub struct CompactCounter {
    pub small_count: u8,    // 0-255
    pub medium_count: u16,  // 0-65,535
    pub large_count: u32,   // 0-4,294,967,295
}
```

#### Implement Time-Bucketed Storage
```rust
// GOOD: Store analytics in daily buckets
pub fn store_daily_analytics(env: &Env, user: &Address, activity_type: u8, data: &Bytes) {
    let day_bucket = (env.ledger().timestamp() / 86400) as u32;
    let key = (user.clone(), activity_type, day_bucket);
    env.storage().persistent().set(&key, data);
}
```

#### Cache Hot Data in Instance Storage
```rust
// GOOD: Cache frequently accessed data
pub fn get_cached_config(env: &Env) -> Config {
    let cache_key = Symbol::short("config_cache");
    if let Some(config) = env.storage().instance().get(&cache_key) {
        config
    } else {
        let config = load_config_from_persistent(env);
        env.storage().instance().set(&cache_key, &config);
        config
    }
}
```

### DON'T ❌

#### Don't Store Redundant Data
```rust
// BAD: Storing duplicate information
env.storage().persistent().set(&DataKey::UserName, &name);
env.storage().persistent().set(&DataKey::UserDisplayName, &name); // Duplicate!
```

#### Don't Use Excessive Storage Writes
```rust
// BAD: Multiple separate writes
env.storage().persistent().set(&DataKey::UserActive, &true);
env.storage().persistent().set(&DataKey::UserVerified, &false);
env.storage().persistent().set(&DataKey::UserPremium, &true);
```

#### Don't Store Large Strings On-Chain
```rust
// BAD: Large string storage
env.storage().persistent().set(&DataKey::UserBio, &very_long_biography);
```

#### Don't Ignore Storage Rent
```rust
// BAD: Storing data that will never be accessed
env.storage().persistent().set(&DataKey::UnusedData, &data);
```

## Function Design Guidelines

### DO ✅

#### Implement Batch Versions of Functions
```rust
// GOOD: Batch operation
pub fn batch_create_assessments(
    env: Env,
    admin: Address,
    assessments: Vec<AssessmentData>,
) -> Result<Vec<u64>, Error> {
    // Process all assessments in a single transaction
    let mut results = Vec::new(&env);
    let mut batch_writes = Map::new(&env);
    
    for assessment in assessments.iter() {
        let id = generate_assessment_id(&env);
        let key = OptimizedKey::Index(id as u16);
        batch_writes.set(key, assessment.to_xdr(&env));
        results.push_back(id);
    }
    
    StorageOptimizer::batch_write(&env, &batch_writes);
    Ok(results)
}
```

#### Use Lazy Loading
```rust
// GOOD: Load data only when needed
pub fn get_user_profile_lazy(env: &Env, user: &Address) -> UserProfile {
    let cache_key = (user.clone(), Symbol::short("profile"));
    if let Some(profile) = StorageOptimizer::get_cached::<UserProfile>(env, cache_key.clone()) {
        return profile;
    }
    
    let profile = load_expensive_profile_data(env, user);
    StorageOptimizer::cache_in_instance(env, cache_key, OptimizedKey::Batch(0), 300);
    profile
}
```

#### Minimize Storage Reads
```rust
// GOOD: Read once, use multiple times
pub fn complex_calculation(env: &Env, user: &Address) -> Result<u64, Error> {
    let profile = get_user_profile(env, user)?; // Single read
    let config = get_config(env, user)?;        // Single read
    
    // Use data multiple times without additional reads
    let result1 = calculate_metric1(&profile, &config);
    let result2 = calculate_metric2(&profile, &config);
    
    Ok(result1 + result2)
}
```

#### Optimize Loops
```rust
// GOOD: Efficient iteration
pub fn process_questions_optimized(env: &Env, question_ids: &[u64]) -> Vec<Question> {
    let mut questions = Vec::new(env);
    
    // Batch load all questions first
    let mut question_map = Map::new(env);
    for &qid in question_ids.iter() {
        if let Some(question) = env.storage().persistent().get::<_, Question>(&OptimizedKey::Index(qid as u16)) {
            question_map.set(qid, question);
        }
    }
    
    // Process in order
    for &qid in question_ids.iter() {
        if let Some(question) = question_map.get(qid) {
            questions.push_back(question);
        }
    }
    
    questions
}
```

### DON'T ❌

#### Don't Create Deep Call Stacks
```rust
// BAD: Excessive recursion or deep nesting
pub fn deeply_nested_function(env: Env, depth: u32) -> Result<(), Error> {
    if depth > 0 {
        return deeply_nested_function(env, depth - 1); // Expensive!
    }
    Ok(())
}
```

#### Don't Ignore Gas Costs in Error Paths
```rust
// BAD: Expensive error handling
pub fn expensive_validation(env: Env, data: LargeData) -> Result<(), Error> {
    // Expensive validation that runs even when it will fail
    let complex_calculation = compute_expensive_hash(&data);
    if !is_valid_format(&data) {
        return Err(Error::InvalidFormat); // Wasted gas!
    }
    Ok(())
}
```

#### Don't Use Inefficient Data Structures
```rust
// BAD: Using Vec for frequent lookups
pub fn find_user_slow(env: &Env, users: &Vec<User>, target: &Address) -> Option<User> {
    for user in users.iter() { // O(n) lookup
        if user.address == *target {
            return Some(user.clone());
        }
    }
    None
}

// GOOD: Use Map for lookups
pub fn find_user_fast(env: &Env, users: &Map<Address, User>, target: &Address) -> Option<User> {
    users.get(target) // O(1) lookup
}
```

## Event Emission Guidelines

### DO ✅

#### Batch Related Events
```rust
// GOOD: Batch event emission
pub fn emit_batch_events(env: &Env, events: Vec<EventData>) {
    let batch_event = BatchEvent {
        events,
        timestamp: env.ledger().timestamp(),
    };
    env.events().publish((Symbol::short("batch"),), batch_event);
}
```

#### Use Compact Event Data
```rust
// GOOD: Compact event structure
#[contracttype]
pub struct CompactEvent {
    pub user: Address,
    pub action: u8,        // Use enum as u8
    pub timestamp: u32,   // Truncated timestamp
    pub metadata: u32,    // Packed metadata
}
```

#### Emit Only Essential Events
```rust
// GOOD: Minimal event emission
pub fn track_essential_activity(env: &Env, user: &Address, action: ActionType) {
    env.events().publish(
        (Symbol::short("activity"),),
        CompactEvent {
            user: user.clone(),
            action: action as u8,
            timestamp: (env.ledger().timestamp() / 1000) as u32, // Second precision
            metadata: 0,
        }
    );
}
```

### DON'T ❌

#### Don't Emit Events for Every Small Change
```rust
// BAD: Excessive event emission
pub fn update_counter_with_events(env: &Env, counter_id: u32) {
    for i in 0..10 {
        increment_counter(env, counter_id);
        env.events().publish((Symbol::short("increment"),), i); // Too many events!
    }
}
```

#### Don't Include Large Data in Events
```rust
// BAD: Large event payloads
pub fn emit_large_event(env: &Env, user: &Address, large_data: LargeData) {
    env.events().publish((Symbol::short("large_event"),), large_data); // Expensive!
}
```

## Contract Interaction Guidelines

### DO ✅

#### Minimize Cross-Contract Calls
```rust
// GOOD: Batch cross-contract operations
pub fn batch_cross_contract_calls(env: &Env, operations: Vec<ContractOperation>) -> Result<Vec<Result>, Error> {
    // Group operations by target contract
    let mut contract_ops = Map::new(env);
    for op in operations.iter() {
        let ops = contract_ops.get(op.target_contract.clone()).unwrap_or(Vec::new(env));
        ops.push_back(op.clone());
        contract_ops.set(op.target_contract.clone(), ops);
    }
    
    // Execute batched calls
    let mut results = Vec::new(env);
    for (contract, ops) in contract_ops.iter() {
        let contract_results = execute_contract_operations(env, contract, ops)?;
        results.push_back(contract_results);
    }
    
    Ok(results)
}
```

#### Use Efficient Data Serialization
```rust
// GOOD: Compact serialization
pub fn serialize_compact<T: contracttype>(env: &Env, data: &T) -> Bytes {
    // Use custom compact serialization
    data.to_compact_bytes(env)
}
```

#### Implement Callback Patterns
```rust
// GOOD: Callback pattern for complex operations
pub fn start_complex_operation(env: &Env, params: OperationParams) -> Result<BytesN<32>, Error> {
    let operation_id = generate_operation_id(env);
    
    // Start operation asynchronously
    env.storage().persistent().set(&OperationKey(operation_id), &OperationState::InProgress);
    
    // Emit event for callback
    env.events().publish((Symbol::short("operation_started"),), operation_id);
    
    Ok(operation_id)
}
```

### DON'T ❌

#### Don't Create Deep Call Stacks
```rust
// BAD: Contract A calls B calls C calls D
pub fn deep_call_chain(env: &Env) -> Result<(), Error> {
    contract_b::expensive_operation(env)?; // Calls contract_c
    // ... which calls contract_d
    // Very expensive!
}
```

#### Don't Transfer Large Data Between Contracts
```rust
// BAD: Large data transfer between contracts
pub fn transfer_large_data(env: &Env, target_contract: Address, large_data: LargeData) -> Result<(), Error> {
    target_contract.require_auth();
    target_contract.call(
        &env,
        &Symbol::short("receive_large_data"),
        large_data, // Expensive transfer!
    )?;
    Ok(())
}
```

## Testing and Profiling Guidelines

### DO ✅

#### Profile Gas Usage
```rust
// GOOD: Gas profiling in tests
#[test]
fn test_gas_optimization() {
    let env = Env::default();
    
    // Profile original implementation
    let start_gas = env.contract().get_current_gas();
    original_function(env.clone(), test_params);
    let original_gas = env.contract().get_current_gas() - start_gas;
    
    // Profile optimized implementation
    let start_gas = env.contract().get_current_gas();
    optimized_function(env.clone(), test_params);
    let optimized_gas = env.contract().get_current_gas() - start_gas;
    
    // Verify optimization
    assert!(optimized_gas < original_gas, "Optimization failed");
    println!("Gas savings: {}%", ((original_gas - optimized_gas) * 100) / original_gas);
}
```

#### Test Edge Cases
```rust
// GOOD: Test gas usage in edge cases
#[test]
fn test_gas_usage_edge_cases() {
    let env = Env::default();
    
    // Test with maximum data sizes
    let large_data = create_large_test_data();
    let gas_used = measure_gas_usage(&env, || {
        process_large_data(env.clone(), large_data)
    });
    
    assert!(gas_used < MAX_GAS_LIMIT, "Exceeded gas limit with large data");
}
```

#### Benchmark Batch Operations
```rust
// GOOD: Benchmark batch vs individual operations
#[test]
fn benchmark_batch_operations() {
    let env = Env::default();
    let operations = create_test_operations(100);
    
    // Individual operations
    let individual_gas = measure_gas_usage(&env, || {
        for op in operations.iter() {
            execute_individual_operation(env.clone(), op);
        }
    });
    
    // Batch operations
    let batch_gas = measure_gas_usage(&env, || {
        execute_batch_operations(env.clone(), &operations);
    });
    
    let savings = ((individual_gas - batch_gas) * 100) / individual_gas;
    assert!(savings > 20, "Batch operations should save at least 20% gas");
}
```

### DON'T ❌

#### Don't Ignore Gas Costs in Tests
```rust
// BAD: Tests without gas consideration
#[test]
fn test_functionality_only() {
    let env = Env::default();
    let result = expensive_function(env, large_params);
    assert!(result.is_ok()); // No gas measurement!
}
```

#### Don't Test Only with Small Data
```rust
// BAD: Only testing with minimal data
#[test]
fn test_with_small_data() {
    let env = Env::default();
    let small_data = create_small_test_data(); // Not realistic!
    let result = process_data(env, small_data);
    assert!(result.is_ok());
}
```

## Deployment and Monitoring Guidelines

### DO ✅

#### Implement Gas Monitoring
```rust
// GOOD: Production gas monitoring
pub fn monitor_gas_usage(env: &Env, function_name: Symbol, gas_used: u64) {
    let threshold = get_gas_threshold(function_name);
    if gas_used > threshold {
        env.events().publish(
            (Symbol::short("gas_alert"),),
            GasAlert {
                function: function_name,
                gas_used,
                threshold,
                timestamp: env.ledger().timestamp(),
            }
        );
    }
}
```

#### Set Gas Limits
```rust
// GOOD: Enforce gas limits
pub fn execute_with_gas_limit<T>(
    env: &Env,
    operation: impl FnOnce(&Env) -> Result<T, Error>,
    limit: u64
) -> Result<T, Error> {
    let start_gas = env.contract().get_current_gas();
    let result = operation(env);
    let gas_used = env.contract().get_current_gas() - start_gas;
    
    if gas_used > limit {
        return Err(Error::GasLimitExceeded);
    }
    
    result
}
```

#### Use Gas Optimization Flags
```rust
// GOOD: Feature flags for optimization
pub fn conditional_optimization(env: &Env, data: &Data) -> Result<(), Error> {
    if is_optimization_enabled(env) {
        process_data_optimized(env, data)
    } else {
        process_data_standard(env, data)
    }
}
```

### DON'T ❌

#### Don't Deploy Without Gas Testing
```rust
// BAD: No gas testing before deployment
// Deploying directly without comprehensive gas profiling
```

#### Don't Ignore Production Gas Usage
```rust
// BAD: No monitoring in production
pub fn unmonitored_function(env: &Env, params: Params) -> Result<(), Error> {
    // Complex operations without gas monitoring
    expensive_computation(env, params);
    Ok(())
}
```

## Code Review Checklist

### Storage Optimization
- [ ] Are storage operations batched?
- [ ] Is data packed efficiently?
- [ ] Are boolean flags packed into u32?
- [ ] Is time-bucketed storage used for analytics?
- [ ] Is caching implemented for hot data?

### Function Design
- [ ] Are batch operations available?
- [ ] Is lazy loading implemented?
- [ ] Are storage reads minimized?
- [ ] Are loops optimized?
- [ ] Are data structures appropriate for use case?

### Event Emission
- [ ] Are events batched when possible?
- [ ] Is event data compact?
- [ ] Are only essential events emitted?
- [ ] Are event payloads minimal?

### Contract Interactions
- [ ] Are cross-contract calls minimized?
- [ ] Is data serialization efficient?
- [ ] Are callback patterns used appropriately?
- [ ] Is call stack depth controlled?

### Testing and Profiling
- [ ] Is gas usage profiled?
- [ ] Are edge cases tested?
- [ ] Are batch operations benchmarked?
- [ ] Are gas limits enforced?

### Monitoring
- [ ] Is gas usage monitored in production?
- [ ] Are gas alerts implemented?
- [ ] Are optimization flags available?
- [ ] Is performance tracked over time?

## Common Gas Optimization Patterns

### Pattern 1: Packed Structs
```rust
// Use for frequently accessed, small data
#[contracttype]
pub struct PackedUserData {
    pub flags: u32,        // 32 booleans
    pub counters: [u8; 4], // 4 small counters
    pub timestamp: u32,    // Last update
}
```

### Pattern 2: Batch Operations
```rust
// Use for multiple similar operations
pub fn batch_operation<T>(
    env: &Env,
    operations: Vec<T>,
    processor: impl Fn(&Env, &T) -> Result<(), Error>
) -> Result<BatchResult, Error> {
    let mut successful = 0;
    let mut failed = 0;
    
    for op in operations.iter() {
        match processor(env, op) {
            Ok(()) => successful += 1,
            Err(_) => failed += 1,
        }
    }
    
    Ok(BatchResult { successful, failed })
}
```

### Pattern 3: Cached Access
```rust
// Use for frequently accessed data
pub fn cached_get<T: contracttype>(
    env: &Env,
    key: &T,
    cache_ttl: u32
) -> Option<T> {
    let cache_key = format!("cache_{:?}", key);
    
    if let Some(cached) = env.storage().instance().get::<_, T>(&cache_key) {
        return Some(cached);
    }
    
    if let Some(data) = env.storage().persistent().get(key) {
        env.storage().instance().set(&cache_key, &data);
        // Set expiry (simplified)
        Some(data)
    } else {
        None
    }
}
```

### Pattern 4: Time-Bucketed Analytics
```rust
// Use for time-series data
pub fn store_analytics_bucket(
    env: &Env,
    user: &Address,
    metric: u8,
    value: u32
) {
    let bucket = (env.ledger().timestamp() / 86400) as u32; // Daily bucket
    let key = (user.clone(), metric, bucket);
    
    let mut current: u32 = env.storage().persistent()
        .get(&key)
        .unwrap_or(0);
    current += value;
    
    env.storage().persistent().set(&key, &current);
}
```

## Conclusion

Following these guidelines will ensure that StrellerMinds smart contracts are gas-efficient, cost-effective, and performant at scale. Regular profiling, monitoring, and optimization are essential for maintaining optimal gas usage as the platform evolves.

Remember:
1. **Storage is expensive** - minimize and optimize storage operations
2. **Batch when possible** - group operations to reduce overhead
3. **Profile continuously** - monitor gas usage in development and production
4. **Test thoroughly** - include gas usage in all testing scenarios
5. **Monitor production** - track real-world gas consumption and optimize accordingly

By consistently applying these principles, we can ensure the StrellerMinds platform remains cost-effective and efficient for all users.
