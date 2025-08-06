# Contributing to Caxton

Thank you for your interest in contributing to Caxton! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please read it before contributing.

## How to Contribute

### Reporting Issues

- Check if the issue has already been reported
- Use the issue templates when available
- Provide a clear description of the problem
- Include steps to reproduce the issue
- Mention your environment (OS, Rust version, etc.)

### Submitting Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the code style** - Run `cargo fmt` and `cargo clippy`
3. **Write tests** - Ensure your code has appropriate test coverage
4. **Update documentation** - Keep docs in sync with code changes
5. **Use conventional commits** - This is required for our release automation

### Conventional Commit Format

We use [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages. This enables automatic changelog generation and semantic versioning.

Format: `<type>(<scope>): <subject>`

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions or modifications
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Other changes that don't modify src or test files
- `revert`: Revert a previous commit

**Examples:**
```
feat(agent): add WebAssembly instance pooling
fix(fipa): correct message routing logic
docs(api): update agent lifecycle documentation
perf(runtime): optimize memory allocation
```

### Development Setup

1. **Install Rust** (1.70.0 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**
   ```bash
   git clone https://github.com/jwilger/caxton.git
   cd caxton
   ```

3. **Install development tools**
   ```bash
   cargo install cargo-nextest
   cargo install cargo-watch
   cargo install mdbook
   ```

4. **Run tests**
   ```bash
   cargo nextest run
   cargo test --doc
   ```

5. **Run benchmarks**
   ```bash
   cargo bench
   ```

### Testing Guidelines

- Write property-based tests for core domain logic
- Use integration tests for system behavior
- Aim for 80% code coverage on core modules
- Test error paths and edge cases
- Use `testcontainers` for external dependencies

### Documentation

- Update rustdoc comments for public APIs
- Keep architectural decisions in `_adrs/` directory
- Update user guides in `docs/` when adding features
- Include examples in documentation

## Release Process

Releases are automated using [release-plz](https://release-plz.ieni.dev/):

1. **Automatic PR Creation**: When commits land on `main`, release-plz creates/updates a PR with:
   - Version bumps based on conventional commits
   - Updated CHANGELOG.md
   - Updated Cargo.toml versions

2. **Review and Merge**: Maintainers review and merge the release PR

3. **Automatic Release**: Upon merge, the system automatically:
   - Creates git tags
   - Publishes to crates.io
   - Creates GitHub releases with binaries
   - Updates documentation

### Manual Release (Maintainers Only)

If needed, maintainers can trigger a release manually:
```bash
cargo install release-plz
release-plz release
```

## Architecture Guidelines

- Follow type-driven development principles
- Make illegal states unrepresentable
- Use the type system for compile-time guarantees
- Implement the functional core, imperative shell pattern
- Ensure all code is observable with structured logging

## Getting Help

- Check the [documentation](https://jwilger.github.io/caxton/)
- Ask questions in GitHub Discussions
- Join our community chat (if available)
- Review existing issues and PRs

## Recognition

Contributors will be recognized in:
- The project README
- Release notes
- The contributors page in documentation

Thank you for contributing to Caxton!
