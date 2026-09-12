use digital_canvas::{BrushId, Color, Vec2, EntityId};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brush {
    id: BrushId,
    name: String,
    category: BrushCategory,
    brush_type: BrushType,
    params: BrushParams,
    texture: Option<BrushTexture>,
    dual_brush: Option<DualBrush>,
    dynamics: BrushDynamics,
    stamps: Vec<BrushStamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushCategory {
    Pencil,
    Ink,
    Pen,
    Marker,
    Airbrush,
    Paint,
    Oil,
    Acrylic,
    Watercolor,
    Charcoal,
    Pastel,
    Crayon,
    Texture,
    Pixel,
    Sketch,
    Smudge,
    Eraser,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushType {
    Standard,
    Textured,
    Dual,
    Scatter,
    Pattern,
    Mixer,
    Smudge,
    Eraser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushParams {
    pub size: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,
    pub spacing: f32,
    pub rotation: f32,
    pub roundness: f32,
    pub angle: f32,
    pub scatter: f32,
    pub scatter_count: u32,
    pub jitter_size: f32,
    pub jitter_opacity: f32,
    pub jitter_flow: f32,
    pub jitter_angle: f32,
    pub jitter_roundness: f32,
    pub jitter_position: f32,
    pub smoothing: f32,
    pub stabilization: f32,
    pub pressure_size: bool,
    pub pressure_opacity: bool,
    pub pressure_flow: bool,
    pub pressure_scatter: bool,
    pub tilt_size: bool,
    pub tilt_opacity: bool,
    pub velocity_size: bool,
    pub velocity_opacity: bool,
    pub blend_mode: digital_canvas::BlendMode,
    pub wetness: f32,
    pub color_dynamics: ColorDynamics,
    pub texture: Option<BrushTexture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDynamics {
    pub hue_jitter: f32,
    pub saturation_jitter: f32,
    pub brightness_jitter: f32,
    pub purity_jitter: f32,
    pub foreground_background_jitter: f32,
    pub pressure_hue: bool,
    pub pressure_saturation: bool,
    pub pressure_brightness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub scale: f32,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    R8,
    Rg8,
    Rgba8,
    R16Float,
    Rg16Float,
    Rgba16Float,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualBrush {
    pub brush: Box<Brush>,
    pub mode: DualBrushMode,
    pub size_ratio: f32,
    pub spacing: f32,
    pub scatter: f32,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DualBrushMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Replace,
    Behind,
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushDynamics {
    pub size: DynamicsCurve,
    pub opacity: DynamicsCurve,
    pub flow: DynamicsCurve,
    pub scatter: DynamicsCurve,
    pub rotation: DynamicsCurve,
    pub roundness: DynamicsCurve,
    pub angle: DynamicsCurve,
    pub spacing: DynamicsCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsCurve {
    pub enabled: bool,
    pub curve: Vec<(f32, f32)>,
    pub source: DynamicsSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicsSource {
    Pressure,
    Tilt,
    Velocity,
    Rotation,
    Direction,
    Fade,
    Random,
}

impl Default for DynamicsCurve {
    fn default() -> Self {
        Self {
            enabled: false,
            curve: vec![(0., 0.), (1.0, 1.0)],
            source: DynamicsSource::Pressure,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushStamp {
    pub texture: BrushTexture,
    pub spacing: f32,
    pub rotation: f32,
    pub scale: f32,
    pub jitter: f32,
}

pub struct BrushEngine {
    brushes: HashMap<BrushId, Brush>,
    active_brush: Option<BrushId>,
    categories: HashMap<BrushCategory, Vec<BrushId>>,
    recent_brushes: Vec<BrushId>,
    favorites: Vec<BrushId>,
    size: f32,
    opacity: f32,
    flow: f32,
    color: Color,
    settings: BrushEngineSettings,
}

#[derive(Debug, Clone)]
pub struct BrushEngineSettings {
    pub default_size: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub smoothing_enabled: bool,
    pub stabilization_enabled: bool,
    pub pressure_curve_enabled: bool,
}

impl Default for BrushEngineSettings {
    fn default() -> Self {
        Self {
            default_size: 20.0,
            min_size: 0.5,
            max_size: 5000.0,
            smoothing_enabled: true,
            stabilization_enabled: true,
            pressure_curve_enabled: true,
        }
    }
}

impl BrushEngine {
    pub fn new(settings: &crate::settings::BrushSettings) -> Self {
        let mut engine = Self {
            brushes: HashMap::new(),
            active_brush: None,
            categories: HashMap::new(),
            recent_brushes: Vec::new(),
            favorites: Vec::new(),
            size: settings.default_size,
            opacity: 1.0,
            flow: 1.0,
            color: Color::BLACK,
            settings: BrushEngineSettings::default(),
        };

        engine.load_default_brushes();
        engine
    }

    fn load_default_brushes(&mut self) {
        // Create default brushes for each category
        let default_brushes = vec![
            ("HB Pencil", BrushCategory::Pencil, BrushType::Standard, Self::pencil_brush()),
            ("Sketching Pencil", BrushCategory::Pencil, BrushType::Textured, Self::sketching_pencil()),
            ("Inking Pen", BrushCategory::Ink, BrushType::Standard, Self::inking_pen()),
            ("Technical Pen", BrushCategory::Pen, BrushType::Standard, Self::technical_pen()),
            ("Marker", BrushCategory::Marker, BrushType::Standard, Self::marker_brush()),
            ("Airbrush", BrushCategory::Airbrush, BrushType::Standard, Self::airbrush()),
            ("Oil Paint", BrushCategory::Oil, BrushType::Textured, Self::oil_paint()),
            ("Watercolor", BrushCategory::Watercolor, BrushType::Textured, Self::watercolor()),
            ("Soft Brush", BrushCategory::Paint, BrushType::Standard, Self::soft_brush()),
            ("Hard Brush", BrushCategory::Paint, BrushType::Standard, Self::hard_brush()),
            ("Texture Brush", BrushCategory::Texture, BrushType::Textured, Self::texture_brush()),
            ("Smudge", BrushCategory::Smudge, BrushType::Smudge, Self::smudge_brush()),
            ("Eraser", BrushCategory::Eraser, BrushType::Eraser, Self::eraser_brush()),
        ];

        for (name, category, brush_type, params) in default_brushes {
            let brush = Brush {
                id: BrushId::new(),
                name: name.to_string(),
                category,
                brush_type,
                params,
                texture: None,
                dual_brush: None,
                dynamics: BrushDynamics::default(),
                stamps: vec![],
            };
            let id = brush.id;
            self.categories.entry(category).or_default().push(id);
            self.brushes.insert(id, brush);
        }

        // Set first brush as active
        if let Some(first) = self.brushes.keys().next().copied() {
            self.active_brush = Some(first);
        }
    }

    fn pencil_brush() -> BrushParams {
        BrushParams {
            size: 10.,
            min_size: 0.5,
            max_size: 100.,
            opacity: 0.8,
            flow: 0.6,
            hardness: 0.3,
            spacing: 0.5,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.,
            scatter_count: 1,
            jitter_size: 0.1,
            jitter_opacity: 0.1,
            jitter_flow: 0.5,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.5,
            smoothing: 0.3,
            stabilization: 0.2,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: false,
            tilt_size: false,
            tilt_opacity: true,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn sketching_pencil() -> BrushParams {
        let mut p = Self::pencil_brush();
        p.texture = Some(BrushTexture::paper_texture());
        p.hardness = 0.2;
        p.jitter_size = 0.2;
        p.jitter_position = 1.0;
        p
    }

    fn inking_pen() -> BrushParams {
        BrushParams {
            size: 5.0,
            min_size: 0.5,
            max_size: 50.,
            opacity: 1.0,
            flow: 1.0,
            hardness: 1.0,
            spacing: 0.2,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.,
            scatter_count: 1,
            jitter_size: 0.,
            jitter_opacity: 0.,
            jitter_flow: 0.,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.,
            smoothing: 0.8,
            stabilization: 0.5,
            pressure_size: true,
            pressure_opacity: false,
            pressure_flow: false,
            pressure_scatter: false,
            tilt_size: false,
            tilt_opacity: false,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn technical_pen() -> BrushParams {
        let mut p = Self::inking_pen();
        p.hardness = 1.0;
        p.pressure_size = false;
        p.smoothing = 0.9;
        p.stabilization = 0.7;
        p
    }

    fn marker_brush() -> BrushParams {
        BrushParams {
            size: 20.,
            min_size: 2.0,
            max_size: 200.,
            opacity: 0.4,
            flow: 0.3,
            hardness: 0.1,
            spacing: 0.1,
            rotation: 0.,
            roundness: 0.5,
            angle: 45.0,
            scatter: 0.,
            scatter_count: 1,
            jitter_size: 0.5,
            jitter_opacity: 0.1,
            jitter_flow: 0.5,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.2,
            smoothing: 0.2,
            stabilization: 0.1,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: false,
            tilt_size: true,
            tilt_opacity: true,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Multiply,
            wetness: 0.3,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn airbrush() -> BrushParams {
        BrushParams {
            size: 50.,
            min_size: 10.,
            max_size: 500.,
            opacity: 0.2,
            flow: 0.1,
            hardness: 0.,
            spacing: 0.15,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.5,
            scatter_count: 5,
            jitter_size: 0.1,
            jitter_opacity: 0.2,
            jitter_flow: 0.1,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 2.0,
            smoothing: 0.,
            stabilization: 0.,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: true,
            tilt_size: false,
            tilt_opacity: false,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn oil_paint() -> BrushParams {
        BrushParams {
            size: 30.,
            min_size: 5.0,
            max_size: 300.,
            opacity: 0.9,
            flow: 0.7,
            hardness: 0.2,
            spacing: 0.5,
            rotation: 0.,
            roundness: 0.8,
            angle: 0.,
            scatter: 0.1,
            scatter_count: 3,
            jitter_size: 0.5,
            jitter_opacity: 0.1,
            jitter_flow: 0.1,
            jitter_angle: 10.,
            jitter_roundness: 0.1,
            jitter_position: 0.3,
            smoothing: 0.3,
            stabilization: 0.2,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: false,
            tilt_size: true,
            tilt_opacity: true,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.8,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn watercolor() -> BrushParams {
        BrushParams {
            size: 40.,
            min_size: 5.0,
            max_size: 400.,
            opacity: 0.3,
            flow: 0.2,
            hardness: 0.,
            spacing: 0.1,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.2,
            scatter_count: 4,
            jitter_size: 0.1,
            jitter_opacity: 0.3,
            jitter_flow: 0.2,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.5,
            smoothing: 0.1,
            stabilization: 0.,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: true,
            tilt_size: false,
            tilt_opacity: false,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Multiply,
            wetness: 1.0,
            texture: None,\n            color_dynamics: ColorDynamics {
                hue_jitter: 0.5,
                saturation_jitter: 0.1,
                brightness_jitter: 0.1,
                purity_jitter: 0.,
                foreground_background_jitter: 0.1,
                pressure_hue: false,
                pressure_saturation: true,
                pressure_brightness: true,
            },
        }
    }

    fn soft_brush() -> BrushParams {
        BrushParams {
            size: 30.,
            min_size: 1.0,
            max_size: 1000.,
            opacity: 0.5,
            flow: 0.4,
            hardness: 0.,
            spacing: 0.5,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.,
            scatter_count: 1,
            jitter_size: 0.,
            jitter_opacity: 0.,
            jitter_flow: 0.,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.,
            smoothing: 0.2,
            stabilization: 0.1,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: false,
            tilt_size: false,
            tilt_opacity: false,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn hard_brush() -> BrushParams {
        let mut p = Self::soft_brush();
        p.hardness = 1.0;
        p.opacity = 1.0;
        p.flow = 1.0;
        p
    }

    fn texture_brush() -> BrushParams {
        let mut p = Self::soft_brush();
        p.texture = Some(BrushTexture::canvas_texture());
        p.spacing = 0.8;
        p.jitter_position = 0.5;
        p
    }

    fn smudge_brush() -> BrushParams {
        BrushParams {
            size: 25.0,
            min_size: 1.0,
            max_size: 500.,
            opacity: 0.5,
            flow: 0.3,
            hardness: 0.1,
            spacing: 0.5,
            rotation: 0.,
            roundness: 1.0,
            angle: 0.,
            scatter: 0.,
            scatter_count: 1,
            jitter_size: 0.,
            jitter_opacity: 0.,
            jitter_flow: 0.,
            jitter_angle: 0.,
            jitter_roundness: 0.,
            jitter_position: 0.,
            smoothing: 0.2,
            stabilization: 0.1,
            pressure_size: true,
            pressure_opacity: true,
            pressure_flow: true,
            pressure_scatter: false,
            tilt_size: false,
            tilt_opacity: false,
            velocity_size: false,
            velocity_opacity: false,
            blend_mode: digital_canvas::BlendMode::Normal,
            wetness: 0.5,
            color_dynamics: ColorDynamics::default(),\n            texture: None,
        }
    }

    fn eraser_brush() -> BrushParams {
        let mut p = Self::soft_brush();
        p.blend_mode = digital_canvas::BlendMode::Normal; // Eraser uses destination-out blending
        p.opacity = 1.0;
        p.flow = 1.0;
        p
    }
}

impl BrushEngine {
    pub fn active_brush(&self) -> Option<&Brush> {
        self.active_brush.and_then(|id| self.brushes.get(&id))
    }

    pub fn active_brush_mut(&mut self) -> Option<&mut Brush> {
        self.active_brush.and_then(|id| self.brushes.get_mut(&id))
    }

    pub fn set_active_brush(&mut self, id: BrushId) -> bool {
        if self.brushes.contains_key(&id) {
            self.active_brush = Some(id);
            self.add_to_recent(id);
            true
        } else {
            false
        }
    }

    pub fn active_brush_id(&self) -> Option<BrushId> {
        self.active_brush
    }

    pub fn brushes(&self) -> &HashMap<BrushId, Brush> {
        &self.brushes
    }

    pub fn brushes_by_category(&self, category: BrushCategory) -> Vec<&Brush> {
        self.categories.get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.brushes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn recent_brushes(&self) -> Vec<&Brush> {
        self.recent_brushes.iter()
            .filter_map(|id| self.brushes.get(id))
            .collect()
    }

    pub fn favorite_brushes(&self) -> Vec<&Brush> {
        self.favorites.iter()
            .filter_map(|id| self.brushes.get(id))
            .collect()
    }

    pub fn add_brush(&mut self, brush: Brush) -> BrushId {
        let id = brush.id;
        self.categories.entry(brush.category).or_default().push(id);
        self.brushes.insert(id, brush);
        id
    }

    pub fn remove_brush(&mut self, id: BrushId) -> bool {
        if let Some(brush) = self.brushes.remove(&id) {
            if let Some(list) = self.categories.get_mut(&brush.category) {
                list.retain(|&bid| bid != id);
            }
            self.recent_brushes.retain(|&bid| bid != id);
            self.favorites.retain(|&bid| bid != id);
            if self.active_brush == Some(id) {
                self.active_brush = self.brushes.keys().next().copied();
            }
            true
        } else {
            false
        }
    }

    pub fn duplicate_brush(&mut self, id: BrushId) -> Option<BrushId> {
        if let Some(brush) = self.brushes.get(&id).cloned() {
            let mut new_brush = brush;
            new_brush.id = BrushId::new();
            new_brush.name = format!("{} copy", new_brush.name);
            let new_id = self.add_brush(new_brush);
            Some(new_id)
        } else {
            None
        }
    }

    pub fn toggle_favorite(&mut self, id: BrushId) {
        if let Some(pos) = self.favorites.iter().position(|&bid| bid == id) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(id);
        }
    }

    pub fn is_favorite(&self, id: BrushId) -> bool {
        self.favorites.contains(&id)
    }

    fn add_to_recent(&mut self, id: BrushId) {
        self.recent_brushes.retain(|&bid| bid != id);
        self.recent_brushes.insert(0, id);
        if self.recent_brushes.len() > 20 {
            self.recent_brushes.truncate(20);
        }
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_size(&mut self, size: f32) {
        let new_size = size.clamp(self.settings.min_size, self.settings.max_size);
        self.size = new_size;
        if let Some(brush) = self.active_brush_mut() {
            brush.params.size = new_size;
        }
    }

    pub fn increase_size(&mut self) {
        self.set_size(self.size * 1.1);
    }

    pub fn decrease_size(&mut self) {
        self.set_size(self.size / 1.1);
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0., 1.0);
    }

    pub fn flow(&self) -> f32 {
        self.flow
    }

    pub fn set_flow(&mut self, flow: f32) {
        self.flow = flow.clamp(0., 1.0);
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn calculate_brush_params(
        &self,
        pressure: f32,
        tilt: Vec2,
        velocity: f32,
    ) -> ComputedBrushParams {
        let brush = self.active_brush().unwrap();
        let params = &brush.params;

        let mut size = params.size * self.size;
        let mut opacity = params.opacity * self.opacity;
        let mut flow = params.flow * self.flow;
        let mut scatter = params.scatter;
        let mut rotation = params.rotation;
        let mut roundness = params.roundness;

        if params.pressure_size {
            size *= pressure;
        }
        if params.pressure_opacity {
            opacity *= pressure;
        }
        if params.pressure_flow {
            flow *= pressure;
        }
        if params.pressure_scatter {
            scatter *= pressure;
        }

        // Apply dynamics curves
        size *= self.evaluate_dynamics(&brush.dynamics.size, pressure);
        opacity *= self.evaluate_dynamics(&brush.dynamics.opacity, pressure);
        flow *= self.evaluate_dynamics(&brush.dynamics.flow, pressure);
        scatter *= self.evaluate_dynamics(&brush.dynamics.scatter, pressure);

        ComputedBrushParams {
            size: size.clamp(params.min_size, params.max_size),
            opacity: opacity.clamp(0., 1.0),
            flow: flow.clamp(0., 1.0),
            hardness: params.hardness,
            spacing: params.spacing,
            rotation,
            roundness,
            scatter,
            scatter_count: params.scatter_count,
            blend_mode: params.blend_mode,
            wetness: params.wetness,
            color: self.color,
        }
    }

    fn evaluate_dynamics(&self, curve: &DynamicsCurve, input: f32) -> f32 {
        if !curve.enabled || curve.curve.is_empty() {
            return 1.0;
        }

        let t = input.clamp(0., 1.0);
        for i in 0..curve.curve.len() - 1 {
            let (x1, y1) = curve.curve[i];
            let (x2, y2) = curve.curve[i + 1];
            if t >= x1 && t <= x2 {
                let local_t = (t - x1) / (x2 - x1);
                return y1 + (y2 - y1) * local_t;
            }
        }
        1.0
    }

    pub fn generate_stroke_points(
        &self,
        start: Vec2,
        end: Vec2,
        pressure: f32,
        spacing: f32,
    ) -> Vec<BrushPoint> {
        let distance = start.distance(end);
        let step = (self.active_brush().map(|b| b.params.spacing).unwrap_or(0.5) * self.size).max(1.0);
        let count = ((distance / step) as usize).max(1);

        let mut points = Vec::with_capacity(count);
        for i in 0..=count {
            let t = if count > 1 { i as f32 / count as f32 } else { 0.5 };
            let position = Vec2::new(
                start.x + (end.x - start.x) * t,
                start.y + (end.y - start.y) * t,
            );
            points.push(BrushPoint {
                position,
                pressure,
                tilt: Vec2::zero(),
                rotation: 0.,
                timestamp: 0.,
            });
        }
        points
    }

    pub fn search_brushes(&self, query: &str) -> Vec<&Brush> {
        let query = query.to_lowercase();
        self.brushes.values()
            .filter(|b| b.name.to_lowercase().contains(&query))
            .collect()
    }

    pub fn import_brushes(&mut self, data: &[u8]) -> Result<Vec<BrushId>> {
        // Import brushes from file format
        Ok(vec![])
    }

    pub fn export_brushes(&self, ids: &[BrushId]) -> Result<Vec<u8>> {
        // Export brushes to file format
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct ComputedBrushParams {
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub hardness: f32,
    pub spacing: f32,
    pub rotation: f32,
    pub roundness: f32,
    pub scatter: f32,
    pub scatter_count: u32,
    pub blend_mode: digital_canvas::BlendMode,
    pub wetness: f32,
    pub color: Color,
}

impl Default for BrushDynamics {
    fn default() -> Self {
        Self {
            size: DynamicsCurve::default(),
            opacity: DynamicsCurve::default(),
            flow: DynamicsCurve::default(),
            scatter: DynamicsCurve::default(),
            rotation: DynamicsCurve::default(),
            roundness: DynamicsCurve::default(),
            angle: DynamicsCurve::default(),
            spacing: DynamicsCurve::default(),
        }
    }
}

impl Default for ColorDynamics {
    fn default() -> Self {
        Self {
            hue_jitter: 0.,
            saturation_jitter: 0.,
            brightness_jitter: 0.,
            purity_jitter: 0.,
            foreground_background_jitter: 0.,
            pressure_hue: false,
            pressure_saturation: false,
            pressure_brightness: false,
        }
    }
}

impl BrushTexture {
    pub fn paper_texture() -> Self {
        // Generate paper texture
        Self {
            data: vec![],
            width: 256,
            height: 256,
            format: TextureFormat::R8,
            scale: 1.0,
            repeat: true,
        }
    }

    pub fn canvas_texture() -> Self {
        Self {
            data: vec![],
            width: 256,
            height: 256,
            format: TextureFormat::R8,
            scale: 1.0,
            repeat: true,
        }
    }
}
