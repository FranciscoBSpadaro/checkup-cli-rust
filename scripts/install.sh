#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# checkup — Quick Install Script for Linux / macOS
# ─────────────────────────────────────────────────────────────
# Installs Rust (via rustup) if missing, then builds and
# installs the checkup CLI from source.
# ─────────────────────────────────────────────────────────────

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ──────────────────────────────────────────────────

info()    { echo -e "${BLUE}[INFO]${RESET}    $1"; }
success() { echo -e "${GREEN}[✓]${RESET}       $1"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}    $1"; }
error()   { echo -e "${RED}[ERROR]${RESET}   $1"; exit 1; }
header()  { echo -e "\n${BOLD}${BLUE}── $1 ──${RESET}\n"; }

detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux*)   echo "linux" ;;
        Darwin*)  echo "macos" ;;
        *)        echo "unknown" ;;
    esac
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        echo "${ID_LANGUAGE:-$ID}"
    else
        echo "unknown"
    fi
}

# ── Step 1: Detect environment ───────────────────────────────

header "checkup Installer"

OS="$(detect_os)"
info "Detected OS: ${OS}"

if [ "$OS" = "unknown" ]; then
    error "Unsupported operating system. This script supports Linux and macOS."
fi

# ── Step 2: Install system dependencies ──────────────────────

header "Checking System Dependencies"

if ! command -v curl &>/dev/null; then
    info "curl is required. Installing..."
    if [ "$OS" = "macos" ] && command -v brew &>/dev/null; then
        brew install curl
    elif [ "$OS" = "linux" ]; then
        DISTRO="$(detect_distro)"
        case "$DISTRO" in
            ubuntu|debian|linuxmint|pop)
                sudo apt-get update -qq && sudo apt-get install -y -qq curl ;;
            fedora)
                sudo dnf install -y curl ;;
            centos|rhel)
                sudo yum install -y curl ;;
            arch|manjaro)
                sudo pacman -Sy --noconfirm curl ;;
            alpine)
                sudo apk add curl ;;
            *)
                error "Unknown distro '${DISTRO}'. Please install curl manually and re-run." ;;
        esac
    else
        error "Please install curl and re-run this script."
    fi
fi
success "curl is available"

if ! command -v git &>/dev/null; then
    info "git is required. Installing..."
    if [ "$OS" = "macos" ] && command -v brew &>/dev/null; then
        brew install git
    elif [ "$OS" = "linux" ]; then
        DISTRO="$(detect_distro)"
        case "$DISTRO" in
            ubuntu|debian|linuxmint|pop)
                sudo apt-get update -qq && sudo apt-get install -y -qq git ;;
            fedora)
                sudo dnf install -y git ;;
            centos|rhel)
                sudo yum install -y git ;;
            arch|manjaro)
                sudo pacman -Sy --noconfirm git ;;
            alpine)
                sudo apk add git ;;
            *)
                error "Unknown distro. Please install git manually and re-run." ;;
        esac
    fi
fi
success "git is available"

# ── Step 3: Install Rust (via rustup) ────────────────────────

header "Checking Rust Toolchain"

if command -v cargo &>/dev/null && command -v rustc &>/dev/null; then
    RUST_VERSION="$(rustc --version | cut -d' ' -f2)"
    success "Rust is already installed (rustc ${RUST_VERSION})"

    # Check minimum version 1.75
    MAJOR="$(echo "$RUST_VERSION" | cut -d. -f1)"
    MINOR="$(echo "$RUST_VERSION" | cut -d. -f2)"
    if [ "$MAJOR" -lt 1 ] || { [ "$MAJOR" -eq 1 ] && [ "$MINOR" -lt 75 ]; }; then
        warn "Rust ${RUST_VERSION} detected, but 1.75+ is required. Updating..."
        rustup update stable
        success "Rust updated to $(rustc --version)"
    fi
else
    info "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

    # Source cargo env for current session
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"

    success "Rust installed: $(rustc --version)"
    info "You may need to restart your shell or run: source ~/.cargo/env"
fi

# Set PATH in case cargo was just installed
export PATH="${HOME}/.cargo/bin:${PATH}"

# ── Step 4: Clone and build ──────────────────────────────────

header "Building checkup"

INSTALL_DIR="${HOME}/.checkup-src"
REPO_URL="https://github.com/FranciscoBSpadaro/checkup-cli-rust"

if [ -d "$INSTALL_DIR" ]; then
    info "Existing source found at ${INSTALL_DIR}, updating..."
    cd "$INSTALL_DIR"
    git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || warn "Could not pull latest changes"
else
    info "Cloning repository..."
    git clone "$REPO_URL" "$INSTALL_DIR"
fi
success "Source code ready at ${INSTALL_DIR}"

info "Building release binary (this may take a minute)..."
cd "$INSTALL_DIR"
cargo build --release 2>&1 | while IFS= read -r line; do echo "  $line"; done
success "Build complete"

# ── Step 5: Install binary ────────────────────────────────────

header "Installing Binary"

BINARY_PATH="${INSTALL_DIR}/target/release/checkup"

if [ ! -f "$BINARY_PATH" ]; then
    error "Build artifact not found at ${BINARY_PATH}"
fi

INSTALL_BIN="${HOME}/.cargo/bin"

mkdir -p "$INSTALL_BIN"
cp "$BINARY_PATH" "${INSTALL_BIN}/checkup"
chmod +x "${INSTALL_BIN}/checkup"

success "checkup installed to ${INSTALL_BIN}/checkup"

# ── Step 6: Verify ───────────────────────────────────────────

header "Verification"

export PATH="${INSTALL_BIN}:${PATH}"

if checkup --version &>/dev/null; then
    VERSION="$(checkup --version 2>&1 | tail -1)"
    success "checkup is working! (${VERSION})"
else
    warn "Could not verify installation. Make sure ${INSTALL_BIN} is in your PATH:"
    echo -e "  ${BOLD}export PATH=\"\${HOME}/.cargo/bin:\${PATH}\"${RESET}"
    echo -e "  ${BOLD}echo 'export PATH=\"\${HOME}/.cargo/bin:\${PATH}\"' >> ~/.bashrc${RESET}"
fi

# ── Done ─────────────────────────────────────────────────────

header "Installation Complete 🎉"

echo -e "  ${GREEN}checkup is ready to use!${RESET}"
echo ""
echo "  Quick start:"
echo "    checkup init          # Create checkup.toml config"
echo "    checkup check         # Run diagnostics"
echo "    checkup check --fix   # Auto-fix issues"
echo "    checkup --help        # Show all options"
echo ""

# shellcheck disable=SC2016
if [[ ":$PATH:" != *":${INSTALL_BIN}:"* ]]; then
    warn "${INSTALL_BIN} is not in your PATH."
    echo "  Add it with:"
    echo -e "    ${BOLD}echo 'export PATH=\"\${HOME}/.cargo/bin:\${PATH}\"' >> ~/.bashrc && source ~/.bashrc${RESET}"
    echo "    (or ~/.zshrc if using zsh)"
fi
