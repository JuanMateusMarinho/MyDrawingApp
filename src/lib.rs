pub mod app;
pub mod brush;
pub mod canvas;
pub mod color;
pub mod document;
pub mod filter;
pub mod history;
pub mod input;
pub mod layer;
pub mod plugin;
pub mod psd;
pub mod render;
pub mod selection;
pub mod settings;
pub mod timelapse;
pub mod ui;
pub mod utils;
pub mod file;

pub use app::Application;
pub use canvas::Canvas;
pub use document::Document;
pub use layer::Layer;
pub use brush::BrushEngine;
pub use render::Renderer;
pub use input::InputManager;
pub use color::Color;
pub use history::HistoryManager;
pub use timelapse::TimelapseRecorder;
pub use settings::Settings;
pub use file::FileManager;
pub use selection::SelectionManager;
pub use filter::FilterEngine;
pub use render::LayerUniforms;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

pub type LayerId = EntityId;
pub type DocumentId = EntityId;
pub type BrushId = EntityId;
pub type SelectionId = EntityId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, point: glam::Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl From<glam::Vec2> for Vec2 {
    fn from(v: glam::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec2> for glam::Vec2 {
    fn from(v: Vec2) -> Self {
        glam::Vec2::new(v.x, v.y)
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn distance(&self, other: Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<glam::Vec3> for Vec3 {
    fn from(v: glam::Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Vec3> for glam::Vec3 {
    fn from(v: Vec3) -> Self {
        glam::Vec3::new(v.x, v.y, v.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<glam::Vec4> for Vec4 {
    fn from(v: glam::Vec4) -> Self {
        Self { x: v.x, y: v.y, z: v.z, w: v.w }
    }
}

impl From<Vec4> for glam::Vec4 {
    fn from(v: Vec4) -> Self {
        glam::Vec4::new(v.x, v.y, v.z, v.w)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Darken,
    Lighten,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl BlendMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::HardLight => "Hard Light",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::Difference => "Difference",
            BlendMode::Exclusion => "Exclusion",
            BlendMode::Hue => "Hue",
            BlendMode::Saturation => "Saturation",
            BlendMode::Color => "Color",
            BlendMode::Luminosity => "Luminosity",
        }
    }

    pub fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::SoftLight,
            BlendMode::HardLight,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub skew: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            skew: Vec2::zero(),
        }
    }
}

impl Transform2D {
    pub fn to_matrix(&self) -> glam::Mat3 {
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        glam::Mat3::from_cols(
            glam::Vec3::new(cos * self.scale.x, sin * self.scale.x, 0.0),
            glam::Vec3::new(-sin * self.scale.y, cos * self.scale.y, 0.0),
            glam::Vec3::new(self.translation.x, self.translation.y, 1.0),
        )
    }
}

pub fn initialize_logging() {
    use tracing_subscriber::{fmt, EnvFilter};
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("digital_canvas=debug".parse().unwrap()))
        .init();
}

pub fn setup_panic_hook() {
    color_eyre::install().ok();
}