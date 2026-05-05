# Contributing to Mahakala Agent

Thank you for your interest in contributing to Mahakala Agent! We welcome contributions from the community.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/Yamantaka-mandala01/mahakala-agent/issues)
2. If not, create a new issue with:
   - A clear title and description
   - Steps to reproduce the issue
   - Expected vs actual behavior
   - Your environment (OS, Rust version, etc.)

### Suggesting Features

1. Open an issue describing the feature
2. Explain why it would be useful
3. Provide examples of how it would work

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Development Guidelines

### Code Style

- Follow Rust conventions and idioms
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes
- Add comments for complex logic

### Testing

- Write unit tests for new functionality
- Ensure all tests pass: `cargo test`
- Add integration tests where appropriate

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add new tool for file searching
fix: resolve memory leak in conversation history
docs: update API documentation
refactor: simplify tool registry logic
```

## Project Structure

See [README.md](README.md) for the full project structure.

## Getting Help

If you have questions, feel free to open an issue or reach out to the maintainers.
