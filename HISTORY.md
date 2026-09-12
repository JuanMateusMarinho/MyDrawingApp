# DigitalCanvas - Development History & Remaining Tasks

## Project Status: Architecture Complete, Blocked by Dependency Conflict

The core architecture is fully implemented with all 17 modules. **Currently blocked by a fundamental dependency conflict** that prevents compilation.

### Current Blocking Issue (Must Resolve First)

**Fundamental Dependency Conflict: slint 1.17.1 requires image 0.25.10, but codebase uses image 0.20 API**

- slint 1.17.1 transitively depends on image 0.25.10 via i-slint-core
- Codebase was written for image 0.20 API (different module structure)
- Cargo's workspace resolver requires a single version of `image`
- All attempts to patch/force image 0.20 failed due to transitive dependency from slint

**Recommended Resolution: Downgrade slint to version compatible with image ≤0.24**

```toml
slint = { version = "1.2", features = ["renderer-winit-femtovg", "std"] }
```

This avoids the version conflict entirely. Estimated effort: 1-2 hours to update slint imports/API.

---

## Critical Compilation Errors (Must Fix After Dependency Resolution)

### 1. wgpu 23 API Changes
- [ ] `wgpu::CullMode` - moved to `wgpu::CullMode::Back` etc. (enum variant syntax)
- [ ] `Instance::new()` - takes `InstanceDescriptor` by value, not reference
- [ ] `DeviceExt::create_buffer_init` - needs `use wgpu::util::DeviceExt;`
- [ ] `TextureView` - doesn't implement `Clone`, need custom clone for LayerTexture
- [ ] `SurfaceConfiguration` - field changes

### 2. slint 1.17 API Changes
- [ ] `Weak::upgrade()` - moves the weak reference, can't reuse in multiple closures
- [ ] Property accessors - `set_brush_size()`, `set_document_title()` may have different names
- [ ] Callback registration - `on_tool_selected()` etc. need correct signatures

### 3. image 0.25 API Changes
- [ ] `RgbaImage`, `ImageBuffer`, `Rgba` - module paths changed
- [ ] `DynamicImage` - import path changed
- [ ] `FilterType::Lanczos3` - may be renamed

### 4. psd 0.3 API Changes
- [ ] `Psd::from_bytes()` instead of `from_file()`
- [ ] Layer blend mode is string ("norm", "mul ") not enum
- [ ] `layer.blend_mode()` returns &str

### 5. winit 0.30 API Changes
- [ ] `Modifiers::state()` returns `ModifiersState` with different methods
- [ ] `PhysicalKey::Code(key_code)` pattern matching needed
- [ ] `EventLoop::run_app()` takes ownership, not reference
- [ ] `ControlFlow::Poll` -> `ControlFlow::Poll` (same but check)

### 6. EntityId Field Access
- [ ] All `.0` access changed to `.value()` method
- [ ] Update all modules: history, layer, document, brush, file, timelapse, plugin, selection

### 7. BrushParams Missing Field
- [ ] Add `texture: Option<BrushTexture>` field to all brush preset constructors
- [ ] Add `texture: None,` to brush param structs

### 8. Settings Access
- [ ] `settings.document` -> `settings.canvas` (field renamed)
- [ ] `settings.brush` exists but check field names

### 9. Copy Trait Issues
- [ ] `LayerTexture` - implement manual `Clone` (TextureView not Clone)
- [ ] `Document::ColorMode` - add `#[derive(Default)]`
- [ ] `AdjustmentType` - add serde derives

### 10. Ownership/Borrowing
- [ ] `Application::run()` - take `self` not `&mut self`
- [ ] `UiManager` callbacks - create new `weak` for each closure
- [ ] `BrushEngine::set_size()` - borrow conflict

---

## Core Features to Complete (After Compilation)

### Rendering System
- [ ] Implement actual brush stroke rendering to layer textures
- [ ] Layer compositing with blend modes in fragment shader
- [ ] Selection rendering (marching ants)
- [ ] Canvas grid and rulers
- [ ] High-DPI display support

### Brush Engine
- [ ] Pressure curve editor UI
- [ ] Brush texture loading (paper, canvas textures)
- [ ] Dual brush rendering
- [ ] Smudge tool implementation
- [ ] Color dynamics (hue/saturation/brightness jitter)

### Layer System
- [ ] Layer masks (add, edit, disable, feather)
- [ ] Clipping masks
- [ ] Adjustment layers (Brightness/Contrast, Levels, Curves, Hue/Saturation)
- [ ] Layer groups with passthrough blend mode
- [ ] Layer effects (drop shadow, stroke, etc.)

### Document System
- [ ] Native format (.dcanvas) with full serialization
- [ ] PSD import with full layer structure
- [ ] PSD export (basic)
- [ ] Auto-save with crash recovery
- [ ] Document templates

### Selection Tools
- [ ] Rectangle/Ellipse selection
- [ ] Lasso/Polygon selection
- [ ] Magic Wand (flood fill)
- [ ] Quick Selection (brush-based)
- [ ] Select by Color
- [ ] Feather, Expand, Contract, Feather
- [ ] Selection to path, Path to selection

### Image Adjustments
- [ ] Brightness/Contrast
- [ ] Levels
- [ ] Curves
- [ ] Hue/Saturation
- [ ] Color Balance
- [ ] Exposure
- [ ] Vibrance
- [ ] Black & White
- [ ] Invert, Posterize, Threshold
- [ ] Gradient Map
- [ ] Selective Color

### Filters
- [ ] Gaussian Blur (GPU compute shader)
- [ ] Motion Blur
- [ ] Box Blur
- [ ] Sharpen / Unsharp Mask
- [ ] Add Noise
- [ ] Reduce Noise
- [ ] High Pass
- [ ] Lens Distortion
- [ ] Pixelate

### Color Management
- [ ] ICC profile support (lcms2)
- [ ] sRGB, AdobeRGB, ProPhotoRGB, Display P3
- [ ] Soft proofing
- [ ] Color picker with gamut warning

### Text Tools
- [ ] Text layers (editable)
- [ ] Font selection, size, style
- [ ] Text on path
- [ ] Vertical text

### Transform Tools
- [ ] Free Transform (Ctrl+T)
- [ ] Perspective Transform
- [ ] Warp
- [ ] Flip Horizontal/Vertical
- [ ] Rotate 90° CW/CCW

### Retouch Tools
- [ ] Clone Stamp
- [ ] Healing Brush
- [ ] Spot Healing
- [ ] Dodge/Burn
- [ ] Blur/Sharpen/Smudge brushes

### Timelapse
- [ ] Efficient stroke recording (not frame-based)
- [ ] Playback controls
- [ ] Video export (FFmpeg integration)
- [ ] Configurable FPS, resolution, speed

### File Formats
- [ ] PNG (with compression level)
- [ ] JPEG (quality, progressive)
- [ ] WebP (lossy/lossless)
- [ ] TIFF (LZW, ZIP compression)
- [ ] BMP
- [ ] GIF
- [ ] PSD (import priority, export basic)
- [ ] Native .dcanvas (full fidelity)

### UI/UX
- [ ] Dark/Light/System themes
- [ ] Customizable workspace (dockable panels)
- [ ] Fullscreen canvas mode (Tab to toggle)
- [ ] Distraction-free mode
- [ ] Keyboard shortcut editor
- [ ] Tablet configuration panel
- [ ] Brush library with search/favorites
- [ ] Color palette management
- [ ] History panel with thumbnails
- [ ] Layer panel with thumbnails, drag-reorder
- [ ] Property panel (context-sensitive)
- [ ] Status bar (zoom, color, coordinates)
- [ ] Rulers with guides
- [ ] Snap to guides/grid

### Input System
- [ ] Windows Ink (default)
- [ ] Wintab support (optional)
- [ ] Pressure curve editor (graph)
- [ ] Pen button mapping UI
- [ ] Touch gestures (pinch zoom, two-finger pan)
- [ ] Keyboard shortcut customization

### Performance
- [ ] Multi-threaded filter processing
- [ ] Tile-based rendering for large canvases
- [ ] GPU compute shaders for filters
- [ ] Lazy layer thumbnail generation
- [ ] Background auto-save
- [ ] Memory pooling for brush strokes

### Platform Support
- [ ] Windows: .exe + NSIS installer + MSI
- [ ] Linux: AppImage + .deb + .rpm
- [ ] macOS: .app bundle + DMG (when Metal backend works)

### Testing & Quality
- [ ] Unit tests for core math (color, transform, brush)
- [ ] Integration tests for document save/load
- [ ] Benchmark rendering performance
- [ ] Tablet input testing (CI with virtual tablet)
- [ ] Memory leak detection
- [ ] Crash reporting (sentry/rust-minidump)

### Documentation
- [ ] User manual (Markdown + in-app help)
- [ ] Keyboard shortcut reference
- [ ] Brush creation guide
- [ ] Plugin API documentation
- [ ] Build instructions per platform

---

## Version Milestones

### v0.1.0 - MVP (Core Drawing)
- [x] Project structure
- [x] Module architecture
- [x] Basic UI
- [ ] Compilation fixes
- [ ] Basic brush rendering
- [ ] Layer visibility/opacity
- [ ] Save/Load PNG
- [ ] Basic undo/redo

### v0.2.0 - Professional Drawing
- [ ] Full brush engine
- [ ] Layer masks
- [ ] Selection tools
- [ ] Transform tools
- [ ] Tablet pressure/tilt
- [ ] Brush library UI

### v0.3.0 - Image Editing
- [ ] Adjustment layers
- [ ] Filters (GPU)
- [ ] Retouch tools
- [ ] Text tool
- [ ] PSD import
- [ ] Color management

### v0.4.0 - Timelapse & Export
- [ ] Timelapse recording
- [ ] Video export
- [ ] All export formats
- [ ] Batch export

### v1.0.0 - Production Release
- [ ] Windows installer
- [ ] Linux AppImage
- [ ] macOS .app
- [ ] Plugin system
- [ ] Full documentation
- [ ] Code signing
- [ ] Auto-updater

---

## Known Issues Tracker

| Issue | Module | Priority | Status |
|-------|--------|----------|--------|
| slint/image version conflict | all | **Critical (Blocking)** | Open |
| wgpu CullMode enum | render | Critical | Open |
| slint Weak reuse | ui | Critical | Open |
| psd BlendMode string | psd | High | Open |
| EntityId .value() | all | Critical | In Progress |
| BrushParams texture | brush | High | Open |
| LayerTexture Clone | layer | High | Open |
| DocumentId private | lib | Critical | Fixed |
| LayerUniforms private | render | Critical | Fixed |

---

## Notes for Contributors

1. **Run `cargo check` frequently** - catches errors early
2. **Use `cargo update`** - keep dependencies current but pin in Cargo.lock
3. **Test on target platforms** - Windows (Vulkan/DX12), Linux (Vulkan), macOS (Metal)
4. **Profile rendering** - use `cargo flamegraph` or `tracy`
5. **Follow semver** - breaking changes only in major versions

---

*Last Updated: 2026-09-11*
*Run `cargo check 2>&1 | grep -c "error\["` to track error count*

**Current Status: BLOCKED** - Need to downgrade slint from 1.17.1 to ~1.2 to resolve image version conflict. All other compilation issues are secondary to this fundamental dependency conflict.