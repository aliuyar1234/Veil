# Contributing to Veil

Thank you for your interest in contributing to Veil! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## Getting Started

### Prerequisites

- Rust 1.75 or later (stable)
- Git
- (Optional) `just` command runner
- (Optional) `pre-commit` for git hooks

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/veil.git
   cd veil
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/your-org/veil.git
   ```

## Development Setup

### Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-audit

# Optional: Install additional tools
cargo install cargo-llvm-cov cargo-mutants
pip install pre-commit
cargo install just

# Set up pre-commit hooks (recommended)
pre-commit install
```

### Build the Project

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --workspace --release
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p veil-detect

# Run with verbose output
cargo test --workspace -- --nocapture
```

## Making Changes

### Branching Strategy

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes in small, focused commits

3. Keep your branch up to date:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

### Commit Messages

Follow conventional commits format:

```
type(scope): short description

Longer description if needed.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, semicolons, etc.)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes

Examples:
```
feat(detect): add IBAN validation for Swiss format
fix(redact): handle overlapping findings correctly
docs(api): add OpenAPI examples for batch endpoint
test(crypto): add property tests for encryption
```

## Testing

### Unit Tests

All new code should have unit tests:

```bash
cargo test --workspace
```

### Integration Tests

Add integration tests in `tests/` directories:

```bash
cargo test --test '*'
```

### Property-Based Tests

For validators and parsers, add `proptest` tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_validator(input in ".*") {
        let _ = validate(&input);
    }
}
```

### Coverage

Check test coverage:

```bash
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html
```

### Mutation Testing

Verify test quality:

```bash
cargo mutants -p veil-detect -- --lib
```

## Pull Request Process

### Before Submitting

1. Run the full test suite:
   ```bash
   cargo test --workspace --all-features
   ```

2. Run clippy:
   ```bash
   cargo clippy --workspace --all-features -- -D warnings
   ```

3. Format code:
   ```bash
   cargo fmt --all
   ```

4. Run security audit:
   ```bash
   cargo audit
   ```

### Submitting

1. Push your branch:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Open a Pull Request on GitHub

3. Fill out the PR template completely

4. Wait for CI checks to pass

5. Address review feedback

### PR Requirements

- [ ] Tests pass
- [ ] No clippy warnings
- [ ] Code is formatted
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG updated (for user-facing changes)
- [ ] No security audit warnings

## Code Style

### Rust Guidelines

- Follow standard Rust naming conventions
- Use `rustfmt` defaults (no custom configuration)
- Run `clippy` with `-D warnings`
- Document all public items
- Prefer `Result<T, E>` over `panic!`
- Use `#[must_use]` for functions that return values that shouldn't be ignored

### Documentation

- All public items must have doc comments
- Include examples in documentation where helpful
- Keep README and ARCHITECTURE.md up to date

### Error Handling

- Use custom error types via `thiserror`
- Provide meaningful error messages
- Don't expose internal details in user-facing errors

### Security

- Never log PII values
- Use `SensitiveString` for PII data
- Use constant-time comparisons for secrets
- Run `cargo audit` before releases

## Questions?

If you have questions, please:

1. Check existing issues and discussions
2. Search the documentation
3. Open a new issue with the "question" label

Thank you for contributing!
