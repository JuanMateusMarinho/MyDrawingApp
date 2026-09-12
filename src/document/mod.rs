use digital_canvas::{DocumentId, Layer, LayerId, Color, Rect, Vec2, BlendMode, EntityId};
use anyhow::Result;
use image::{Rgba, ImageBuffer};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use crate::render::Renderer;

type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct Document {
    id: DocumentId,
    name: String,
    width: u32,
    height: u32,
    dpi: f32,
    background_color: Color,
    layers: Vec<Arc<RwLock<Layer>>>,
    layer_order: Vec<LayerId>,
    active_layer: Option<LayerId>,
    modified: bool,
    file_path: Option<std::path::PathBuf>,
    metadata: DocumentMetadata,
    color_profile: Option<ColorProfile>,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub author: String,
    pub description: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub color_mode: ColorMode,
    pub bit_depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    RGB,
    Grayscale,
    CMYK,
    Lab,
}

#[derive(Debug, Clone)]
pub struct ColorProfile {
    pub name: String,
    pub data: Vec<u8>,
}

impl Document {
    pub fn new(
        id: DocumentId,
        name: &str,
        width: u32,
        height: u32,
        settings: &crate::settings::Settings,
    ) -> Result<Self> {
        let background_layer = Layer::new(
            LayerId::new(),
            "Background",
            width,
            height,
            true,
            settings.document.default_background_color,
        )?;

        let layer_id = background_layer.id();
        let mut doc = Self {
            id,
            name: name.to_string(),
            width,
            height,
            dpi: settings.document.default_dpi,
            background_color: settings.document.default_background_color,
            layers: vec![Arc::new(RwLock::new(background_layer))],
            layer_order: vec![layer_id],
            active_layer: Some(layer_id),
            modified: false,
            file_path: None,
            metadata: DocumentMetadata {
                color_mode: ColorMode::RGB,
                bit_depth: 8,
                ..Default::default()
            },
            color_profile: None,
        };

        doc.metadata.created = chrono::Utc::now();
        doc.metadata.modified = chrono::Utc::now();

        Ok(doc)
    }

    pub fn id(&self) -> DocumentId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
        self.mark_modified();
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    pub fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi.max(1.value());
        self.mark_modified();
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
        self.mark_modified();
    }

    pub fn layers(&self) -> Vec<Arc<RwLock<Layer>>> {
        self.layer_order.iter()
            .filter_map(|id| self.layers.iter().find(|l| l.read().unwrap().id() == *id))
            .cloned()
            .collect()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn layer_order(&self) -> &[LayerId] {
        &self.layer_order
    }

    pub fn active_layer(&self) -> Option<LayerId> {
        self.active_layer
    }

    pub fn set_active_layer(&mut self, layer_id: LayerId) -> bool {
        if self.layers.iter().any(|l| l.read().unwrap().id() == layer_id) {
            self.active_layer = Some(layer_id);
            true
        } else {
            false
        }
    }

    pub fn get_layer(&self, layer_id: LayerId) -> Option<Arc<RwLock<Layer>>> {
        self.layers.iter()
            .find(|l| l.read().unwrap().id() == layer_id)
            .cloned()
    }

    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        let layer_id = layer.id();
        let arc_layer = Arc::new(RwLock::new(layer));
        self.layers.push(arc_layer);
        self.layer_order.push(layer_id);
        self.active_layer = Some(layer_id);
        self.mark_modified();
        layer_id
    }

    pub fn insert_layer(&mut self, index: usize, layer: Layer) -> LayerId {
        let layer_id = layer.id();
        let arc_layer = Arc::new(RwLock::new(layer));
        self.layers.push(arc_layer);
        self.layer_order.insert(index.min(self.layer_order.len()), layer_id);
        self.active_layer = Some(layer_id);
        self.mark_modified();
        layer_id
    }

    pub fn remove_layer(&mut self, layer_id: LayerId) -> bool {
        if self.layers.len() <= 1 {
            return false; // Keep at least one layer
        }

        if let Some(pos) = self.layer_order.iter().position(|&id| id == layer_id) {
            self.layer_order.remove(pos);
            self.layers.retain(|l| l.read().unwrap().id() != layer_id);

            if self.active_layer == Some(layer_id) {
                self.active_layer = self.layer_order.last().copied();
            }
            self.mark_modified();
            true
        } else {
            false
        }
    }

    pub fn duplicate_layer(&mut self, layer_id: LayerId) -> Option<LayerId> {
        if let Some(layer) = self.get_layer(layer_id) {
            let mut new_layer = layer.read().unwrap().clone();
            new_layer.id = LayerId::new();
            new_layer.name = format!("{} copy", new_layer.name);
            let new_id = self.add_layer(new_layer);
            Some(new_id)
        } else {
            None
        }
    }

    pub fn merge_down(&mut self, layer_id: LayerId) -> bool {
        if let Some(pos) = self.layer_order.iter().position(|&id| id == layer_id) {
            if pos == 0 {
                return false; // Can't merge background layer down
            }

            let upper_id = self.layer_order[pos];
            let lower_id = self.layer_order[pos - 1];

            if let (Some(upper), Some(lower)) = (self.get_layer(upper_id), self.get_layer(lower_id)) {
                let mut upper_write = upper.write().unwrap();
                let lower_read = lower.read().unwrap();

                upper_write.merge_from(&lower_read)?;
                self.remove_layer(lower_id);
                self.mark_modified();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn merge_visible(&mut self) -> Option<LayerId> {
        let visible_layers: Vec<LayerId> = self.layer_order.iter()
            .filter(|&&id| {
                self.get_layer(id).map(|l| l.read().unwrap().visible()).unwrap_or(false)
            })
            .copied()
            .collect();

        if visible_layers.len() < 2 {
            return None;
        }

        let mut merged = Layer::new(
            LayerId::new(),
            "Merged",
            self.width,
            self.height,
            true,
            Color::TRANSPARENT,
        ).ok()?;

        for &layer_id in visible_layers.iter().rev() {
            if let Some(layer) = self.get_layer(layer_id) {
                merged.merge_from(&layer.read().unwrap()).ok()?;
            }
        }

        let merged_id = self.add_layer(merged);
        
        // Hide the original visible layers
        for &layer_id in &visible_layers {
            if let Some(layer) = self.get_layer(layer_id) {
                layer.write().unwrap().set_visible(false);
            }
        }

        Some(merged_id)
    }

    pub fn flatten(&mut self) {
        if let Some(merged_id) = self.merge_visible() {
            // Remove all other layers
            let layers_to_remove: Vec<LayerId> = self.layer_order.iter()
                .filter(|&&id| id != merged_id)
                .copied()
                .collect();
            
            for id in layers_to_remove {
                self.remove_layer(id);
            }
        }
    }

    pub fn reorder_layer(&mut self, layer_id: LayerId, new_index: usize) -> bool {
        if let Some(current_pos) = self.layer_order.iter().position(|&id| id == layer_id) {
            self.layer_order.remove(current_pos);
            let new_index = new_index.min(self.layer_order.len());
            self.layer_order.insert(new_index, layer_id);
            self.mark_modified();
            true
        } else {
            false
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width.max(1);
        self.height = height.max(1);

        for layer in &self.layers {
            layer.write().unwrap().resize(width, height)?;
        }

        self.mark_modified();
        Ok(())
    }

    pub fn crop(&mut self, rect: Rect) -> Result<()> {
        let x = rect.x.max(0.value()) as u32;
        let y = rect.y.max(0.value()) as u32;
        let width = (rect.width.min(self.width as f32 - x as f32)).max(1.value()) as u32;
        let height = (rect.height.min(self.height as f32 - y as f32)).max(1.value()) as u32;

        for layer in &self.layers {
            layer.write().unwrap().crop(x, y, width, height)?;
        }

        self.width = width;
        self.height = height;
        self.mark_modified();
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn mark_modified(&mut self) {
        self.modified = true;
        self.metadata.modified = chrono::Utc::now();
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, path: std::path::PathBuf) {
        self.file_path = Some(path);
        self.mark_saved();
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut DocumentMetadata {
        &mut self.metadata
    }

    pub fn color_profile(&self) -> Option<&ColorProfile> {
        self.color_profile.as_ref()
    }

    pub fn set_color_profile(&mut self, profile: ColorProfile) {
        self.color_profile = Some(profile);
        self.mark_modified();
    }

    pub fn create_thumbnail(&self, max_size: u32) -> Result<RgbaImage> {
        let scale = (max_size as f32 / self.width.max(self.height) as f32).min(1.0);
        let thumb_width = (self.width as f32 * scale) as u32;
        let thumb_height = (self.height as f32 * scale) as u32;

        let mut thumbnail = RgbaImage::new(thumb_width, thumb_height);
        
        // Render visible layers to thumbnail
        for layer_id in &self.layer_order {
            if let Some(layer) = self.get_layer(*layer_id) {
                let layer_read = layer.read().unwrap();
                if layer_read.visible() {
                    if let Some(texture) = layer_read.texture() {
                        // Would need to read back from GPU texture
                        // For now, return blank
                    }
                }
            }
        }

        Ok(thumbnail)
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        Self {
            id: DocumentId::new(),
            name: self.name.clone(),
            width: self.width,
            height: self.height,
            dpi: self.dpi,
            background_color: self.background_color,
            layers: self.layers.iter().map(|l| Arc::new(RwLock::new(l.read().unwrap().clone()))).collect(),
            layer_order: self.layer_order.clone(),
            active_layer: self.active_layer,
            modified: self.modified,
            file_path: self.file_path.clone(),
            metadata: self.metadata.clone(),
            color_profile: self.color_profile.clone(),
        }
    }
}
