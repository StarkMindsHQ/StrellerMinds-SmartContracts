# Security Guidelines

## Security-First Mindset

All development for StrellerMinds smart contracts must follow these security principles:

1. **Principle of Least Privilege**: Contracts should request only the permissions they need.
2. **Input Validation**: All inputs must be validated before processing.
3. **Error Handling**: Proper error handling must be implemented for all operations.
4. **Access Control**: Clear access control mechanisms must be in place.
5. **Audit Readiness**: Code should be written with clarity for future audits.

## Security Review Process

1. All PRs must undergo security review before merging.
2. Static analysis tools must be run on all code.
3. Test coverage must include security-focused test cases.

## Vulnerability Reporting

If you discover a security vulnerability, please do NOT open an issue. Email security@strellerminds.com instead.
