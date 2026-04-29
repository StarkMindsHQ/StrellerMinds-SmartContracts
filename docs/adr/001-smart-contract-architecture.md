# ADR-001: Smart Contract Architecture

## Status
Accepted

## Context
The StrellerMinds educational platform requires a robust smart contract architecture to handle various educational functions including learning analytics, token incentives, progress tracking, and credentialing. The system needs to support:

1. **Educational Data Management**: Tracking student progress, sessions, and achievements
2. **Token Economy**: Managing educational tokens with incentives and staking
3. **Analytics**: Comprehensive learning analytics and reporting
4. **Security**: Role-based access control and data protection
5. **Scalability**: Support for multiple educational institutions and courses

Key constraints include:
- Built on Stellar blockchain using Soroban SDK
- Must handle complex educational data structures
- Require gas optimization for cost-effective operations
- Need modular design for maintainability
- Must support upgradeable contracts

## Decision
We adopted a **modular smart contract architecture** with the following key principles:

### 1. Contract Separation by Domain
- **Analytics Contract**: Handles learning sessions, progress tracking, and achievement systems
- **Token Contract**: Manages educational tokens, incentives, and staking mechanisms
- **Shared Contract**: Provides common utilities, RBAC, validation, and security patterns

### 2. Storage Architecture
- **Instance Storage**: For contract-wide configuration and admin settings
- **Persistent Storage**: For long-term data like analytics and token balances
- **Temporary Storage**: For session data and intermediate calculations
- **Efficient Key Design**: Structured storage keys using enums and composite keys

### 3. Event-Driven Communication
- **Standardized Event Schema**: Common event structures across all contracts
- **Event Types**: Access control, analytics, token operations, and system events
- **Event Filtering**: Support for event filtering and aggregation

### 4. Security Framework
- **Role-Based Access Control (RBAC)**: Hierarchical permission system
- **Rate Limiting**: Protection against spam and abuse
- **Reentrancy Guards**: Prevent reentrancy attacks
- **Input Validation**: Comprehensive validation for all inputs

### 5. Gas Optimization Strategies
- **Batch Operations**: Support for bulk operations to reduce transaction costs
- **Lazy Computation**: Compute analytics on-demand rather than pre-computing
- **Efficient Data Structures**: Use of optimized data types and storage patterns
- **Selective Storage**: Only store essential data on-chain

## Consequences

### Benefits
1. **Modularity**: Each contract has a clear responsibility, making the system easier to understand and maintain
2. **Reusability**: Shared contract provides common functionality across all contracts
3. **Security**: Centralized security patterns reduce the risk of vulnerabilities
4. **Scalability**: Modular design allows for independent scaling of different components
5. **Testability**: Each contract can be tested independently
6. **Upgradeability**: Individual contracts can be upgraded without affecting others

### Drawbacks
1. **Complexity**: More contracts to deploy and manage
2. **Inter-contract Communication**: Requires careful handling of contract interactions
3. **Gas Costs**: Multiple contract calls may increase transaction costs
4. **Deployment Overhead**: Need to deploy and configure multiple contracts

### Trade-offs
- **Modularity vs Simplicity**: Chose modularity for long-term maintainability over deployment simplicity
- **On-chain vs Off-chain**: Store essential data on-chain for immutability, use off-chain for large datasets
- **Real-time vs Batch Processing**: Use batch processing for analytics to optimize gas costs

## Implementation

### Contract Structure
```
contracts/
├── analytics/          # Learning analytics and progress tracking
├── token/              # Token management and incentives
├── shared/             # Common utilities and security
├── mobile-optimizer/   # Mobile-specific optimizations
├── progress/           # Simple progress tracking
├── proxy/              # Upgradeable contract pattern
└── search/             # Search functionality
```

### Key Patterns
1. **Initialization Pattern**: All contracts follow a consistent initialization pattern with admin setup
2. **Error Handling**: Standardized error types and error codes across contracts
3. **Event Emission**: Consistent event schema for all contract operations
4. **Access Control**: Unified RBAC implementation in shared contract
5. **Validation**: Common validation functions in shared contract

### Storage Design
- Use composite keys for efficient data retrieval
- Implement lazy loading for large datasets
- Use TTL for temporary data where applicable
- Implement data archiving for old analytics data

## Alternatives Considered

### 1. Monolithic Contract
**Pros**: Simpler deployment, lower gas costs for inter-contract calls
**Cons**: Harder to maintain, larger attack surface, difficult to upgrade individual components
**Rejected**: Maintainability concerns outweighed simplicity benefits

### 2. Micro-contracts (One function per contract)
**Pros**: Maximum modularity, minimal attack surface per contract
**Cons**: Extremely high deployment overhead, complex inter-contract communication
**Rejected**: Too granular, would create unnecessary complexity

### 3. Hybrid Architecture (Core contracts + micro-services)
**Pros**: Balance between modularity and simplicity
**Cons**: Still requires complex inter-contract communication
**Rejected**: Current modular approach provides better balance

## References

- [Soroban Contract Documentation](https://soroban.stellar.org/docs)
- [Stellar Smart Contract Best Practices](https://stellar.org/developers/smart-contracts)
- [Analytics Contract Implementation](../contracts/analytics/README.md)
- [Token Contract Implementation](../contracts/token/README.md)
- [Shared Contract Documentation](../contracts/shared/README.md)
