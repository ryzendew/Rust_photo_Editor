use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CellRendererText, Label, ListStore, Orientation,
    ScrolledWindow, TreeView, TreeViewColumn,
};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

pub struct HistoryAction {
    pub id: u32,
    pub action_type: String,
    pub description: String,
    pub timestamp: String,
}

pub struct HistoryPanel {
    pub widget: GtkBox,
    pub history_store: ListStore,
    pub tree_view: TreeView,
}

impl HistoryPanel {
    pub fn new() -> Self {
        info!("Creating history panel");
        
        // Create the main box
        let widget = GtkBox::new(Orientation::Vertical, 5);
        widget.set_margin_start(10);
        widget.set_margin_end(10);
        widget.set_margin_top(10);
        widget.set_margin_bottom(10);
        
        // Create a label for the panel
        let label = Label::new(Some("History"));
        label.set_halign(gtk4::Align::Start);
        label.set_margin_bottom(5);
        widget.append(&label);
        
        // Create the list store for history
        // Columns: 0 = ID, 1 = Action Type, 2 = Description, 3 = Timestamp
        let history_store = ListStore::new(&[
            u32::static_type(),     // ID
            String::static_type(),  // Action Type
            String::static_type(),  // Description
            String::static_type(),  // Timestamp
        ]);
        
        // Create the tree view
        let tree_view = TreeView::with_model(&history_store);
        tree_view.set_headers_visible(true);
        
        // Create the description column
        let desc_renderer = CellRendererText::new();
        let desc_column = TreeViewColumn::new();
        desc_column.set_title("Action");
        desc_column.pack_start(&desc_renderer, true);
        desc_column.add_attribute(&desc_renderer, "text", 2);
        tree_view.append_column(&desc_column);
        
        // Create the timestamp column
        let time_renderer = CellRendererText::new();
        let time_column = TreeViewColumn::new();
        time_column.set_title("Time");
        time_column.pack_start(&time_renderer, true);
        time_column.add_attribute(&time_renderer, "text", 3);
        tree_view.append_column(&time_column);
        
        // Add the tree view to a scrolled window
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&tree_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_margin_bottom(10);
        widget.append(&scrolled_window);
        
        // Create buttons box
        let buttons_box = GtkBox::new(Orientation::Horizontal, 5);
        
        // Undo button
        let undo_button = Button::with_label("Undo");
        buttons_box.append(&undo_button);
        
        // Redo button
        let redo_button = Button::with_label("Redo");
        buttons_box.append(&redo_button);
        
        // Add the buttons box to the main widget
        widget.append(&buttons_box);
        
        // Add some sample history items
        let panel = HistoryPanel {
            widget,
            history_store,
            tree_view,
        };
        
        // Connect signals
        undo_button.connect_clicked(|_| {
            info!("Undo button clicked");
            // Here you would call the undo functionality
        });
        
        redo_button.connect_clicked(|_| {
            info!("Redo button clicked");
            // Here you would call the redo functionality
        });
        
        info!("History panel created");
        panel
    }
    
    pub fn get_widget(&self) -> GtkBox {
        self.widget.clone()
    }
    
    pub fn add_action(&self, action: HistoryAction) {
        // Add the action to the store
        self.history_store.insert_with_values(
            None,
            &[
                (0, &action.id),
                (1, &action.action_type),
                (2, &action.description),
                (3, &action.timestamp),
            ],
        );
        
        // Select the latest action
        let index = self.history_store.iter_n_children(None) - 1;
        let path = gtk4::TreePath::from_indices(&[index as i32]);
        gtk4::prelude::TreeViewExt::set_cursor(&self.tree_view, &path, Option::<&TreeViewColumn>::None, false);
    }
    
    pub fn clear(&self) {
        self.history_store.clear();
    }
} 