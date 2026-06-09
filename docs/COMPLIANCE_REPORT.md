# Compliance Report Automation

## Overview
This system automates the generation of audit-ready compliance reports for the StrellerMinds platform. It aggregates on-chain events from smart contracts and off-chain access logs from the API server.

## Components

### 1. Smart Contract Role: `ComplianceAdmin`
A new privileged role `ComplianceAdmin` has been added to the RBAC system. This role has the `GenerateComplianceReport` permission.
- **Permissions:** `GenerateComplianceReport`, `ViewAudit`, `ViewSystemStats`, `ViewAllUsers`, `ViewAllCourses`, `ViewAllCertificates`.
- **Granting:** The `Admin` role can grant the `ComplianceAdmin` role.

### 2. API Endpoint: `POST /api/v1/admin/compliance-report`
A protected endpoint that allows an authorized compliance administrator to trigger report generation manually.
- **Authentication:** Bearer JWT required.
- **Scope:** Must have the `compliance` scope in the JWT.

### 3. Generation Script: `scripts/generate_compliance_report.sh`
The core logic for data aggregation and report formatting.
- **On-chain:** Uses `soroban contract events` to pull contract activity.
- **Off-chain:** Pulls `audit_logs` from the PostgreSQL database.
- **Formatting:** Uses Markdown templates and `pandoc` to generate HTML/PDF reports.
- **Storage:** Uploads generated artifacts to the configured S3 bucket.

### 4. Scheduled Workflow: `.github/workflows/compliance-report.yml`
A GitHub Action that runs automatically on the first day of every month at midnight UTC.

## Configuration

The following environment variables/secrets are required for the automated job:
- `DATABASE_URL`: Connection string for the production PostgreSQL database.
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`: Credentials for S3 upload.
- `AWS_S3_BUCKET`: The destination bucket (default: `streminderminds-compliance-reports`).

## Manual Execution
To generate a report manually from the command line:
```bash
make compliance-report YEAR=2024 MONTH=05
```

The reports will be saved in `target/compliance/YEAR/MONTH/`.
