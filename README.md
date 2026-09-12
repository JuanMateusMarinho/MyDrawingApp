# DigitalCanvas

Professional digital art and image editing application combining Procreate's simplicity with Photoshop's power.

## Features

- **GPU-Accelerated Canvas** - Vulkan/DirectX 12/Metal via WGPU
- **Professional Brush Engine** - 13 categories, pressure/tilt/velocity dynamics
- **Layer System** - Masks, adjustments, blend modes, groups
- **Photoshop-Compatible Shortcuts** - B, E, V, M, L, W, I, G, C, Z, H, R, etc.
- **Tablet Support** - Pressure (8192 levels), tilt, rotation, eraser
- **Timelapse Recording** - Efficient stroke-based recording
- **PSD Import/Export** - Layer preservation
- **Plugin Architecture** - Extensible via dynamic libraries

## Building

### Prerequisites

- Rust 1.75+ (stable)
- Windows 10/11 SDK
- Visual Studio 2022 Build Tools with C++ workload

```bash
# Clone and build
git clone https://github.com/digitalcanvas/digital-canvas
cd digital-canvas
cargo build --release --features vulkan
```

### Running

```bash
# Run from build directory
./target/release/digital_canvas.exe
```

## Distribution

### Portable Package

```powershell
# Build and create portable ZIP
.\build.ps1 -Portable
# Creates: dist/DigitalCanvas-Portable-Win64.zip
```

### NSIS Installer

```powershell
# Build and create NSIS installer
.\build.ps1 -Installer
# Creates: DigitalCanvas-Setup.exe
```

### Signed Release

```powershell
# Build, package, and sign
.\build.ps1 -Portable -Installer -Sign
```

## Project Structure

```
src/
├── app/           # Application entry, event loop
├── brush/         # Brush engine, dynamics, presets
├── canvas/        # Viewport, zoom, pan, rotation
├── color/         # Color spaces, blend modes, palettes
├── document/      # Document model, metadata
├── file/          # Import/export, native format, PSD
├── filter/        # Image filters (blur, sharpen, etc.)
├── history/       # Undo/redo with action grouping
├── input/         # Keyboard, mouse, tablet input
├── layer/         # Layers, masks, adjustments
├── plugin/        # Plugin system
├── psd/           # PSD import/export
├── render/        # WGPU renderer
├── selection/     # Selection tools
├── settings/      # Configuration system
├── timelapse/     # Timelapse recording/playback
├── ui/            # Slint UI
└── utils/         # Math, color, image utilities
```

## Keyboard Shortcuts (Photoshop-style)

| Key | Tool |
|-----|------|
| B | Brush |
| E | Eraser |
| V | Move |
| M | Marquee |
| L | Lasso |
| W | Magic Wand |
| I | Eyedropper |
| G | Gradient |
| C | Crop |
| Z | Zoom |
| H | Hand/Pan |
| R | Rotate Canvas |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save |
| [ / ] | Brush Size -/+ |

## Tablet Support

- Windows Ink (default)
- Wintab (optional)
- Pressure curves configurable
- Pen button mapping
- Touch gestures

## License

MIT OR Apache-2.0