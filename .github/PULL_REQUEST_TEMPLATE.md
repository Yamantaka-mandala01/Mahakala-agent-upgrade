name: Pull Request
description: Submit a pull request
title: ""
labels: []
body:
  - type: markdown
    attributes:
      value: |
        Thanks for submitting a pull request! Please fill out the information below.
  - type: textarea
    id: description
    attributes:
      label: Description
      description: Describe the changes in this pull request.
      placeholder: This PR adds/fixes...
    validations:
      required: true
  - type: textarea
    id: related
    attributes:
      label: Related Issue
      description: Link to the related issue(s) if applicable.
      placeholder: Fixes #123
  - type: checkboxes
    id: checklist
    attributes:
      label: Checklist
      description: Please ensure all items are checked before submitting.
      options:
        - label: I have run `cargo fmt` to format the code
          required: true
        - label: I have run `cargo clippy` and fixed any warnings
          required: true
        - label: I have run `cargo test` and all tests pass
          required: true
        - label: I have added tests for new functionality (if applicable)
          required: false
        - label: I have updated documentation (if applicable)
          required: false
  - type: textarea
    id: testing
    attributes:
      label: How Has This Been Tested?
      description: Describe the tests you ran and how to reproduce them.
  - type: textarea
    id: additional
    attributes:
      label: Additional Notes
      description: Any additional information or context.
