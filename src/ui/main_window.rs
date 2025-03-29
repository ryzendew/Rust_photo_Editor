use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea, FileChooserAction,
    FileChooserDialog, HeaderBar, Label, Notebook, Overlay, ResponseType, ScrolledWindow,
    Stack, StackSwitcher, ToggleButton, TreeView, Orientation, Scale, CellRendererText, ComboBoxText, TreeViewColumn,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar as AdwHeaderBar, Clamp, WindowTitle};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use log::{debug, info, warn, error};

use crate::core::document::{Document, DocumentFormat};
use crate::core::layer::Layer;
use crate::tools::{ToolType, ToolManager};
use crate::ui::canvas::Canvas;
use crate::ui::layers_panel::LayersPanel;
use crate::ui::tools_panel::ToolsPanel;
use crate::ui::history_panel::HistoryPanel;
use crate::ui::settings::SettingsDialog;
use crate::ui::filters_panel::FiltersPanel;

pub struct MainWindow {
    pub window: ApplicationWindow,
    pub document: Option<Rc<RefCell<Document>>>,
    pub canvas: Option<Canvas>,
    pub tool_manager: ToolManager,
    pub current_file_path: Option<PathBuf>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        info!("Creating main window");
        
        // Create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Rust Photo")
            .default_width(1200)
            .default_height(800)
            .build();
            
        // Create the main layout box
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        
        // Create header bar
        let header = AdwHeaderBar::builder()
            .css_classes(vec!["flat"])
            .build();
        
        // Add main action buttons
        let open_button = Button::with_label("Open");
        let save_button = Button::with_label("Save");
        header.pack_start(&open_button);
        header.pack_start(&save_button);
        
        // Add settings button
        let settings_button = Button::from_icon_name("emblem-system-symbolic");
        header.pack_end(&settings_button);
        
        main_box.append(&header);
        
        // Create the content area
        let content = GtkBox::new(Orientation::Horizontal, 0);
        content.set_hexpand(true);
        content.set_vexpand(true);
        
        // Create toolbar
        let toolbar = GtkBox::new(Orientation::Vertical, 4);
        toolbar.set_margin_start(8);
        toolbar.set_margin_end(8);
        toolbar.set_margin_top(8);
        toolbar.set_margin_bottom(8);
        toolbar.add_css_class("toolbar");
        
        // Add tool buttons
        let tools = vec![
            ("Move", "object-select-symbolic"),
            ("Select", "edit-select-all-symbolic"),
            ("Brush", "edit-clear-symbolic"),
            ("Eraser", "edit-clear-all-symbolic"),
            ("Fill", "color-fill-symbolic"),
            ("Gradient", "gradient-symbolic"),
        ];
        
        for (name, icon) in tools {
            let button = Button::builder()
                .icon_name(icon)
                .tooltip_text(name)
                .css_classes(vec!["flat", "tool-button"])
                .build();
            toolbar.append(&button);
        }
        
        content.append(&toolbar);
        
        // Create canvas area
        let canvas_box = GtkBox::new(Orientation::Vertical, 0);
        canvas_box.set_hexpand(true);
        canvas_box.set_vexpand(true);
        
        let scroll = ScrolledWindow::new();
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);
        
        let placeholder = Label::new(Some("Open or create a document to start editing"));
        placeholder.add_css_class("dim-label");
        placeholder.set_vexpand(true);
        
        scroll.set_child(Some(&placeholder));
        canvas_box.append(&scroll);
        
        content.append(&canvas_box);
        
        // Create right sidebar
        let sidebar = GtkBox::new(Orientation::Vertical, 0);
        sidebar.set_width_request(300);
        
        // Add tab buttons
        let tab_box = GtkBox::new(Orientation::Horizontal, 0);
        tab_box.add_css_class("linked");
        
        let layers_button = Button::with_label("Layers");
        layers_button.add_css_class("active");
        let history_button = Button::with_label("History");
        let filters_button = Button::with_label("Filters");
        
        tab_box.append(&layers_button);
        tab_box.append(&history_button);
        tab_box.append(&filters_button);
        
        sidebar.append(&tab_box);
        
        // Add panels stack
        let stack = Stack::new();
        stack.set_vexpand(true);
        
        // Layers panel
        let layers_panel = GtkBox::new(Orientation::Vertical, 0);
        let layers_list = TreeView::new();
        layers_list.set_vexpand(true);
        
        let name_column = TreeViewColumn::new();
        name_column.set_title("Name");
        let name_cell = CellRendererText::new();
        name_column.pack_start(&name_cell, true);
        layers_list.append_column(&name_column);
        
        let type_column = TreeViewColumn::new();
        type_column.set_title("Type");
        let type_cell = CellRendererText::new();
        type_column.pack_start(&type_cell, true);
        layers_list.append_column(&type_column);
        
        layers_panel.append(&layers_list);
        
        // Layer controls
        let layer_controls = GtkBox::new(Orientation::Vertical, 4);
        layer_controls.set_margin_start(8);
        layer_controls.set_margin_end(8);
        layer_controls.set_margin_top(8);
        layer_controls.set_margin_bottom(8);
        
        let opacity_box = GtkBox::new(Orientation::Horizontal, 4);
        opacity_box.append(&Label::new(Some("Opacity")));
        let opacity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        opacity_box.append(&opacity_scale);
        
        let blend_box = GtkBox::new(Orientation::Horizontal, 4);
        blend_box.append(&Label::new(Some("Blend Mode")));
        let blend_combo = ComboBoxText::new();
        blend_combo.append(Some("normal"), "Normal");
        blend_combo.append(Some("multiply"), "Multiply");
        blend_combo.append(Some("screen"), "Screen");
        blend_combo.set_active_id(Some("normal"));
        blend_box.append(&blend_combo);
        
        layer_controls.append(&opacity_box);
        layer_controls.append(&blend_box);
        layers_panel.append(&layer_controls);
        
        stack.add_titled(&layers_panel, Some("layers"), "Layers");
        
        // History panel
        let history_panel = GtkBox::new(Orientation::Vertical, 0);
        let history_list = TreeView::new();
        history_list.set_vexpand(true);
        history_panel.append(&history_list);
        stack.add_titled(&history_panel, Some("history"), "History");
        
        // Filters panel  
        let filters_panel = GtkBox::new(Orientation::Vertical, 0);
        let filters_list = TreeView::new();
        filters_list.set_vexpand(true);
        filters_panel.append(&filters_list);
        stack.add_titled(&filters_panel, Some("filters"), "Filters");
        
        sidebar.append(&stack);
        content.append(&sidebar);
        
        main_box.append(&content);
        
        // Set the window content
        window.set_child(Some(&main_box));
        
        // Create tool manager
        let tool_manager = ToolManager::new();
        
        MainWindow {
            window,
            document: None,
            canvas: None,
            tool_manager,
            current_file_path: None,
        }
    }
    
    pub fn clone(&self) -> Self {
        MainWindow {
            window: self.window.clone(),
            document: self.document.clone(),
            canvas: self.canvas.clone().map(|c| c.clone()),
            tool_manager: self.tool_manager.clone(),
            current_file_path: self.current_file_path.clone(),
        }
    }
    
    fn on_open_clicked(window: &ApplicationWindow) {
        info!("Open button clicked");
        
        // Create a file chooser dialog
        let dialog = FileChooserDialog::new(
            Some("Open Image"),
            Some(window),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        
        // Add file filters
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Image Files"));
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/tiff");
        filter.add_mime_type("image/webp");
        dialog.add_filter(&filter);
        
        // Show the dialog
        dialog.set_modal(true);
        dialog.show();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        info!("Selected file: {:?}", path);
                        // Here you would load the image and update the canvas
                        // For now, just close the dialog
                    }
                }
            }
            dialog.destroy();
        });
    }
    
    fn on_save_clicked(window: &ApplicationWindow, document: Option<&Rc<RefCell<Document>>>, current_path: Option<PathBuf>) {
        info!("Save button clicked");
        
        // Check if we have a document to save
        let document = match document {
            Some(doc) => doc,
            None => {
                warn!("No document to save");
                return;
            }
        };
        
        // If we already have a path, save directly
        if let Some(path) = current_path {
            info!("Saving to existing path: {:?}", path);
            // Here you would save the document to the path
            return;
        }
        
        // Create a file chooser dialog
        let dialog = FileChooserDialog::new(
            Some("Save Image"),
            Some(window),
            FileChooserAction::Save,
            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
        );
        
        // Add file filters
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Image Files"));
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/tiff");
        filter.add_mime_type("image/webp");
        dialog.add_filter(&filter);
        
        // Set a default filename
        dialog.set_current_name("untitled.png");
        
        // Show the dialog
        dialog.set_modal(true);
        dialog.show();
        
        let doc_clone = document.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        info!("Saving to: {:?}", path);
                        // Here you would save the document to the path
                        let _doc = doc_clone.borrow();
                        // For now, just log the action
                        info!("Document would be saved to {:?}", path);
                    }
                }
            }
            dialog.destroy();
        });
    }
    
    fn on_settings_clicked(window: &ApplicationWindow) {
        info!("Settings button clicked");
        
        // Create and show the settings dialog
        let settings_dialog = SettingsDialog::new(window);
        settings_dialog.show();
    }
    
    pub fn set_document(&mut self, document: Document) {
        info!("Setting document in main window");
        
        // Create a new canvas with the document
        let canvas = self.create_canvas(&document);
        
        // Find the scrolled window and replace its content
        if let Some(window_child) = self.window.child() {
            if let Some(main_box) = window_child.downcast_ref::<GtkBox>() {
                if let Some(content_box) = main_box.first_child().and_then(|c| c.next_sibling()) {
                    if let Some(content) = content_box.downcast_ref::<GtkBox>() {
                        if let Some(canvas_box) = content.first_child().and_then(|c| c.next_sibling()) {
                            if let Some(canvas_container) = canvas_box.downcast_ref::<GtkBox>() {
                                if let Some(scrolled_window) = canvas_container.first_child() {
                                    if let Some(scroll) = scrolled_window.downcast_ref::<ScrolledWindow>() {
                                        // Replace the placeholder with the canvas
                                        let canvas_widget = canvas.get_widget();
                                        scroll.set_child(Some(&canvas_widget));
                                        
                                        // Store the document and canvas
                                        self.document = Some(Rc::new(RefCell::new(document)));
                                        self.canvas = Some(canvas);
                                        
                                        info!("Document set and canvas updated");
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        error!("Failed to find the canvas container in the widget hierarchy");
    }
    
    fn create_canvas(&self, document: &Document) -> Canvas {
        let mut canvas = Canvas::new();
        canvas.width = document.width as i32;
        canvas.height = document.height as i32;
        canvas.set_document(document.clone());
        canvas
    }
} 