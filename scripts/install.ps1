# ─────────────────────────────────────────────────────────────
# checkup — Quick Install Script for Windows
# ─────────────────────────────────────────────────────────────
# Installs Rust (via rustup) if missing, then builds and
# installs the checkup CLI from source.
# ─────────────────────────────────────────────────────────────
#
# Usage (PowerShell as Administrator recommended):
#   Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
#   .\scripts\install.ps1

$ErrorActionPreference = "Stop"

# ── Helpers ──────────────────────────────────────────────────

function Write-Info    { param($msg) Write-Host "[INFO]    $msg" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host "[OK]       $msg" -ForegroundColor Green }
function Write-Warn    { param($msg) Write-Host "[WARN]     $msg" -ForegroundColor Yellow }
function Write-Error   { param($msg) Write-Host "[ERROR]    $msg" -ForegroundColor Red; exit 1 }
function Write-Header  { param($msg) Write-Host "`n── $msg ──`n" -ForegroundColor Cyan }

# ── Step 1: Detect environment ───────────────────────────────

Write-Header "checkup Installer (Windows)"

$osVersion = [System.Environment]::OSVersion.Version
Write-Info "Detected Windows version: $($osVersion.Major).$($osVersion.Minor).$($osVersion.Build)"

if ($osVersion.Major -lt 10) {
    Write-Error "Windows 10 or later is required."
}

# ── Step 2: Check system dependencies ────────────────────────

Write-Header "Checking System Dependencies"

# Check for winget (Windows 10 1709+ / Windows 11)
$hasWinget = $false
try {
    $null = Get-Command winget -ErrorAction Stop
    $hasWinget = $true
    Write-Success "winget is available"
} catch {
    Write-Warn "winget not found. Some dependencies may need manual installation."
}

# Check for git
try {
    $gitVersion = git --version 2>&1
    Write-Success "git is available ($gitVersion)"
} catch {
    Write-Info "git not found. Installing via winget..."
    if ($hasWinget) {
        winget install --id Git.Git -e --source winget --accept-package-agreements --accept-source-agreements
        # Refresh PATH
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        Write-Success "git installed"
    } else {
        Write-Error "Please install git manually from https://git-scm.com/download/win and re-run."
    }
}

# ── Step 3: Install Rust (via rustup) ────────────────────────

Write-Header "Checking Rust Toolchain"

$rustInstalled = $false
try {
    $cargoVersion = cargo --version 2>&1
    $rustcVersion = rustc --version 2>&1
    Write-Success "Rust is already installed ($rustcVersion)"

    # Check minimum version 1.75
    $versionStr = ($rustcVersion -split ' ')[1]
    $major = [int]($versionStr -split '\.')[0]
    $minor = [int]($versionStr -split '\.')[1]
    if ($major -lt 1 -or ($major -eq 1 -and $minor -lt 75)) {
        Write-Warn "Rust $versionStr detected, but 1.75+ is required. Updating..."
        rustup update stable
        Write-Success "Rust updated to $(rustc --version)"
    }
    $rustInstalled = $true
} catch {
    Write-Info "Rust not found. Installing via rustup..."
}

if (-not $rustInstalled) {
    # Download and run rustup-init.exe
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    Write-Info "Downloading rustup..."
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing

    Write-Info "Running rustup installer..."
    & $rustupPath -y --default-toolchain stable
    if ($LASTEXITCODE -ne 0) {
        Write-Error "rustup installation failed."
    }

    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

    Write-Success "Rust installed: $(rustc --version)"
    Write-Info "You may need to restart your terminal for PATH changes to take effect."
}

# Ensure cargo is on PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# ── Step 4: Clone and build ──────────────────────────────────

Write-Header "Building checkup"

$installDir = "$env:USERPROFILE\.checkup-src"
$repoUrl = "https://github.com/FranciscoBSpadaro/checkup-cli-rust"

if (Test-Path $installDir) {
    Write-Info "Existing source found at $installDir, updating..."
    Push-Location $installDir
    git pull origin main 2>$null
    if ($LASTEXITCODE -ne 0) { git pull origin master 2>$null }
    Pop-Location
} else {
    Write-Info "Cloning repository..."
    git clone $repoUrl $installDir
}
Write-Success "Source code ready at $installDir"

Write-Info "Building release binary (this may take a minute)..."
Push-Location $installDir
cargo build --release 2>&1 | ForEach-Object { Write-Host "  $_" }
Pop-Location
Write-Success "Build complete"

# ── Step 5: Install binary ────────────────────────────────────

Write-Header "Installing Binary"

$binaryPath = "$installDir\target\release\checkup.exe"

if (-not (Test-Path $binaryPath)) {
    Write-Error "Build artifact not found at $binaryPath"
}

$installBin = "$env:USERPROFILE\.cargo\bin"
if (-not (Test-Path $installBin)) {
    New-Item -ItemType Directory -Path $installBin -Force | Out-Null
}

Copy-Item $binaryPath "$installBin\checkup.exe" -Force
Write-Success "checkup installed to $installBin\checkup.exe"

# ── Step 6: Verify ───────────────────────────────────────────

Write-Header "Verification"

$env:PATH = "$installBin;$env:PATH"

try {
    $versionOutput = checkup --version 2>&1 | Select-Object -Last 1
    Write-Success "checkup is working! ($versionOutput)"
} catch {
    Write-Warn "Could not verify installation. Make sure $installBin is in your PATH:"
    Write-Host "  [System.Environment]::SetEnvironmentVariable('PATH', '$installBin;' + [System.Environment]::GetEnvironmentVariable('PATH', 'User'), 'User')"
}

# ── Done ─────────────────────────────────────────────────────

Write-Header "Installation Complete"

Write-Host "  checkup is ready to use!" -ForegroundColor Green
Write-Host ""
Write-Host "  Quick start:"
Write-Host "    checkup init          # Create checkup.toml config"
Write-Host "    checkup check         # Run diagnostics"
Write-Host "    checkup fix           # Auto-fix issues"
Write-Host "    checkup --help        # Show all options"
Write-Host ""

$userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installBin*") {
    Write-Warn "$installBin is not in your user PATH."
    Write-Host "  Add it with:"
    Write-Host "    [System.Environment]::SetEnvironmentVariable('PATH', '$installBin;' + [System.Environment]::GetEnvironmentVariable('PATH', 'User'), 'User')"
    Write-Host "  Or add it via System Properties > Environment Variables."
}
