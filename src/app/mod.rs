use digital_canvas::{
    Canvas, Document, Renderer, InputManager, Settings, FileManager,
    HistoryManager, TimelapseRecorder, SelectionManager, FilterEngine,
    BrushEngine, Color, Layer, EntityId, LayerId, DocumentId,
};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, KeyEvent, ElementState, MouseButton, DeviceId},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

pub struct Application {
    event_loop: Option<EventLoop<()>>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    canvas: Option<Canvas>,
    input_manager: InputManager,
    document: Option<Document>,
    settings: Settings,
    file_manager: FileManager,
    history: HistoryManager,
    timelapse: TimelapseRecorder,
    selection: SelectionManager,
    filter_engine: FilterEngine,
    brush_engine: BrushEngine,
    last_frame_time: Instant,
    frame_count: u64,
    running: bool,
}

impl Application {
    pub fn new() -> Result<Self> {
        let settings = Settings::load()?;
        let file_manager = FileManager::new(&settings)?;
        let history = HistoryManager::new(settings.history.max_history_states);
        let timelapse = TimelapseRecorder::new(&settings.timelapse)?;
        let selection = SelectionManager::new();
        let filter_engine = FilterEngine::new();
        let brush_engine = BrushEngine::new(&settings.brush)?;
        let input_manager = InputManager::new();

        Ok(Self {
            event_loop: None,
            window: None,
            renderer: None,
            canvas: None,
            input_manager,
            document: None,
            settings,
            file_manager,
            history,
            timelapse,
            selection,
            filter_engine,
            brush_engine,
            last_frame_time: Instant::now(),
            frame_count: 0,
            running: false,
        })
    }

    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        
        let mut app_handler = AppHandler::new(&mut self);
        event_loop.run_app(&mut app_handler)?;

        Ok(())
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let attrs = WindowAttributes::default()
            .with_title("DigitalCanvas")
            .with_inner_size(winit::dpi::LogicalSize::new(1600, 1000))
            .with_min_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .with_resizable(true)
            .with_visible(true);

        let window = Arc::new(event_loop.create_window(attrs)?);
        window.set_cursor_icon(winit::window::CursorIcon::Crosshair);

        self.window = Some(window.clone());

        let renderer = Renderer::new(window.clone(), &self.settings)?;
        self.renderer = Some(renderer);

        let renderer = self.renderer.as_mut().unwrap();
        let canvas = Canvas::new(renderer, &self.settings)?;
        self.canvas = Some(canvas);

        let document = Document::new(
            DocumentId::new(),
            "Untitled",
            1920,
            1080,
            &self.settings,
        )?;
        self.document = Some(document);

        Ok(())
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
                if let Some(el) = self.event_loop.as_ref() {
                    el.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                if let Some(canvas) = &mut self.canvas {
                    canvas.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_manager.handle_key_event(event);
                self.handle_shortcuts(event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_manager.handle_cursor_move(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.input_manager.handle_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_manager.handle_mouse_wheel(delta);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_manager.update_modifiers(modifiers.state());
            }
            WindowEvent::Focused(focused) => {
                self.input_manager.set_focused(focused);
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.input_manager.handle_mouse_motion(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::Key(key_event) => {
                self.input_manager.handle_raw_key_event(key_event);
            }
            _ => {}
        }
    }

    fn handle_shortcuts(&mut self, event: KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        let ctrl = self.input_state().ctrl;
        let shift = self.input_state().shift;
        let alt = self.input_state().alt;

        match (event.physical_key, ctrl, shift, alt) {
            (PhysicalKey::Code(KeyCode::KeyN), true, false, false) => self.new_document(),
            (PhysicalKey::Code(KeyCode::KeyO), true, false, false) => self.open_document(),
            (PhysicalKey::Code(KeyCode::KeyS), true, false, false) => self.save_document(),
            (PhysicalKey::Code(KeyCode::KeyS), true, true, false) => self.save_document_as(),
            (PhysicalKey::Code(KeyCode::KeyZ), true, false, false) => self.undo(),
            (PhysicalKey::Code(KeyCode::KeyZ), true, true, false) => self.redo(),
            (PhysicalKey::Code(KeyCode::KeyY), true, false, false) => self.redo(),
            (PhysicalKey::Code(KeyCode::KeyC), true, false, false) => self.copy(),
            (PhysicalKey::Code(KeyCode::KeyV), true, false, false) => self.paste(),
            (PhysicalKey::Code(KeyCode::KeyX), true, false, false) => self.cut(),
            (PhysicalKey::Code(KeyCode::KeyA), true, false, false) => self.select_all(),
            (PhysicalKey::Code(KeyCode::KeyD), true, false, false) => self.deselect(),
            (PhysicalKey::Code(KeyCode::KeyT), true, false, false) => self.transform(),
            (PhysicalKey::Code(KeyCode::BracketLeft), false, false, false) => self.decrease_brush_size(),
            (PhysicalKey::Code(KeyCode::BracketRight), false, false, false) => self.increase_brush_size(),
            (PhysicalKey::Code(KeyCode::KeyB), false, false, false) => self.select_tool(Tool::Brush),
            (PhysicalKey::Code(KeyCode::KeyE), false, false, false) => self.select_tool(Tool::Eraser),
            (PhysicalKey::Code(KeyCode::KeyV), false, false, false) => self.select_tool(Tool::Move),
            (PhysicalKey::Code(KeyCode::KeyM), false, false, false) => self.select_tool(Tool::Marquee),
            (PhysicalKey::Code(KeyCode::KeyL), false, false, false) => self.select_tool(Tool::Lasso),
            (PhysicalKey::Code(KeyCode::KeyW), false, false, false) => self.select_tool(Tool::MagicWand),
            (PhysicalKey::Code(KeyCode::KeyI), false, false, false) => self.select_tool(Tool::Eyedropper),
            (PhysicalKey::Code(KeyCode::KeyG), false, false, false) => self.select_tool(Tool::Gradient),
            (PhysicalKey::Code(KeyCode::KeyC), false, false, false) => self.select_tool(Tool::Crop),
            (PhysicalKey::Code(KeyCode::KeyZ), false, false, false) => self.select_tool(Tool::Zoom),
            (PhysicalKey::Code(KeyCode::KeyH), false, false, false) => self.select_tool(Tool::Hand),
            (PhysicalKey::Code(KeyCode::KeyR), false, false, false) => self.select_tool(Tool::Rotate),
            _ => {}
        }
    }

    fn input_state(&self) -> &digital_canvas::input::InputState {
        self.input_manager.state()
    }

    fn render_frame(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_count += 1;

        if let (Some(renderer), Some(canvas), Some(document)) = 
            (&mut self.renderer, &mut self.canvas, &self.document) {
            
            let input_state = self.input_manager.state().clone();
            canvas.update(&input_state, delta_time, document);
            
            if let Err(e) = renderer.render(canvas, document, &self.selection) {
                tracing::error!("Render error: {}", e);
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn new_document(&mut self) {
        tracing::info!("New document");
    }

    fn open_document(&mut self) {
        tracing::info!("Open document");
    }

    fn save_document(&mut self) {
        tracing::info!("Save document");
    }

    fn save_document_as(&mut self) {
        tracing::info!("Save document as");
    }

    fn undo(&mut self) {
        if let Some(document) = &mut self.document {
            self.history.undo(document);
        }
    }

    fn redo(&mut self) {
        if let Some(document) = &mut self.document {
            self.history.redo(document);
        }
    }

    fn copy(&mut self) {
        tracing::info!("Copy");
    }

    fn paste(&mut self) {
        tracing::info!("Paste");
    }

    fn cut(&mut self) {
        tracing::info!("Cut");
    }

    fn select_all(&mut self) {
        if let Some(document) = &self.document {
            self.selection.select_all(document.width(), document.height());
        }
    }

    fn deselect(&mut self) {
        self.selection.clear();
    }

    fn transform(&mut self) {
        tracing::info!("Transform");
    }

    fn decrease_brush_size(&mut self) {
        self.brush_engine.decrease_size();
    }

    fn increase_brush_size(&mut self) {
        self.brush_engine.increase_size();
    }

    fn select_tool(&mut self, tool: Tool) {
        self.input_manager.set_active_tool(tool);
        tracing::info!("Tool selected: {:?}", tool);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Brush,
    Eraser,
    Move,
    Marquee,
    Lasso,
    MagicWand,
    Eyedropper,
    Gradient,
    Crop,
    Zoom,
    Hand,
    Rotate,
    Text,
    Pen,
    CloneStamp,
    HealingBrush,
    Dodge,
    Burn,
    Blur,
    Sharpen,
    Smudge,
}

struct AppHandler<'a> {
    app: &'a mut Application,
}

impl<'a> AppHandler<'a> {
    fn new(app: &'a mut Application) -> Self {
        Self { app }
    }
}

impl ApplicationHandler for AppHandler<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.window.is_none() {
            if let Err(e) = self.app.create_window(event_loop) {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.app.handle_window_event(event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.app.handle_device_event(event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.app.running {
            if let Some(window) = &self.app.window {
                window.request_redraw();
            }
        }
    }
}
