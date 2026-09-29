# PowerShell Build Script for WeMod Pro Unlocker
# Compiles CLI (Rust) and Updater (Go) into optimized release binaries in dist/

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

# 2. Setup dist directory
$rootDir = $PSScriptRoot
$distDir = Join-Path $rootDir "dist"

if (Test-Path $distDir) {
    Write-Host "[1/4] Cleaning existing dist/ folder..." -ForegroundColor Yellow
    Remove-Item -Path $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir | Out-Null

# 3. Build CLI (Rust)
Write-Host "`n[2/4] Building CLI (Rust) in release mode..." -ForegroundColor Green
Push-Location (Join-Path $rootDir "cli")
try {
    & cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "Cargo build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

$cliBinary = Join-Path $rootDir "cli\target\release\wemod-pro-unlocker.exe"
if (-not (Test-Path $cliBinary)) {
    throw "CLI executable not found at $cliBinary"
}
Copy-Item -Path $cliBinary -Destination (Join-Path $distDir "wemod-pro-unlocker.exe") -Force

# 4. Build Updater (Go)
Write-Host "`n[3/4] Building Updater (Go) with optimizations (-ldflags='-s -w')..." -ForegroundColor Green
Push-Location (Join-Path $rootDir "updater")
try {
    $updaterBinary = Join-Path $distDir "updater.exe"
    & go build -ldflags="-s -w" -o $updaterBinary .
    if ($LASTEXITCODE -ne 0) { throw "Go build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

# 5. Create release zip artifact
Write-Host "`n[4/4] Creating release ZIP archive..." -ForegroundColor Green
$zipPath = Join-Path $distDir "wemod-pro-unlocker-windows-x64.zip"
Compress-Archive -Path "$distDir\wemod-pro-unlocker.exe", "$distDir\updater.exe" -DestinationPath $zipPath -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " Build Completed Successfully!          " -ForegroundColor Cyan
Write-Host " Output files location: $distDir        " -ForegroundColor Cyan
Get-ChildItem -Path $distDir | Select-Name Name, @{Name="Size (KB)"; Expression={[math]::Round($_.Length / 1KB, 2)}} | Format-Table -AutoSize
Write-Host "========================================" -ForegroundColor Cyan
