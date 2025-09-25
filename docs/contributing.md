# Contribution Guidelines

## Before Contributing

1. Ensure you have an assigned issue before starting work.
2. Discuss major changes in the issue before implementing.

## Pull Request Process

1. Ensure your PR links to the related issue ID.
2. Update documentation as needed.
3. Add tests for new functionality.
4. Ensure all tests pass and CI checks are successful.
5. Request review from at least one maintainer.

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
