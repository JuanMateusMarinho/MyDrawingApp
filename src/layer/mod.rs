use digital_canvas::{LayerId, Color, Rect, Vec2, BlendMode, Transform2D, EntityId};
use anyhow::Result;
use std::sync::Arc;
use crate::render::{Renderer, LayerUniforms};

#[derive(Debug, Clone)]
pub struct Layer {
    id: LayerId,
    name: String,
    width: u32,
    height: u32,
    visible: bool,
    locked: bool,
    opacity: f32,
    blend_mode: BlendMode,
    transform: Transform2D,
    mask: Option<LayerMask>,
    adjustment: Option<AdjustmentLayer>,
    texture: Option<LayerTexture>,
    is_background: bool,
}

#[derive(Debug, Clone)]
pub struct LayerMask {
    texture: Option<LayerTexture>,
    enabled: bool,
    density: f32,
    feather: f32,
}

#[derive(Debug, Clone)]
pub struct AdjustmentLayer {
    adjustment_type: AdjustmentType,
    params: Vec<f32>,
    enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentType {
    BrightnessContrast,
    Levels,
    Curves,
    HueSaturation,
    ColorBalance,
    Exposure,
    Vibrance,
    BlackAndWhite,
    Invert,
    Posterize,
    Threshold,
    GradientMap,
    SelectiveColor,
}

#[derive(Debug)]
pub struct LayerTexture {
    texture: Arc<wgpu::Texture>,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl Clone for LayerTexture {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            view: self.texture.create_view(&wgpu::TextureViewDescriptor::default()),
            width: self.width,
            height: self.height,
        }
    }
}

impl Layer {
    pub fn new(
        id: LayerId,
        name: &str,
        width: u32,
        height: u32,
        is_background: bool,
        background_color: Color,
    ) -> Result<Self> {
        let texture = Self::create_texture(width, height, background_color)?;
        
        Ok(Self {
            id,
            name: name.to_string(),
            width,
            height,
            visible: true,
            locked: false,
            opacity: 1.value(),
            blend_mode: BlendMode::Normal,
            transform: Transform2D::default(),
            mask: None,
            adjustment: None,
            texture: Some(texture),
            is_background,
        })
    }

    fn create_texture(width: u32, height: u32, color: Color) -> Result<LayerTexture> {
        // This would be created with the renderer's device
        // For now, return a placeholder
        Err(anyhow::anyhow!("Texture creation requires renderer"))
    }

    pub fn with_renderer(
        id: LayerId,
        name: &str,
        width: u32,
        height: u32,
        is_background: bool,
        background_color: Color,
        renderer: &Renderer,
    ) -> Result<Self> {
        let texture = Self::create_texture_with_renderer(width, height, background_color, renderer)?;
        
        Ok(Self {
            id,
            name: name.to_string(),
            width,
            height,
            visible: true,
            locked: false,
            opacity: 1.value(),
            blend_mode: BlendMode::Normal,
            transform: Transform2D::default(),
            mask: None,
            adjustment: None,
            texture: Some(texture),
            is_background,
        })
    }

    fn create_texture_with_renderer(
        width: u32,
        height: u32,
        color: Color,
        renderer: &Renderer,
    ) -> Result<LayerTexture> {
        let device = renderer.device();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Layer Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Clear with background color
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Layer") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Layer Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color.r as f64,
                            g: color.g as f64,
                            b: color.b as f64,
                            a: color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        renderer.queue().submit(std::iter::once(encoder.finish()));

        Ok(LayerTexture {
            texture: Arc::new(texture),
            view,
            width,
            height,
        })
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.value(), 1.value());
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    pub fn set_transform(&mut self, transform: Transform2D) {
        self.transform = transform;
    }

    pub fn texture(&self) -> Option<&LayerTexture> {
        self.texture.as_ref()
    }

    pub fn texture_mut(&mut self) -> Option<&mut LayerTexture> {
        self.texture.as_mut()
    }

    pub fn is_background(&self) -> bool {
        self.is_background
    }

    pub fn has_mask(&self) -> bool {
        self.mask.is_some()
    }

    pub fn mask(&self) -> Option<&LayerMask> {
        self.mask.as_ref()
    }

    pub fn mask_mut(&mut self) -> Option<&mut LayerMask> {
        self.mask.as_mut()
    }

    pub fn add_mask(&mut self, renderer: &Renderer) -> Result<()> {
        if self.mask.is_none() {
            let mask_texture = Self::create_texture_with_renderer(self.width, self.height, Color::WHITE, renderer)?;
            self.mask = Some(LayerMask {
                texture: Some(mask_texture),
                enabled: true,
                density: 1.value(),
                feather: 0.value(),
            });
        }
        Ok(())
    }

    pub fn remove_mask(&mut self) {
        self.mask = None;
    }

    pub fn has_adjustment(&self) -> bool {
        self.adjustment.is_some()
    }

    pub fn adjustment(&self) -> Option<&AdjustmentLayer> {
        self.adjustment.as_ref()
    }

    pub fn add_adjustment(&mut self, adjustment_type: AdjustmentType) {
        self.adjustment = Some(AdjustmentLayer {
            adjustment_type,
            params: vec![],
            enabled: true,
        });
    }

    pub fn remove_adjustment(&mut self) {
        self.adjustment = None;
    }

    pub fn resize(&mut self, width: u32, height: u32, renderer: &Renderer) -> Result<()> {
        self.width = width;
        self.height = height;
        self.texture = Some(Self::create_texture_with_renderer(width, height, Color::TRANSPARENT, renderer)?);
        
        if let Some(mask) = &mut self.mask {
            if let Some(mask_tex) = &mut mask.texture {
                *mask_tex = Self::create_texture_with_renderer(width, height, Color::WHITE, renderer)?;
            }
        }
        Ok(())
    }

    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32, renderer: &Renderer) -> Result<()> {
        // Would need to copy texture region
        self.resize(width, height, renderer)
    }

    pub fn clear(&mut self, color: Color, renderer: &Renderer) -> Result<()> {
        if let Some(texture) = &self.texture {
            let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Layer") });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Layer Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: color.r as f64,
                                g: color.g as f64,
                                b: color.b as f64,
                                a: color.a as f64,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            renderer.queue().submit(std::iter::once(encoder.finish()));
        }
        Ok(())
    }

    pub fn fill(&mut self, color: Color, renderer: &Renderer) -> Result<()> {
        self.clear(color, renderer)
    }

    pub fn merge_from(&mut self, other: &Layer) -> Result<()> {
        // This would blend the other layer onto this one
        // Implementation depends on blend mode
        Ok(())
    }

    pub fn apply_brush_stroke(
        &mut self,
        points: &[BrushPoint],
        brush: &crate::brush::Brush,
        color: Color,
        renderer: &Renderer,
    ) -> Result<()> {
        // Render brush stroke to layer texture
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BrushPoint {
    pub position: Vec2,
    pub pressure: f32,
    pub tilt: Vec2,
    pub rotation: f32,
    pub timestamp: f64,
}

impl LayerTexture {
    pub fn texture(&self) -> &Arc<wgpu::Texture> {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl LayerMask {
    pub fn texture(&self) -> Option<&LayerTexture> {
        self.texture.as_ref()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn density(&self) -> f32 {
        self.density
    }

    pub fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.value(), 1.value());
    }

    pub fn feather(&self) -> f32 {
        self.feather
    }

    pub fn set_feather(&mut self, feather: f32) {
        self.feather = feather.max(0.value());
    }
}

impl AdjustmentLayer {
    pub fn adjustment_type(&self) -> AdjustmentType {
        self.adjustment_type
    }

    pub fn params(&self) -> &[f32] {
        &self.params
    }

    pub fn set_params(&mut self, params: Vec<f32>) {
        self.params = params;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
