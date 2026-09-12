# DigitalCanvas Build Script
# Run this after fixing compilation errors

param(
    [switch]$Portable,
    [switch]$Installer,
    [switch]$Sign
)

$ErrorActionPreference = "Stop"

Write-Host "Building DigitalCanvas..." -ForegroundColor Green

# Build release
cargo build --release --features vulkan

$exePath = "target/release/digital_canvas.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "Build failed - executable not found at $exePath"
    exit 1
}

Write-Host "Build successful!" -ForegroundColor Green

if ($Portable) {
    Write-Host "Creating portable package..." -ForegroundColor Cyan
    
    $packageDir = "dist/DigitalCanvas-Portable"
    if (Test-Path $packageDir) {
        Remove-Item -Recurse -Force $packageDir
    }
    New-Item -ItemType Directory -Path $packageDir | Out-Null
    
    # Copy executable
    Copy-Item $exePath -Destination "$packageDir/digital_canvas.exe"
    
    # Copy assets
    if (Test-Path "assets") {
        Copy-Item -Path "assets" -Destination "$packageDir/assets" -Recurse
    }
    
    # Copy default settings
    Copy-Item "src/settings/default_settings.toml" -Destination "$packageDir/settings.toml"
    
    # Create README
    @"
DigitalCanvas - Professional Digital Art Application
====================================================

Portable version - no installation required.

1. Extract this ZIP to any folder
2. Run digital_canvas.exe
3. Start creating!

System Requirements:
- Windows 10/11 (64-bit)
- DirectX 12 compatible GPU
- 4GB+ RAM (8GB recommended)
- Drawing tablet supported (Wacom, Huion, XP-Pen, etc.)

For more information, visit: https://github.com/digitalcanvas/digital-canvas
"@ | Out-File -Encoding UTF8 "$packageDir/README.txt"
    
    # Create ZIP
    $zipPath = "dist/DigitalCanvas-Portable-Win64.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath }
    Compress-Archive -Path "$packageDir/*" -DestinationPath $zipPath -Force
    
    Write-Host "Portable package created: $zipPath" -ForegroundColor Green
}

if ($Installer) {
    Write-Host "Building NSIS installer..." -ForegroundColor Cyan
    
    # Check for NSIS
    $nsisPath = "${env:ProgramFiles}\NSIS\makensis.exe"
    if (-not (Test-Path $nsisPath)) {
        $nsisPath = "${env:ProgramFiles(x86)}\NSIS\makensis.exe"
    }
    
    if (Test-Path $nsisPath) {
        & $nsisPath installer.nsi
        Write-Host "NSIS installer created" -ForegroundColor Green
    } else {
        Write-Warning "NSIS not found. Skipping installer creation."
        Write-Host "Install NSIS from https://nsis.sourceforge.io/" -ForegroundColor Yellow
    }
}

if ($Sign) {
    Write-Host "Signing binaries..." -ForegroundColor Cyan
    
    $signtool = "signtool.exe"
    $timestampUrl = "http://timestamp.digicert.com"
    
    if (Get-Command $signtool -ErrorAction SilentlyContinue) {
        & $signtool sign /fd sha256 /tr $timestampUrl /td sha256 /a $exePath
        Write-Host "Executable signed" -ForegroundColor Green
        
        if ($Installer -and (Test-Path "DigitalCanvas-Setup.exe")) {
            & $signtool sign /fd sha256 /tr $timestampUrl /td sha256 /a DigitalCanvas-Setup.exe
            Write-Host "Installer signed" -ForegroundColor Green
        }
    } else {
        Write-Warning "signtool not found. Skipping code signing."
    }
}

Write-Host "`nDone!" -ForegroundColor Green