use digital_canvas::{
    Document, Layer, LayerId, Color, Vec2, Rect, BlendMode,
    input::InputState, brush::BrushEngine,
};
use anyhow::Result;
use std::sync::Arc;
use crate::render::Renderer;

pub struct Canvas {
    viewport_offset: Vec2,
    viewport_scale: f32,
    viewport_rotation: f32,
    canvas_width: u32,
    canvas_height: u32,
    background_color: Color,
    show_grid: bool,
    grid_size: f32,
    is_panning: bool,
    pan_start: Vec2,
    last_mouse_pos: Vec2,
}

impl Canvas {
    pub fn new(renderer: &mut Renderer, settings: &crate::settings::Settings) -> Result<Self> {
        Ok(Self {
            viewport_offset: Vec2::zero(),
            viewport_scale: 1.0,
            viewport_rotation: 0.0,
            canvas_width: renderer.width(),
            canvas_height: renderer.height(),
            background_color: Color::new(0.15, 0.15, 0.18, 1.0),
            show_grid: settings.canvas.show_grid,
            grid_size: settings.canvas.grid_size,
            is_panning: false,
            pan_start: Vec2::zero(),
            last_mouse_pos: Vec2::zero(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    pub fn update(&mut self, input: &InputState, delta_time: f32, document: &Document) {
        self.last_mouse_pos = input.mouse_position;

        // Handle panning (Space + drag or middle mouse)
        if (input.is_key_pressed(winit::keyboard::KeyCode::Space) && input.is_mouse_pressed(winit::event::MouseButton::Left))
            || input.is_mouse_pressed(winit::event::MouseButton::Middle) {
            if !self.is_panning {
                self.is_panning = true;
                self.pan_start = input.mouse_position - self.viewport_offset;
            }
            self.viewport_offset = input.mouse_position - self.pan_start;
        } else {
            self.is_panning = false;
        }

        // Handle zoom (Ctrl + mouse wheel)
        if input.is_key_pressed(winit::keyboard::KeyCode::ControlLeft) || input.is_key_pressed(winit::keyboard::KeyCode::ControlRight) {
            let wheel_delta = input.mouse_wheel_delta;
            if wheel_delta != 0.0 {
                let zoom_factor = 1.0 + wheel_delta * 0.1;
                let mouse_pos = input.mouse_position;
                let world_pos_before = self.screen_to_world(mouse_pos);
                self.viewport_scale = (self.viewport_scale * zoom_factor).clamp(0.01, 100.0);
                let world_pos_after = self.screen_to_world(mouse_pos);
                self.viewport_offset += world_pos_before - world_pos_after;
            }
        }

        // Handle canvas rotation (R key + drag)
        if input.is_key_pressed(winit::keyboard::KeyCode::KeyR) && input.is_mouse_pressed(winit::event::MouseButton::Left) {
            let center = Vec2::new(self.canvas_width as f32 / 2.0, self.canvas_height as f32 / 2.0);
            let delta = input.mouse_position - self.last_mouse_pos;
            self.viewport_rotation += delta.x * 0.01;
        }

        // Reset view (double-tap H or shortcut)
        if input.was_key_just_pressed(winit::keyboard::KeyCode::KeyH) && input.is_key_pressed(winit::keyboard::KeyCode::ControlLeft) {
            self.reset_view();
        }
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let centered = Vec2::new(
            screen_pos.x - self.canvas_width as f32 / 2.0,
            screen_pos.y - self.canvas_height as f32 / 2.0,
        );
        let rotated = Vec2::new(
            centered.x * self.viewport_rotation.cos() - centered.y * self.viewport_rotation.sin(),
            centered.x * self.viewport_rotation.sin() + centered.y * self.viewport_rotation.cos(),
        );
        let scaled = Vec2::new(
            rotated.x / self.viewport_scale,
            rotated.y / self.viewport_scale,
        );
        Vec2::new(
            scaled.x + self.canvas_width as f32 / 2.0 - self.viewport_offset.x,
            scaled.y + self.canvas_height as f32 / 2.0 - self.viewport_offset.y,
        )
    }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let offset = Vec2::new(
            world_pos.x - self.canvas_width as f32 / 2.0 + self.viewport_offset.x,
            world_pos.y - self.canvas_height as f32 / 2.0 + self.viewport_offset.y,
        );
        let scaled = Vec2::new(
            offset.x * self.viewport_scale,
            offset.y * self.viewport_scale,
        );
        let rotated = Vec2::new(
            scaled.x * self.viewport_rotation.cos() + scaled.y * self.viewport_rotation.sin(),
            -scaled.x * self.viewport_rotation.sin() + scaled.y * self.viewport_rotation.cos(),
        );
        Vec2::new(
            rotated.x + self.canvas_width as f32 / 2.0,
            rotated.y + self.canvas_height as f32 / 2.0,
        )
    }

    pub fn reset_view(&mut self) {
        self.viewport_offset = Vec2::zero();
        self.viewport_scale = 1.0;
        self.viewport_rotation = 0.0;
    }

    pub fn fit_to_screen(&mut self, document: &Document) {
        let doc_width = document.width() as f32;
        let doc_height = document.height() as f32;
        let scale_x = self.canvas_width as f32 / doc_width;
        let scale_y = self.canvas_height as f32 / doc_height;
        self.viewport_scale = scale_x.min(scale_y) * 0.95;
        self.viewport_offset = Vec2::zero();
        self.viewport_rotation = 0.0;
    }

    pub fn set_zoom(&mut self, zoom: f32, center: Option<Vec2>) {
        if let Some(center_pos) = center {
            let world_before = self.screen_to_world(center_pos);
            self.viewport_scale = zoom.clamp(0.01, 100.0);
            let world_after = self.screen_to_world(center_pos);
            self.viewport_offset += world_before - world_after;
        } else {
            self.viewport_scale = zoom.clamp(0.01, 100.0);
        }
    }

    pub fn zoom_in(&mut self) {
        self.set_zoom(self.viewport_scale * 1.25, None);
    }

    pub fn zoom_out(&mut self) {
        self.set_zoom(self.viewport_scale / 1.25, None);
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.viewport_rotation = rotation;
    }

    pub fn rotate_canvas(&mut self, delta: f32) {
        self.viewport_rotation += delta;
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.viewport_offset += delta;
    }

    pub fn mirror_horizontal(&mut self) {
        self.viewport_scale.x = -self.viewport_scale.x.abs();
    }

    pub fn viewport_offset(&self) -> Vec2 {
        self.viewport_offset
    }

    pub fn viewport_scale(&self) -> f32 {
        self.viewport_scale
    }

    pub fn viewport_rotation(&self) -> f32 {
        self.viewport_rotation
    }

    pub fn canvas_size(&self) -> (u32, u32) {
        (self.canvas_width, self.canvas_height)
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    pub fn set_grid_size(&mut self, size: f32) {
        self.grid_size = size.max(1.0);
    }

    pub fn get_transform_matrix(&self) -> glam::Mat3 {
        let cos = self.viewport_rotation.cos();
        let sin = self.viewport_rotation.sin();
        glam::Mat3::from_cols(
            glam::Vec3::new(cos * self.viewport_scale, sin * self.viewport_scale, 0.0),
            glam::Vec3::new(-sin * self.viewport_scale, cos * self.viewport_scale, 0.0),
            glam::Vec3::new(
                -self.viewport_offset.x * self.viewport_scale + self.canvas_width as f32 / 2.0,
                -self.viewport_offset.y * self.viewport_scale + self.canvas_height as f32 / 2.0,
                1.0,
            ),
        )
    }
}