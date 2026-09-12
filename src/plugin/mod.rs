use digital_canvas::{Document, Layer, LayerId, Color, Rect, Vec2, Tool, BlendMode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    
    fn initialize(&mut self, context: &mut PluginContext) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

pub struct PluginContext {
    pub document: Option<Arc<Mutex<Document>>>,
    pub settings: Arc<Mutex<digital_canvas::settings::Settings>>,
    pub brush_engine: Arc<Mutex<digital_canvas::brush::BrushEngine>>,
    pub file_manager: Arc<Mutex<digital_canvas::file::FileManager>>,
    pub commands: Arc<Mutex<CommandRegistry>>,
    pub ui_extension: Arc<Mutex<UiExtension>>,
}

impl PluginContext {
    pub fn new() -> Self {
        Self {
            document: None,
            settings: Arc::new(Mutex::new(digital_canvas::settings::Settings::default())),
            brush_engine: Arc::new(Mutex::new(digital_canvas::brush::BrushEngine::new(&digital_canvas::settings::BrushSettings::default()).unwrap())),
            file_manager: Arc::new(Mutex::new(digital_canvas::file::FileManager::new(&digital_canvas::settings::Settings::default()).unwrap())),
            commands: Arc::new(Mutex::new(CommandRegistry::new())),
            ui_extension: Arc::new(Mutex::new(UiExtension::new())),
        }
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self { commands: HashMap::new() }
    }

    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.insert(command.name().to_string(), command);
    }

    pub fn execute(&self, name: &str, args: &CommandArgs) -> Result<CommandResult> {
        if let Some(command) = self.commands.get(name) {
            command.execute(args)
        } else {
            Err(anyhow::anyhow!("Command not found: {}", name))
        }
    }

    pub fn list_commands(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &CommandArgs) -> Result<CommandResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArgs {
    pub values: HashMap<String, serde_json::Value>,
}

impl CommandArgs {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) {
        self.values.insert(key.to_string(), serde_json::to_value(value).unwrap());
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.values.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl CommandResult {
    pub fn success(message: &str) -> Self {
        Self { success: true, message: message.to_string(), data: None }
    }

    pub fn error(message: &str) -> Self {
        Self { success: false, message: message.to_string(), data: None }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

pub struct UiExtension {
    panels: HashMap<String, PanelInfo>,
    menu_items: HashMap<String, MenuItemInfo>,
}

impl UiExtension {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            menu_items: HashMap::new(),
        }
    }

    pub fn add_panel(&mut self, id: &str, info: PanelInfo) {
        self.panels.insert(id.to_string(), info);
    }

    pub fn add_menu_item(&mut self, id: &str, info: MenuItemInfo) {
        self.menu_items.insert(id.to_string(), info);
    }

    pub fn panels(&self) -> &HashMap<String, PanelInfo> {
        &self.panels
    }

    pub fn menu_items(&self) -> &HashMap<String, MenuItemInfo> {
        &self.menu_items
    }
}

#[derive(Debug, Clone)]
pub struct PanelInfo {
    pub title: String,
    pub widget_id: String,
    pub default_visible: bool,
    pub default_position: PanelPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelPosition {
    Left,
    Right,
    Bottom,
    Floating,
}

#[derive(Debug, Clone)]
pub struct MenuItemInfo {
    pub menu_path: String, // e.g., "File/Export/My Format"
    pub label: String,
    pub shortcut: Option<String>,
    pub action: String, // Command name to execute
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    context: PluginContext,
    plugin_dir: std::path::PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: std::path::PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&plugin_dir)?;
        Ok(Self {
            plugins: Vec::new(),
            context: PluginContext::new(),
            plugin_dir,
        })
    }

    pub fn load_plugins(&mut self) -> Result<()> {
        // Would load dynamic libraries from plugin_dir
        // For now, just register built-in plugins
        Ok(())
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        plugin.initialize(&mut self.context)?;
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn unload_plugins(&mut self) -> Result<()> {
        for mut plugin in self.plugins.drain(..) {
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub fn execute_command(&self, name: &str, args: &CommandArgs) -> Result<CommandResult> {
        self.context.commands.lock().unwrap().execute(name, args)
    }

    pub fn context(&self) -> &PluginContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut PluginContext {
        &mut self.context
    }
}

// Built-in plugin example
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str { "Example Plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn author(&self) -> &str { "DigitalCanvas Team" }
    fn description(&self) -> &str { "An example plugin" }

    fn initialize(&mut self, context: &mut PluginContext) -> Result<()> {
        context.commands.lock().unwrap().register(Box::new(ExampleCommand));
        context.ui_extension.lock().unwrap().add_panel("example_panel", PanelInfo {
            title: "Example Panel".to_string(),
            widget_id: "example_panel_widget".to_string(),
            default_visible: false,
            default_position: PanelPosition::Right,
        });
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

struct ExampleCommand;

impl Command for ExampleCommand {
    fn name(&self) -> &str { "example.hello" }
    fn description(&self) -> &str { "Say hello" }
    fn execute(&self, _args: &CommandArgs) -> Result<CommandResult> {
        Ok(CommandResult::success("Hello from plugin!"))
    }
}

// Plugin manifest for dynamic loading
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: String,
    pub min_app_version: String,
    pub dependencies: Vec<PluginDependency>,
    pub permissions: Vec<PluginPermission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PluginPermission {
    FileSystemRead,
    FileSystemWrite,
    NetworkAccess,
    DocumentAccess,
    LayerAccess,
    BrushAccess,
    SettingsAccess,
    UiExtension,
}
