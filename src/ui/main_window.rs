use gtk4::{
    Application, ApplicationWindow as Window, Box as GtkBox, FileChooserAction,
    FileChooserDialog, HeaderBar, MenuButton, ResponseType, ScrolledWindow,
    Orientation, PopoverMenu, PopoverMenuBar,
};
use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;
use log::{info, error};

use crate::core::document::Document;
use crate::core::canvas::Canvas;
use crate::tools::ToolManager;
use crate::ui::CanvasWidget;
use crate::ui::tools_panel::ToolsPanel;
use crate::ui::layers_panel::LayersPanel;
use crate::ui::preferences::PreferencesDialog;
use crate::ui::menu_manager::MenuManager;

pub struct MainWindow {
    pub window: Window,
    pub document: RefCell<Option<Rc<RefCell<Document>>>>,
    pub canvas: Rc<RefCell<Canvas>>,
    pub canvas_widget: Rc<RefCell<CanvasWidget>>,
    pub tools_panel: Rc<RefCell<ToolsPanel>>,
    pub layers_panel: Rc<RefCell<LayersPanel>>,
    pub tool_manager: Rc<RefCell<ToolManager>>,
    pub menu_manager: Rc<RefCell<MenuManager>>,
    pub current_file_path: Option<PathBuf>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window = Window::new(app);
        window.set_title(Some("Rust Photo Editor"));
        window.set_default_size(1200, 800);

        // Create the main layout
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        // Create header bar
        let header = HeaderBar::new();
        window.set_titlebar(Some(&header));

        // Create menu bar
        let menu_manager = Rc::new(RefCell::new(MenuManager::new()));
        let menu_bar = gtk4::PopoverMenuBar::from_model(Some(&menu_manager.borrow().get_menu_bar()));
        main_box.prepend(&menu_bar);

        // Add actions to the window
        window.insert_action_group("app", Some(menu_manager.borrow().get_actions()));

        // Create tool manager
        let tool_manager = Rc::new(RefCell::new(ToolManager::new()));

        // Create canvas
        let canvas = Rc::new(RefCell::new(Canvas::new(800, 600)));
        let canvas_widget = Rc::new(RefCell::new(CanvasWidget::new()));

        // Create tools panel
        let tools_panel = Rc::new(RefCell::new(ToolsPanel::new(tool_manager.clone())));

        // Create layers panel
        let layers_panel = Rc::new(RefCell::new(LayersPanel::new()));

        // Create scrolled window for canvas
        let canvas_scroll = ScrolledWindow::new();
        canvas_scroll.set_hexpand(true);
        canvas_scroll.set_vexpand(true);
        canvas_scroll.set_child(Some(canvas_widget.borrow().widget()));

        // Create content box for main area
        let content_box = GtkBox::new(Orientation::Horizontal, 0);
        content_box.set_hexpand(true);
        content_box.set_vexpand(true);

        // Set up layout with panes
        let h_paned = gtk4::Paned::new(Orientation::Horizontal);
        h_paned.set_start_child(Some(&tools_panel.borrow().widget));
        h_paned.set_end_child(Some(&canvas_scroll));
        h_paned.set_resize_start_child(false);
        h_paned.set_shrink_start_child(false);
        h_paned.set_position(200);

        let v_paned = gtk4::Paned::new(Orientation::Horizontal);
        v_paned.set_start_child(Some(&h_paned));
        v_paned.set_end_child(Some(&layers_panel.borrow().widget));
        v_paned.set_resize_end_child(false);
        v_paned.set_shrink_end_child(false);
        v_paned.set_position(1000);

        content_box.append(&v_paned);
        main_box.append(&content_box);
        window.set_child(Some(&main_box));

        Self {
            window,
            document: RefCell::new(None),
            canvas,
            canvas_widget,
            tools_panel,
            layers_panel,
            tool_manager,
            menu_manager,
            current_file_path: None,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            document: RefCell::new(self.document.borrow().clone()),
            canvas: self.canvas.clone(),
            canvas_widget: self.canvas_widget.clone(),
            tools_panel: self.tools_panel.clone(),
            layers_panel: self.layers_panel.clone(),
            tool_manager: self.tool_manager.clone(),
            menu_manager: self.menu_manager.clone(),
            current_file_path: self.current_file_path.clone(),
        }
    }

    fn create_canvas(&self, document: &Document) -> Canvas {
        let mut canvas = Canvas::new(document.width as u32, document.height as u32);
        canvas.width = document.width as u32;
        canvas.height = document.height as u32;
        canvas
    }

    pub fn open_document(&self, document: Document) {
        let document = Rc::new(RefCell::new(document));
        *self.document.borrow_mut() = Some(document.clone());
        
        // Update the core canvas
        self.canvas.borrow_mut().document = Some(document.clone());
        
        // Update the canvas widget
        let mut canvas_widget = self.canvas_widget.borrow_mut();
        canvas_widget.set_document(Some(document.borrow().clone()));
    }
} 