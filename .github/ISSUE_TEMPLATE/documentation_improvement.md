---
name: 📚 Documentation Improvement
description: Suggest improvements or additions to documentation
title: "[DOCS] <Brief description>"
labels: ["documentation", "enhancement"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for helping improve our documentation! 📖
        
  - type: textarea
    id: current-state
    attributes:
      label: Current State
      description: What documentation exists and what's missing/inaccurate?
      placeholder: Describe the current documentation state
    validations:
      required: true
      
  - type: textarea
    id: improvement
    attributes:
      label: Proposed Improvement
      description: What would you like to see added or changed?
      placeholder: Describe your proposed documentation improvements
    validations:
      required: true
      
  - type: dropdown
    id: doc-type
    attributes:
      label: Documentation Type
      options:
        - README update
        - API documentation
        - Tutorial/Guide
        - Code examples
        - Troubleshooting guide
        - Architecture documentation
        - Other (specify below)
    validations:
      required: true
      
  - type: textarea
    id: additional-context
    attributes:
      label: Additional Context
      description: Any other relevant information
      placeholder: Add any other context about the documentation improvement
      
  - type: checkboxes
    id: terms
    attributes:
      label: Code of Conduct
      description: By submitting this issue, you agree to follow our [Code of Conduct](../CODE_OF_CONDUCT.md)
      options:
        - label: I have read and agree to the Code of Conduct
          required: true
