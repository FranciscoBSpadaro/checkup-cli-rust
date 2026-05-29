# 🏥 checkup

> Environment diagnostics and auto-fix for development machines.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/FranciscoBSpadaro/checkup/actions/workflows/ci.yml/badge.svg)](https://github.com/FranciscoBSpadaro/checkup/actions/workflows/ci.yml)

Stop wasting hours debugging your dev machine. One command tells you **exactly**
what's wrong — and how to fix it.

**🇧🇷 [Versão em Português](README-PT.md)**

```bash
$ checkup
```

```
┌──────────────────────────────────────────────────────┐
│  checkup v0.1.1 — Environment Diagnostics            │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ✓ Node.js          v20.10.0 (>= 20)                 │
│  ✓ PostgreSQL       running on :5432                 │
│  ✗ Docker           not running                      │
│    └─ Fix: systemctl start docker                    │
│  ✗ Port 3000        already in use                   │
│    └─ Fix: kill $(lsof -t -i:3000)                   │
│  ✗ .env             missing: DATABASE_URL, JWT_SECRET│
│                                                      │
│  Summary: 3 issues found, 2 passed                   │
│  Run 'checkup fix' to auto-resolve what's possible   │
└──────────────────────────────────────────────────────┘
```

---

## Table of Contents

- [Why?](#why)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Checks](#checks)
- [Auto-Fix](#auto-fix)
- [Roadmap](#roadmap)
- [CI/CD](#cicd)
- [Contributing](#contributing)
- [License](#license)

---

## Why?

Every developer has been there:

```bash
$ npm install
Error: node-gyp build failed
# ... 30 minutes later ...
# Oh, wrong Node version.
# ... 20 minutes later ...
# Oh, Docker isn't running.
# ... 15 minutes later ...
# Oh, DATABASE_URL isn't set.
```

**Three unrelated problems. 65 minutes wasted.**

`checkup` diagnoses your entire dev environment in **seconds**. Define what your
project needs, run one command, see everything that's wrong — with the fix
right next to it.

---

## Installation

### From source

```bash
git clone https://github.com/FranciscoBSpadaro/checkup-cli-rust
cd checkup
cargo install --path .
```

### Quick install script

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/FranciscoBSpadaro/checkup-cli-rust/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/FranciscoBSpadaro/checkup-cli-rust/main/scripts/install.ps1" -OutFile install.ps1; .\install.ps1
```

### Platform Support

| Platform | Status       |
| -------- | ------------ |
| Linux    | ✅ Supported |
| macOS    | ✅ Supported |
| Windows  | 🚧 Planned   |

---

## Quick Start

**Step 1:** Initialize config in your project root:

```bash
checkup init
```

This creates a `checkup.toml` file with sensible defaults.

**Step 2:** Edit `checkup.toml` for your project:

```toml
[commands.node]
command = "node"
fix = "fnm install 20"

[versions.node]
version_flag = "--version"
expected = ">=20"
fix = "fnm install 20"

[services.postgres]
port = 5432

[ports]
free = [3000, 8080]

[env]
required = ["DATABASE_URL", "JWT_SECRET"]

[envfile]
path = ".env"
required = ["DATABASE_URL", "JWT_SECRET"]
```

**Step 3:** Run diagnostics:

```bash
checkup
```

**Step 4:** Auto-fix what's possible:

```bash
checkup fix
```

---

## Configuration

The `checkup.toml` file defines what to check:

```toml
# Commands that must exist in PATH
[commands.node]
command = "node"
fix = "fnm install 20"

# Version checks (semver constraints: >=, >, ==, <, <=)
[versions.node]
version_flag = "--version"
expected = ">=20"
fix = "fnm install 20"

# Services (TCP port checks)
[services.postgres]
port = 5432

[services.redis]
port = 6379

# Ports that must be free
[ports]
free = [3000, 8080]

# Required environment variables
[env]
required = ["DATABASE_URL", "JWT_SECRET", "REDIS_URL"]

# .env file validation
[envfile]
path = ".env"
required = ["DATABASE_URL", "JWT_SECRET"]
```

### Command Line Options

```
checkup                    Run all checks (default)
checkup init               Create checkup.toml with auto-detected tools
checkup check              Run all checks
checkup fix                Auto-fix failures
checkup list               List configured checks
checkup completions        Generate shell completions

Options:
  -c, --config <FILE>      Path to config file [default: checkup.toml]
      --json               Output results as JSON
  -q, --quiet              Only show failures
  -h, --help               Print help
  -V, --version            Print version
```

---

## Checks

Each check tests one aspect of your environment:

| Check     | What it does                              |
| --------- | ----------------------------------------- |
| `command` | Verifies a command exists in PATH         |
| `version` | Checks tool version (semver comparison)   |
| `service` | TCP port check — is the service running?  |
| `port`    | Is the port free to use?                  |
| `envfile` | Does `.env` exist and have required keys? |
| `envvar`  | Are environment variables set?            |

Checks run **in parallel** using async I/O — typically completing in under 1 second.

---

## Auto-Fix

The `fix` subcommand attempts to automatically resolve issues:

```bash
$ checkup fix
```

```
Attempting auto-fix for 3 issues...

  [1/3] Docker: starting service...          ✓ Done
  [2/3] .env: creating from .env.example...  ✓ Done
  [3/3] Port 3000: killing process...        ✗ Permission denied (need sudo)

  2/3 fixed automatically. 1 requires manual action.
  Run: sudo kill $(sudo lsof -t -i:3000)
```

When auto-fix isn't possible, the tool provides the exact command to run.

---

## Roadmap

- ✅ Project structure and architecture
- ✅ Core: trait `Check` + plugin system
- ✅ Checks: command, version, service, port, envfile, envvar
- ✅ Parallel execution with async (`futures::join_all`)
- ✅ `checkup.toml` config parser (serde)
- ✅ Auto-fix suggestions
- ✅ JSON output mode (`--json`)
- ✅ CLI with clap (`init`, `check`, `fix`, `list`, `completions`)
- ✅ Shell completions (bash, zsh, fish, powershell, elvish)
- ✅ 38 tests passing (19 unit + 10 integration)
- 🚧 Windows support

---

## CI/CD

This project uses **GitHub Actions** for continuous integration and delivery. Every push and pull request triggers the CI pipeline automatically.

### Pipeline Stages

| Stage       | Description                                     |
| ----------- | ----------------------------------------------- |
| **Check**   | `cargo check` — fast compilation verification   |
| **Format**  | `cargo fmt` — ensures consistent code style     |
| **Lint**    | `cargo clippy` — catches common mistakes        |
| **Test**    | `cargo test` — runs all tests (38 passing)      |
| **Build**   | `cargo build --release` — production artifact   |

### Supported Platforms (CI)

| Platform | Status       |
| -------- | ------------ |
| Linux    | ✅ Tested    |
| macOS    | ✅ Tested    |
| Windows  | ✅ Tested    |

The CI pipeline runs on **all three platforms** on every push, ensuring cross-platform compatibility. See [`.github/workflows/ci.yml`](.github/workflows/ci.yml) for details.

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on how to submit issues, propose features, and open pull requests.

---

## License

This project is licensed under the [MIT License](LICENSE).
