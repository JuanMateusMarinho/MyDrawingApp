use digital_canvas::{Document, DocumentId, Layer, LayerId, Color, Rect, Vec2};
use anyhow::Result;
use image::{DynamicImage, ImageFormat, GenericImageView, Rgba, ImageBuffer};
use psd::Psd;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct FileManager {
    recent_files: Vec<RecentFile>,
    max_recent_files: usize,
    autosave_dir: std::path::PathBuf,
    project_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: std::path::PathBuf,
    pub name: String,
    pub thumbnail: Option<Vec<u8>>,
    pub last_opened: chrono::DateTime<chrono::Utc>,
    pub is_project: bool,
}

impl FileManager {
    pub fn new(settings: &crate::settings::Settings) -> Result<Self> {
        let config_dir = crate::settings::Settings::config_dir()?;
        let autosave_dir = config_dir.join("autosave");
        let project_dir = config_dir.join("projects");
        
        std::fs::create_dir_all(&autosave_dir)?;
        std::fs::create_dir_all(&project_dir)?;

        let mut manager = Self {
            recent_files: Vec::new(),
            max_recent_files: 20,
            autosave_dir,
            project_dir,
        };

        manager.load_recent_files()?;
        Ok(manager)
    }

    pub fn new_document(&self, width: u32, height: u32, settings: &crate::settings::Settings) -> Result<Document> {
        Document::new(
            DocumentId::new(),
            "Untitled",
            width,
            height,
            settings,
        )
    }

    pub fn open(&self, path: &Path, settings: &crate::settings::Settings) -> Result<Document> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let document = match extension.as_str() {
            "psd" => self.open_psd(path, settings)?,
            "dcanvas" | "dcv" => self.open_native(path, settings)?,
            _ => self.open_image(path, settings)?,
        };

        Ok(document)
    }

    pub fn save(&self, document: &Document, path: &Path) -> Result<()> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "psd" => self.save_psd(document, path)?,
            "dcanvas" | "dcv" => self.save_native(document, path)?,
            "png" => self.save_image(document, path, ImageFormat::Png)?,
            "jpg" | "jpeg" => self.save_image(document, path, ImageFormat::Jpeg)?,
            "webp" => self.save_image(document, path, ImageFormat::WebP)?,
            "tiff" | "tif" => self.save_image(document, path, ImageFormat::Tiff)?,
            "bmp" => self.save_image(document, path, ImageFormat::Bmp)?,
            "gif" => self.save_image(document, path, ImageFormat::Gif)?,
            _ => return Err(anyhow::anyhow!("Unsupported file format: {}", extension)),
        }

        self.add_recent_file(path, document.name(), document.file_path().is_some());
        Ok(())
    }

    pub fn export(&self, document: &Document, path: &Path, format: ExportFormat, options: &ExportOptions) -> Result<()> {
        match format {
            ExportFormat::PNG => self.save_image(document, path, ImageFormat::Png)?,
            ExportFormat::JPEG => self.save_image(document, path, ImageFormat::Jpeg)?,
            ExportFormat::WEBP => self.save_image(document, path, ImageFormat::WebP)?,
            ExportFormat::TIFF => self.save_image(document, path, ImageFormat::Tiff)?,
            ExportFormat::BMP => self.save_image(document, path, ImageFormat::Bmp)?,
            ExportFormat::GIF => self.save_image(document, path, ImageFormat::Gif)?,
            ExportFormat::PSD => self.save_psd(document, path)?,
            ExportFormat::Native => self.save_native(document, path)?,
        }
        Ok(())
    }

    pub fn autosave(&self, document: &Document) -> Result<()> {
        let filename = format!("autosave_{}_{}.dcv", 
            document.id().0, 
            chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let path = self.autosave_dir.join(filename);
        self.save_native(document, &path)
    }

    pub fn recover_autosaves(&self) -> Result<Vec<Document>> {
        let mut documents = Vec::new();
        
        for entry in std::fs::read_dir(&self.autosave_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "dcv").unwrap_or(false) {
                if let Ok(doc) = self.open_native(&path, &crate::settings::Settings::default()) {
                    documents.push(doc);
                }
            }
        }

        // Sort by modification time, newest first
        documents.sort_by(|a, b| {
            b.file_path()
                .and_then(|p| std::fs::metadata(p).ok())
                .and_then(|m| m.modified().ok())
                .cmp(&a.file_path()
                    .and_then(|p| std::fs::metadata(p).ok())
                    .and_then(|m| m.modified().ok()))
        });

        Ok(documents)
    }

    pub fn clean_autosaves(&self, max_age_days: u32) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);
        let mut count = 0;

        for entry in std::fs::read_dir(&self.autosave_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified: chrono::DateTime<chrono::Utc> = modified.into();
                    if modified < cutoff {
                        std::fs::remove_file(path)?;
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    fn open_image(&self, path: &Path, settings: &crate::settings::Settings) -> Result<Document> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let mut doc = Document::new(
            DocumentId::new(),
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported"),
            width,
            height,
            settings,
        )?;

        // Set background layer content
        if let Some(layer) = doc.get_layer(doc.active_layer().unwrap()) {
            // Would upload texture to GPU
        }

        doc.set_file_path(path.to_path_buf());
        doc.mark_saved();

        Ok(doc)
    }

    fn save_image(&self, document: &Document, path: &Path, format: ImageFormat) -> Result<()> {
        // Would render document to image and save
        let img = self.render_document_to_image(document)?;
        img.save_with_format(path, format)?;
        Ok(())
    }

    fn render_document_to_image(&self, document: &Document) -> Result<RgbaImage> {
        // Composite all visible layers
        let mut composite = RgbaImage::new(document.width(), document.height());
        
        // Fill with background
        let bg = document.background_color();
        for pixel in composite.pixels_mut() {
            *pixel = Rgba([bg.r as u8, bg.g as u8, bg.b as u8, 255]);
        }

        // Blend layers (simplified - would use GPU in reality)
        for layer_id in document.layer_order() {
            if let Some(layer) = document.get_layer(*layer_id) {
                let layer_read = layer.read().unwrap();
                if layer_read.visible() {
                    // Would blend layer texture
                }
            }
        }

        Ok(composite)
    }

    fn open_psd(&self, path: &Path, settings: &crate::settings::Settings) -> Result<Document> {
        let bytes = fs::read(path)?;
        let psd = Psd::from_bytes(&bytes)?;
        let (width, height) = (psd.width(), psd.height());

        let mut doc = Document::new(
            DocumentId::new(),
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("PSD Import"),
            width,
            height,
            settings,
        )?;

        // Parse PSD layers
        for (i, layer) in psd.layers().iter().enumerate() {
            let layer_name = layer.name().to_string();
            let visible = layer.visible();
            let opacity = layer.opacity() as f32 / 255.0;
            let blend_mode = self.psd_blend_mode_to_blend_mode(layer.blend_mode());

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

            // Would load layer pixel data
            
            doc.add_layer(new_layer);
        }

        doc.set_file_path(path.to_path_buf());
        doc.mark_saved();

        Ok(doc)
    }

    fn save_psd(&self, document: &Document, path: &Path) -> Result<()> {
        // Would save as PSD
        Ok(())
    }

    fn open_native(&self, path: &Path, settings: &crate::settings::Settings) -> Result<Document> {
        let data = std::fs::read(path)?;
        let project: NativeProject = bincode::deserialize(&data)?;
        
        let mut doc = Document::new(
            project.document_id,
            &project.name,
            project.width,
            project.height,
            settings,
        )?;

        // Restore layers
        for layer_data in project.layers {
            let mut layer = Layer::new(
                layer_data.id,
                &layer_data.name,
                project.width,
                project.height,
                layer_data.is_background,
                layer_data.background_color,
            )?;
            
            layer.set_visible(layer_data.visible);
            layer.set_locked(layer_data.locked);
            layer.set_opacity(layer_data.opacity);
            layer.set_blend_mode(layer_data.blend_mode);
            layer.set_transform(layer_data.transform);
            
            doc.add_layer(layer);
        }

        doc.set_file_path(path.to_path_buf());
        doc.mark_saved();

        Ok(doc)
    }

    fn save_native(&self, document: &Document, path: &Path) -> Result<()> {
        let project = NativeProject {
            document_id: document.id(),
            name: document.name().to_string(),
            width: document.width(),
            height: document.height(),
            dpi: document.dpi(),
            background_color: document.background_color(),
            layers: document.layers().iter().map(|l| {
                let layer = l.read().unwrap();
                NativeLayerData {
                    id: layer.id(),
                    name: layer.name().to_string(),
                    visible: layer.visible(),
                    locked: layer.locked(),
                    opacity: layer.opacity(),
                    blend_mode: layer.blend_mode(),
                    transform: *layer.transform(),
                    is_background: layer.is_background(),
                    background_color: document.background_color(),
                }
            }).collect(),
            timelapse: document.metadata().get("timelapse").cloned(),
            metadata: document.metadata().clone(),
        };

        let data = bincode::serialize(&project)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    fn psd_blend_mode_to_blend_mode(&self, psd_mode: &str) -> digital_canvas::BlendMode {
        match psd_mode {
            "norm" => digital_canvas::BlendMode::Normal,
            "mul " => digital_canvas::BlendMode::Multiply,
            "scrn" => digital_canvas::BlendMode::Screen,
            "over" => digital_canvas::BlendMode::Overlay,
            "sLit" => digital_canvas::BlendMode::SoftLight,
            "hLit" => digital_canvas::BlendMode::HardLight,
            "div " => digital_canvas::BlendMode::ColorDodge,
            "idiv" => digital_canvas::BlendMode::ColorBurn,
            "dark" => digital_canvas::BlendMode::Darken,
            "lite" => digital_canvas::BlendMode::Lighten,
            "diff" => digital_canvas::BlendMode::Difference,
            "smud" => digital_canvas::BlendMode::Exclusion,
            "hue " => digital_canvas::BlendMode::Hue,
            "sat " => digital_canvas::BlendMode::Saturation,
            "colr" => digital_canvas::BlendMode::Color,
            "lum " => digital_canvas::BlendMode::Luminosity,
            _ => digital_canvas::BlendMode::Normal,
        }
    }

    pub fn recent_files(&self) -> &[RecentFile] {
        &self.recent_files
    }

    pub fn add_recent_file(&mut self, path: &Path, name: &str, is_project: bool) {
        self.recent_files.retain(|f| f.path != path);
        self.recent_files.insert(0, RecentFile {
            path: path.to_path_buf(),
            name: name.to_string(),
            thumbnail: None,
            last_opened: chrono::Utc::now(),
            is_project,
        });
        if self.recent_files.len() > self.max_recent_files {
            self.recent_files.truncate(self.max_recent_files);
        }
        self.save_recent_files().ok();
    }

    pub fn remove_recent_file(&mut self, path: &Path) {
        self.recent_files.retain(|f| f.path != path);
        self.save_recent_files().ok();
    }

    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
        self.save_recent_files().ok();
    }

    fn load_recent_files(&mut self) -> Result<()> {
        let path = crate::settings::Settings::config_dir()?.join("recent_files.json");
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            self.recent_files = serde_json::from_str(&data)?;
        }
        Ok(())
    }

    fn save_recent_files(&self) -> Result<()> {
        let path = crate::settings::Settings::config_dir()?.join("recent_files.json");
        let data = serde_json::to_string_pretty(&self.recent_files)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn create_thumbnail(&self, document: &Document, size: (u32, u32)) -> Result<Vec<u8>> {
        let img = self.render_document_to_image(document)?;
        let thumb = image::imageops::resize(&img, size.0, size.1, image::imageops::FilterType::Lanczos3);
        let mut buf = Vec::new();
        thumb.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeProject {
    pub document_id: DocumentId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
    pub background_color: Color,
    pub layers: Vec<NativeLayerData>,
    pub timelapse: Option<Vec<u8>>,
    pub metadata: crate::document::DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeLayerData {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: digital_canvas::BlendMode,
    pub transform: digital_canvas::Transform2D,
    pub is_background: bool,
    pub background_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub quality: u8,
    pub lossless: bool,
    pub embed_profile: bool,
    pub include_layers: bool,
    pub flatten: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            quality: 90,
            lossless: false,
            embed_profile: true,
            include_layers: false,
            flatten: true,
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