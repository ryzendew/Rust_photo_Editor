// UI module: Contains user interface components

use gtk4::{Application, ApplicationWindow, Box, Button, DrawingArea, 
           ScrolledWindow, Notebook, Label, Image, 
           CellRendererText, TreeView, TreeViewColumn, ListStore, ComboBoxText, 
           Align, PolicyType, DropDown, Orientation, CssProvider, StyleContext,
           STYLE_PROVIDER_PRIORITY_APPLICATION, ToggleButton, CheckButton, SpinButton, Adjustment,
           Entry, FileChooserAction, FileChooserDialog, Grid, HeaderBar, IconSize,
           MenuButton, NotebookPage, Paned, ResponseType, FileFilter, Overlay, Fixed};
use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use gtk4::gdk;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs;
use log::{info, debug, warn, error};
use crate::core::canvas::Tool;
use crate::core::layer::Layer;
use gtk4::glib::clone;
use gtk4::glib;
use crate::core::canvas::Canvas;
use crate::vector::{VectorDocument, Point};

// UI components
pub mod settings;
pub mod main_window;
pub mod canvas;
pub mod layers_panel;
pub mod tools_panel;
pub mod history_panel;
pub mod filters_panel;
pub mod color_picker;

// Re-export the main components
pub use settings::SettingsDialog;
pub use crate::core::settings::Settings;
pub use main_window::MainWindow;
pub use canvas::Canvas as UiCanvas;
pub use layers_panel::LayersPanel;
pub use tools_panel::ToolsPanel;
pub use history_panel::HistoryPanel;
pub use filters_panel::FiltersPanel;
pub use color_picker::ColorPicker;

// For showing settings dialog
pub fn show_settings_dialog(parent: &ApplicationWindow) -> SettingsDialog {
    let dialog = SettingsDialog::new(parent);
    dialog.show();
    dialog
}

// CSS styles for the application
const CSS_STYLES: &str = r#"
    window {
        background-color: #2d2d2d;
        color: #e0e0e0;
    }
    
    .toolbar {
        background-color: #353535;
        border-bottom: 1px solid #1a1a1a;
    }
    
    .tool-button {
        min-width: 40px;
        min-height: 40px;
        padding: 4px;
        margin: 2px;
        border-radius: 4px;
        background-color: #404040;
    }
    
    .tool-button:hover {
        background-color: #505050;
    }
    
    .tool-button:active {
        background-color: #606060;
    }
    
    .tool-button.selected {
        background-color: #0066cc;
    }
    
    .canvas-area {
        background-color: #1e1e1e;
    }
    
    .side-panel {
        background-color: #353535;
        border-left: 1px solid #1a1a1a;
    }
    
    .histogram-panel {
        background-color: #353535;
    }
    
    .layers-panel {
        background-color: #353535;
    }
    
    .layer-row {
        background-color: #404040;
        border-radius: 3px;
        padding: 2px;
    }
    
    .layer-row:hover {
        background-color: #505050;
    }
    
    .layer-row.selected {
        background-color: #0066cc;
    }
    
    .status-bar {
        background-color: #353535;
        border-top: 1px solid #1a1a1a;
    }
    
    /* Vector-specific styles */
    
    .vector-panel {
        background-color: #353535;
        border-radius: 4px;
        padding: 4px;
    }
    
    frame {
        border: 1px solid #505050;
        border-radius: 4px;
        padding: 4px;
        margin: 4px 0;
    }
    
    frame > label:first-child {
        color: #a0a0a0;
        font-weight: bold;
        font-size: 13px;
        margin-bottom: 4px;
    }
    
    .vector-shape-button {
        min-width: 30px;
        min-height: 30px;
        padding: 4px;
        border-radius: 4px;
        background-color: #404040;
    }
    
    .vector-shape-button:hover {
        background-color: #505050;
    }
    
    .vector-shape-button.selected {
        background-color: #0066cc;
    }
    
    colorbutton {
        min-width: 28px;
        min-height: 28px;
        border-radius: 4px;
        border: 1px solid #505050;
    }
    
    spinbutton {
        background-color: #404040;
        color: #e0e0e0;
        border: 1px solid #505050;
        border-radius: 4px;
    }
    
    .highlight-label {
        color: #39a9dc;
        font-weight: bold;
    }
    
    notebook {
        background-color: #353535;
    }
    
    notebook tab {
        background-color: #404040;
        padding: 4px 8px;
        border-radius: 4px 4px 0 0;
    }
    
    notebook tab:checked {
        background-color: #505050;
    }
    
    notebook tab label {
        color: #e0e0e0;
    }
    
    scrolledwindow {
        background-color: #2d2d2d;
        border-radius: 4px;
    }
    
    /* Adwaita style overrides */
    
    entry {
        background-color: #404040;
        color: #e0e0e0;
        border: 1px solid #505050;
        border-radius: 4px;
    }
    
    dropdown {
        background-color: #404040;
        color: #e0e0e0;
        border: 1px solid #505050;
        border-radius: 4px;
    }
    
    button {
        background-color: #404040;
        color: #e0e0e0;
        border: 1px solid #505050;
        border-radius: 4px;
    }
    
    button:hover {
        background-color: #505050;
    }
    
    button:active {
        background-color: #606060;
    }
    
    togglebutton {
        background-color: #404040;
        color: #e0e0e0;
        border: 1px solid #505050;
        border-radius: 4px;
    }
    
    togglebutton:hover {
        background-color: #505050;
    }
    
    togglebutton:checked {
        background-color: #0066cc;
    }
"#;

// Main window implementation
// This is now defined in main_window.rs and imported above
/*
pub struct MainWindow {
    pub window: ApplicationWindow,
    pub canvas: DrawingArea,
    pub toolbar: Box,
    pub sidebar: Paned,
    pub status_bar: Box,
    pub layers_panel: Box,
    pub tools_panel: Box,
    pub filters_panel: Box,
    pub history_panel: Box,
    pub document: Option<Document>,
    pub tool_manager: ToolManager,
}

impl MainWindow {
    pub fn new(application: &Application) -> Self {
        // Implementation...
    }
    
    pub fn setup_ui(&self) {
        // Implementation...
    }
    
    // Other methods...
}
*/

// Helper function to convert tool index to enum
impl From<usize> for Tool {
    fn from(index: usize) -> Self {
        match index {
            0 => Tool::Selection,
            1 => Tool::Transform,
            2 => Tool::Vector,
            3 => Tool::Paint,
            4 => Tool::Eraser,
            5 => Tool::Clone,
            6 => Tool::Healing,
            7 => Tool::Text,
            8 => Tool::Zoom,
            9 => Tool::Hand,
            _ => Tool::Selection,
        }
    }
}

// Helper function to convert tool enum to index
impl From<Tool> for usize {
    fn from(tool: Tool) -> Self {
        match tool {
            Tool::Selection => 0,
            Tool::Transform => 1,
            Tool::Vector => 2,
            Tool::Paint => 3,
            Tool::Eraser => 4,
            Tool::Clone => 5,
            Tool::Healing => 6,
            Tool::Text => 7,
            Tool::Zoom => 8,
            Tool::Hand => 9,
        }
    }
} 