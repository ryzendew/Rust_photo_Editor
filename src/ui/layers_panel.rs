use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CellRendererText, Label, ListStore, Orientation,
    ScrolledWindow, TreeView, TreeViewColumn, Image, Entry, Separator,
};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

use crate::core::document::Document;
use crate::core::layer::{Layer, LayerManager};

#[derive(Clone)]
pub struct LayersPanel {
    pub widget: GtkBox,
    pub layer_store: ListStore,
    pub tree_view: TreeView,
    pub document: Option<Rc<RefCell<Document>>>,
}

impl LayersPanel {
    pub fn new() -> Self {
        info!("Creating layers panel");
        
        // Create the main box
        let widget = GtkBox::new(Orientation::Vertical, 5);
        widget.set_margin_start(10);
        widget.set_margin_end(10);
        widget.set_margin_top(10);
        widget.set_margin_bottom(10);
        
        // Create a label for the panel
        let label = Label::new(Some("Layers"));
        label.set_halign(gtk4::Align::Start);
        label.set_margin_bottom(5);
        widget.append(&label);
        
        // Create a separator
        let separator = Separator::new(Orientation::Horizontal);
        widget.append(&separator);
        
        // Create the list store for layers
        // Columns: 0 = Layer ID, 1 = Layer Name, 2 = Visible, 3 = Type
        let layer_store = ListStore::new(&[
            u32::static_type(),     // Layer ID
            String::static_type(),  // Layer Name
            bool::static_type(),    // Visible
            String::static_type(),  // Type
        ]);
        
        // Create the tree view
        let tree_view = TreeView::with_model(&layer_store);
        tree_view.set_headers_visible(true);
        
        // Create columns
        let name_renderer = CellRendererText::new();
        name_renderer.set_property("editable", &true);
        let name_column = TreeViewColumn::new();
        name_column.set_title("Name");
        name_column.pack_start(&name_renderer, true);
        name_column.add_attribute(&name_renderer, "text", 1);
        tree_view.append_column(&name_column);
        
        let type_renderer = CellRendererText::new();
        let type_column = TreeViewColumn::new();
        type_column.set_title("Type");
        type_column.pack_start(&type_renderer, true);
        type_column.add_attribute(&type_renderer, "text", 3);
        tree_view.append_column(&type_column);
        
        // Add the tree view to a scrolled window
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&tree_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_margin_bottom(10);
        scrolled_window.set_has_frame(true);
        widget.append(&scrolled_window);
        
        // Create buttons box
        let buttons_box = GtkBox::new(Orientation::Horizontal, 5);
        
        // Add layer button
        let add_button = Button::from_icon_name("list-add");
        add_button.set_tooltip_text(Some("Add Layer"));
        buttons_box.append(&add_button);
        
        // Delete layer button
        let delete_button = Button::from_icon_name("list-remove");
        delete_button.set_tooltip_text(Some("Delete Layer"));
        buttons_box.append(&delete_button);
        
        // Group layers button
        let group_button = Button::from_icon_name("object-group-symbolic");
        group_button.set_tooltip_text(Some("Group Layers"));
        buttons_box.append(&group_button);
        
        // Duplicate layer button
        let duplicate_button = Button::from_icon_name("edit-copy-symbolic");
        duplicate_button.set_tooltip_text(Some("Duplicate Layer"));
        buttons_box.append(&duplicate_button);
        
        // Add the buttons box to the main widget
        widget.append(&buttons_box);
        
        // Create layer properties box
        let properties_box = GtkBox::new(Orientation::Vertical, 5);
        properties_box.set_margin_top(10);
        
        // Layer opacity slider (to be implemented)
        let opacity_label = Label::new(Some("Opacity"));
        opacity_label.set_halign(gtk4::Align::Start);
        properties_box.append(&opacity_label);
        
        // Layer blend mode dropdown (to be implemented)
        let blend_label = Label::new(Some("Blend Mode"));
        blend_label.set_halign(gtk4::Align::Start);
        properties_box.append(&blend_label);
        
        // Add the properties box to the main widget
        widget.append(&properties_box);
        
        // Create the layers panel
        let panel = LayersPanel {
            widget,
            layer_store,
            tree_view,
            document: None,
        };
        
        // Connect signals (to be implemented with document binding)
        
        info!("Layers panel created");
        panel
    }
    
    pub fn get_widget(&self) -> GtkBox {
        self.widget.clone()
    }
    
    pub fn set_document(&mut self, document: Rc<RefCell<Document>>) {
        info!("Setting document in layers panel");
        self.document = Some(document.clone());
        
        // Clear the layer store
        self.layer_store.clear();
        
        // Populate the layer store with layers from the document
        let doc = document.borrow();
        self.update_layer_list(&doc.layer_manager);
    }
    
    fn update_layer_list(&self, layer_manager: &LayerManager) {
        // Clear the store
        self.layer_store.clear();
        
        // Get layer indices instead of ordering
        let layer_count = layer_manager.layer_count();
        let layer_indices: Vec<usize> = (0..layer_count).collect();
        
        // Add layers to the store
        for &layer_id in layer_indices.iter() {
            if let Some(layer) = layer_manager.get_layer(layer_id) {
                self.add_layer_to_store(layer, layer_id.to_string());
            }
        }
    }
    
    fn add_layer_to_store(&self, layer: &Layer, layer_id: String) {
        // Determine a type string based on what's available in the Layer struct
        let type_str = "Raster"; // Default to raster since we only have image data in the Layer struct
        
        // Add to the store
        self.layer_store.insert_with_values(
            None,
            &[
                (0, &(layer_id.parse::<u32>().unwrap())),
                (1, &layer.name),
                (2, &layer.visible),
                (3, &type_str),
            ],
        );
    }
    
    pub fn update_from_document(&self) {
        if let Some(doc) = &self.document {
            let doc = doc.borrow();
            self.update_layer_list(&doc.layer_manager);
        }
    }
} 