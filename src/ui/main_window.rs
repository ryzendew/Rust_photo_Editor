use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea, FileChooserAction,
    FileChooserDialog, HeaderBar, Label, Notebook, Overlay, ResponseType, ScrolledWindow,
    Stack, StackSwitcher, ToggleButton,
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
            
        // Create the HeaderBar
        let header_bar = AdwHeaderBar::builder()
            .title_widget(&WindowTitle::new("Rust Photo", ""))
            .build();
            
        // Create the main layout
        let main_box = GtkBox::new(gtk4::Orientation::Vertical, 0);
        
        // Create the menu button
        let menu_button = Button::with_label("Open");
        menu_button.set_margin_start(10);
        menu_button.set_margin_end(10);
        
        // Create the save button
        let save_button = Button::with_label("Save");
        save_button.set_margin_start(10);
        save_button.set_margin_end(10);
        
        // Create the settings button
        let settings_button = Button::with_label("Settings");
        settings_button.set_margin_start(10);
        settings_button.set_margin_end(10);
        
        // Add the buttons to the header bar
        header_bar.pack_start(&menu_button);
        header_bar.pack_start(&save_button);
        header_bar.pack_end(&settings_button);
        
        // Create the main content area
        let content_box = GtkBox::new(gtk4::Orientation::Horizontal, 0);
        
        // Create the left panel for tools
        let tools_panel = ToolsPanel::new();
        let tools_box = tools_panel.get_widget();
        tools_box.set_size_request(200, -1);
        content_box.append(&tools_box);
        
        // Create the center area for the canvas
        let canvas_box = GtkBox::new(gtk4::Orientation::Vertical, 0);
        canvas_box.set_hexpand(true);
        canvas_box.set_vexpand(true);
        
        // Create a scrolled window for the canvas
        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .build();
        scrolled_window.set_hexpand(true);
        scrolled_window.set_vexpand(true);
        
        // Create a placeholder for the canvas
        let placeholder = Label::new(Some("Open or create a document to start editing"));
        placeholder.set_hexpand(true);
        placeholder.set_vexpand(true);
        scrolled_window.set_child(Some(&placeholder));
        
        canvas_box.append(&scrolled_window);
        content_box.append(&canvas_box);
        
        // Create the right panel for layers, history, etc.
        let right_panel = Notebook::new();
        right_panel.set_size_request(250, -1);
        
        // Create the layers panel
        let layers_panel = LayersPanel::new();
        let layers_widget = layers_panel.get_widget();
        right_panel.append_page(&layers_widget, Some(&Label::new(Some("Layers"))));
        
        // Create the history panel
        let history_panel = HistoryPanel::new();
        let history_widget = history_panel.get_widget();
        right_panel.append_page(&history_widget, Some(&Label::new(Some("History"))));
        
        // Create the filters panel
        let filters_panel = FiltersPanel::new();
        let filters_widget = filters_panel.get_widget();
        right_panel.append_page(&filters_widget, Some(&Label::new(Some("Filters"))));
        
        content_box.append(&right_panel);
        
        // Add all components to the main box
        main_box.append(&header_bar);
        main_box.append(&content_box);
        
        // Set the window content
        window.set_child(Some(&main_box));
        
        // Create the main window instance
        let main_window = MainWindow {
            window,
            document: None,
            canvas: None,
            tool_manager: ToolManager::new(),
            current_file_path: None,
        };
        
        // Connect signals
        let window_clone = main_window.window.clone();
        menu_button.connect_clicked(move |_| {
            MainWindow::on_open_clicked(&window_clone);
        });
        
        let window_clone = main_window.window.clone();
        let main_window_clone = Rc::new(RefCell::new(main_window.clone()));
        save_button.connect_clicked(move |_| {
            let main_window = main_window_clone.borrow();
            MainWindow::on_save_clicked(&window_clone, main_window.document.as_ref(), main_window.current_file_path.clone());
        });
        
        let window_clone = main_window.window.clone();
        settings_button.connect_clicked(move |_| {
            MainWindow::on_settings_clicked(&window_clone);
        });
        
        info!("Main window created");
        main_window
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