use digital_canvas::{Rect, Vec2, Color, EntityId};
use anyhow::Result;
use image::{RgbaImage, GrayImage, ImageBuffer};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    None,
    Rectangle,
    Ellipse,
    Lasso,
    Polygon,
    MagicWand,
    QuickSelection,
    ColorRange,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub selection_type: SelectionType,
    pub bounds: Rect,
    pub mask: Option<GrayImage>,
    pub feather: f32,
    pub anti_aliased: bool,
    pub path_points: Vec<Vec2>, // For lasso/polygon
}

impl Selection {
    pub fn new() -> Self {
        Self {
            selection_type: SelectionType::None,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            mask: None,
            feather: 0.0,
            anti_aliased: true,
            path_points: Vec::new(),
        }
    }

    pub fn rectangle(rect: Rect, feather: f32) -> Self {
        Self {
            selection_type: SelectionType::Rectangle,
            bounds: rect,
            mask: None,
            feather,
            anti_aliased: true,
            path_points: Vec::new(),
        }
    }

    pub fn ellipse(rect: Rect, feather: f32) -> Self {
        Self {
            selection_type: SelectionType::Ellipse,
            bounds: rect,
            mask: None,
            feather,
            anti_aliased: true,
            path_points: Vec::new(),
        }
    }

    pub fn from_mask(mask: GrayImage, feather: f32) -> Self {
        let bounds = Rect::new(0.0, 0.0, mask.width() as f32, mask.height() as f32);
        Self {
            selection_type: SelectionType::MagicWand,
            bounds,
            mask: Some(mask),
            feather,
            anti_aliased: true,
            path_points: Vec::new(),
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        if !self.bounds.contains(point) {
            return false;
        }

        match self.selection_type {
            SelectionType::Rectangle => true,
            SelectionType::Ellipse => {
                let cx = self.bounds.x + self.bounds.width / 2.0;
                let cy = self.bounds.y + self.bounds.height / 2.0;
                let rx = self.bounds.width / 2.0;
                let ry = self.bounds.height / 2.0;
                let dx = (point.x - cx) / rx;
                let dy = (point.y - cy) / ry;
                dx * dx + dy * dy <= 1.0
            }
            SelectionType::MagicWand | SelectionType::QuickSelection | SelectionType::ColorRange => {
                if let Some(mask) = &self.mask {
                    let x = point.x as u32;
                    let y = point.y as u32;
                    if x < mask.width() && y < mask.height() {
                        mask.get_pixel(x, y).0[0] > 128
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            SelectionType::Lasso | SelectionType::Polygon => {
                // Point in polygon test
                self.point_in_polygon(point)
            }
            SelectionType::None => false,
        }
    }

    fn point_in_polygon(&self, point: Vec2) -> bool {
        let mut inside = false;
        let points = &self.path_points;
        let n = points.len();
        if n < 3 {
            return false;
        }

        for i in 0..n {
            let j = (i + 1) % n;
            let pi = points[i];
            let pj = points[j];

            if ((pi.y > point.y) != (pj.y > point.y)) &&
               (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
                inside = !inside;
            }
        }
        inside
    }

    pub fn get_mask(&self, width: u32, height: u32) -> GrayImage {
        if let Some(mask) = &self.mask {
            return mask.clone();
        }

        let mut mask = GrayImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let point = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if self.contains(point) {
                    let alpha = if self.feather > 0.0 {
                        self.calculate_feather(point)
                    } else {
                        255
                    };
                    mask.put_pixel(x, y, image::Luma([alpha]));
                }
            }
        }

        mask
    }

    fn calculate_feather(&self, point: Vec2) -> u8 {
        // Simplified feather calculation
        255
    }

    pub fn invert(&mut self, width: u32, height: u32) {
        if let Some(mask) = &mut self.mask {
            for pixel in mask.pixels_mut() {
                pixel.0[0] = 255 - pixel.0[0];
            }
        } else {
            self.mask = Some(self.get_mask(width, height));
            self.invert(width, height);
        }
    }

    pub fn expand(&mut self, radius: f32) {
        // Morphological dilation
        if let Some(mask) = &mut self.mask {
            *mask = morphology::dilate(mask, radius as u32);
        }
        self.bounds = Rect::new(
            self.bounds.x - radius,
            self.bounds.y - radius,
            self.bounds.width + 2.0 * radius,
            self.bounds.height + 2.0 * radius,
        );
    }

    pub fn contract(&mut self, radius: f32) {
        // Morphological erosion
        if let Some(mask) = &mut self.mask {
            *mask = morphology::erode(mask, radius as u32);
        }
        self.bounds = Rect::new(
            self.bounds.x + radius,
            self.bounds.y + radius,
            self.bounds.width - 2.0 * radius,
            self.bounds.height - 2.0 * radius,
        );
    }

    pub fn feather(&mut self, radius: f32) {
        self.feather = radius;
        if let Some(mask) = &mut self.mask {
            *mask = morphology::blur(mask, radius);
        }
    }

    pub fn intersect(&mut self, other: &Selection, width: u32, height: u32) {
        let mask1 = self.get_mask(width, height);
        let mask2 = other.get_mask(width, height);
        
        let mut result = GrayImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let v1 = mask1.get_pixel(x, y).0[0];
                let v2 = mask2.get_pixel(x, y).0[0];
                result.put_pixel(x, y, image::Luma([v1.min(v2)]));
            }
        }
        
        self.mask = Some(result);
        self.selection_type = SelectionType::MagicWand;
    }

    pub fn union(&mut self, other: &Selection, width: u32, height: u32) {
        let mask1 = self.get_mask(width, height);
        let mask2 = other.get_mask(width, height);
        
        let mut result = GrayImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let v1 = mask1.get_pixel(x, y).0[0];
                let v2 = mask2.get_pixel(x, y).0[0];
                result.put_pixel(x, y, image::Luma([v1.max(v2)]));
            }
        }
        
        self.mask = Some(result);
        self.selection_type = SelectionType::MagicWand;
    }

    pub fn subtract(&mut self, other: &Selection, width: u32, height: u32) {
        let mask1 = self.get_mask(width, height);
        let mask2 = other.get_mask(width, height);
        
        let mut result = GrayImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let v1 = mask1.get_pixel(x, y).0[0];
                let v2 = mask2.get_pixel(x, y).0[0];
                let v = if v1 > v2 { v1 - v2 } else { 0 };
                result.put_pixel(x, y, image::Luma([v]));
            }
        }
        
        self.mask = Some(result);
        self.selection_type = SelectionType::MagicWand;
    }

    pub fn is_empty(&self) -> bool {
        match self.selection_type {
            SelectionType::None => true,
            SelectionType::Rectangle | SelectionType::Ellipse => {
                self.bounds.width <= 0.0 || self.bounds.height <= 0.0
            }
            _ => {
                if let Some(mask) = &self.mask {
                    mask.pixels().all(|p| p.0[0] == 0)
                } else {
                    true
                }
            }
        }
    }

    pub fn area(&self, width: u32, height: u32) -> u32 {
        let mask = self.get_mask(width, height);
        mask.pixels().filter(|p| p.0[0] > 0).count() as u32
    }

    pub fn centroid(&self, width: u32, height: u32) -> Option<Vec2> {
        let mask = self.get_mask(width, height);
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut count = 0;

        for y in 0..height {
            for x in 0..width {
                let alpha = mask.get_pixel(x, y).0[0] as f32 / 255.0;
                if alpha > 0.0 {
                    sum_x += x as f32 * alpha;
                    sum_y += y as f32 * alpha;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(Vec2::new(sum_x / count as f32, sum_y / count as f32))
        } else {
            None
        }
    }
}

pub struct SelectionManager {
    current: Selection,
    history: VecDeque<Selection>,
    max_history: usize,
    active_tool: SelectionTool,
    tool_state: SelectionToolState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionTool {
    Rectangle,
    Ellipse,
    Lasso,
    Polygon,
    MagicWand,
    QuickSelection,
    ColorRange,
    Move,
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone)]
pub enum SelectionToolState {
    Idle,
    Drawing { start: Vec2, current: Vec2 },
    Polygon { points: Vec<Vec2> },
    MagicWand { tolerance: f32, contiguous: bool, sample_all_layers: bool },
    QuickSelection { brush_size: f32, mode: QuickSelectMode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSelectMode {
    Add,
    Subtract,
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            current: Selection::new(),
            history: VecDeque::new(),
            max_history: 50,
            active_tool: SelectionTool::Rectangle,
            tool_state: SelectionToolState::Idle,
        }
    }

    pub fn current(&self) -> &Selection {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut Selection {
        self.push_history();
        &mut self.current
    }

    pub fn set_tool(&mut self, tool: SelectionTool) {
        self.active_tool = tool;
        self.tool_state = SelectionToolState::Idle;
    }

    pub fn active_tool(&self) -> SelectionTool {
        self.active_tool
    }

    pub fn handle_mouse_down(&mut self, pos: Vec2, modifier: SelectionModifier) {
        match self.active_tool {
            SelectionTool::Rectangle | SelectionTool::Ellipse => {
                self.tool_state = SelectionToolState::Drawing { start: pos, current: pos };
            }
            SelectionTool::Lasso => {
                self.tool_state = SelectionToolState::Drawing { start: pos, current: pos };
                self.current = Selection::new();
                self.current.selection_type = SelectionType::Lasso;
                self.current.path_points.push(pos);
            }
            SelectionTool::Polygon => {
                if let SelectionToolState::Polygon { points } = &mut self.tool_state {
                    points.push(pos);
                    self.current.path_points.push(pos);
                    self.update_polygon_bounds();
                } else {
                    self.tool_state = SelectionToolState::Polygon { points: vec![pos] };
                    self.current = Selection::new();
                    self.current.selection_type = SelectionType::Polygon;
                    self.current.path_points.push(pos);
                }
            }
            SelectionTool::MagicWand => {
                if let SelectionToolState::MagicWand { tolerance, contiguous, sample_all_layers } = self.tool_state {
                    // Would perform flood fill
                }
            }
            SelectionTool::QuickSelection => {
                // Would perform quick selection
            }
            SelectionTool::Move => {
                // Move selection
            }
            _ => {}
        }
    }

    pub fn handle_mouse_move(&mut self, pos: Vec2) {
        match &mut self.tool_state {
            SelectionToolState::Drawing { start, current } => {
                *current = pos;
                let rect = Rect::new(
                    start.x.min(pos.x),
                    start.y.min(pos.y),
                    (start.x - pos.x).abs(),
                    (start.y - pos.y).abs(),
                );
                self.current.bounds = rect;
                self.current.selection_type = match self.active_tool {
                    SelectionTool::Rectangle => SelectionType::Rectangle,
                    SelectionTool::Ellipse => SelectionType::Ellipse,
                    SelectionTool::Lasso => SelectionType::Lasso,
                    _ => SelectionType::Rectangle,
                };
                if self.active_tool == SelectionTool::Lasso {
                    self.current.path_points.push(pos);
                }
            }
            SelectionToolState::Polygon { .. } => {
                // Preview line to current position
            }
            _ => {}
        }
    }

    pub fn handle_mouse_up(&mut self, pos: Vec2, modifier: SelectionModifier, width: u32, height: u32) {
        match self.active_tool {
            SelectionTool::Rectangle | SelectionTool::Ellipse => {
                if let SelectionToolState::Drawing { start, current } = &self.tool_state {
                    let rect = Rect::new(
                        start.x.min(*current.x.max(pos.x)),
                        start.y.min(*current.y.max(pos.y)),
                        (start.x - pos.x).abs(),
                        (start.y - pos.y).abs(),
                    );
                    
                    self.apply_modifier(modifier, width, height);
                    self.current = if self.active_tool == SelectionTool::Rectangle {
                        Selection::rectangle(rect, 0.0)
                    } else {
                        Selection::ellipse(rect, 0.0)
                    };
                }
                self.tool_state = SelectionToolState::Idle;
            }
            SelectionTool::Lasso => {
                if let SelectionToolState::Drawing { start, .. } = &self.tool_state {
                    // Close the lasso
                    self.current.path_points.push(*start);
                    self.apply_modifier(modifier, width, height);
                }
                self.tool_state = SelectionToolState::Idle;
            }
            SelectionTool::Polygon => {
                // Polygon stays open until double-click or enter
            }
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        match (key, pressed) {
            (winit::keyboard::KeyCode::Enter, true) => {
                if let SelectionToolState::Polygon { points } = &self.tool_state {
                    if points.len() >= 3 {
                        self.current.path_points.push(points[0]); // Close polygon
                        self.apply_modifier(SelectionModifier::None, 1920, 1080);
                        self.tool_state = SelectionToolState::Idle;
                    }
                }
            }
            (winit::keyboard::KeyCode::Escape, true) => {
                self.tool_state = SelectionToolState::Idle;
                self.current = Selection::new();
            }
            _ => {}
        }
    }

    fn apply_modifier(&mut self, modifier: SelectionModifier, width: u32, height: u32) {
        // In a real implementation, we'd combine with existing selection based on modifier
        match modifier {
            SelectionModifier::Add => {
                // Union with existing
            }
            SelectionModifier::Subtract => {
                // Subtract from existing
            }
            SelectionModifier::Intersect => {
                // Intersect with existing
            }
            SelectionModifier::None => {
                // Replace
            }
        }
    }

    fn update_polygon_bounds(&mut self) {
        if self.current.path_points.is_empty() {
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for point in &self.current.path_points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        self.current.bounds = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
    }

    fn push_history(&mut self) {
        self.history.push_back(self.current.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.pop_back() {
            self.current = prev;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.push_history();
        self.current = Selection::new();
    }

    pub fn select_all(&mut self, width: u32, height: u32) {
        self.push_history();
        self.current = Selection::rectangle(
            Rect::new(0.0, 0.0, width as f32, height as f32),
            0.0,
        );
    }

    pub fn deselect(&mut self) {
        self.push_history();
        self.current = Selection::new();
    }

    pub fn invert(&mut self, width: u32, height: u32) {
        self.push_history();
        self.current.invert(width, height);
    }

    pub fn has_selection(&self) -> bool {
        !self.current.is_empty()
    }

    pub fn bounds(&self) -> Rect {
        self.current.bounds
    }

    pub fn feather(&mut self, radius: f32) {
        self.push_history();
        self.current.feather(radius);
    }

    pub fn expand(&mut self, radius: f32) {
        self.push_history();
        self.current.expand(radius);
    }

    pub fn contract(&mut self, radius: f32) {
        self.push_history();
        self.current.contract(radius);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionModifier {
    None,
    Add,
    Subtract,
    Intersect,
}

impl SelectionModifier {
    pub fn from_modifiers(shift: bool, alt: bool) -> Self {
        match (shift, alt) {
            (true, false) => SelectionModifier::Add,
            (false, true) => SelectionModifier::Subtract,
            (true, true) => SelectionModifier::Intersect,
            _ => SelectionModifier::None,
        }
    }
}

mod morphology {
    use image::{GrayImage, ImageBuffer, Luma};

    pub fn dilate(image: &GrayImage, radius: u32) -> GrayImage {
        let mut result = GrayImage::new(image.width(), image.height());
        let r = radius as i32;

        for y in 0..image.height() {
            for x in 0..image.width() {
                let mut max_val = 0u8;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx*dx + dy*dy <= r*r {
                            let nx = (x as i32 + dx).clamp(0, image.width() as i32 - 1) as u32;
                            let ny = (y as i32 + dy).clamp(0, image.height() as i32 - 1) as u32;
                            max_val = max_val.max(image.get_pixel(nx, ny).0[0]);
                        }
                    }
                }
                result.put_pixel(x, y, Luma([max_val]));
            }
        }
        result
    }

    pub fn erode(image: &GrayImage, radius: u32) -> GrayImage {
        let mut result = GrayImage::new(image.width(), image.height());
        let r = radius as i32;

        for y in 0..image.height() {
            for x in 0..image.width() {
                let mut min_val = 255u8;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx*dx + dy*dy <= r*r {
                            let nx = (x as i32 + dx).clamp(0, image.width() as i32 - 1) as u32;
                            let ny = (y as i32 + dy).clamp(0, image.height() as i32 - 1) as u32;
                            min_val = min_val.min(image.get_pixel(nx, ny).0[0]);
                        }
                    }
                }
                result.put_pixel(x, y, Luma([min_val]));
            }
        }
        result
    }

    pub fn blur(image: &GrayImage, radius: f32) -> GrayImage {
        // Gaussian blur approximation
        let r = radius.ceil() as u32;
        let mut result = GrayImage::new(image.width(), image.height());
        
        // Horizontal pass
        let mut temp = GrayImage::new(image.width(), image.height());
        for y in 0..image.height() {
            for x in 0..image.width() {
                let mut sum = 0.0f32;
                let mut weight_sum = 0.0f32;
                for dx in -(r as i32)..=(r as i32) {
                    let nx = (x as i32 + dx).clamp(0, image.width() as i32 - 1) as u32;
                    let weight = (-(dx as f32).powi(2) / (2.0 * radius * radius)).exp();
                    sum += image.get_pixel(nx, y).0[0] as f32 * weight;
                    weight_sum += weight;
                }
                temp.put_pixel(x, y, Luma([(sum / weight_sum) as u8]));
            }
        }

        // Vertical pass
        for y in 0..image.height() {
            for x in 0..image.width() {
                let mut sum = 0.0f32;
                let mut weight_sum = 0.0f32;
                for dy in -(r as i32)..=(r as i32) {
                    let ny = (y as i32 + dy).clamp(0, image.height() as i32 - 1) as u32;
                    let weight = (-(dy as f32).powi(2) / (2.0 * radius * radius)).exp();
                    sum += temp.get_pixel(x, ny).0[0] as f32 * weight;
                    weight_sum += weight;
                }
                result.put_pixel(x, y, Luma([(sum / weight_sum) as u8]));
            }
        }
        result
    }
}
