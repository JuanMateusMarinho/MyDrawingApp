use digital_canvas::{Color, Vec2};
use anyhow::Result;
use config::{Config, File, FileFormat};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub general: GeneralSettings,
    pub interface: InterfaceSettings,
    pub canvas: CanvasSettings,
    pub brush: BrushSettings,
    pub tablet: TabletSettings,
    pub shortcuts: ShortcutSettings,
    pub performance: PerformanceSettings,
    pub color: ColorSettings,
    pub timelapse: TimelapseSettings,
    pub autosave: AutosaveSettings,
    pub file_formats: FileFormatSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub language: String,
    pub check_updates: bool,
    pub anonymous_usage_stats: bool,
    pub reset_warnings: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            check_updates: true,
            anonymous_usage_stats: false,
            reset_warnings: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSettings {
    pub theme: Theme,
    pub ui_scale: f32,
    pub font_size: f32,
    pub show_tooltips: bool,
    pub tooltip_delay: u64,
    pub panel_opacity: f32,
    pub compact_mode: bool,
    pub show_scrollbars: bool,
    pub animate_panels: bool,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            ui_scale: 1.0,
            font_size: 13.0,
            show_tooltips: true,
            tooltip_delay: 500,
            panel_opacity: 0.95,
            compact_mode: false,
            show_scrollbars: true,
            animate_panels: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSettings {
    pub default_width: u32,
    pub default_height: u32,
    pub default_dpi: f32,
    pub default_background_color: Color,
    pub max_canvas_size: u32,
    pub show_grid: bool,
    pub grid_size: f32,
    pub grid_color: Color,
    pub show_rulers: bool,
    pub ruler_units: RulerUnits,
    pub snap_to_grid: bool,
    pub snap_to_guides: bool,
    pub canvas_rotation_snap: f32,
    pub zoom_steps: Vec<f32>,
    pub fit_to_screen_on_open: bool,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            default_width: 1920,
            default_height: 1080,
            default_dpi: 300.0,
            default_background_color: Color::WHITE,
            max_canvas_size: 16384,
            show_grid: false,
            grid_size: 50.0,
            grid_color: Color::new(0.5, 0.5, 0.5, 0.3),
            show_rulers: true,
            ruler_units: RulerUnits::Pixels,
            snap_to_grid: false,
            snap_to_guides: true,
            canvas_rotation_snap: 15.0,
            zoom_steps: vec![0.125, 0.25, 0.33, 0.5, 0.67, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 8.0, 10.0, 16.0, 25.0, 50.0, 100.0],
            fit_to_screen_on_open: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulerUnits {
    Pixels,
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
    Percent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushSettings {
    pub default_size: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub default_opacity: f32,
    pub default_flow: f32,
    pub default_hardness: f32,
    pub smoothing_enabled: bool,
    pub smoothing_factor: f32,
    pub stabilization_enabled: bool,
    pub stabilization_factor: f32,
    pub pressure_curve_enabled: bool,
    pub cursor_type: CursorType,
    pub show_brush_outline: bool,
    pub outline_color: Color,
    pub crosshair_size: f32,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            default_size: 20.0,
            min_size: 0.5,
            max_size: 5000.0,
            default_opacity: 1.0,
            default_flow: 1.0,
            default_hardness: 0.5,
            smoothing_enabled: true,
            smoothing_factor: 0.3,
            stabilization_enabled: true,
            stabilization_factor: 0.5,
            pressure_curve_enabled: true,
            cursor_type: CursorType::BrushOutline,
            show_brush_outline: true,
            outline_color: Color::new(0.0, 0.0, 0.0, 0.5),
            crosshair_size: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorType {
    BrushOutline,
    Crosshair,
    Dot,
    Precise,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabletSettings {
    pub enable_windows_ink: bool,
    pub enable_wintab: bool,
    pub pressure_curve: Vec<(f32, f32)>,
    pub click_threshold: f32,
    pub double_click_distance: f32,
    pub double_click_time: u64,
    pub pen_buttons: PenButtonMapping,
    pub touch_enabled: bool,
    pub touch_gestures: bool,
}

impl Default for TabletSettings {
    fn default() -> Self {
        Self {
            enable_windows_ink: true,
            enable_wintab: false,
            pressure_curve: vec![(0.0, 0.0), (0.25, 0.15), (0.5, 0.5), (0.75, 0.85), (1.0, 1.0)],
            click_threshold: 0.5,
            double_click_distance: 10.0,
            double_click_time: 300,
            pen_buttons: PenButtonMapping::default(),
            touch_enabled: true,
            touch_gestures: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenButtonMapping {
    pub lower_button: PenButtonAction,
    pub upper_button: PenButtonAction,
    pub eraser_tip: PenButtonAction,
    pub eraser_button: PenButtonAction,
}

impl Default for PenButtonMapping {
    fn default() -> Self {
        Self {
            lower_button: PenButtonAction::RightClick,
            upper_button: PenButtonAction::Pan,
            eraser_tip: PenButtonAction::Eraser,
            eraser_button: PenButtonAction::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenButtonAction {
    None,
    RightClick,
    Pan,
    Eraser,
    ColorPicker,
    Undo,
    BrushSize,
    BrushOpacity,
    ToggleBrush,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    pub scheme: ShortcutScheme,
    pub shortcuts: HashMap<String, Shortcut>,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        
        // Photoshop-style defaults
        let defaults = vec![
            ("tool.brush", Shortcut::new("B", false, false, false)),
            ("tool.eraser", Shortcut::new("E", false, false, false)),
            ("tool.move", Shortcut::new("V", false, false, false)),
            ("tool.marquee", Shortcut::new("M", false, false, false)),
            ("tool.lasso", Shortcut::new("L", false, false, false)),
            ("tool.magic_wand", Shortcut::new("W", false, false, false)),
            ("tool.eyedropper", Shortcut::new("I", false, false, false)),
            ("tool.gradient", Shortcut::new("G", false, false, false)),
            ("tool.crop", Shortcut::new("C", false, false, false)),
            ("tool.zoom", Shortcut::new("Z", false, false, false)),
            ("tool.hand", Shortcut::new("H", false, false, false)),
            ("tool.rotate", Shortcut::new("R", false, false, false)),
            ("tool.text", Shortcut::new("T", false, false, false)),
            ("tool.pen", Shortcut::new("P", false, false, false)),
            ("tool.clone_stamp", Shortcut::new("S", false, false, false)),
            ("tool.healing", Shortcut::new("J", false, false, false)),
            ("edit.undo", Shortcut::new("Z", true, false, false)),
            ("edit.redo", Shortcut::new("Z", true, true, false)),
            ("edit.copy", Shortcut::new("C", true, false, false)),
            ("edit.paste", Shortcut::new("V", true, false, false)),
            ("edit.cut", Shortcut::new("X", true, false, false)),
            ("edit.select_all", Shortcut::new("A", true, false, false)),
            ("edit.deselect", Shortcut::new("D", true, false, false)),
            ("edit.transform", Shortcut::new("T", true, false, false)),
            ("file.new", Shortcut::new("N", true, false, false)),
            ("file.open", Shortcut::new("O", true, false, false)),
            ("file.save", Shortcut::new("S", true, false, false)),
            ("file.save_as", Shortcut::new("S", true, true, false)),
            ("brush.size_decrease", Shortcut::new("[", false, false, false)),
            ("brush.size_increase", Shortcut::new("]", false, false, false)),
            ("brush.hardness_decrease", Shortcut::new("{", false, false, false)),
            ("brush.hardness_increase", Shortcut::new("}", false, false, false)),
            ("view.zoom_in", Shortcut::new("=", true, false, false)),
            ("view.zoom_out", Shortcut::new("-", true, false, false)),
            ("view.fit_screen", Shortcut::new("0", true, false, false)),
            ("view.actual_pixels", Shortcut::new("1", true, false, false)),
            ("view.toggle_fullscreen", Shortcut::new("F", false, false, false)),
            ("layer.new", Shortcut::new("N", true, true, false)),
            ("layer.duplicate", Shortcut::new("J", true, false, false)),
            ("layer.merge_down", Shortcut::new("E", true, false, false)),
            ("layer.group", Shortcut::new("G", true, false, false)),
        ];

        for (action, shortcut) in defaults {
            shortcuts.insert(action.to_string(), shortcut);
        }

        Self {
            scheme: ShortcutScheme::Photoshop,
            shortcuts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutScheme {
    Photoshop,
    Procreate,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Shortcut {
    pub fn new(key: &str, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key: key.to_string(),
            ctrl,
            shift,
            alt,
        }
    }

    pub fn matches(&self, key: &str, ctrl: bool, shift: bool, alt: bool) -> bool {
        self.key.eq_ignore_ascii_case(key) && self.ctrl == ctrl && self.shift == shift && self.alt == alt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub gpu_acceleration: bool,
    pub preferred_backend: GpuBackend,
    pub max_texture_size: u32,
    pub enable_multithreading: bool,
    pub thread_count: usize,
    pub memory_limit_mb: u32,
    pub cache_size_mb: u32,
    pub vsync: bool,
    pub low_latency_mode: bool,
    pub background_processing: bool,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            gpu_acceleration: true,
            preferred_backend: GpuBackend::Auto,
            max_texture_size: 8192,
            enable_multithreading: true,
            thread_count: num_cpus::get(),
            memory_limit_mb: 4096,
            cache_size_mb: 512,
            vsync: true,
            low_latency_mode: true,
            background_processing: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackend {
    Auto,
    Vulkan,
    Metal,
    Dx12,
    Gl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSettings {
    pub working_space: WorkingSpace,
    pub display_profile: Option<String>,
    pub proof_profile: Option<String>,
    pub proof_intent: RenderingIntent,
    pub black_point_compensation: bool,
    pub warn_profile_mismatch: bool,
    pub warn_missing_profile: bool,
    pub rgb_working_space: RgbWorkingSpace,
    pub cmyk_working_space: CmykWorkingSpace,
    pub gray_working_space: GrayWorkingSpace,
    pub spot_working_space: SpotWorkingSpace,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            working_space: WorkingSpace::SRGB,
            display_profile: None,
            proof_profile: None,
            proof_intent: RenderingIntent::RelativeColorimetric,
            black_point_compensation: true,
            warn_profile_mismatch: true,
            warn_missing_profile: true,
            rgb_working_space: RgbWorkingSpace::SRGB,
            cmyk_working_space: CmykWorkingSpace::USWebCoatedSWOP,
            gray_working_space: GrayWorkingSpace::GrayGamma22,
            spot_working_space: SpotWorkingSpace::DotGain20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkingSpace {
    SRGB,
    AdobeRGB,
    ProPhotoRGB,
    DisplayP3,
    Rec2020,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RgbWorkingSpace {
    SRGB,
    AdobeRGB,
    ProPhotoRGB,
    DisplayP3,
    Rec2020,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmykWorkingSpace {
    USWebCoatedSWOP,
    USWebUncoated,
    USSheetfedCoated,
    USSheetfedUncoated,
    EuropeISOCoated,
    EuropeISOUncoated,
    JapanColor2001Coated,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrayWorkingSpace {
    GrayGamma18,
    GrayGamma22,
    DotGain10,
    DotGain15,
    DotGain20,
    DotGain25,
    DotGain30,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpotWorkingSpace {
    DotGain10,
    DotGain15,
    DotGain20,
    DotGain25,
    DotGain30,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseSettings {
    pub enabled: bool,
    pub fps: u32,
    pub playback_speed: f32,
    pub export_resolution: (u32, u32),
    pub max_events: usize,
    pub capture_interval_ms: u64,
    pub include_ui: bool,
    pub include_cursor: bool,
}

impl Default for TimelapseSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            fps: 30,
            playback_speed: 1.0,
            export_resolution: (1920, 1080),
            max_events: 100000,
            capture_interval_ms: 100,
            include_ui: false,
            include_cursor: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveSettings {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub max_versions: u32,
    pub save_on_focus_loss: bool,
    pub save_on_idle: bool,
    pub idle_timeout_seconds: u32,
}

impl Default for AutosaveSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 5,
            max_versions: 10,
            save_on_focus_loss: true,
            save_on_idle: true,
            idle_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFormatSettings {
    pub default_export_format: ExportFormat,
    pub png_compression: u8,
    pub jpeg_quality: u8,
    pub webp_quality: f32,
    pub webp_lossless: bool,
    pub tiff_compression: TiffCompression,
    pub psd_compatibility: bool,
    pub include_thumbnail: bool,
    pub embed_color_profile: bool,
}

impl Default for FileFormatSettings {
    fn default() -> Self {
        Self {
            default_export_format: ExportFormat::PNG,
            png_compression: 6,
            jpeg_quality: 90,
            webp_quality: 85.0,
            webp_lossless: false,
            tiff_compression: TiffCompression::LZW,
            psd_compatibility: true,
            include_thumbnail: true,
            embed_color_profile: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    PNG,
    JPEG,
    WEBP,
    TIFF,
    BMP,
    GIF,
    PSD,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TiffCompression {
    None,
    LZW,
    PackBits,
    Deflate,
    JPEG,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("settings.toml");

        let builder = Config::builder()
            .add_source(File::new(config_path.to_str().unwrap(), FileFormat::Toml).required(false))
            .add_source(File::from_str(include_str!("default_settings.toml"), FileFormat::Toml).required(false));

        let config = builder.build()?;
        let mut settings: Settings = config.try_deserialize()?;
        
        // Apply migrations if needed
        settings.migrate();
        
        Ok(settings)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("settings.toml");
        
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(config_path, toml)?;
        
        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "digitalcanvas", "DigitalCanvas") {
            Ok(proj_dirs.config_dir().to_path_buf())
        } else {
            Ok(std::env::current_dir()?.join(".config"))
        }
    }

    fn migrate(&mut self) {
        // Apply settings migrations for version upgrades
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            interface: InterfaceSettings::default(),
            canvas: CanvasSettings::default(),
            brush: BrushSettings::default(),
            tablet: TabletSettings::default(),
            shortcuts: ShortcutSettings::default(),
            performance: PerformanceSettings::default(),
            color: ColorSettings::default(),
            timelapse: TimelapseSettings::default(),
            autosave: AutosaveSettings::default(),
            file_formats: FileFormatSettings::default(),
        }
    }
}
