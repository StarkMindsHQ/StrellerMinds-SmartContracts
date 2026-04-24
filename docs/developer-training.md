# StrellerMinds Developer Training Guide

## Overview

This training guide helps developers get started with the StrellerMinds Smart Contracts project and effectively use the repo workflows to build, test, debug, and ship Soroban contracts safely.

## Training Tracks

### Track A: Contract Contributor (Onboarding)
- Goal: build and test the full workspace, understand contract boundaries, land safe PRs
- Target outcome: can make small changes with tests + clippy, follows repo conventions

### Track B: Contract Maintainer
- Goal: review PRs, enforce security and code quality, own incident/debug workflows
- Target outcome: can debug failures end-to-end, understands authorization boundaries and invariants

### Track C: Release Engineer
- Goal: produce artifacts, run release checks, monitor pipeline and post-release health
- Target outcome: can run the release workflow and interpret audits and metrics

## Training Materials Library

- Development workflow: [development.md](development.md)
- Architecture overview: [ARCHITECTURE.md](ARCHITECTURE.md)
- Public API overview: [API.md](API.md)
- Code style and conventions: [CODE_STYLE.md](CODE_STYLE.md)
- Security baseline: [security.md](security.md), [SECURITY_TESTING.md](SECURITY_TESTING.md), [SECURITY_AUDIT_REPORT.md](SECURITY_AUDIT_REPORT.md)
- Release process: [RELEASE_MANAGEMENT.md](RELEASE_MANAGEMENT.md), [RELEASE_PROCESS.md](RELEASE_PROCESS.md)
- Upgrade model: [UPGRADE_FRAMEWORK.md](UPGRADE_FRAMEWORK.md)
- Tooling and CLI: [developer-tools.md](developer-tools.md)
- Effectiveness measurement: [tool-effectiveness-review.md](tool-effectiveness-review.md)

## Training Modules

### Module 1: Environment Setup

#### Learning Objectives
- Set up development environment
- Install required tools
- Verify installation

#### Prerequisites
- Basic knowledge of command-line interfaces
- Understanding of Git version control
- Familiarity with Rust programming language

#### Step-by-Step Instructions

1. **Clone Repository**
   ```bash
   git clone <repository-url>
   cd StrellerMinds-SmartContracts
   ```

2. **Install prerequisites**
   - Follow the repo development guide: [development.md](development.md)
   - If you are on Windows, use WSL for the shell scripts and Docker workflow

3. **Verify Setup**
   ```bash
   make check
   ```

#### Hands-on Exercise
- Run the environment check
- Fix any missing dependencies
- Start the local development network
  ```bash
  make localnet-start
  make localnet-status
  make localnet-stop
  ```

#### Assessment
- ✅ All prerequisite tools installed
- ✅ Environment check passes
- ✅ Localnet starts successfully

---

### Module 2: Using the Developer CLI

#### Learning Objectives
- Navigate the CLI interface
- Execute common development tasks
- Understand menu options

#### CLI Navigation

1. **Start the CLI**
   ```bash
   cd utils/streller-cli
   cargo run
   ```

2. **Main Menu Navigation**
   - Use arrow keys to navigate
   - Press Enter to select
   - Explore different menu options

3. **Common Tasks**
   - Check system prerequisites
   - Start/stop local network
   - Build contracts
   - Run tests

#### Hands-on Exercise
- Navigate through all menu options
- Execute system check
- Start localnet
- Build contracts
- Run tests

#### Assessment
- ✅ Can navigate CLI menus
- ✅ Can execute basic tasks
- ✅ Understand menu options

---

### Module 3: Debugging Tools

#### Learning Objectives
- Use debugging utilities
- Analyze contract structure
- Diagnose network issues
- Analyze logs and performance

#### Debugging Tools Overview

1. **Contract Analysis**
   - Analyzes smart contract structure
   - Checks dependencies and source files
   - Provides contract statistics

2. **Network Diagnostics**
   - Checks localnet status
   - Validates Docker containers
   - Tests network connectivity

3. **Log Analysis**
   - Scans for log files
   - Analyzes errors and warnings
   - Provides log statistics

4. **Performance Metrics**
   - Collects system metrics
   - Analyzes build performance
   - Tracks artifact sizes

#### Hands-on Exercise

1. **Contract Analysis**
   - Navigate to Debug Tools → Contract Analysis
   - Review the analysis results
   - Identify contract structure

2. **Network Diagnostics**
   - Navigate to Debug Tools → Network Diagnostics
   - Check localnet status
   - Verify connectivity

3. **Log Analysis**
   - Navigate to Debug Tools → Log Analysis
   - Analyze available logs
   - Check for errors

#### Assessment
- ✅ Can use all debugging tools
- ✅ Can interpret debugging results
- ✅ Can identify common issues

---

### Module 4: Testing Utilities

#### Learning Objectives
- Run different types of tests
- Analyze test coverage
- Use performance testing
- Configure test environment

#### Testing Tools Overview

1. **Unit Tests**
   - Test individual contract modules
   - Run with different configurations
   - Analyze test results

2. **Integration Tests**
   - Test contract interactions
   - Validate network operations
   - Run E2E scenarios

3. **Coverage Analysis**
   - Generate coverage reports
   - Analyze test coverage
   - Identify untested code

4. **Performance Testing**
   - Measure build performance
   - Analyze execution speed
   - Monitor resource usage

#### Hands-on Exercise

1. **Run Unit Tests**
   - Navigate to Test Utils → Unit Tests Only
   - Run all unit tests
   - Try specific contract tests

2. **Integration Testing**
   - Navigate to Test Utils → Integration Tests
   - Run quick smoke tests
   - Check network connectivity

3. **Coverage Analysis**
   - Navigate to Test Utils → Test Coverage
   - Generate coverage report
   - Review coverage statistics

#### Assessment
- ✅ Can run all test types
- ✅ Can interpret test results
- ✅ Can generate coverage reports

---

### Module 5: CI/CD Pipeline

#### Learning Objectives
- Understand CI/CD workflow
- Run pipeline locally
- Interpret pipeline results
- Troubleshoot pipeline issues

#### Pipeline Overview

1. **CI Workflow Stages**
   - Format Check
   - Clippy Lint
   - Build Check
   - Test Suite

2. **Pipeline Commands**
   - `make ci-test` - Fast CI-style checks (build + unit tests + quick E2E)
   - `make check-code` - Formatting + strict linting
   - `make dev-test` - Clean, build, and full E2E

3. **Pipeline Monitoring**
   - Check GitHub Actions status
   - Analyze build logs
   - Troubleshoot failures

#### Hands-on Exercise

1. **Run Pipeline Locally**
   ```bash
   make ci-test
   ```

2. **Check Code Quality**
   ```bash
   make check-code
   ```

3. **Analyze Results**
   - Review build output
   - Check for warnings/errors
   - Fix any issues found

#### Assessment
- ✅ Understands pipeline stages
- ✅ Can run pipeline locally
- ✅ Can troubleshoot issues

---

### Module 6: Development Workflow

#### Learning Objectives
- Follow best practices
- Use tools effectively
- Maintain code quality
- Collaborate with team

#### Development Workflow

1. **Daily Development**
   ```bash
   make build
   make unit-test
   ```

2. **Before Commit**
   ```bash
   make check-code
   make unit-test
   ```

3. **Before Deployment**
   ```bash
   make ci-test
   make e2e-test
   ```

#### Best Practices

1. **Code Quality**
   - Run formatting checks
   - Address clippy warnings
   - Maintain test coverage

2. **Testing**
   - Write unit tests for new features
   - Run integration tests regularly
   - Monitor test coverage

3. **Debugging**
   - Use debugging tools early
   - Analyze logs for errors
   - Monitor performance metrics

#### Hands-on Exercise

1. **Complete Development Cycle**
   - Make a small code change
   - Run tests
   - Check code quality
   - Fix any issues

2. **Team Collaboration**
   - Create a feature branch
   - Make changes
   - Run tests
   - Simulate pull request

#### Assessment
- ✅ Follows development workflow
- ✅ Maintains code quality
- ✅ Uses tools effectively

---

## Advanced Topics

### Module 7: Advanced Debugging

#### Advanced Debugging Techniques
- Memory leak detection
- Performance profiling
- Network packet analysis
- Contract state inspection

### Module 8: Custom Tool Development

#### Extending the CLI
- Adding new menu options
- Creating custom debugging tools
- Developing new testing utilities

### Module 9: Performance Optimization

#### Optimization Strategies
- Build time optimization
- Test execution optimization
- Resource usage optimization
- CI/CD pipeline optimization

## Resources

### Documentation
- [Developer Tools](developer-tools.md)
- [Development Guide](development.md)
- [Architecture](ARCHITECTURE.md)
- [API](API.md)
- [Security](security.md)
- [Release Management](RELEASE_MANAGEMENT.md)

### Community
- Discord Channel
- GitHub Discussions
- Stack Overflow Tag

### Support
- Issue Tracker
- Office Hours
- Code Reviews

## Video Tutorials

These are recording-ready outlines with demo steps that map directly to repo commands and docs. Host the videos outside the repository and link them from this section once published.

### Video 1: Repo Tour and Contract Layout
- Audience: new contributors
- Outcome: can find a contract crate, its entrypoints, storage, events, and tests
- Demo path: open `contracts/<name>/src/lib.rs`, `storage.rs`, `events.rs`, `types.rs`, then run a unit test for that crate

### Video 2: Build and Test Loop (Fast Feedback)
- Audience: contributors
- Outcome: can iterate locally with a reliable, repeatable loop
- Demo commands:
  ```bash
  make build
  make unit-test
  make check-code
  ```

### Video 3: Localnet + Quick E2E (Smoke Testing)
- Audience: contributors and maintainers
- Outcome: can use localnet lifecycle and run quick end-to-end checks
- Demo commands:
  ```bash
  make localnet-start
  make e2e-test-quick
  make localnet-logs
  make localnet-stop
  ```

### Video 4: Reading Events, Errors, and Storage Patterns
- Audience: contributors
- Outcome: can trace an entrypoint to storage mutations and emitted events
- Demo path: pick one contract (e.g., `contracts/certificate/`), follow storage helpers, then run its tests and inspect snapshots

### Video 5: Security in Practice (RBAC + Reentrancy)
- Audience: maintainers
- Outcome: can review authorization boundaries and threat mitigations
- Reading set: [RBAC_IMPLEMENTATION.md](RBAC_IMPLEMENTATION.md), [REENTRANCY_PROTECTION.md](REENTRANCY_PROTECTION.md), [SECURITY_TESTING.md](SECURITY_TESTING.md)

### Video 6: Release Workflow Walkthrough
- Audience: release engineers
- Outcome: can run pre-release validation and interpret the outputs
- Reading set: [RELEASE_MANAGEMENT.md](RELEASE_MANAGEMENT.md), [RELEASE_PROCESS.md](RELEASE_PROCESS.md)

## Workshops

Workshops are designed to be facilitated, hands-on, and repeatable. Each workshop includes a short prep checklist, exercises, and a wrap-up review.

### Workshop 1 (90–120 min): Build, Test, and Navigate a Contract
- Prep checklist: Rust + Soroban CLI installed, Docker running, `make check` passes
- Exercises:
  - Run `make build` and explain where artifacts land
  - Run `make unit-test` and identify at least one contract test module
  - Pick one contract and map: entrypoints → storage → events → errors
- Wrap-up: 10-minute recap of repo structure and what “done” means for a PR (tests + lint)

### Workshop 2 (120 min): Localnet + E2E Debugging Loop
- Prep checklist: ports 8000/6379 free, Docker healthy
- Exercises:
  - Start localnet and confirm status/logs
  - Run `make e2e-test-quick`, then re-run with logs open to correlate failures
  - Identify one E2E scenario and explain the contract interactions at a high level
- Wrap-up: document one “common failure mode” and its fix in the team knowledge base

### Workshop 3 (120–180 min): Security Review Lab
- Prep checklist: read [security.md](security.md) and [SECURITY_TESTING.md](SECURITY_TESTING.md)
- Exercises:
  - For a selected contract, list all privileged entrypoints and the authorization model
  - Identify the state that must never be corrupted and the invariants that protect it
  - Run clippy in strict mode and explain one lint finding (real or simulated) and the fix
- Wrap-up: produce a short review note following the project’s security checklist style

## Training Delivery

### Session Runbook
- Before: pick a track, assign a facilitator, and select modules/workshops to deliver
- During: keep a shared notes document for questions and follow-ups
- After: capture action items (doc gaps, tooling friction, recurring failures) and file issues

### Team Training Plan Template
- Week 1: Modules 1–2 + Workshop 1
- Week 2: Modules 3–4 + Workshop 2
- Week 3: Modules 5–6 + Workshop 3 (maintainers/release engineers)

### Skills Matrix Template
- Fundamentals: build, unit tests, repo layout, basic Soroban concepts
- Quality: formatting, clippy, snapshots/tests, CI expectations
- Operations: localnet lifecycle, E2E debugging, releases (as applicable)

## Training Effectiveness

Use the measurement framework in [tool-effectiveness-review.md](tool-effectiveness-review.md) to evaluate training outcomes. Track onboarding time, PR cycle time, incident/debug resolution time, and survey-based confidence before and after training.

## Certification

### Developer Certification Levels

1. **Junior Developer**
   - Complete Modules 1-3
   - Pass basic assessment
   - Demonstrate tool usage

2. **Intermediate Developer**
   - Complete Modules 1-5
   - Pass intermediate assessment
   - Show workflow understanding

3. **Senior Developer**
   - Complete all modules
   - Pass advanced assessment
   - Demonstrate best practices

### Assessment Process

1. **Practical Exercises**
   - Complete hands-on exercises
   - Submit code solutions
   - Demonstrate tool usage

2. **Knowledge Assessment**
   - Complete written assessment
   - Answer conceptual questions
   - Explain best practices

3. **Code Review**
   - Submit code for review
   - Address feedback
   - Demonstrate quality standards

## Continuous Learning

### Staying Updated

1. **Regular Updates**
   - Check tool updates
   - Review documentation changes
   - Attend training sessions

2. **Community Involvement**
   - Participate in discussions
   - Share best practices
   - Contribute to improvements

3. **Skill Development**
   - Learn new debugging techniques
   - Explore advanced testing methods
   - Stay current with tools

### Feedback and Improvement

1. **Training Feedback**
   - Provide training feedback
   - Suggest improvements
   - Share experiences

2. **Tool Enhancement**
   - Report tool issues
   - Suggest new features
   - Contribute to development

## Conclusion

This training guide provides comprehensive coverage of the StrellerMinds development tools and workflows. By completing these modules, developers will be equipped to work efficiently with the smart contract project and contribute effectively to the team.

For additional support or questions, refer to the project documentation or contact the development team.
