use digital_canvas::{Document, Layer, LayerId, Color, Rect, BlendMode};
use anyhow::Result;
use psd::Psd;
use std::path::Path;
use std::sync::Arc;
use std::fs;

pub struct PsdImporter;

impl PsdImporter {
    pub fn import(path: &Path, settings: &digital_canvas::settings::Settings) -> Result<Document> {
        let bytes = fs::read(path)?;
        let psd = Psd::from_bytes(&bytes)?;
        let (width, height) = (psd.width(), psd.height());

        let mut doc = Document::new(
            digital_canvas::document::DocumentId::new(),
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("PSD Import"),
            width,
            height,
            settings,
        )?;

        // Parse PSD layers in reverse order (PSD stores bottom-to-top)
        for layer in psd.layers().iter().rev() {
            let layer_name = layer.name().to_string();
            let visible = layer.visible();
            let opacity = layer.opacity() as f32 / 255.0;
            let blend_mode = Self::psd_blend_mode_to_blend_mode(layer.blend_mode());

            let mut new_layer = Layer::new(
                LayerId::new(),
                &layer_name,
                width,
                height,
                false,
                Color::TRANSPARENT,
            )?;

            new_layer.set_visible(visible);
            new_layer.set_opacity(opacity);
            new_layer.set_blend_mode(blend_mode);

            // Load layer pixel data if available
            if let Some(channel_data) = layer.channel_data() {
                // Would convert channel data to texture
            }

            doc.add_layer(new_layer);
        }

        // Handle layer groups
        Self::process_layer_groups(&mut doc, psd.layer_groups());

        doc.set_file_path(path.to_path_buf());
        doc.mark_saved();

        Ok(doc)
    }

    fn process_layer_groups(doc: &mut Document, groups: &[psd::LayerGroup]) {
        // Would process layer group hierarchy
    }

    fn psd_blend_mode_to_blend_mode(psd_mode: &str) -> BlendMode {
        match psd_mode {
            "norm" => BlendMode::Normal,
            "mul " => BlendMode::Multiply,
            "scrn" => BlendMode::Screen,
            "over" => BlendMode::Overlay,
            "sLit" => BlendMode::SoftLight,
            "hLit" => BlendMode::HardLight,
            "div " => BlendMode::ColorDodge,
            "idiv" => BlendMode::ColorBurn,
            "dark" => BlendMode::Darken,
            "lite" => BlendMode::Lighten,
            "diff" => BlendMode::Difference,
            "smud" => BlendMode::Exclusion,
            "hue " => BlendMode::Hue,
            "sat " => BlendMode::Saturation,
            "colr" => BlendMode::Color,
            "lum " => BlendMode::Luminosity,
            _ => BlendMode::Normal,
        }
    }
}

pub struct PsdExporter;

impl PsdExporter {
    pub fn export(document: &Document, path: &Path) -> Result<()> {
        // Would create PSD file from document
        // This requires implementing PSD writing
        Err(anyhow::anyhow!("PSD export not yet implemented"))
    }
}

pub struct PsdCompatibility;

impl PsdCompatibility {
    pub fn supported_blend_modes() -> &'static [BlendMode] {
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

    pub fn supported_features() -> PsdFeatures {
        PsdFeatures {
            layers: true,
            layer_groups: true,
            layer_masks: true,
            layer_effects: false, // Not yet supported
            adjustment_layers: false, // Not yet supported
            smart_objects: false,
            text_layers: false,
            vector_masks: false,
            clipping_masks: true,
            blend_modes: Self::supported_blend_modes().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PsdFeatures {
    pub layers: bool,
    pub layer_groups: bool,
    pub layer_masks: bool,
    pub layer_effects: bool,
    pub adjustment_layers: bool,
    pub smart_objects: bool,
    pub text_layers: bool,
    pub vector_masks: bool,
    pub clipping_masks: bool,
    pub blend_modes: Vec<BlendMode>,
}
