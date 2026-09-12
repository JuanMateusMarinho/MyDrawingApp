use digital_canvas::{Document, Layer, LayerId, Color, Rect, Vec2, BlendMode, Transform2D};
use anyhow::Result;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryAction {
    // Layer operations
    AddLayer { layer_id: u64, layer_data: LayerData },
    RemoveLayer { layer_id: u64, layer_data: LayerData },
    DuplicateLayer { layer_id: u64, new_layer_id: u64, layer_data: LayerData },
    ReorderLayer { layer_id: u64, old_index: usize, new_index: usize },
    
    // Layer property changes
    LayerVisibility { layer_id: u64, old_visible: bool, new_visible: bool },
    LayerOpacity { layer_id: u64, old_opacity: f32, new_opacity: f32 },
    LayerBlendMode { layer_id: u64, old_mode: BlendMode, new_mode: BlendMode },
    LayerName { layer_id: u64, old_name: String, new_name: String },
    LayerLock { layer_id: u64, old_locked: bool, new_locked: bool },
    LayerTransform { layer_id: u64, old_transform: Transform2D, new_transform: Transform2D },
    
    // Layer content changes
    LayerDraw { layer_id: u64, region: Rect, old_texture_data: Option<Vec<u8>> },
    LayerFill { layer_id: u64, color: Color, old_texture_data: Option<Vec<u8>> },
    LayerClear { layer_id: u64, old_texture_data: Option<Vec<u8>> },
    
    // Mask operations
    AddMask { layer_id: u64, mask_data: MaskData },
    RemoveMask { layer_id: u64, mask_data: MaskData },
    MaskDraw { layer_id: u64, region: Rect, old_mask_data: Option<Vec<u8>> },
    MaskProperty { layer_id: u64, property: MaskProperty, old_value: f32, new_value: f32 },
    
    // Adjustment layer operations
    AddAdjustment { layer_id: u64, adjustment_type: crate::layer::AdjustmentType, params: Vec<f32> },
    RemoveAdjustment { layer_id: u64, adjustment_type: crate::layer::AdjustmentType, params: Vec<f32> },
    AdjustmentParams { layer_id: u64, old_params: Vec<f32>, new_params: Vec<f32> },
    
    // Document operations
    Resize { old_width: u32, old_height: u32, new_width: u32, new_height: u32 },
    Crop { old_rect: Rect, new_rect: Rect },
    BackgroundColor { old_color: Color, new_color: Color },
    
    // Selection operations
    SelectionChange { old_selection: Option<SelectionData>, new_selection: Option<SelectionData> },
    
    // Tool operations (for macro recording)
    ToolStroke { tool: String, points: Vec<StrokePoint>, brush_id: u64, color: Color },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerData {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub transform: Transform2D,
    pub is_background: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskData {
    pub enabled: bool,
    pub density: f32,
    pub feather: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskProperty {
    Density,
    Feather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionData {
    pub selection_type: SelectionType,
    pub data: Vec<u8>, // Serialized selection mask or path data
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionType {
    None,
    Rectangle,
    Ellipse,
    Lasso,
    Polygon,
    MagicWand,
    Mask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Vec2,
    pub pressure: f32,
    pub tilt: Vec2,
    pub rotation: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub action: HistoryAction,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub group_id: Option<u64>, // For grouping related actions
}

pub struct HistoryManager {
    history: VecDeque<HistoryEntry>,
    undo_stack: VecDeque<HistoryEntry>,
    max_history: usize,
    current_group: Option<u64>,
    group_counter: u64,
}

impl HistoryManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            undo_stack: VecDeque::with_capacity(max_history),
            max_history,
            current_group: None,
            group_counter: 0,
        }
    }

    pub fn push(&mut self, action: HistoryAction, description: &str) {
        let entry = HistoryEntry {
            action,
            timestamp: chrono::Utc::now(),
            description: description.to_string(),
            group_id: self.current_group,
        };

        self.history.push_back(entry);
        self.undo_stack.clear(); // Clear redo stack on new action

        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn begin_group(&mut self, description: &str) -> u64 {
        self.group_counter += 1;
        self.current_group = Some(self.group_counter);
        self.group_counter
    }

    pub fn end_group(&mut self) {
        self.current_group = None;
    }

    pub fn undo(&mut self, document: &mut Document) -> bool {
        if let Some(entry) = self.history.pop_back() {
            self.apply_undo(document, &entry);
            self.undo_stack.push_back(entry);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, document: &mut Document) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            self.apply_redo(document, &entry);
            self.history.push_back(entry);
            true
        } else {
            false
        }
    }

    fn apply_undo(&mut self, document: &mut Document, entry: &HistoryEntry) {
        use HistoryAction::*;
        
        match &entry.action {
            AddLayer { layer_id, .. } => {
                document.remove_layer(LayerId(layer_id.value()));
            }
            RemoveLayer { layer_id, layer_data } => {
                // Restore layer
                let mut layer = Layer::from_data(layer_data.clone()).unwrap();
                layer.id = LayerId(layer_id.value());
                document.layers.push(std::sync::Arc::new(std::sync::RwLock::new(layer)));
                document.layer_order.insert(layer_data.name.parse().unwrap_or(0), LayerId(layer_id.value()));
            }
            DuplicateLayer { new_layer_id, .. } => {
                document.remove_layer(LayerId(new_layer_id.value()));
            }
            ReorderLayer { layer_id, old_index, .. } => {
                document.reorder_layer(LayerId(layer_id.value()), *old_index);
            }
            LayerVisibility { layer_id, old_visible, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_visible(*old_visible);
                }
            }
            LayerOpacity { layer_id, old_opacity, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_opacity(*old_opacity);
                }
            }
            LayerBlendMode { layer_id, old_mode, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_blend_mode(*old_mode);
                }
            }
            LayerName { layer_id, old_name, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_name(old_name);
                }
            }
            LayerLock { layer_id, old_locked, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_locked(*old_locked);
                }
            }
            LayerTransform { layer_id, old_transform, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_transform(*old_transform);
                }
            }
            LayerDraw { layer_id, region, old_texture_data: Some(data) } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().restore_texture_region(region, data);
                }
            }
            LayerFill { layer_id, .. } | LayerClear { layer_id, .. } => {
                // Would restore from old_texture_data
            }
            AddMask { layer_id, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().remove_mask();
                }
            }
            RemoveMask { layer_id, mask_data } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().restore_mask(mask_data);
                }
            }
            MaskDraw { layer_id, region, old_mask_data: Some(data) } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().restore_mask_region(region, data);
                }
            }
            MaskProperty { layer_id, property, old_value, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    if let Some(mask) = layer.write().unwrap().mask_mut() {
                        match property {
                            MaskProperty::Density => mask.set_density(*old_value),
                            MaskProperty::Feather => mask.set_feather(*old_value),
                        }
                    }
                }
            }
            AddAdjustment { layer_id, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().remove_adjustment();
                }
            }
            RemoveAdjustment { layer_id, adjustment_type, params } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().add_adjustment(*adjustment_type);
                }
            }
            AdjustmentParams { layer_id, old_params, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    if let Some(adj) = layer.write().unwrap().adjustment_mut() {
                        adj.set_params(old_params.clone());
                    }
                }
            }
            Resize { old_width, old_height, .. } => {
                document.resize(*old_width, *old_height).ok();
            }
            Crop { old_rect, .. } => {
                // Would restore from old_rect
            }
            BackgroundColor { old_color, .. } => {
                document.set_background_color(*old_color);
            }
            SelectionChange { old_selection, .. } => {
                // Restore selection
            }
            ToolStroke { .. } => {
                // Would undo the stroke
            }
        }
    }

    fn apply_redo(&mut self, document: &mut Document, entry: &HistoryEntry) {
        use HistoryAction::*;
        
        match &entry.action {
            AddLayer { layer_id, layer_data } => {
                let mut layer = Layer::from_data(layer_data.clone()).unwrap();
                layer.id = LayerId(layer_id.value());
                document.layers.push(std::sync::Arc::new(std::sync::RwLock::new(layer)));
                document.layer_order.push(LayerId(layer_id.value()));
            }
            RemoveLayer { layer_id, .. } => {
                document.remove_layer(LayerId(layer_id.value()));
            }
            DuplicateLayer { new_layer_id, layer_data } => {
                let mut layer = Layer::from_data(layer_data.clone()).unwrap();
                layer.id = LayerId(new_layer_id.value());
                document.layers.push(std::sync::Arc::new(std::sync::RwLock::new(layer)));
                document.layer_order.push(LayerId(new_layer_id.value()));
            }
            ReorderLayer { layer_id, new_index, .. } => {
                document.reorder_layer(LayerId(layer_id.value()), *new_index);
            }
            LayerVisibility { layer_id, new_visible, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_visible(*new_visible);
                }
            }
            LayerOpacity { layer_id, new_opacity, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_opacity(*new_opacity);
                }
            }
            LayerBlendMode { layer_id, new_mode, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_blend_mode(*new_mode);
                }
            }
            LayerName { layer_id, new_name, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_name(new_name);
                }
            }
            LayerLock { layer_id, new_locked, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_locked(*new_locked);
                }
            }
            LayerTransform { layer_id, new_transform, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().set_transform(*new_transform);
                }
            }
            LayerDraw { .. } | LayerFill { .. } | LayerClear { .. } => {
                // Would apply the drawing operation
            }
            AddMask { layer_id, mask_data } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().apply_mask(mask_data);
                }
            }
            RemoveMask { .. } => {
                // Mask already removed
            }
            MaskDraw { .. } => {
                // Would apply mask drawing
            }
            MaskProperty { layer_id, property, new_value, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    if let Some(mask) = layer.write().unwrap().mask_mut() {
                        match property {
                            MaskProperty::Density => mask.set_density(*new_value),
                            MaskProperty::Feather => mask.set_feather(*new_value),
                        }
                    }
                }
            }
            AddAdjustment { layer_id, adjustment_type, params } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().add_adjustment(*adjustment_type);
                }
            }
            RemoveAdjustment { layer_id, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    layer.write().unwrap().remove_adjustment();
                }
            }
            AdjustmentParams { layer_id, new_params, .. } => {
                if let Some(layer) = document.get_layer(LayerId(layer_id.value())) {
                    if let Some(adj) = layer.write().unwrap().adjustment_mut() {
                        adj.set_params(new_params.clone());
                    }
                }
            }
            Resize { new_width, new_height, .. } => {
                document.resize(*new_width, *new_height).ok();
            }
            Crop { new_rect, .. } => {
                document.crop(*new_rect).ok();
            }
            BackgroundColor { new_color, .. } => {
                document.set_background_color(*new_color);
            }
            SelectionChange { new_selection, .. } => {
                // Apply selection
            }
            ToolStroke { .. } => {
                // Would redo the stroke
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.undo_stack.clear();
    }

    pub fn entries(&self) -> Vec<&HistoryEntry> {
        self.history.iter().collect()
    }

    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        while self.history.len() > max {
            self.history.pop_front();
        }
    }
}

impl Layer {
    fn from_data(_data: LayerData) -> Result<Self> {
        // Would reconstruct layer from data
        Err(anyhow::anyhow!("Not implemented"))
    }

    fn restore_texture_region(&mut self, _region: Rect, _data: &[u8]) {}
    fn restore_mask(&mut self, _data: &MaskData) {}
    fn restore_mask_region(&mut self, _region: Rect, _data: &[u8]) {}
    fn adjustment_mut(&mut self) -> Option<&mut crate::layer::AdjustmentLayer> {
        None
    }
    fn apply_mask(&mut self, _data: &MaskData) {}
}
