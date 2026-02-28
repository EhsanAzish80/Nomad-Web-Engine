# Contributing to Nomad Web Engine

Thank you for your interest in contributing! This guide will help you get started.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please treat all contributors with respect.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/nomad-web-engine.git
   cd nomad-web-engine
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (bundled with Rust)
- Git

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p nomad-core

# Run tests with output
cargo test -- --nocapture
```

### Code Formatting

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Run clippy on all targets
cargo clippy --all-targets --all-features -- -D warnings
```

## Contribution Guidelines

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for consistent formatting
- Address all `clippy` warnings
- Write idiomatic Rust code

### Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments where helpful
- Update README.md and ARCHITECTURE.md for significant changes
- Keep comments clear and concise

### Testing

- Write unit tests for new functionality
- Ensure tests pass before submitting PR
- Add integration tests for cross-crate features
- Maintain or improve test coverage

### Commits

- Write clear, descriptive commit messages
- Use conventional commit format:
  ```
  type(scope): description
  
  [optional body]
  ```
  
  Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
  
  Example:
  ```
  feat(html): add HTML5 tokenizer
  
  Implements the tokenization phase of HTML parsing
  according to the WHATWG HTML spec.
  ```

### Pull Requests

1. **Keep PRs focused**: One feature or fix per PR
2. **Update documentation**: Reflect changes in docs
3. **Add tests**: Cover new code paths
4. **Pass CI**: All checks must pass
5. **Describe changes**: Explain what and why in PR description

### PR Checklist

- [ ] Code builds without errors
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] Tests added for new functionality
- [ ] Commit messages are clear

## Areas for Contribution

### Priority Areas

- Core engine functionality
- HTML parser implementation
- CSS parser and styling
- Layout algorithms
- Test coverage improvements
- Performance optimizations
- Documentation enhancements

### Good First Issues

Look for issues labeled `good-first-issue` or `help-wanted` in the issue tracker.

## Review Process

1. A maintainer will review your PR
2. Address feedback through additional commits
3. Once approved, a maintainer will merge your PR
4. Your contribution will be in the next release!

## Communication

- **Issues**: For bug reports and feature requests
- **Discussions**: For questions and ideas
- **Pull Requests**: For code contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue or discussion if you have questions about contributing.

---

Thank you for making Nomad Web Engine better! 🚀
