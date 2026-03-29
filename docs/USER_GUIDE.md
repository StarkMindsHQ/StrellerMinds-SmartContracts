# User Guide

Welcome to the **StarkMinds Smart Contracts User Guide**. This document is designed for educators, institutional administrators, and platform integrators who rely on the StrellerMinds infrastructure to manage learning data, credentials, and achievements.

## 🎓 For Educators and Institutions

### Managing Course Progress
The StarkMinds platform utilizes a robust on-chain **Analytics Contract** to register learning sessions and compute performance. 
- When your students begin a module, the platform automatically triggers a `record_session` on the smart contract.
- Their session is securely validated and subsequently updated upon completion. Performance history is entirely verifiable on the Stellar ledger.

### Issuing Credentials
Once a user completes a predefined learning track, the educational body can issue a verifiable credential:
1. Ensure the administrator account holds the correct credentials and RBAC roles.
2. Sign the transaction payload linking the `student_address`, `achievement`, and `metadata_hash`.
3. The credential becomes a permanent entity on-chain. (Reference: [Cross-Chain Credentials](CROSS_CHAIN_DEPLOYMENT.md))

### Distributing Rewards
You can incentivize student completion through the **Token Contract**. Administrators holding `Minter` roles can reward top performers or issue stakes for platform engagement.

---

## 💻 For End-Users (Students)

### Verifying Your Credentials
As a student, any credential minted accurately to your public Stellar address is cryptographically yours.
- **View Achievements:** Platform dashboards query the ledger directly, displaying real-time updates on your achievements.
- **Portability:** Utilizing the cross-chain features, your StarkMinds credentials can be verified seamlessly across integrated network applications (e.g., Ethereum vs. Stellar).

### Leaderboards and Gamification
Participate actively in courses to rank on performance-based **Leaderboards**. Rankings are recalculated on-chain considering your `final_score` and `completion_percentage`. These metrics dictate access to potential tokenized learning incentives.

---

## 🛠️ Typical Workflows

**1. Onboarding**
Institutional Admins are granted roles through the `Shared Utils` (RBAC) by SuperAdmins. They initialize courses and map specific contract paths.

**2. Active Learning**
Students engage with the curriculum on mobile or web. The `Mobile Optimizer` batches interactions into efficient transactions to record progress.

**3. Analytics & Reporting**
Institutions generate daily or weekly metrics directly from the smart contract via `generate_progress_report`. These are immutable historical reports to measure curriculum efficacy.
