# StarkMinds Smart Contracts

Welcome to the official documentation for the StarkMinds smart contracts, built on the Stellar network using Soroban.

## 🏗️ System Architecture
This diagram shows the flow of course validation and credential issuance.

```mermaid
graph LR
    Student((Student)) -->|Submits Proof| Contract[Soroban Contract]
    Contract -->|Verifies Logic| Storage[(Stellar Ledger)]
    Storage -->|Emits Event| Mint[Credential Issued]
    Mint -->|Success| Student