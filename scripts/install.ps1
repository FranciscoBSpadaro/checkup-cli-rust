# ─────────────────────────────────────────────────────────────
# checkup — Quick Install Script for Windows (PowerShell)
# ─────────────────────────────────────────────────────────────
# Installs Rust (via rustup) if missing, then builds and
# installs the checkup CLI from source.
# ─────────────────────────────────────────────────────────────
#
# Usage:
#   .\scripts\install.ps1
#
# Or from an elevated PowerShell:
#   Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy Bypass -Force
#   .\scripts\install.ps1
# ─────────────────────────────────────────────────────────────

$ErrorActionPreference = "Stop"

# ── Helpers ──────────────────────────────────────────────────

function Write-Info    { param($msg) Write-Host "[INFO]    " -ForegroundColor Cyan -NoNewline; Write-Host $msg }
function Write-Success { param($msg) Write-Host "[OK]       " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn    { param($msg) Write-Host "[WARN]     " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Error   { param($msg) Write-Host "[ERROR]    " -ForegroundColor Red -NoNewline; Write-Host $msg; exit 1 }
function Write-Header  { param($msg) Write-Host ""; Write-Host "── $msg ──" -ForegroundColor Cyan; Write-Host "" }

# ── Configuration ────────────────────────────────────────────

$RepoUrl    = "https://github.com/FranciscoBSpadaro/checkup-cli-rust"
$SourceDir  = Join-Path $env:USERPROFILE ".checkup-src"
$CargoBin   = Join-Path $env:USERPROFILE ".cargo\bin"

# ── Step 1: Welcome ──────────────────────────────────────────

Write-Header "checkup Installer for Windows"

Write-Info "Detected OS: Windows $([Environment]::OSVersion.VersionString)"

# ── Step 2: Check / Install Git ──────────────────────────────

Write-Header "Checking System Dependencies"

if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Success "git is available ($(git --version))"
} else {
    Write-Info "git not found. Please install Git for Windows:"
    Write-Host "  https://git-scm.com/download/win"
    Write-Host "Then re-run this script."
    Write-Error "git is required"
}

# ── Step 3: Check / Install Rust ─────────────────────────────

Write-Header "Checking Rust Toolchain"

$CargoPath = Join-Path $CargoBin "cargo.exe"
$RustcPath = Join-Path $CargoBin "rustc.exe"

if ((Test-Path $CargoPath) -and (Test-Path $RustcPath)) {
    $RustVersion = & rustc --version 2>&1
    Write-Success "Rust is already installed ($RustVersion)"

    # Check minimum version 1.75
    $VerString = ($RustVersion -split '\s+')[1]
    $Parts = $VerString -split '\.'
    $Major = [int]$Parts[0]
    $Minor = [int]$Parts[1]

    if ($Major -lt 1 -or ($Major -eq 1 -and $Minor -lt 75)) {
        Write-Warn "Rust ${VerString} detected, but 1.75+ is required. Updating..."
        & rustup update stable
        Write-Success "Rust updated to $(rustc --version)"
    }
} elseif (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Info "rustup found but toolchain not installed. Installing stable..."
    & rustup default stable
    Write-Success "Rust installed: $(rustc --version)"
} else {
    Write-Info "Rust not found. Downloading rustup installer..."

    $RustupInit = "$env:TEMP\rustup-init.exe"

    Invoke-WebRequest `
        -Uri "https://win.rustup.rs/x86_64" `
        -OutFile $RustupInit `
        -UseBasicParsing

    Write-Info "Running rustup installer..."
    & $RustupInit -y --default-toolchain stable

    if ($LASTEXITCODE -ne 0) {
        Write-Error "rustup installation failed (exit code $LASTEXITCODE)"
    }

    # Refresh PATH for current session
    $env:PATH = "$CargoBin;$env:PATH"

    # Verify
    if (Test-Path $CargoPath) {
        Write-Success "Rust installed: $(rustc --version)"
        Write-Info "You may need to restart your terminal after this script."
    } else {
        Write-Error "Installation appeared to succeed but cargo was not found."
    }
}

# Ensure cargo bin is on PATH for the rest of this script
$env:PATH = "$CargoBin;$env:PATH"

# ── Step 4: Clone and build ──────────────────────────────────

Write-Header "Building checkup"

if (Test-Path $SourceDir) {
    Write-Info "Existing source found at ${SourceDir}, updating..."
    Push-Location $SourceDir
    git pull origin master 2>&1 | Out-String | ForEach-Object { Write-Host "  $_" }
    Pop-Location
} else {
    Write-Info "Cloning repository..."
    git clone $RepoUrl $SourceDir
}

Write-Success "Source code ready at ${SourceDir}"

Write-Info "Building release binary (this may take a minute)..."
Push-Location $SourceDir
& cargo build --release 2>&1 | ForEach-Object { Write-Host "  $_" }
Pop-Location

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed with exit code $LASTEXITCODE"
}
Write-Success "Build complete"

# ── Step 5: Install binary ────────────────────────────────────

Write-Header "Installing Binary"

$BuiltBinary = Join-Path $SourceDir "target\release\checkup.exe"

if (-not (Test-Path $BuiltBinary)) {
    Write-Error "Build artifact not found at ${BuiltBinary}"
}

if (-not (Test-Path $CargoBin)) {
    New-Item -ItemType Directory -Path $CargoBin -Force | Out-Null
}

Copy-Item $BuiltBinary (Join-Path $CargoBin "checkup.exe") -Force

Write-Success "checkup installed to ${CargoBin}\checkup.exe"

# ── Step 6: Add to PATH if needed ────────────────────────────

Write-Header "Updating PATH"

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

if ($UserPath -notlike "*$CargoBin*") {
    Write-Info "Adding ${CargoBin} to user PATH..."
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$UserPath;$CargoBin",
        "User"
    )
    # Also update current session
    $env:PATH = "$CargoBin;$env:PATH"
    Write-Success "Added to PATH (will be available in new terminals)"
} else {
    Write-Success "Already on user PATH"
}

# ── Step 7: Verify ───────────────────────────────────────────

Write-Header "Verification"

try {
    $Version = & checkup --version 2>&1 | Select-Object -Last 1
    Write-Success "checkup is working! ($Version)"
} catch {
    Write-Warn "Could not verify installation."
    Write-Host "  Try opening a new terminal and run: checkup --version"
}

# ── Done ─────────────────────────────────────────────────────

Write-Header "Installation Complete"

Write-Host "  " -NoNewline
Write-Host "checkup is ready to use!" -ForegroundColor Green
Write-Host ""
Write-Host "  Quick start:"
Write-Host "    checkup init          # Create checkup.toml config"
Write-Host "    checkup check         # Run diagnostics"
Write-Host "    checkup fix           # Auto-fix issues"
Write-Host "    checkup --help        # Show all options"
Write-Host ""

Write-Warn "If 'checkup' is not recognized, restart your terminal."
