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

// CSS styles for the application
const APP_CSS: &str = "
    .toolbar {
        background-color: #2d2d2d;
        border-right: 1px solid rgba(0,0,0,0.3);
        padding: 8px;
    }

    .tool-button {
        padding: 12px;
        margin: 4px;
        border-radius: 6px;
        color: #e0e0e0;
        background-color: rgba(255,255,255,0.05);
        min-width: 24px;
        min-height: 24px;
    }

    .tool-button:hover {
        background-color: rgba(255,255,255,0.1);
    }

    .tool-button:active,
    .tool-button.active {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    .sidebar {
        background-color: #2d2d2d;
        border-left: 1px solid rgba(0,0,0,0.3);
        padding: 8px;
    }

    .sidebar tab-box {
        padding: 8px;
        background-color: #232323;
        border-bottom: 1px solid rgba(0,0,0,0.3);
    }

    .sidebar button {
        padding: 8px 12px;
        margin: 2px;
        border-radius: 6px;
        color: #e0e0e0;
    }

    .sidebar button.active {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    .canvas-area {
        background-color: #1e1e1e;
    }

    .canvas-area .placeholder {
        color: rgba(255,255,255,0.7);
        font-size: 1.1em;
    }

    headerbar {
        background-color: #1d1d1d;
        border-bottom: 1px solid rgba(0,0,0,0.3);
        padding: 8px;
    }

    headerbar button {
        padding: 8px 12px;
        margin: 2px;
        border-radius: 6px;
        color: #e0e0e0;
        background-color: rgba(255,255,255,0.05);
    }

    headerbar button:hover {
        background-color: rgba(255,255,255,0.1);
    }

    scale {
        margin: 8px 0;
    }

    scale trough {
        background-color: rgba(255,255,255,0.1);
        border-radius: 4px;
        min-height: 6px;
    }

    scale highlight {
        background-color: @accent_bg_color;
        border-radius: 3px;
    }

    combobox button {
        background-color: rgba(255,255,255,0.05);
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 6px;
        padding: 6px;
        color: #e0e0e0;
    }

    combobox button:hover {
        background-color: rgba(255,255,255,0.1);
    }
";

pub fn init_styles() {
    let provider = CssProvider::new();
    provider.load_from_data(APP_CSS);
    
    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

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