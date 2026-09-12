use digital_canvas::{Color, Tool};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

slint::include_modules!();

pub struct UiManager {
    main_window: MainWindow,
    settings: digital_canvas::settings::Settings,
}

impl UiManager {
    pub fn new(settings: &digital_canvas::settings::Settings) -> Result<Self, slint::PlatformError> {
        let main_window = MainWindow::new()?;
        
        let mut ui = Self {
            main_window,
            settings: settings.clone(),
        };

        ui.setup_callbacks();
        
        Ok(ui)
    }

    fn setup_callbacks(&mut self) {
        // Tool selection
        let weak = self.main_window.as_weak();
        self.main_window.on_tool_selected(move |tool_name: SharedString| {
            if let Some(_window) = weak.upgrade() {
                println!("Tool selected: {}", tool_name);
            }
        });

        // Brush size change
        let weak = self.main_window.as_weak();
        self.main_window.on_brush_size_changed(move |size: f32| {
            if let Some(_window) = weak.upgrade() {
                println!("Brush size: {}", size);
            }
        });

        // Brush opacity change
        let weak = self.main_window.as_weak();
        self.main_window.on_brush_opacity_changed(move |opacity: f32| {
            if let Some(_window) = weak.upgrade() {
                println!("Brush opacity: {}", opacity);
            }
        });

        // File operations
        let weak = self.main_window.as_weak();
        self.main_window.on_new_document(move || {
            if let Some(_window) = weak.upgrade() {
                println!("New document");
            }
        });

        let weak = self.main_window.as_weak();
        self.main_window.on_open_document(move || {
            if let Some(_window) = weak.upgrade() {
                println!("Open document");
            }
        });

        let weak = self.main_window.as_weak();
        self.main_window.on_save_document(move || {
            if let Some(_window) = weak.upgrade() {
                println!("Save document");
            }
        });

        // Edit operations
        let weak = self.main_window.as_weak();
        self.main_window.on_undo(move || {
            if let Some(_window) = weak.upgrade() {
                println!("Undo");
            }
        });

        let weak = self.main_window.as_weak();
        self.main_window.on_redo(move || {
            if let Some(_window) = weak.upgrade() {
                println!("Redo");
            }
        });
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.main_window.run()
    }

    pub fn window(&self) -> &MainWindow {
        &self.main_window
    }

    pub fn set_active_tool(&self, tool: Tool) {
        self.main_window.set_active_tool(tool.as_str().into());
    }

    pub fn set_brush_size(&self, size: f32) {
        self.main_window.set_brush_size(size);
    }

    pub fn set_brush_opacity(&self, opacity: f32) {
        self.main_window.set_brush_opacity(opacity);
    }
}

// Tool string conversion
impl Tool {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tool::Brush => "brush",
            Tool::Eraser => "eraser",
            Tool::Move => "move",
            Tool::Marquee => "marquee",
            Tool::Lasso => "lasso",
            Tool::MagicWand => "magic_wand",
            Tool::Eyedropper => "eyedropper",
            Tool::Gradient => "gradient",
            Tool::Crop => "crop",
            Tool::Zoom => "zoom",
            Tool::Hand => "hand",
            Tool::Rotate => "rotate",
            Tool::Text => "text",
            Tool::Pen => "pen",
            Tool::CloneStamp => "clone_stamp",
            Tool::HealingBrush => "healing",
            Tool::Dodge => "dodge",
            Tool::Burn => "burn",
            Tool::Blur => "blur",
            Tool::Sharpen => "sharpen",
            Tool::Smudge => "smudge",
        }
    }
}
