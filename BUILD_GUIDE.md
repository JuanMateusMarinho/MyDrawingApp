# DigitalCanvas - Windows Installer Build Guide

## Prerequisites
- Rust 1.75+ (stable)
- Windows 10/11 SDK
- Visual Studio 2022 Build Tools (for MSVC)
- WiX Toolset v3.11+ (for MSI installer)
- NSIS 3.0+ (for NSIS installer)

## Build Release Binary

```bash
# Build optimized release
cargo build --release --features vulkan

# Binary location: target/release/digital_canvas.exe
```

## Create Portable ZIP Package

```bash
# Create package directory
mkdir dist/DigitalCanvas-Portable

# Copy binary
cp target/release/digital_canvas.exe dist/DigitalCanvas-Portable/

# Copy assets
cp -r assets dist/DigitalCanvas-Portable/

# Create default settings
cp src/settings/default_settings.toml dist/DigitalCanvas-Portable/settings.toml

# Create README
cat > dist/DigitalCanvas-Portable/README.txt << 'EOF'
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
EOF

# Create ZIP
cd dist
powershell -Command "Compress-Archive -Path 'DigitalCanvas-Portable\*' -DestinationPath 'DigitalCanvas-Portable-Win64.zip' -Force"
```

## Create NSIS Installer

Create `installer.nsi`:

```nsis
!include "MUI2.nsh"

Name "DigitalCanvas"
OutFile "DigitalCanvas-Setup.exe"
InstallDir "$PROGRAMFILES64\DigitalCanvas"
InstallDirRegKey HKCU "Software\DigitalCanvas" "InstallDir"
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Section "Main"
    SetOutPath "$INSTDIR"
    File "target\release\digital_canvas.exe"
    File /r "assets\*"
    File "src\settings\default_settings.toml"
    
    WriteRegStr HKCU "Software\DigitalCanvas" "InstallDir" "$INSTDIR"
    
    CreateDirectory "$SMPROGRAMS\DigitalCanvas"
    CreateShortCut "$SMPROGRAMS\DigitalCanvas\DigitalCanvas.lnk" "$INSTDIR\digital_canvas.exe"
    CreateShortCut "$DESKTOP\DigitalCanvas.lnk" "$INSTDIR\digital_canvas.exe"
    
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR"
    Delete "$SMPROGRAMS\DigitalCanvas\*.*"
    RMDir "$SMPROGRAMS\DigitalCanvas"
    Delete "$DESKTOP\DigitalCanvas.lnk"
    DeleteRegKey HKCU "Software\DigitalCanvas"
SectionEnd
```

Build:
```bash
makensis installer.nsi
```

## Create WiX MSI Installer

Create `installer.wxs`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="DigitalCanvas" Language="1033" Version="1.0.0" Manufacturer="DigitalCanvas Team" UpgradeCode="PUT-GUID-HERE">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    <MajorUpgrade DowngradeErrorMessage="A newer version of DigitalCanvas is already installed." />
    <MediaTemplate EmbedCab="yes" />
    
    <Feature Id="ProductFeature" Title="DigitalCanvas" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
      <ComponentRef Id="ApplicationShortcut" />
      <ComponentRef Id="DesktopShortcut" />
    </Feature>
  </Product>

  <Fragment>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="INSTALLFOLDER" Name="DigitalCanvas" />
      </Directory>
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="DigitalCanvas" />
      </Directory>
      <Directory Id="DesktopFolder" Name="Desktop" />
    </Directory>
  </Fragment>

  <Fragment>
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="PUT-GUID-HERE">
        <File Id="DigitalCanvasExe" Source="target\release\digital_canvas.exe" KeyPath="yes" />
      </Component>
      <Component Id="Assets" Guid="PUT-GUID-HERE">
        <File Source="assets\*" />
      </Component>
      <Component Id="Settings" Guid="PUT-GUID-HERE">
        <File Source="src\settings\default_settings.toml" Name="settings.toml" />
      </Component>
    </ComponentGroup>
  </Fragment>

  <Fragment>
    <Component Id="ApplicationShortcut" Directory="ApplicationProgramsFolder">
      <Shortcut Id="ApplicationStartMenuShortcut" Name="DigitalCanvas" Description="Professional Digital Art Application" Target="[INSTALLFOLDER]digital_canvas.exe" WorkingDirectory="INSTALLFOLDER" />
      <RemoveFolder Id="CleanupApplicationProgramsFolder" On="uninstall" />
      <RegistryValue Root="HKCU" Key="Software\DigitalCanvas" Name="installed" Type="integer" Value="1" KeyPath="yes" />
    </Component>
    <Component Id="DesktopShortcut" Directory="DesktopFolder">
      <Shortcut Id="ApplicationDesktopShortcut" Name="DigitalCanvas" Description="Professional Digital Art Application" Target="[INSTALLFOLDER]digital_canvas.exe" WorkingDirectory="INSTALLFOLDER" />
      <RemoveFolder Id="CleanupDesktopFolder" On="uninstall" />
    </Component>
  </Fragment>
</Wix>
```

Build:
```bash
candle.exe installer.wxs
light.exe installer.wixobj -ext WixUIExtension -o DigitalCanvas.msi
```

## GitHub Actions CI/CD

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  release:
    types: [published]

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          target: x86_64-pc-windows-msvc
          
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
        
      - name: Build release
        run: cargo build --release --features vulkan
        
      - name: Create portable ZIP
        run: |
          mkdir dist/DigitalCanvas-Portable
          cp target/release/digital_canvas.exe dist/DigitalCanvas-Portable/
          cp -r assets dist/DigitalCanvas-Portable/
          cp src/settings/default_settings.toml dist/DigitalCanvas-Portable/settings.toml
          Compress-Archive -Path "dist/DigitalCanvas-Portable/*" -DestinationPath "DigitalCanvas-Portable-Win64.zip" -Force
          
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: DigitalCanvas-Win64
          path: |
            target/release/digital_canvas.exe
            DigitalCanvas-Portable-Win64.zip
```

## Code Signing (Production)

```bash
# Sign the executable
signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /a target/release/digital_canvas.exe

# Sign the installer
signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /a DigitalCanvas-Setup.exe
```

## Distribution Checklist

- [ ] Build passes all tests
- [ ] Binary runs on clean Windows 10/11 VM
- [ ] Tablet input works (Wacom, Huion, XP-Pen)
- [ ] GPU acceleration works (Vulkan/DX12)
- [ ] Portable ZIP extracts and runs
- [ ] NSIS installer installs/uninstalls cleanly
- [ ] MSI installer installs/uninstalls cleanly
- [ ] Code signing applied (production)
- [ ] Checksums published (SHA256)
- [ ] Release notes written
- [ ] GitHub release created with assets