use digital_canvas::{Document, Layer, LayerId, Color, Vec2, Rect, EntityId};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseSettings {
    pub enabled: bool,
    pub fps: u32,
    pub playback_speed: f32,
    pub export_resolution: (u32, u32),
    pub max_events: usize,
    pub capture_interval: Duration,
}

impl Default for TimelapseSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            fps: 30,
            playback_speed: 1.0,
            export_resolution: (1920, 1080),
            max_events: 100000,
            capture_interval: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseEvent {
    pub timestamp: f64, // Relative to start
    pub event_type: TimelapseEventType,
    pub layer_id: Option<u64>,
    pub data: TimelapseEventData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelapseEventType {
    StrokeStart,
    StrokePoint,
    StrokeEnd,
    LayerCreate,
    LayerDelete,
    LayerVisibility,
    LayerOpacity,
    LayerBlendMode,
    LayerTransform,
    LayerReorder,
    MaskCreate,
    MaskDelete,
    MaskDraw,
    AdjustmentAdd,
    AdjustmentRemove,
    AdjustmentChange,
    DocumentResize,
    DocumentCrop,
    SelectionChange,
    ToolChange,
    ColorChange,
    BrushChange,
    CanvasPan,
    CanvasZoom,
    CanvasRotate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelapseEventData {
    Stroke {
        points: Vec<StrokePoint>,
        brush_id: u64,
        color: Color,
        blend_mode: digital_canvas::BlendMode,
    },
    LayerProperty {
        property: LayerProperty,
        old_value: String,
        new_value: String,
    },
    LayerContent {
        region: Rect,
        // Texture diff data would go here
    },
    DocumentProperty {
        property: DocumentProperty,
        old_value: String,
        new_value: String,
    },
    Selection {
        selection_type: String,
        data: Vec<u8>,
    },
    Tool {
        tool_name: String,
    },
    Color {
        foreground: Color,
        background: Color,
    },
    Brush {
        brush_id: u64,
        brush_name: String,
    },
    CanvasTransform {
        offset: Vec2,
        scale: f32,
        rotation: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerProperty {
    Visibility,
    Opacity,
    BlendMode,
    Name,
    Lock,
    Transform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentProperty {
    Width,
    Height,
    BackgroundColor,
    Dpi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Vec2,
    pub pressure: f32,
    pub tilt: Vec2,
    pub rotation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelapseMetadata {
    pub document_id: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration: Duration,
    pub total_strokes: u64,
    pub total_events: u64,
    pub canvas_size: (u32, u32),
    pub settings: TimelapseSettings,
}

pub struct TimelapseRecorder {
    settings: TimelapseSettings,
    events: VecDeque<TimelapseEvent>,
    metadata: Option<TimelapseMetadata>,
    start_time: Option<Instant>,
    last_capture: Instant,
    is_recording: bool,
    current_stroke: Option<StrokeBuilder>,
    event_counter: u64,
}

struct StrokeBuilder {
    brush_id: u64,
    color: Color,
    blend_mode: digital_canvas::BlendMode,
    points: Vec<StrokePoint>,
    layer_id: Option<LayerId>,
    start_time: f64,
}

impl TimelapseRecorder {
    pub fn new(settings: &TimelapseSettings) -> Result<Self> {
        Ok(Self {
            settings: settings.clone(),
            events: VecDeque::with_capacity(settings.max_events),
            metadata: None,
            start_time: None,
            last_capture: Instant::now(),
            is_recording: false,
            current_stroke: None,
            event_counter: 0,
        })
    }

    pub fn start(&mut self, document: &Document) {
        if !self.settings.enabled || self.is_recording {
            return;
        }

        self.is_recording = true;
        self.start_time = Some(Instant::now());
        self.last_capture = Instant::now();
        self.events.clear();
        self.event_counter = 0;

        self.metadata = Some(TimelapseMetadata {
            document_id: document.id().0,
            start_time: chrono::Utc::now(),
            end_time: None,
            duration: Duration::ZERO,
            total_strokes: 0,
            total_events: 0,
            canvas_size: (document.width(), document.height()),
            settings: self.settings.clone(),
        });

        // Record initial document state
        self.record_initial_state(document);
    }

    pub fn stop(&mut self) {
        if !self.is_recording {
            return;
        }

        self.end_current_stroke();
        self.is_recording = false;

        if let Some(meta) = &mut self.metadata {
            meta.end_time = Some(chrono::Utc::now());
            if let Some(start) = self.start_time {
                meta.duration = start.elapsed();
            }
            meta.total_events = self.event_counter;
        }
    }

    pub fn pause(&mut self) {
        self.end_current_stroke();
        self.is_recording = false;
    }

    pub fn resume(&mut self, document: &Document) {
        if self.is_recording {
            return;
        }
        self.is_recording = true;
        self.start_time = Some(Instant::now());
        self.last_capture = Instant::now();
    }

    pub fn record_stroke_start(
        &mut self,
        layer_id: LayerId,
        brush_id: u64,
        color: Color,
        blend_mode: digital_canvas::BlendMode,
        position: Vec2,
        pressure: f32,
        tilt: Vec2,
        rotation: f32,
    ) {
        if !self.is_recording || !self.settings.enabled {
            return;
        }

        self.end_current_stroke();

        let timestamp = self.start_time.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
        
        self.current_stroke = Some(StrokeBuilder {
            brush_id,
            color,
            blend_mode,
            points: vec![StrokePoint { position, pressure, tilt, rotation }],
            layer_id: Some(layer_id),
            start_time: timestamp,
        });

        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::StrokeStart,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::Stroke {
                points: vec![StrokePoint { position, pressure, tilt, rotation }],
                brush_id,
                color,
                blend_mode,
            },
        });
    }

    pub fn record_stroke_point(
        &mut self,
        position: Vec2,
        pressure: f32,
        tilt: Vec2,
        rotation: f32,
    ) {
        if !self.is_recording || !self.settings.enabled {
            return;
        }

        if let Some(stroke) = &mut self.current_stroke {
            let point = StrokePoint { position, pressure, tilt, rotation };
            stroke.points.push(point.clone());

            // Throttle event recording
            if self.last_capture.elapsed() >= self.settings.capture_interval {
                let timestamp = self.start_time.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
                self.add_event(TimelapseEvent {
                    timestamp,
                    event_type: TimelapseEventType::StrokePoint,
                    layer_id: stroke.layer_id.map(|id| id.0),
                    data: TimelapseEventData::Stroke {
                        points: vec![point],
                        brush_id: stroke.brush_id,
                        color: stroke.color,
                        blend_mode: stroke.blend_mode,
                    },
                });
                self.last_capture = Instant::now();
            }
        }
    }

    pub fn record_stroke_end(&mut self) {
        self.end_current_stroke();
    }

    fn end_current_stroke(&mut self) {
        if let Some(stroke) = self.current_stroke.take() {
            if stroke.points.len() > 1 {
                if let Some(meta) = &mut self.metadata {
                    meta.total_strokes += 1;
                }
            }

            let timestamp = self.start_time.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
            self.add_event(TimelapseEvent {
                timestamp,
                event_type: TimelapseEventType::StrokeEnd,
                layer_id: stroke.layer_id.map(|id| id.0),
                data: TimelapseEventData::Stroke {
                    points: stroke.points,
                    brush_id: stroke.brush_id,
                    color: stroke.color,
                    blend_mode: stroke.blend_mode,
                },
            });
        }
    }

    pub fn record_layer_create(&mut self, layer_id: LayerId, name: &str) {
        self.record_layer_property(layer_id, LayerProperty::Name, "none", name);
    }

    pub fn record_layer_delete(&mut self, layer_id: LayerId) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::LayerDelete,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Name,
                old_value: "deleted".to_string(),
                new_value: "none".to_string(),
            },
        });
    }

    pub fn record_layer_visibility(&mut self, layer_id: LayerId, visible: bool) {
        self.record_layer_property(layer_id, LayerProperty::Visibility, 
            if visible { "hidden" } else { "visible" }, 
            if visible { "visible" } else { "hidden" });
    }

    pub fn record_layer_opacity(&mut self, layer_id: LayerId, old_opacity: f32, new_opacity: f32) {
        self.record_layer_property(layer_id, LayerProperty::Opacity, 
            &old_opacity.to_string(), &new_opacity.to_string());
    }

    pub fn record_layer_blend_mode(&mut self, layer_id: LayerId, old_mode: digital_canvas::BlendMode, new_mode: digital_canvas::BlendMode) {
        self.record_layer_property(layer_id, LayerProperty::BlendMode, 
            &old_mode.as_str().to_string(), &new_mode.as_str().to_string());
    }

    pub fn record_layer_transform(&mut self, layer_id: LayerId, old_transform: digital_canvas::Transform2D, new_transform: digital_canvas::Transform2D) {
        self.record_layer_property(layer_id, LayerProperty::Transform, 
            &format!("{:?}", old_transform), &format!("{:?}", new_transform));
    }

    pub fn record_layer_reorder(&mut self, layer_id: LayerId, old_index: usize, new_index: usize) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::LayerReorder,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Transform, // Reusing for reorder
                old_value: old_index.to_string(),
                new_value: new_index.to_string(),
            },
        });
    }

    pub fn record_mask_create(&mut self, layer_id: LayerId) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::MaskCreate,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Visibility, // Reusing
                old_value: "none".to_string(),
                new_value: "mask".to_string(),
            },
        });
    }

    pub fn record_mask_delete(&mut self, layer_id: LayerId) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::MaskDelete,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Visibility,
                old_value: "mask".to_string(),
                new_value: "none".to_string(),
            },
        });
    }

    pub fn record_adjustment_add(&mut self, layer_id: LayerId, adj_type: digital_canvas::layer::AdjustmentType) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::AdjustmentAdd,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Visibility,
                old_value: "none".to_string(),
                new_value: format!("{:?}", adj_type),
            },
        });
    }

    pub fn record_adjustment_remove(&mut self, layer_id: LayerId) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::AdjustmentRemove,
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property: LayerProperty::Visibility,
                old_value: "adjustment".to_string(),
                new_value: "none".to_string(),
            },
        });
    }

    pub fn record_document_resize(&mut self, old_width: u32, old_height: u32, new_width: u32, new_height: u32) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::DocumentResize,
            layer_id: None,
            data: TimelapseEventData::DocumentProperty {
                property: DocumentProperty::Width,
                old_value: format!("{}x{}", old_width, old_height),
                new_value: format!("{}x{}", new_width, new_height),
            },
        });

        if let Some(meta) = &mut self.metadata {
            meta.canvas_size = (new_width, new_height);
        }
    }

    pub fn record_tool_change(&mut self, tool_name: &str) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::ToolChange,
            layer_id: None,
            data: TimelapseEventData::Tool {
                tool_name: tool_name.to_string(),
            },
        });
    }

    pub fn record_color_change(&mut self, foreground: Color, background: Color) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::ColorChange,
            layer_id: None,
            data: TimelapseEventData::Color { foreground, background },
        });
    }

    pub fn record_brush_change(&mut self, brush_id: u64, brush_name: &str) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::BrushChange,
            layer_id: None,
            data: TimelapseEventData::Brush {
                brush_id,
                brush_name: brush_name.to_string(),
            },
        });
    }

    pub fn record_canvas_transform(&mut self, offset: Vec2, scale: f32, rotation: f32) {
        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: TimelapseEventType::CanvasPan, // Using pan for all transforms
            layer_id: None,
            data: TimelapseEventData::CanvasTransform { offset, scale, rotation },
        });
    }

    fn record_layer_property(&mut self, layer_id: LayerId, property: LayerProperty, old_value: &str, new_value: &str) {
        if !self.is_recording || !self.settings.enabled {
            return;
        }

        let timestamp = self.get_timestamp();
        self.add_event(TimelapseEvent {
            timestamp,
            event_type: match property {
                LayerProperty::Visibility => TimelapseEventType::LayerVisibility,
                LayerProperty::Opacity => TimelapseEventType::LayerOpacity,
                LayerProperty::BlendMode => TimelapseEventType::LayerBlendMode,
                LayerProperty::Transform => TimelapseEventType::LayerTransform,
                _ => TimelapseEventType::LayerTransform,
            },
            layer_id: Some(layer_id.0),
            data: TimelapseEventData::LayerProperty {
                property,
                old_value: old_value.to_string(),
                new_value: new_value.to_string(),
            },
        });
    }

    fn record_initial_state(&mut self, document: &Document) {
        let timestamp = 0.0;
        // Record initial layers
        for layer_id in document.layer_order() {
            if let Some(layer) = document.get_layer(*layer_id) {
                let layer_read = layer.read().unwrap();
                self.add_event(TimelapseEvent {
                    timestamp,
                    event_type: TimelapseEventType::LayerCreate,
                    layer_id: Some(layer_id.0),
                    data: TimelapseEventData::LayerProperty {
                        property: LayerProperty::Name,
                        old_value: "none".to_string(),
                        new_value: layer_read.name().to_string(),
                    },
                });
            }
        }
    }

    fn get_timestamp(&self) -> f64 {
        self.start_time.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0)
    }

    fn add_event(&mut self, event: TimelapseEvent) {
        self.events.push_back(event);
        self.event_counter += 1;

        if self.events.len() > self.settings.max_events {
            self.events.pop_front();
        }
    }

    pub fn events(&self) -> &VecDeque<TimelapseEvent> {
        &self.events
    }

    pub fn metadata(&self) -> Option<&TimelapseMetadata> {
        self.metadata.as_ref()
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn export_video(&self, output_path: &std::path::Path) -> Result<()> {
        // Would use ffmpeg or similar to generate video from events
        // This is a placeholder for the actual implementation
        Ok(())
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let data = TimelapseData {
            metadata: self.metadata.clone(),
            events: self.events.clone(),
        };
        Ok(bincode::serialize(&data)?)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let data: TimelapseData = bincode::deserialize(data)?;
        let event_count = data.events.len() as u64;
        let mut recorder = Self::new(&data.metadata.as_ref().unwrap().settings)?;
        recorder.metadata = data.metadata;
        recorder.events = data.events;
        recorder.event_counter = event_count;
        Ok(recorder)
    }

    pub fn settings(&self) -> &TimelapseSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut TimelapseSettings {
        &mut self.settings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimelapseData {
    metadata: Option<TimelapseMetadata>,
    events: VecDeque<TimelapseEvent>,
}

pub struct TimelapsePlayer {
    events: VecDeque<TimelapseEvent>,
    current_index: usize,
    playback_speed: f32,
    is_playing: bool,
    last_frame_time: Instant,
    frame_callback: Option<Box<dyn Fn(&TimelapseEvent) + Send>>,
}

impl TimelapsePlayer {
    pub fn new(events: VecDeque<TimelapseEvent>, fps: u32) -> Self {
        Self {
            events,
            current_index: 0,
            playback_speed: 1.0,
            is_playing: false,
            last_frame_time: Instant::now(),
            frame_callback: None,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_frame_time = Instant::now();
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_index = 0;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1);
    }

    pub fn set_frame_callback<F>(&mut self, callback: F)
    where
        F: Fn(&TimelapseEvent) + Send + 'static,
    {
        self.frame_callback = Some(Box::new(callback));
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.is_playing || self.current_index >= self.events.len() {
            return;
        }

        let frame_duration = 1.0 / 30.0 / self.playback_speed; // Assuming 30 fps base
        let elapsed = self.last_frame_time.elapsed().as_secs_f32();

        if elapsed >= frame_duration {
            while self.current_index < self.events.len() {
                let event = &self.events[self.current_index];
                if let Some(callback) = &self.frame_callback {
                    callback(event);
                }
                self.current_index += 1;
                
                // Skip to next significant event
                if matches!(event.event_type, TimelapseEventType::StrokeEnd | TimelapseEventType::LayerCreate) {
                    break;
                }
            }
            self.last_frame_time = Instant::now();
        }
    }

    pub fn seek(&mut self, time: f64) {
        self.current_index = self.events.iter()
            .position(|e| e.timestamp > time)
            .unwrap_or(self.events.len());
    }

    pub fn progress(&self) -> f32 {
        if self.events.is_empty() {
            0.0
        } else {
            self.current_index as f32 / self.events.len() as f32
        }
    }

    pub fn current_time(&self) -> f64 {
        self.events.get(self.current_index).map(|e| e.timestamp).unwrap_or(0.0)
    }

    pub fn total_time(&self) -> f64 {
        self.events.back().map(|e| e.timestamp).unwrap_or(0.0)
    }
}
