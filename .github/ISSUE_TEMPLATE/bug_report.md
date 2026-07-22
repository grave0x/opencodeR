name: Bug Report
description: Report a bug or unexpected behavior in opencodeR
title: "[bug] "
labels: ["bug"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for reporting! Please fill out as much detail as possible.

  - type: dropdown
    id: area
    attributes:
      label: Area
      options:
        - Server API
        - CLI / Client
        - TUI
        - Build / Compilation
        - PTY / Terminal
        - SSE / Events
        - Crash / Panic
        - Other
    validations:
      required: true

  - type: dropdown
    id: severity
    attributes:
      label: Severity
      options:
        - Critical (data loss, crash on startup)
        - High (feature broken, no workaround)
        - Medium (feature broken, has workaround)
        - Low (cosmetic, minor)
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: Description
      description: What happened? What did you expect to happen?
      placeholder: "A clear and concise description of the bug"
    validations:
      required: true

  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: Minimal reproduction steps
      placeholder: |
        1. Start server with `opencodeR-server --headless`
        2. Send request `curl ...`
        3. See error
    validations:
      required: true

  - type: input
    id: version
    attributes:
      label: Version
      description: Output of `opencodeR --version`
    validations:
      required: true

  - type: input
    id: os
    attributes:
      label: OS / Environment
      placeholder: "e.g. Linux x86_64, Arch Linux, kernel 6.5"
    validations:
      required: false

  - type: textarea
    id: logs
    attributes:
      label: Logs / Backtrace
      description: Include any relevant logs, error messages, or RUST_BACKTRACE output
      render: shell
    validations:
      required: false
