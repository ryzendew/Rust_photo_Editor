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
    pub document: RefCell<Option<Rc<RefCell<Document>>>>,
    pub canvas: Rc<RefCell<Canvas>>,
    pub tool_manager: ToolManager,
    pub current_file_path: Option<PathBuf>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        info!("Creating main window");
        
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Rust Photo Editor")
            .default_width(1200)
            .default_height(800)
            .build();

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&main_box));

        // Create toolbar
        let toolbar = GtkBox::new(Orientation::Horizontal, 5);
        toolbar.add_css_class("toolbar");
        main_box.append(&toolbar);

        // Create open button
        let open_button = Button::with_label("Open");
        open_button.add_css_class("tool-button");
        toolbar.append(&open_button);

        // Create save button
        let save_button = Button::with_label("Save");
        save_button.add_css_class("tool-button");
        toolbar.append(&save_button);

        // Create settings button
        let settings_button = Button::with_label("Settings");
        settings_button.add_css_class("tool-button");
        toolbar.append(&settings_button);

        // Create canvas area
        let canvas_scroll = ScrolledWindow::new();
        canvas_scroll.set_hexpand(true);
        canvas_scroll.set_vexpand(true);
        canvas_scroll.add_css_class("canvas-area");

        let canvas = Canvas::new();
        canvas_scroll.set_child(Some(canvas.widget()));
        main_box.append(&canvas_scroll);

        let window_obj = Self {
            window,
            canvas: Rc::new(RefCell::new(canvas)),
            document: RefCell::new(None),
            tool_manager: ToolManager::new(),
            current_file_path: None,
        };

        // Connect signals
        let window_ref = window_obj.clone();
        open_button.connect_clicked(move |_| {
            window_ref.on_open_clicked();
        });

        let window_ref = window_obj.clone();
        save_button.connect_clicked(move |_| {
            window_ref.on_save_clicked();
        });

        let window_ref = window_obj.clone();
        settings_button.connect_clicked(move |_| {
            window_ref.on_settings_clicked();
        });

        window_obj
    }
    
    pub fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            canvas: self.canvas.clone(),
            document: RefCell::new(self.document.borrow().clone()),
            tool_manager: self.tool_manager.clone(),
            current_file_path: self.current_file_path.clone(),
        }
    }
    
    fn on_open_clicked(&self) {
        info!("Open button clicked");
        
        let dialog = FileChooserDialog::new(
            Some("Open Image"),
            Some(&self.window),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        
        let filter = gtk4::FileFilter::new();
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/tiff");
        filter.add_mime_type("image/webp");
        dialog.add_filter(&filter);
        
        let window_ref = self.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        match Document::from_file(&path) {
                            Ok(document) => {
                                window_ref.set_document(document);
                            }
                            Err(err) => {
                                error!("Failed to open document: {}", err);
                            }
                        }
                    }
                }
            }
            dialog.close();
        });
        
        dialog.show();
    }
    
    fn on_save_clicked(&self) {
        info!("Save button clicked");
        
        if let Some(doc) = self.document.borrow().as_ref() {
            let dialog = FileChooserDialog::new(
                Some("Save Image"),
                Some(&self.window),
                FileChooserAction::Save,
                &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
            );
            
            let filter = gtk4::FileFilter::new();
            filter.add_mime_type("image/jpeg");
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/tiff");
            filter.add_mime_type("image/webp");
            dialog.add_filter(&filter);
            
            let doc_ref = doc.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let mut doc = doc_ref.borrow_mut();
                            if let Err(err) = doc.save(&path) {
                                error!("Failed to save document: {}", err);
                            }
                        }
                    }
                }
                dialog.close();
            });
            
            dialog.show();
        }
    }
    
    fn on_settings_clicked(&self) {
        let dialog = SettingsDialog::new(&self.window);
        dialog.show();
    }
    
    fn set_document(&self, document: Document) {
        *self.document.borrow_mut() = Some(Rc::new(RefCell::new(document.clone())));
        let mut canvas = self.canvas.borrow_mut();
        canvas.set_document(Some(document));
    }
    
    fn create_canvas(&self, document: &Document) -> Canvas {
        let mut canvas = Canvas::new();
        canvas.width = document.width as i32;
        canvas.height = document.height as i32;
        canvas.set_document(Some(document.clone()));
        canvas
    }
} 