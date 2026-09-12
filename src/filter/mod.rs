use digital_canvas::{Document, Layer, LayerId, Color, Rect, EntityId};
use anyhow::Result;
use image::{RgbaImage, ImageBuffer, Rgba};
use std::sync::Arc;

pub struct FilterEngine {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterEngine {
    pub fn new() -> Self {
        let mut engine = Self { filters: Vec::new() };
        engine.register_builtin_filters();
        engine
    }

    fn register_builtin_filters(&mut self) {
        self.filters.push(Box::new(GaussianBlur::default()));
        self.filters.push(Box::new(MotionBlur::default()));
        self.filters.push(Box::new(BoxBlur::default()));
        self.filters.push(Box::new(Sharpen::default()));
        self.filters.push(Box::new(Noise::default()));
        self.filters.push(Box::new(ReduceNoise::default()));
        self.filters.push(Box::new(HighPass::default()));
        self.filters.push(Box::new(ColorAdjustment::default()));
        self.filters.push(Box::new(Distortion::default()));
        self.filters.push(Box::new(Pixelate::default()));
    }

    pub fn apply_filter(&self, filter_name: &str, layer: &mut Layer, params: &FilterParams, renderer: &crate::render::Renderer) -> Result<()> {
        if let Some(filter) = self.filters.iter().find(|f| f.name() == filter_name) {
            filter.apply(layer, params, renderer)
        } else {
            Err(anyhow::anyhow!("Filter not found: {}", filter_name))
        }
    }

    pub fn filter_names(&self) -> Vec<&str> {
        self.filters.iter().map(|f| f.name()).collect()
    }

    pub fn filter_info(&self, name: &str) -> Option<FilterInfo> {
        self.filters.iter().find(|f| f.name() == name).map(|f| f.info())
    }

    pub fn register_filter(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }
}

pub trait Filter: Send + Sync {
    fn name(&self) -> &str;
    fn info(&self) -> FilterInfo;
    fn apply(&self, layer: &mut Layer, params: &FilterParams, renderer: &crate::render::Renderer) -> Result<()>;
    fn default_params(&self) -> FilterParams;
    fn validate_params(&self, params: &FilterParams) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterInfo {
    pub name: String,
    pub description: String,
    pub category: FilterCategory,
    pub params: Vec<FilterParamInfo>,
    pub real_time_preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterCategory {
    Blur,
    Sharpen,
    Noise,
    Distortion,
    Color,
    Artistic,
    Stylize,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParamInfo {
    pub name: String,
    pub label: String,
    pub param_type: FilterParamType,
    pub default_value: serde_json::Value,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub step: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterParamType {
    Float,
    Int,
    Bool,
    Color,
    Enum,
    Vector2,
    Vector3,
    Curve,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterParams {
    pub values: std::collections::HashMap<String, FilterParamValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterParamValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Color(Color),
    Enum(String),
    Vector2(f32, f32),
    Vector3(f32, f32, f32),
    Curve(Vec<(f32, f32)>),
}

impl FilterParams {
    pub fn new() -> Self {
        Self { values: std::collections::HashMap::new() }
    }

    pub fn get_float(&self, name: &str) -> f32 {
        match self.values.get(name) {
            Some(FilterParamValue::Float(v)) => *v,
            Some(FilterParamValue::Int(v)) => *v as f32,
            _ => 0.0,
        }
    }

    pub fn get_int(&self, name: &str) -> i32 {
        match self.values.get(name) {
            Some(FilterParamValue::Int(v)) => *v,
            Some(FilterParamValue::Float(v)) => *v as i32,
            _ => 0,
        }
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.values.get(name) {
            Some(FilterParamValue::Bool(v)) => *v,
            _ => false,
        }
    }

    pub fn get_color(&self, name: &str) -> Color {
        match self.values.get(name) {
            Some(FilterParamValue::Color(v)) => *v,
            _ => Color::WHITE,
        }
    }

    pub fn set(&mut self, name: &str, value: impl Into<FilterParamValue>) {
        self.values.insert(name.to_string(), value.into());
    }
}

impl From<f32> for FilterParamValue {
    fn from(v: f32) -> Self { FilterParamValue::Float(v) }
}
impl From<i32> for FilterParamValue {
    fn from(v: i32) -> Self { FilterParamValue::Int(v) }
}
impl From<bool> for FilterParamValue {
    fn from(v: bool) -> Self { FilterParamValue::Bool(v) }
}
impl From<Color> for FilterParamValue {
    fn from(v: Color) -> Self { FilterParamValue::Color(v) }
}
impl From<(f32, f32)> for FilterParamValue {
    fn from(v: (f32, f32)) -> Self { FilterParamValue::Vector2(v.0, v.1) }
}
impl From<(f32, f32, f32)> for FilterParamValue {
    fn from(v: (f32, f32, f32)) -> Self { FilterParamValue::Vector3(v.0, v.1, v.2) }
}

macro_rules! filter_struct {
    ($name:ident, $category:expr, $params:expr) => {
        #[derive(Default)]
        pub struct $name;

        impl Filter for $name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn info(&self) -> FilterInfo {
                FilterInfo {
                    name: stringify!($name).to_string(),
                    description: stringify!($name).to_string(),
                    category: $category,
                    params: $params,
                    real_time_preview: true,
                }
            }

            fn apply(&self, layer: &mut Layer, params: &FilterParams, renderer: &crate::render::Renderer) -> Result<()> {
                Self::apply_impl(layer, params, renderer)
            }

            fn default_params(&self) -> FilterParams {
                Self::default_params_impl()
            }

            fn validate_params(&self, params: &FilterParams) -> Result<()> {
                Self::validate_params_impl(params)
            }
        }

        impl $name {
            fn apply_impl(layer: &mut Layer, params: &FilterParams, renderer: &crate::render::Renderer) -> Result<()> {
                // Implementation would use compute shaders or CPU fallback
                Ok(())
            }

            fn default_params_impl() -> FilterParams {
                let mut p = FilterParams::new();
                $(
                    p.set($0.0, $0.1);
                )*
                p
            }

            fn validate_params_impl(_params: &FilterParams) -> Result<()> {
                Ok(())
            }
        }
    };
}

// Gaussian Blur
filter_struct!(GaussianBlur, FilterCategory::Blur, vec![
    FilterParamInfo { name: "radius".to_string(), label: "Radius".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(5.0), min_value: Some(serde_json::json!(0.1)), max_value: Some(serde_json::json!(100.0)), step: Some(0.1) },
    FilterParamInfo { name: "sigma".to_string(), label: "Sigma".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(1.0), min_value: Some(serde_json::json!(0.1)), max_value: Some(serde_json::json!(50.0)), step: Some(0.1) },
]);

// Motion Blur
filter_struct!(MotionBlur, FilterCategory::Blur, vec![
    FilterParamInfo { name: "distance".to_string(), label: "Distance".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(10.0), min_value: Some(serde_json::json!(0.0)), max_value: Some(serde_json::json!(500.0)), step: Some(1.0) },
    FilterParamInfo { name: "angle".to_string(), label: "Angle".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-180.0)), max_value: Some(serde_json::json!(180.0)), step: Some(1.0) },
]);

// Box Blur
filter_struct!(BoxBlur, FilterCategory::Blur, vec![
    FilterParamInfo { name: "radius".to_string(), label: "Radius".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(3), min_value: Some(serde_json::json!(1)), max_value: Some(serde_json::json!(50)), step: Some(1.0) },
]);

// Sharpen
filter_struct!(Sharpen, FilterCategory::Sharpen, vec![
    FilterParamInfo { name: "amount".to_string(), label: "Amount".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(1.0), min_value: Some(serde_json::json!(0.0)), max_value: Some(serde_json::json!(5.0)), step: Some(0.1) },
    FilterParamInfo { name: "radius".to_string(), label: "Radius".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(1.0), min_value: Some(serde_json::json!(0.1)), max_value: Some(serde_json::json!(10.0)), step: Some(0.1) },
    FilterParamInfo { name: "threshold".to_string(), label: "Threshold".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(0), min_value: Some(serde_json::json!(0)), max_value: Some(serde_json::json!(255)), step: Some(1.0) },
]);

// Noise
filter_struct!(Noise, FilterCategory::Noise, vec![
    FilterParamInfo { name: "amount".to_string(), label: "Amount".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.1), min_value: Some(serde_json::json!(0.0)), max_value: Some(serde_json::json!(1.0)), step: Some(0.01) },
    FilterParamInfo { name: "monochromatic".to_string(), label: "Monochromatic".to_string(), param_type: FilterParamType::Bool, default_value: serde_json::json!(false), min_value: None, max_value: None, step: None },
    FilterParamInfo { name: "distribution".to_string(), label: "Distribution".to_string(), param_type: FilterParamType::Enum, default_value: serde_json::json!("gaussian"), min_value: None, max_value: None, step: None },
]);

// Reduce Noise
filter_struct!(ReduceNoise, FilterCategory::Noise, vec![
    FilterParamInfo { name: "strength".to_string(), label: "Strength".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(5), min_value: Some(serde_json::json!(1)), max_value: Some(serde_json::json!(10)), step: Some(1.0) },
    FilterParamInfo { name: "preserve_details".to_string(), label: "Preserve Details".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(50), min_value: Some(serde_json::json!(0)), max_value: Some(serde_json::json!(100)), step: Some(1.0) },
    FilterParamInfo { name: "reduce_color_noise".to_string(), label: "Reduce Color Noise".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(50), min_value: Some(serde_json::json!(0)), max_value: Some(serde_json::json!(100)), step: Some(1.0) },
    FilterParamInfo { name: "sharpen_details".to_string(), label: "Sharpen Details".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(0), min_value: Some(serde_json::json!(0)), max_value: Some(serde_json::json!(100)), step: Some(1.0) },
]);

// High Pass
filter_struct!(HighPass, FilterCategory::Sharpen, vec![
    FilterParamInfo { name: "radius".to_string(), label: "Radius".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(10.0), min_value: Some(serde_json::json!(0.1)), max_value: Some(serde_json::json!(500.0)), step: Some(0.1) },
    FilterParamInfo { name: "blend_mode".to_string(), label: "Blend Mode".to_string(), param_type: FilterParamType::Enum, default_value: serde_json::json!("overlay"), min_value: None, max_value: None, step: None },
]);

// Color Adjustment
filter_struct!(ColorAdjustment, FilterCategory::Color, vec![
    FilterParamInfo { name: "brightness".to_string(), label: "Brightness".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-1.0)), max_value: Some(serde_json::json!(1.0)), step: Some(0.01) },
    FilterParamInfo { name: "contrast".to_string(), label: "Contrast".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-1.0)), max_value: Some(serde_json::json!(1.0)), step: Some(0.01) },
    FilterParamInfo { name: "saturation".to_string(), label: "Saturation".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-1.0)), max_value: Some(serde_json::json!(1.0)), step: Some(0.01) },
    FilterParamInfo { name: "hue".to_string(), label: "Hue".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-180.0)), max_value: Some(serde_json::json!(180.0)), step: Some(1.0) },
    FilterParamInfo { name: "temperature".to_string(), label: "Temperature".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-100.0)), max_value: Some(serde_json::json!(100.0)), step: Some(1.0) },
    FilterParamInfo { name: "tint".to_string(), label: "Tint".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-100.0)), max_value: Some(serde_json::json!(100.0)), step: Some(1.0) },
]);

// Distortion
filter_struct!(Distortion, FilterCategory::Distortion, vec![
    FilterParamInfo { name: "type".to_string(), label: "Type".to_string(), param_type: FilterParamType::Enum, default_value: serde_json::json!("pinch"), min_value: None, max_value: None, step: None },
    FilterParamInfo { name: "amount".to_string(), label: "Amount".to_string(), param_type: FilterParamType::Float, default_value: serde_json::json!(0.0), min_value: Some(serde_json::json!(-1.0)), max_value: Some(serde_json::json!(1.0)), step: Some(0.01) },
    FilterParamInfo { name: "center".to_string(), label: "Center".to_string(), param_type: FilterParamType::Vector2, default_value: serde_json::json!([0.5, 0.5]), min_value: None, max_value: None, step: None },
]);

// Pixelate
filter_struct!(Pixelate, FilterCategory::Stylize, vec![
    FilterParamInfo { name: "cell_size".to_string(), label: "Cell Size".to_string(), param_type: FilterParamType::Int, default_value: serde_json::json!(10), min_value: Some(serde_json::json!(1)), max_value: Some(serde_json::json!(200)), step: Some(1.0) },
    FilterParamInfo { name: "grid_type".to_string(), label: "Grid Type".to_string(), param_type: FilterParamType::Enum, default_value: serde_json::json!("square"), min_value: None, max_value: None, step: None },
]);