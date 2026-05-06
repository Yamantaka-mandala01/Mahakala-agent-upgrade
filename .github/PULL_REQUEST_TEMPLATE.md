name: Pull Request
description: Submit a pull request
title: ""
labels: []
body:
  - type: markdown
    attributes:
      value: |
        ## ⚠️ 分支策略提醒

        **严禁直接向 `main` 分支提交代码！** 所有修改必须通过以下流程：

        1. 从 `main` 创建新的功能分支：`git checkout -b feature/your-feature-name`
        2. 在功能分支上完成开发和测试
        3. 提交 Pull Request 到 `main` 分支
        4. 等待代码审查和CI检查通过后合并

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
        - label: I have created this PR from a **separate branch** (not main)
          required: true
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
