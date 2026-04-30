# Walkthrough: Compliance Report Automation Updates

This document provides a technical overview of the changes made to implement the automated compliance reporting system. 

## 🏗️ Architecture Overview

The compliance reporting system is a cross-functional feature that spans smart contracts, the API backend, and CI/CD pipelines. It follows a "Collect -> Process -> Distribute" pattern.

### 1. Security & RBAC (Shared Library)
We expanded the Role-Based Access Control (RBAC) system to support compliance-specific operations.
- **Files touched:** 
    - `contracts/shared/src/roles.rs`: Added `RoleLevel::ComplianceAdmin` and `Permission::GenerateComplianceReport`.
    - `contracts/shared/src/permissions.rs`: Added `compliance_admin_permissions` helper and updated level-to-permission mapping.
- **Key Logic:** The `Admin` (Level 4) and `SuperAdmin` (Level 5) can now grant the `ComplianceAdmin` (Level 6) role, despite it being a higher numeric level, thanks to an explicit override in `can_grant`.

### 2. API Backend Integration
The API acts as the bridge between the user (admin) and the generation script.
- **Service Layer (`api/src/services/complianceService.ts`):** 
    - Spawns a shell process to execute the core generation script.
    - Passes environment variables like `YEAR`, `MONTH`, and `DATABASE_URL`.
    - Captures the script's output to return the final S3 URL of the report.
- **Route Layer (`api/src/routes/admin.ts`):** 
    - Mounts at `POST /api/v1/admin/compliance-report`.
    - Protected by `authenticate` and `requireScope("compliance")`.

### 3. Core Logic (Scripts & Templates)
The heavy lifting is done outside the Node.js event loop to ensure stability and reusability.
- **Generation Script (`scripts/generate_compliance_report.sh`):** 
    - **On-chain:** Uses `soroban contract events` to fetch raw contract activity.
    - **Off-chain:** Uses `psql` to export audit logs from the PostgreSQL `audit_logs` table.
    - **Rendering:** Combines data into a Markdown report based on a central template.
- **Template (`docs/compliance_report_template.md`):** A standardized format for all generated reports, ensuring consistency for auditors.

### 4. CI/CD & Automation
To ensure regular reporting without manual intervention:
- **GitHub Workflow (`.github/workflows/compliance-report.yml`):** 
    - Runs on a `cron` schedule (1st of every month).
    - Can be triggered manually via `workflow_dispatch`.
    - Requires secrets: `DATABASE_URL`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`.

## 🧪 Verification & Testing
- **Rust Unit Tests:** Added `contracts/shared/src/compliance_tests.rs` to verify role hierarchy and permission assignments.
- **Integration:** The API endpoint was verified for proper middleware mounting and error handling.
- **Manual Trigger:** Developers can run `make compliance-report` locally to test the generation logic.

## 📖 Further Reading
For usage instructions and configuration details, see [COMPLIANCE_REPORT.md](file:///c:/Users/OBASE%20CLINTON/.gemini/antigravity/scratch/StrellerMinds-SmartContracts-fork/docs/COMPLIANCE_REPORT.md).
