#Requires -Version 5.1
<#
.SYNOPSIS
    Install (or update) bro for PowerShell on Windows.
    Run from the repo root: .\install.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot   = $PSScriptRoot
$BinDir     = Join-Path $env:USERPROFILE "bin"
$BroExe     = Join-Path $BinDir "bro.exe"
$ReleaseBin = Join-Path $RepoRoot "target\release\bro.exe"
$Marker     = "# bro wrapper"

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host " ok  $msg" -ForegroundColor Green }
function Write-Skip($msg) { Write-Host "skip $msg" -ForegroundColor Yellow }

# ── 1. Cargo ─────────────────────────────────────────────────────────────────
Write-Step "Checking for cargo"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo not found. Install Rust from https://rustup.rs then re-run."
}
Write-Ok "cargo $(cargo --version)"

# ── 2. Build ─────────────────────────────────────────────────────────────────
Write-Step "Building release binary"
Push-Location $RepoRoot
cargo build --release
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
    Write-Ok "added $BinDir to user PATH"
} else {
    Write-Skip "$BinDir already in user PATH"
}

# ── 5. Copy binary ───────────────────────────────────────────────────────────
Write-Step "Installing bro.exe → $BroExe"
Copy-Item -Force $ReleaseBin $BroExe
Write-Ok "copied"

# ── 6. PowerShell wrapper ────────────────────────────────────────────────────
Write-Step "Updating PowerShell wrapper in `$PROFILE"

if (-not (Test-Path $PROFILE)) {
    New-Item -ItemType File -Force $PROFILE | Out-Null
    Write-Ok "created $PROFILE"
}

# Always replace — strip old block (marker → next blank line), then re-add
$lines    = Get-Content $PROFILE -ErrorAction SilentlyContinue
$filtered = [System.Collections.Generic.List[string]]::new()
$skip     = $false
foreach ($line in $lines) {
    if ($line -eq $Marker)       { $skip = $true; continue }
    if ($skip -and $line -eq '') { $skip = $false; continue }
    if ($skip)                   { continue }
    $filtered.Add($line)
}
Set-Content $PROFILE $filtered

$WrapperLine = "Invoke-Expression (& '$BroExe' init powershell | Out-String)"
Add-Content $PROFILE "`n$Marker`n$WrapperLine`n"
Write-Ok "wrapper updated"

# ── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Done! Reload your shell:" -ForegroundColor Green
Write-Host "  . `$PROFILE" -ForegroundColor White
Write-Host ""
Write-Host "Then try:  bro add gs `"git status`"  &&  bro gs" -ForegroundColor White
