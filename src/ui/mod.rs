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
    window {
        background-color: #1a1a1a;
        color: #e0e0e0;
    }

    .toolbar {
        background-color: #242424;
        border-right: 1px solid rgba(255,255,255,0.05);
        padding: 8px;
    }

    .tool-button {
        padding: 12px;
        margin: 4px;
        border-radius: 8px;
        color: #e0e0e0;
        background-color: #2d2d2d;
        min-width: 32px;
        min-height: 32px;
        transition: all 200ms ease;
    }

    .tool-button:hover {
        background-color: #353535;
        box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    }

    .tool-button:active,
    .tool-button.active {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
        box-shadow: 0 1px 2px rgba(0,0,0,0.3);
    }

    .sidebar {
        background-color: #242424;
        border-left: 1px solid rgba(255,255,255,0.05);
        padding: 8px;
    }

    .sidebar tab-box {
        padding: 8px;
        background-color: #2d2d2d;
        border-radius: 8px;
        margin-bottom: 8px;
    }

    .sidebar button {
        padding: 8px 12px;
        margin: 2px;
        border-radius: 6px;
        color: #e0e0e0;
        background-color: #2d2d2d;
        border: none;
        box-shadow: none;
    }

    .sidebar button:hover {
        background-color: #353535;
    }

    .sidebar button.active {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    .canvas-area {
        background-color: #1e1e1e;
        border-radius: 8px;
        margin: 8px;
    }

    .canvas-area .placeholder {
        color: rgba(255,255,255,0.7);
        font-size: 1.1em;
    }

    headerbar {
        background-color: #242424;
        border-bottom: 1px solid rgba(255,255,255,0.05);
        padding: 8px;
        min-height: 48px;
    }

    headerbar button {
        padding: 8px 16px;
        margin: 2px;
        border-radius: 6px;
        color: #e0e0e0;
        background-color: #2d2d2d;
        border: none;
        box-shadow: none;
        min-height: 32px;
    }

    headerbar button:hover {
        background-color: #353535;
    }

    scale {
        margin: 8px 0;
    }

    scale trough {
        background-color: #2d2d2d;
        border-radius: 4px;
        min-height: 6px;
    }

    scale highlight {
        background-color: @accent_bg_color;
        border-radius: 3px;
    }

    combobox button {
        background-color: #2d2d2d;
        border: 1px solid rgba(255,255,255,0.05);
        border-radius: 6px;
        padding: 8px;
        color: #e0e0e0;
        min-height: 32px;
    }

    combobox button:hover {
        background-color: #353535;
    }

    treeview {
        background-color: #2d2d2d;
        color: #e0e0e0;
        border-radius: 6px;
    }

    treeview:selected {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    scrolledwindow {
        border-radius: 6px;
    }

    entry {
        background-color: #2d2d2d;
        color: #e0e0e0;
        border: 1px solid rgba(255,255,255,0.05);
        border-radius: 6px;
        padding: 8px;
        min-height: 32px;
    }

    entry:focus {
        border-color: @accent_bg_color;
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