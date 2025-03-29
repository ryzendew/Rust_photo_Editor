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

    headerbar {
        background-color: #242424;
        border-bottom: 1px solid rgba(255,255,255,0.05);
        padding: 4px;
        min-height: 32px;
    }

    .menu-button {
        padding: 4px 8px;
        margin: 2px;
        border-radius: 4px;
        color: #e0e0e0;
        background-color: transparent;
        border: none;
    }

    .menu-button:hover {
        background-color: rgba(255,255,255,0.1);
    }

    .toolbar {
        background-color: #242424;
        border-right: 1px solid rgba(255,255,255,0.05);
        padding: 8px;
        min-width: 48px;
    }

    .tool-button {
        padding: 8px;
        margin: 2px;
        border-radius: 4px;
        color: #e0e0e0;
        background-color: transparent;
        min-width: 32px;
        min-height: 32px;
        transition: all 200ms ease;
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
        background-color: #242424;
        border-left: 1px solid rgba(255,255,255,0.05);
        padding: 8px;
        min-width: 250px;
    }

    .canvas-area {
        background-color: #1e1e1e;
    }

    .canvas-area .placeholder {
        color: rgba(255,255,255,0.7);
        font-size: 1.1em;
    }

    paned separator {
        background-color: rgba(255,255,255,0.05);
        min-width: 1px;
        min-height: 1px;
    }

    scrolledwindow {
        border: none;
    }

    scrolledwindow undershoot,
    scrolledwindow overshoot {
        background: none;
    }

    scrollbar {
        background-color: transparent;
        transition: all 200ms ease;
    }

    scrollbar slider {
        min-width: 6px;
        min-height: 6px;
        border-radius: 3px;
        background-color: rgba(255,255,255,0.2);
    }

    scrollbar slider:hover {
        background-color: rgba(255,255,255,0.3);
    }

    scrollbar slider:active {
        background-color: rgba(255,255,255,0.4);
    }

    treeview {
        background-color: transparent;
        color: #e0e0e0;
    }

    treeview:selected {
        background-color: @accent_bg_color;
        color: @accent_fg_color;
    }

    button {
        padding: 6px 12px;
        border-radius: 4px;
        border: none;
        color: #e0e0e0;
        background-color: rgba(255,255,255,0.1);
    }

    button:hover {
        background-color: rgba(255,255,255,0.15);
    }

    button:active {
        background-color: rgba(255,255,255,0.2);
    }

    .tools-panel flowbox {
        background-color: transparent;
    }

    .tools-panel flowboxchild {
        padding: 2px;
    }

    .tools-panel button {
        padding: 8px;
        min-width: 32px;
        min-height: 32px;
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
pub mod main_window;
pub mod canvas;
pub mod tools_panel;
pub mod layers_panel;
pub mod preferences;
pub mod menu_manager;
pub mod filters_panel;
pub mod color_picker;
pub mod history_panel;

pub use main_window::MainWindow;
pub use canvas::Canvas as CanvasWidget;
pub use tools_panel::ToolsPanel;
pub use layers_panel::LayersPanel;
pub use preferences::PreferencesDialog;
pub use menu_manager::MenuManager;
pub use filters_panel::FiltersPanel;
pub use color_picker::ColorPicker;
pub use history_panel::HistoryPanel;

// For showing preferences dialog
pub fn show_preferences_dialog(parent: &ApplicationWindow) -> PreferencesDialog {
    let dialog = PreferencesDialog::new(parent);
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