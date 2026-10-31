# ChangeRadar Windows Release Build Script
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build_release.ps1

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   ChangeRadar Release Packaging Tool   " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. Environment Verification
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location -Path $ProjectRoot

Write-Host "[1/6] Verifying toolchain requirements..." -ForegroundColor Yellow
$CargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $CargoCheck) {
    if (Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe") {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;C:\msys64\ucrt64\bin;$env:PATH"
    } else {
        Write-Error "Cargo not found in PATH."
    }
}
$NodeCheck = Get-Command npm -ErrorAction SilentlyContinue
if (-not $NodeCheck) {
    Write-Error "Node.js / npm not found in PATH."
}

Write-Host "  -> rustc: $(rustc --version)" -ForegroundColor Green
Write-Host "  -> node:  $(node --version)" -ForegroundColor Green
Write-Host "  -> npm:   $(npm --version)" -ForegroundColor Green

# 2. Prepare Release Output Directory
Write-Host "[2/6] Preparing release directory..." -ForegroundColor Yellow
$ReleaseDir = Join-Path $ProjectRoot "dist-release"
if (Test-Path $ReleaseDir) {
    Remove-Item -Path $ReleaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path (Join-Path $ReleaseDir "bin") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ReleaseDir "docs") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ReleaseDir "ui") -Force | Out-Null

# 3. Build Rust Binaries
Write-Host "[3/6] Building Rust binaries in release mode..." -ForegroundColor Yellow
cargo build --release -p changeradar-cli -p changeradar-daemon
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo release build failed."
}

# Copy binaries
Copy-Item (Join-Path $ProjectRoot "target\release\changeradar.exe") -Destination (Join-Path $ReleaseDir "bin\changeradar.exe")
Copy-Item (Join-Path $ProjectRoot "target\release\changeradard.exe") -Destination (Join-Path $ReleaseDir "bin\changeradard.exe")
Write-Host "  -> Copied changeradar.exe and changeradard.exe" -ForegroundColor Green

# 4. Build Frontend UI
Write-Host "[4/6] Building React desktop UI..." -ForegroundColor Yellow
Set-Location -Path (Join-Path $ProjectRoot "apps\desktop")
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Frontend build failed."
}
Set-Location -Path $ProjectRoot
Copy-Item (Join-Path $ProjectRoot "apps\desktop\dist\*") -Destination (Join-Path $ReleaseDir "ui") -Recurse
Write-Host "  -> Copied compiled UI assets to dist-release\ui" -ForegroundColor Green

# 5. Copy Documentation & Licenses
Write-Host "[5/6] Copying documentation and metadata..." -ForegroundColor Yellow
$Docs = @("README.md", "LICENSE", "PRIVACY.md", "CLI.md", "API.md", "ARCHITECTURE.md", "SECURITY.md", "TROUBLESHOOTING.md", "CHANGELOG.md")
foreach ($doc in $Docs) {
    if (Test-Path (Join-Path $ProjectRoot $doc)) {
        Copy-Item (Join-Path $ProjectRoot $doc) -Destination (Join-Path $ReleaseDir "docs\$doc")
    }
}
Copy-Item (Join-Path $ProjectRoot "LICENSE") -Destination (Join-Path $ReleaseDir "LICENSE")
Copy-Item (Join-Path $ProjectRoot "README.md") -Destination (Join-Path $ReleaseDir "README.md")

# 6. Generate Checksums
Write-Host "[6/6] Generating SHA-256 release checksums..." -ForegroundColor Yellow
$ChecksumFile = Join-Path $ReleaseDir "SHA256SUMS.txt"
$CliHash = (Get-FileHash (Join-Path $ReleaseDir "bin\changeradar.exe") -Algorithm SHA256).Hash.ToLower()
$DaemonHash = (Get-FileHash (Join-Path $ReleaseDir "bin\changeradard.exe") -Algorithm SHA256).Hash.ToLower()
"$CliHash *bin/changeradar.exe" | Out-File -FilePath $ChecksumFile -Encoding utf8
"$DaemonHash *bin/changeradard.exe" | Out-File -FilePath $ChecksumFile -Append -Encoding utf8

Write-Host "========================================" -ForegroundColor Green
Write-Host "   ChangeRadar Release Build Succeeded! " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "Artifacts location: $ReleaseDir"
Write-Host "  - bin/changeradar.exe  (CLI)"
Write-Host "  - bin/changeradard.exe (Daemon)"
Write-Host "  - ui/                  (Desktop Frontend bundle)"
Write-Host "  - docs/                (Full documentation suite)"
Write-Host "  - SHA256SUMS.txt       (Verification checksums)"
