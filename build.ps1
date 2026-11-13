# PowerShell Build Script for WeMod Pro Unlocker
# Compiles Updater (Go) and CLI (Rust) into optimized release binaries in dist/
#
# The CLI embeds the updater (cli/bin/wemod-pro-unlocker-updater.exe) via include_bytes!,
# so the updater is built first and copied there before the CLI is compiled.
#
# Usage:
#   .\build.ps1                   # build updater, sync it into cli/bin, build CLI, zip
#   .\build.ps1 -NoUpdaterSync    # keep the committed cli/bin/wemod-pro-unlocker-updater.exe
#   .\build.ps1 -RunTests         # also run cargo test and go test before building

param(
    [switch]$NoUpdaterSync,
    [switch]$RunTests
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Building WeMod Pro Unlocker Artifacts  " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Check prerequisites
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
$go = Get-Command go -ErrorAction SilentlyContinue

if (-not $cargo) {
    Write-Error "Cargo (Rust) is required but was not found in PATH. Please install Rust: https://rustup.rs/"
}

if (-not $go) {
    Write-Error "Go is required but was not found in PATH. Please install Go: https://go.dev/dl/"
}

$rootDir = $PSScriptRoot
$distDir = Join-Path $rootDir "dist"
$cliDir = Join-Path $rootDir "cli"
$updaterDir = Join-Path $rootDir "updater"

# 2. Optional tests
if ($RunTests) {
    Write-Host "`n[0/5] Running tests..." -ForegroundColor Green
    Push-Location $updaterDir
    try {
        & go test ./...
        if ($LASTEXITCODE -ne 0) { throw "go test failed with exit code $LASTEXITCODE" }
    } finally {
        Pop-Location
    }
    & cargo test --manifest-path (Join-Path $cliDir "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed with exit code $LASTEXITCODE" }
}

# 3. Setup dist directory
if (Test-Path $distDir) {
    Write-Host "[1/5] Cleaning existing dist/ folder..." -ForegroundColor Yellow
    Remove-Item -Path $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir | Out-Null

# 4. Build Updater (Go) first so the CLI can embed it
Write-Host "`n[2/5] Building Updater (Go) with optimizations (-trimpath -ldflags='-s -w')..." -ForegroundColor Green
$updaterBinary = Join-Path $distDir "updater.exe"
Push-Location $updaterDir
try {
    & go build -trimpath -ldflags="-s -w" -o $updaterBinary .
    if ($LASTEXITCODE -ne 0) { throw "Go build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

# 5. Sync the embedded updater
$embeddedUpdater = Join-Path $cliDir "bin\wemod-pro-unlocker-updater.exe"
if ($NoUpdaterSync) {
    Write-Host "`n[3/5] Skipping updater sync (-NoUpdaterSync); CLI embeds the committed $embeddedUpdater" -ForegroundColor Yellow
} else {
    Write-Host "`n[3/5] Syncing updater into $embeddedUpdater..." -ForegroundColor Green
    Copy-Item -Path $updaterBinary -Destination $embeddedUpdater -Force
}

# 6. Build CLI (Rust)
Write-Host "`n[4/5] Building CLI (Rust) in release mode..." -ForegroundColor Green
Push-Location $cliDir
try {
    & cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "Cargo build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

$cliBinary = Join-Path $cliDir "target\release\wemod-pro-unlocker.exe"
if (-not (Test-Path $cliBinary)) {
    throw "CLI executable not found at $cliBinary"
}
Copy-Item -Path $cliBinary -Destination (Join-Path $distDir "wemod-pro-unlocker.exe") -Force

# 7. Create release zip artifact
Write-Host "`n[5/5] Creating release ZIP archive..." -ForegroundColor Green
$zipPath = Join-Path $distDir "wemod-pro-unlocker-windows-x64.zip"
Compress-Archive -Path "$distDir\wemod-pro-unlocker.exe", "$distDir\updater.exe" -DestinationPath $zipPath -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " Build Completed Successfully!          " -ForegroundColor Cyan
Write-Host " Output files location: $distDir        " -ForegroundColor Cyan
Get-ChildItem -Path $distDir | Select-Object Name, @{Name="Size (KB)"; Expression={[math]::Round($_.Length / 1KB, 2)}} | Format-Table -AutoSize
Write-Host "========================================" -ForegroundColor Cyan
