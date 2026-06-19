#Requires -Version 5.1
<#
.SYNOPSIS
    Install bro for PowerShell on Windows.
    Run once from the repo root: .\install.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot  = $PSScriptRoot
$BinDir    = Join-Path $env:USERPROFILE "bin"
$BroExe    = Join-Path $BinDir "bro.exe"
$ReleaseBin = Join-Path $RepoRoot "target\release\bro.exe"

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host " ok  $msg" -ForegroundColor Green }
function Write-Skip($msg) { Write-Host "skip $msg" -ForegroundColor Yellow }

# ── 1. Cargo ────────────────────────────────────────────────────────────────
Write-Step "Checking for cargo"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo not found. Install Rust from https://rustup.rs then re-run."
}
Write-Ok "cargo $(cargo --version)"

# ── 2. Build ─────────────────────────────────────────────────────────────────
Write-Step "Building release binary"
Push-Location $RepoRoot
cargo build --release 2>&1 | ForEach-Object { Write-Host "  $_" }
if ($LASTEXITCODE -ne 0) { Write-Error "cargo build failed." }
Pop-Location
Write-Ok "built $ReleaseBin"

# ── 3. Install dir ───────────────────────────────────────────────────────────
Write-Step "Ensuring $BinDir exists"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Force $BinDir | Out-Null
    Write-Ok "created $BinDir"
} else {
    Write-Skip "$BinDir already exists"
}

# ── 4. PATH ──────────────────────────────────────────────────────────────────
Write-Step "Checking user PATH"
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -split ";" -notcontains $BinDir) {
    [Environment]::SetEnvironmentVariable("PATH", "$BinDir;$userPath", "User")
    $env:PATH = "$BinDir;$env:PATH"
    Write-Ok "added $BinDir to user PATH (effective after restart)"
} else {
    Write-Skip "$BinDir already in user PATH"
}

# ── 5. Copy binary ───────────────────────────────────────────────────────────
Write-Step "Installing bro.exe → $BroExe"
Copy-Item -Force $ReleaseBin $BroExe
Write-Ok "copied"

# ── 6. PowerShell wrapper ────────────────────────────────────────────────────
Write-Step "Installing PowerShell wrapper to `$PROFILE"

# Create profile file if it doesn't exist
if (-not (Test-Path $PROFILE)) {
    New-Item -ItemType File -Force $PROFILE | Out-Null
    Write-Ok "created $PROFILE"
}

$ProfileContent = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
$Marker = "# bro wrapper"

if ($ProfileContent -and $ProfileContent.Contains($Marker)) {
    Write-Skip "wrapper already in `$PROFILE"
} else {
    $Wrapper = "`n$Marker`nInvoke-Expression (& '$BroExe' init powershell | Out-String)`n"
    Add-Content $PROFILE $Wrapper
    Write-Ok "wrapper added to $PROFILE"
}

# ── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Done! Reload your shell:" -ForegroundColor Green
Write-Host "  . `$PROFILE" -ForegroundColor White
Write-Host ""
Write-Host "Then try:  bro add gs `"git status`"  &&  bro gs" -ForegroundColor White
