#Requires -Version 5.1
<#
.SYNOPSIS
    Install (or update) bro for PowerShell on Windows.
.PARAMETER FromSource
    Skip the prebuilt-binary fetch and build from source instead.
.EXAMPLE
    irm https://raw.githubusercontent.com/AdityaSawant0912/bro-cli/master/install.ps1 | iex
.EXAMPLE
    .\install.ps1 -FromSource
#>
param(
    [switch]$FromSource
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo       = "AdityaSawant0912/bro-cli"
$BinDir     = Join-Path $env:USERPROFILE "bin"
$BroExe     = Join-Path $BinDir "bro.exe"
$Marker     = "# bro wrapper"

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host " ok  $msg" -ForegroundColor Green }
function Write-Skip($msg) { Write-Host "skip $msg" -ForegroundColor Yellow }

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Force $BinDir | Out-Null
}

# ── Prebuilt binary ──────────────────────────────────────────────────────────
function Install-Prebuilt {
    $target = "x86_64-pc-windows-msvc"
    $url    = "https://github.com/$Repo/releases/latest/download/bro-$target.zip"
    $tmp    = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Force $tmp | Out-Null

    Write-Step "Fetching prebuilt binary for $target"
    try {
        Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmp "bro.zip") -ErrorAction Stop
    } catch {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
        return $false
    }

    Expand-Archive -Path (Join-Path $tmp "bro.zip") -DestinationPath $tmp -Force
    $extracted = Join-Path $tmp "bro.exe"
    if (-not (Test-Path $extracted)) {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
        return $false
    }

    Copy-Item -Force $extracted $BroExe
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    Write-Ok "installed prebuilt binary → $BroExe"
    return $true
}

# ── Build from source ────────────────────────────────────────────────────────
function Build-FromSource {
    Write-Step "Checking for cargo"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "cargo not found. Install Rust from https://rustup.rs then re-run."
    }
    Write-Ok "cargo $(cargo --version)"

    $repoRoot = $PSScriptRoot
    if (-not (Test-Path (Join-Path $repoRoot "Cargo.toml"))) {
        Write-Step "Cloning $Repo"
        $repoRoot = Join-Path ([System.IO.Path]::GetTempPath()) "bro-cli"
        git clone --depth 1 "https://github.com/$Repo.git" $repoRoot
    }

    Write-Step "Building release binary"
    Push-Location $repoRoot
    cargo build --release
    if ($LASTEXITCODE -ne 0) { Write-Error "cargo build failed." }
    Pop-Location

    $releaseBin = Join-Path $repoRoot "target\release\bro.exe"
    if (-not (Test-Path $releaseBin)) { Write-Error "build produced no binary at $releaseBin" }
    Copy-Item -Force $releaseBin $BroExe
    Write-Ok "built + installed → $BroExe"
}

if ($FromSource) {
    Build-FromSource
} elseif (-not (Install-Prebuilt)) {
    Write-Skip "no matching prebuilt binary, falling back to source build"
    Build-FromSource
}

# ── PATH ──────────────────────────────────────────────────────────────────────
Write-Step "Checking user PATH"
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -split ";" -notcontains $BinDir) {
    [Environment]::SetEnvironmentVariable("PATH", "$BinDir;$userPath", "User")
    $env:PATH = "$BinDir;$env:PATH"
    Write-Ok "added $BinDir to user PATH"
} else {
    Write-Skip "$BinDir already in user PATH"
}

# ── PowerShell wrapper ────────────────────────────────────────────────────────
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

$WrapperLine     = "Invoke-Expression (& '$BroExe' init powershell | Out-String)"
$CompletionsLine = "Invoke-Expression (& '$BroExe' completions powershell | Out-String)"
Add-Content $PROFILE "`n$Marker`n$WrapperLine`n$CompletionsLine`n"
Write-Ok "wrapper + completions updated"

# ── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "Done! Reload your shell:" -ForegroundColor Green
Write-Host "  . `$PROFILE" -ForegroundColor White
Write-Host ""
Write-Host "Then try:  bro add gs `"git status`"  &&  bro gs" -ForegroundColor White
