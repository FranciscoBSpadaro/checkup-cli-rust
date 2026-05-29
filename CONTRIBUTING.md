# Contributing to checkup

Thank you for your interest in contributing! 🎉

This document outlines how to submit issues, propose features, and open pull requests.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [Release Process](#release-process)

---

## Code of Conduct

- Be respectful and constructive.
- Welcome newcomers.
- Focus on what's best for the project and the community.

---

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/<your-username>/checkup.git
   cd checkup
   ```
3. **Create a branch** for your work:
   ```bash
   git checkout -b feature/my-new-check
   ```

---

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:

- A clear, descriptive title.
- Steps to reproduce the behavior.
- Expected vs. actual behavior.
- Your environment (OS, Rust version, etc.).

### Suggesting Features

Feature requests are welcome! Open an issue and include:

- A description of the problem you're solving.
- Your proposed solution (or ideas).
- Any alternatives you've considered.

### Pull Requests

1. Ensure your branch is up to date with `main`:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Run all checks locally before submitting:
   ```bash
   cargo check
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
3. Make sure CI passes on your PR.
4. Reference any related issues in your PR description.
5. Keep PRs focused — one feature or fix per PR.

---

## Development Setup

### Prerequisites

| Requirement | Version    |
| ----------- | ---------- |
| Rust        | 1.75+      |
| Cargo       | (bundled)  |
| Git         | 2.x+       |

### Build from Source

```bash
git clone https://github.com/FranciscoBSpadaro/checkup.git
cd checkup
cargo build --release
```

The binary will be at `target/release/checkup`.

---

## Coding Standards

- **Formatting:** Run `cargo fmt` before every commit.
- **Linting:** Zero `clippy` warnings. Use `cargo clippy -- -D warnings`.
- **Documentation:** Add doc comments (`///`) to all public items.
- **Naming:** Follow Rust conventions (`snake_case` for functions/modules, `PascalCase` for types).
- **Error handling:** Use `anyhow` for application errors, `thiserror` for library errors.

### Check Trait Implementation

When adding a new check, implement the `Check` trait:

```rust
#[async_trait]
impl Check for MyCustomCheck {
    fn name(&self) -> &str { "My Check" }

    async fn check(&self) -> CheckResult {
        // Your logic here
    }
}
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Writing Tests

- Unit tests live in the same file as the code they test (in a `#[cfg(test)]` mod).
- Integration tests go in `tests/`.
- Every new check must include tests for both **pass** and **fail** scenarios.
- Run `cargo test && cargo clippy` before submitting any PR — CI will reject otherwise.

---

## CI/CD Pipeline

The project uses **GitHub Actions** (see `.github/workflows/ci.yml`). Every push and PR triggers:

| Stage       | Command                         |
| ----------- | --------------------------------|
| Check       | `cargo check`                   |
| Format      | `cargo fmt -- --check`          |
| Lint        | `cargo clippy -- -D warnings`   |
| Test        | `cargo test`                    |
| Build       | `cargo build --release`         |

All stages must pass before a PR can be merged.

---

## Releases

Releases are managed via GitHub. To publish a new version:

1. Update `Cargo.toml` version (e.g., `0.1.2`).
2. Run full verification: `make verify`.
3. Commit the bump: `git commit -am "chore: release vX.Y.Z"`.
4. Create a signed git tag: `git tag -s vX.Y.Z -m "Release vX.Y.Z"`.
5. Push: `git push && git push --tags`.
6. The GitHub Actions CI/CD workflow will build, test, and create a GitHub release.

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
