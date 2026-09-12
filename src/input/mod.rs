use digital_canvas::{Vec2, Tool};
use std::collections::HashSet;
use winit::{
    event::{KeyEvent, MouseButton, DeviceEvent, MouseScrollDelta, TouchPhase},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub mouse_wheel_delta: f32,
    pub pressed_keys: HashSet<KeyCode>,
    pub pressed_mouse_buttons: HashSet<MouseButton>,
    pub modifiers: ModifiersState,
    pub active_tool: Tool,
    pub tablet_pressure: f32,
    pub tablet_tilt: Vec2,
    pub tablet_rotation: f32,
    pub is_focused: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModifiersState {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

impl ModifiersState {
    pub fn from_winit(modifiers: winit::event::Modifiers) -> Self {
        Self {
            ctrl: modifiers.state().control_key(),
            shift: modifiers.state().shift_key(),
            alt: modifiers.state().alt_key(),
            meta: modifiers.state().super_key(),
            caps_lock: modifiers.state().caps_lock(),
            num_lock: modifiers.state().num_lock(),
        }
    }
}

pub struct InputManager {
    state: InputState,
    previous_state: InputState,
    pressure_curve: PressureCurve,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            state: InputState::default(),
            previous_state: InputState::default(),
            pressure_curve: PressureCurve::default(),
        }
    }

    pub fn state(&self) -> &InputState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut InputState {
        &mut self.state
    }

    pub fn update(&mut self) {
        self.previous_state = self.state.clone();
        self.state.mouse_delta = Vec2::zero();
        self.state.mouse_wheel_delta = 0.0;
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            if event.state.is_pressed() {
                self.state.pressed_keys.insert(key_code);
            } else {
                self.state.pressed_keys.remove(&key_code);
            }
        }
    }

    pub fn handle_raw_key_event(&mut self, event: KeyEvent) {
        // Handle raw key events for tablet keys, etc.
        if let PhysicalKey::Code(key_code) = event.physical_key {
            if event.state.is_pressed() {
                self.state.pressed_keys.insert(key_code);
            } else {
                self.state.pressed_keys.remove(&key_code);
            }
        }
    }

    pub fn handle_cursor_move(&mut self, x: f32, y: f32) {
        self.state.mouse_position = Vec2::new(x, y);
    }

    pub fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.state.mouse_delta = Vec2::new(dx, dy);
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: winit::event::ElementState) {
        if state.is_pressed() {
            self.state.pressed_mouse_buttons.insert(button);
        } else {
            self.state.pressed_mouse_buttons.remove(&button);
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                self.state.mouse_wheel_delta = y;
            }
            MouseScrollDelta::PixelDelta(pos) => {
                self.state.mouse_wheel_delta = pos.y as f32 * 0.1;
            }
        }
    }

    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.state.modifiers = modifiers;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.state.is_focused = focused;
        if !focused {
            self.state.pressed_keys.clear();
            self.state.pressed_mouse_buttons.clear();
        }
    }

    pub fn set_active_tool(&mut self, tool: Tool) {
        self.state.active_tool = tool;
    }

    pub fn active_tool(&self) -> Tool {
        self.state.active_tool
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.state.pressed_keys.contains(&key)
    }

    pub fn was_key_just_pressed(&self, key: KeyCode) -> bool {
        self.state.pressed_keys.contains(&key) && !self.previous_state.pressed_keys.contains(&key)
    }

    pub fn was_key_just_released(&self, key: KeyCode) -> bool {
        !self.state.pressed_keys.contains(&key) && self.previous_state.pressed_keys.contains(&key)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.state.pressed_mouse_buttons.contains(&button)
    }

    pub fn was_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.state.pressed_mouse_buttons.contains(&button) && !self.previous_state.pressed_mouse_buttons.contains(&button)
    }

    pub fn was_mouse_just_released(&self, button: MouseButton) -> bool {
        !self.state.pressed_mouse_buttons.contains(&button) && self.previous_state.pressed_mouse_buttons.contains(&button)
    }

    pub fn handle_tablet_event(&mut self, event: TabletEvent) {
        match event {
            TabletEvent::Pressure(pressure) => {
                self.state.tablet_pressure = self.pressure_curve.apply(pressure);
            }
            TabletEvent::Tilt(tilt_x, tilt_y) => {
                self.state.tablet_tilt = Vec2::new(tilt_x, tilt_y);
            }
            TabletEvent::Rotation(rotation) => {
                self.state.tablet_rotation = rotation;
            }
            TabletEvent::Button(button, pressed) => {
                // Map tablet buttons to mouse buttons or custom actions
            }
        }
    }

    pub fn pressure_curve(&self) -> &PressureCurve {
        &self.pressure_curve
    }

    pub fn pressure_curve_mut(&mut self) -> &mut PressureCurve {
        &mut self.pressure_curve
    }
}

#[derive(Debug, Clone)]
pub enum TabletEvent {
    Pressure(f32),
    Tilt(f32, f32),
    Rotation(f32),
    Button(u32, bool),
}

#[derive(Debug, Clone)]
pub struct PressureCurve {
    points: Vec<(f32, f32)>,
}

impl Default for PressureCurve {
    fn default() -> Self {
        Self {
            points: vec![
                (0.0, 0.0),
                (0.25, 0.15),
                (0.5, 0.5),
                (0.75, 0.85),
                (1.0, 1.0),
            ],
        }
    }
}

impl PressureCurve {
    pub fn apply(&self, input: f32) -> f32 {
        let t = input.clamp(0.0, 1.0);
        if t <= 0.0 { return 0.0; }
        if t >= 1.0 { return 1.0; }

        for i in 0..self.points.len() - 1 {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[i + 1];
            if t >= x1 && t <= x2 {
                let local_t = (t - x1) / (x2 - x1);
                return y1 + (y2 - y1) * Self::smooth_step(local_t);
            }
        }
        t
    }

    fn smooth_step(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }

    pub fn set_point(&mut self, index: usize, value: (f32, f32)) {
        if index < self.points.len() {
            self.points[index] = value;
            self.points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
    }

    pub fn add_point(&mut self, value: (f32, f32)) {
        self.points.push(value);
        self.points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    pub fn remove_point(&mut self, index: usize) {
        if index < self.points.len() && self.points.len() > 2 {
            self.points.remove(index);
        }
    }

    pub fn points(&self) -> &[(f32, f32)] {
        &self.points
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(target_os = "windows")]
pub mod windows_tablet {
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use std::ffi::c_void;

    pub unsafe fn initialize_tablet_support() -> Result<(), windows::core::Error> {
        // Register for tablet/pen events
        // This would require a window handle and message hook
        Ok(())
    }

    pub fn get_tablet_info() -> Vec<TabletDeviceInfo> {
        // Enumerate tablet devices using Wintab or Windows Ink APIs
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct TabletDeviceInfo {
    pub name: String,
    pub id: String,
    pub max_pressure: u32,
    pub supports_tilt: bool,
    pub supports_rotation: bool,
    pub supports_eraser: bool,
}
