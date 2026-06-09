#Contract Testing for StrellerMinds Smart Contracts

##Issue #471 - Status: In Progress

Goal: Add contract testing between microservices and Soroban smart contracts.

### Current Approach (Phone-Friendly Start)
- Use *Consumer-Driven Contracts* where possible.
- Start with OpenAPI/Swagger schema validation for REST endpoints.
- Plan to migrate to **Pact** for full behavioral testing.

### Key Contracts to Cover
- Soroban contract invocation endpoints
- User authentication & progress APIs
- Transaction submission

### Basic Validation Setup
Add to `package.json` (if Node.js parts exist) or CI:
```json
"scripts": {
  "test:contract": "echo 'Contract tests will run here - using Pact or Spectral for OpenAPI'"
}