# 🏥 checkup

> Environment diagnostics and auto-fix for development machines.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Stop wasting hours debugging your dev machine. One command tells you **exactly**
what's wrong — and how to fix it.

**🇧🇷 [Versão em Português](README-PT.md)**

```bash
$ checkup
```

```
┌──────────────────────────────────────────────────────┐
│  checkup v0.1.0 — Environment Diagnostics            │
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
│  Run 'checkup --fix' to auto-resolve what's possible │
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
git clone https://github.com/seuuser/checkup.git
cd checkup
cargo install --path .
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

This creates a `.checkup.toml` file with sensible defaults.

**Step 2:** Edit `.checkup.toml` for your project:

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
checkup --fix
```

---

## Configuration

The `.checkup.toml` file defines what to check:

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
checkup init               Create a default .checkup.toml
checkup check              Run all checks
checkup check --fix        Show fix suggestions for failures

Options:
  -c, --config <FILE>      Path to config file [default: .checkup.toml]
  -f, --fix                Show fix suggestions
  -j, --json               Output results as JSON
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

The `--fix` flag attempts to automatically resolve issues:

```bash
$ checkup --fix
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

- [x] Project structure and architecture
- [x] Core: trait `Check` + plugin system
- [x] Checks: command, version, service, port, envfile, envvar
- [x] Parallel execution with async (futures::join_all)
- [x] `.checkup.toml` config parser (serde)
- [x] Auto-fix suggestions
- [x] JSON output mode (`--json`)
- [x] CLI with clap (init, check, --fix, --quiet)
- [x] 18 unit tests passing
- [ ] Shell completions (bash, zsh, fish)
- [ ] Windows support

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Fork and clone
git clone https://github.com/youruser/checkup.git
cd checkup

# Run tests
cargo test

# Run lints
cargo clippy

# Format code
cargo fmt

# Build
cargo build --release
```

---

## License

This project is licensed under the [MIT License](LICENSE).
