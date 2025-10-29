# Contributing to Data Designer

Thank you for your interest in contributing to Data Designer! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/data-designer.git
   cd data-designer
   ```
3. **Add the upstream repository**:
   ```bash
   git remote add upstream https://github.com/adamtc007/data-designer.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- PostgreSQL 17 with pgvector extension
- Basic understanding of Rust and WebAssembly

### Initial Setup

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install PostgreSQL and pgvector**:
   ```bash
   # macOS
   brew install postgresql@17
   
   # Install pgvector
   cd /tmp
   git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
   cd pgvector
   PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make
   PG_CONFIG=/opt/homebrew/opt/postgresql@17/bin/pg_config make install
   ```

3. **Create the database**:
   ```bash
   createdb data_designer
   export DATABASE_URL="postgres://$(whoami)@localhost/data_designer"
   ```

4. **Build the project**:
   ```bash
   cargo build
   ```

5. **Run tests**:
   ```bash
   cargo test --all
   ```

### Running the Application

```bash
# Desktop application
./rundesk.sh

# Web application (WASM)
./runwasm.sh
```

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples** (code snippets, screenshots, etc.)
- **Describe the behavior you observed** and what you expected
- **Include system information** (OS, Rust version, PostgreSQL version)
- **Include relevant logs or error messages**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description** of the suggested enhancement
- **Explain why this enhancement would be useful**
- **Include examples** of how the feature would be used
- **List any alternatives** you've considered

### Your First Code Contribution

Unsure where to begin? Look for issues labeled:

- `good first issue` - Simple issues perfect for newcomers
- `help wanted` - Issues where we'd appreciate community help
- `documentation` - Documentation improvements

### Pull Requests

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards

3. **Write or update tests** for your changes

4. **Run the test suite**:
   ```bash
   cargo test --all
   cargo clippy -- -D warnings
   ```

5. **Commit your changes**:
   ```bash
   git commit -m "Add feature: brief description"
   ```

6. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Open a Pull Request** on GitHub

## Coding Standards

### Rust Code Style

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `rustfmt` for code formatting:
  ```bash
  cargo fmt
  ```
- Use `clippy` for linting:
  ```bash
  cargo clippy -- -D warnings
  ```

### Code Quality

- **Write clear, self-documenting code** with meaningful variable names
- **Add comments** for complex logic or non-obvious decisions
- **Keep functions focused** on a single responsibility
- **Avoid unsafe code** unless absolutely necessary (and document why)
- **Handle errors properly** using `Result` and `anyhow::Context`
- **Write comprehensive tests** for new functionality

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Example:
```
Add support for custom DSL functions

- Implement function registry
- Add parser support for user-defined functions
- Update documentation

Closes #123
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific package
cargo test -p data-designer-core

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

### Writing Tests

- Write unit tests in the same file as the code (in a `tests` module)
- Write integration tests in the `tests/` directory
- Use descriptive test names that explain what is being tested
- Test both success and failure cases
- Mock external dependencies when appropriate

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let result = parse_expression("2 + 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Expression::Add(...));
    }
}
```

## Pull Request Process

1. **Update documentation** to reflect any changes to the API or behavior
2. **Update the README.md** if you add or change functionality
3. **Add your changes** to the changelog (if applicable)
4. **Ensure all tests pass** and clippy has no warnings
5. **Request review** from maintainers
6. **Address review feedback** promptly
7. **Squash commits** if requested before merging

### Review Process

- Pull requests require at least one approval from a maintainer
- CI checks must pass (build, tests, clippy)
- Code coverage should not decrease significantly
- All conversations must be resolved before merging

## Community

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and discussions
- **Pull Requests** - Code review and contributions

### Getting Help

If you need help:

1. Check the [README.md](README.md) and [CLAUDE.md](CLAUDE.md) documentation
2. Search existing GitHub issues
3. Ask in GitHub Discussions
4. Open a new issue with the "question" label

## Recognition

Contributors will be recognized in:

- The project's README (Contributors section)
- Release notes for significant contributions
- GitHub's contributor graphs

## License

By contributing to Data Designer, you agree that your contributions will be licensed under the MIT License.

## Thank You!

Your contributions make this project better for everyone. We appreciate your time and effort! 🎉
