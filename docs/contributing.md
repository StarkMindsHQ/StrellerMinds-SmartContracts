# Contribution Guidelines

## Before Contributing

1. Ensure you have an assigned issue before starting work.
2. Discuss major changes in the issue before implementing.
3. Use the appropriate issue templates for bug reports or feature requests.

## Issue Templates

When creating new issues, please use the provided templates to ensure consistent and complete information:

### Bug Reports
Use the **Bug Report** template (`.github/ISSUE_TEMPLATE/bug_report.md`) for:
- Contract functionality issues
- Build or deployment problems
- Security vulnerabilities
- Performance issues

The template includes sections for:
- Clear bug description with reproduction steps
- Environment details (Rust version, Soroban CLI, OS)
- Affected contracts/modules
- Security impact assessment

### Feature Requests
Use the **Feature Request** template (`.github/ISSUE_TEMPLATE/feature_request.md`) for:
- New contract functionality
- Educational platform enhancements
- Performance optimizations
- Developer experience improvements

The template includes sections for:
- Problem description and proposed solution
- Implementation details and affected contracts
- Technical considerations and complexity assessment
- Security and compliance implications

## Pull Request Process

1. Ensure your PR links to the related issue ID.
2. Update documentation as needed.
3. Add tests for new functionality.
4. Ensure all tests pass and CI checks are successful.
5. The CODEOWNERS file will automatically request review from @LaGodxy (maintainer).

## Code Ownership and Review Process

The repository uses a CODEOWNERS file (`.github/CODEOWNERS`) to automatically assign reviewers:

- **Maintainer**: @LaGodxy is the primary maintainer and will be automatically requested for review on all PRs
- **Contract-specific ownership**: Different contracts may have specific ownership requirements
- **Security-sensitive files**: Files related to authentication, authorization, and reentrancy protection require maintainer review

### Review Requirements

- All PRs require approval from the maintainer (@LaGodxy)
- Security-related changes require additional scrutiny
- Large architectural changes should be discussed in issues before implementation

## Code Standards

1. Follow Rust best practices and use `rustfmt` and `clippy`.
2. Write comprehensive tests for all functionality.
3. Document all public functions and modules.
4. Maintain a security-first mindset.

## Documentation Standards

### Contract Documentation

When creating or updating smart contracts, ensure you include comprehensive documentation:

1. **README.md**: Each contract directory must have a README.md following the [standard template](README_TEMPLATE.md)
2. **Function Documentation**: All public functions must have rustdoc comments
3. **Interface Documentation**: Document all public entrypoints with parameters and return types
4. **Event Documentation**: Document all emitted events and their schemas
5. **Usage Examples**: Include code examples for common operations

### README Template

Use the [README_TEMPLATE.md](README_TEMPLATE.md) as a guide for contract documentation. The template includes:

- **Overview**: Brief description of the contract's purpose
- **Interface**: List of public entrypoints with parameters
- **Events**: Description of emitted events and their schemas
- **Configuration**: Constants, settings, and environment variables
- **Testing**: How to run tests and test coverage information
- **Deployment**: Deployment notes and environment setup
- **Related Docs**: Links to other relevant documentation

### Documentation Checklist

Before submitting a PR with contract changes:

- [ ] README.md follows the standard template
- [ ] All public functions have rustdoc comments
- [ ] Events are documented with schemas
- [ ] Configuration parameters are explained
- [ ] Test instructions are provided
- [ ] Deployment steps are documented
- [ ] Usage examples are included
- [ ] Related documentation is linked

## Commit Messages

Follow the conventional commits format:
\`\`\`
feat(scope): add new feature
fix(scope): fix issue
docs(scope): update documentation
test(scope): add tests
chore(scope): maintenance tasks
