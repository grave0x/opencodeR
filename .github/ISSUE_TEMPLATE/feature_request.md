name: Feature Request
description: Suggest a new feature or improvement for opencodeR
title: "[feat] "
labels: ["enhancement"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to suggest a feature!

  - type: dropdown
    id: area
    attributes:
      label: Area
      description: Which part of opencodeR does this affect?
      options:
        - Server API
        - CLI / Client
        - TUI
        - Packaging / CI
        - Documentation
        - Performance
        - Other
    validations:
      required: true

  - type: dropdown
    id: priority
    attributes:
      label: Priority
      options:
        - P0 - Critical (blocker)
        - P1 - High
        - P2 - Medium
        - P3 - Low
        - P4 - Nice to have
    validations:
      required: true

  - type: textarea
    id: problem
    attributes:
      label: Problem / Motivation
      description: What problem does this feature solve? What use case does it enable?
      placeholder: "I'm always frustrated when..."
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: Describe the feature you'd like, including API shape, CLI flags, or UI mockup
      placeholder: "A clear and concise description of what you want to happen"
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: What alternative solutions or workarounds have you considered?
    validations:
      required: false

  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Add any other context, screenshots, or references
    validations:
      required: false

  - type: checkboxes
    id: confirmations
    attributes:
      label: Confirmations
      options:
        - label: I have checked that this feature doesn't already exist
          required: true
        - label: I have checked the existing issues for similar requests
          required: true
