# Contributing to Video Chat Extension

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Maintain a professional environment

## Development Setup

1. **Install Prerequisites**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install wasm-pack
   cargo install wasm-pack
   
   # Install development tools
   cargo install cargo-watch cargo-tarpaulin
   ```

2. **Clone and Build**
   ```bash
   git clone <repository-url>
   cd video_chat_extension
   cargo build --workspace
   ```

3. **Set up pre-commit hooks**
   ```bash
   cp .github/hooks/pre-commit .git/hooks/
   chmod +x .git/hooks/pre-commit
   ```

## Coding Standards

### Rust Style Guide

- **Formatting**: Use `rustfmt` with project configuration
  ```bash
  cargo fmt --all
  ```

- **Linting**: Pass `clippy` with no warnings
  ```bash
  cargo clippy --all-targets --all-features --workspace -- -D warnings
  ```

- **Documentation**: All public APIs must have doc comments
  ```rust
  /// Brief description of the function.
  ///
  /// # Arguments
  ///
  /// * `param` - Description of parameter
  ///
  /// # Returns
  ///
  /// Description of return value
  ///
  /// # Errors
  ///
  /// Description of possible errors
  pub fn example_function(param: Type) -> Result<ReturnType, Error> {
      // implementation
  }
  ```

- **Error Handling**: Use `thiserror` for custom errors, `anyhow` for application errors

- **No Unsafe Code**: Unsafe code is denied by clippy configuration

### Code Organization

- Keep functions small and focused (< 50 lines)
- Use descriptive variable names
- Avoid deep nesting (max 4 levels)
- Separate concerns into modules

## Testing Requirements

### Unit Tests

- Write unit tests for all public functions
- Place tests in the same file as the code
- Use `#[cfg(test)]` module

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // test implementation
    }
}
```

### Integration Tests

- Place integration tests in `tests/` directory
- Test component interactions
- Use mocks for external dependencies

### Coverage Requirements

- Maintain **>80% code coverage**
- Run coverage locally:
  ```bash
  cargo tarpaulin --out Html --workspace
  ```

### WASM Tests

- Use `wasm-bindgen-test` for WASM-specific tests
- Run with:
  ```bash
  wasm-pack test --headless --firefox
  ```

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Follow coding standards
   - Write tests
   - Update documentation

3. **Run quality checks**
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features --workspace -- -D warnings
   cargo test --workspace
   cargo tarpaulin --out Html --workspace
   ```

4. **Commit your changes**
   - Follow commit message guidelines
   - Keep commits atomic and focused

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   - Create PR on GitHub
   - Fill out PR template
   - Link related issues

6. **Code Review**
   - Address reviewer feedback
   - Keep PR updated with main branch
   - Ensure CI passes

7. **Merge**
   - Squash commits if needed
   - Delete feature branch after merge

## Commit Message Guidelines

Use **Conventional Commits** format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(signaling): Add WebSocket signaling support

Implement WebSocket-based signaling for initial peer handshake.
Uses secure WebSocket (wss://) for production environments.

Closes #123
```

```
fix(media): Resolve ICE candidate gathering timeout

Fixed issue where ICE candidates weren't being gathered properly
due to incorrect STUN server configuration.
```

## Feature-Based Development

This project follows a feature development approach:

- Each feature must be complete with tests before moving to the next
- Create feature branches from main: `feature-N-description`
- Each feature gets its own commit and tag
- See `implementation_plan.md` for feature details

## Questions?

- Open a GitHub issue for questions
- Tag maintainers for urgent matters
- Check existing issues and PRs first

---

Thank you for contributing!
