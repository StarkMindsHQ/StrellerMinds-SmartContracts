---
name: 👥 Community Question
description: Ask a question about the project, community, or development
title: "[QUESTION] <Brief description>"
labels: ["question", "community"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for your question! 💬 We're here to help.
        
        Before asking, please:
        1. Check the [documentation](../docs/)
        2. Search existing [issues](../issues) and [discussions](../discussions)
        3. Review the [FAQ](COMMUNITY.md#frequently-asked-questions)
        
  - type: dropdown
    id: category
    attributes:
      label: Question Category
      options:
        - Getting Started
        - Development Setup
        - Smart Contract Usage
        - Testing & Deployment
        - Architecture & Design
        - Contribution Process
        - Community & Events
        - Other (specify below)
    validations:
      required: true
      
  - type: textarea
    id: question
    attributes:
      label: Your Question
      description: Please provide as much detail as possible
      placeholder: What would you like to know?
    validations:
      required: true
      
  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Why are you asking this? What are you trying to accomplish?
      placeholder: Provide context about your question
      
  - type: input
    id: experience-level
    attributes:
      label: Experience Level
      description: How would you describe your blockchain/Rust experience?
      placeholder: e.g., Beginner, Intermediate, Advanced
      
  - type: checkboxes
    id: search
    attributes:
      label: Research
      description: Did you search before asking?
      options:
        - label: I searched existing issues and discussions
          required: true
        - label: I reviewed the documentation
          required: true
