use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CellRendererText, ComboBoxText, Entry, Label, ListStore,
    Orientation, Scale, ScrolledWindow, Separator, TreeView, TreeViewColumn, PositionType,
};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

use crate::filters::{Filter, IntensityFilter};

pub struct FiltersPanel {
    pub widget: GtkBox,
    pub filters_store: ListStore,
    pub tree_view: TreeView,
    pub preview_button: Button,
    pub apply_button: Button,
    pub intensity_scale: Scale,
    pub filter_params_box: GtkBox,
    pub category_combo: ComboBoxText,
    pub filter_combo: ComboBoxText,
}

impl FiltersPanel {
    pub fn new() -> Self {
        info!("Creating filters panel");
        
        let vbox = GtkBox::new(Orientation::Vertical, 10);
        
        // Create filter category combo box
        let category_combo = ComboBoxText::new();
        category_combo.append(Some("color"), "Color");
        category_combo.append(Some("artistic"), "Artistic");
        category_combo.append(Some("distort"), "Distort");
        category_combo.set_active_id(Some("color"));
        vbox.append(&category_combo);
        
        // Create filter combo box
        let filter_combo = ComboBoxText::new();
        vbox.append(&filter_combo);
        
        // Create filter parameters box
        let filter_params_box = GtkBox::new(Orientation::Vertical, 5);
        vbox.append(&filter_params_box);
        
        // Create intensity scale
        let intensity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 1.0, 0.1);
        intensity_scale.set_value(0.5);
        intensity_scale.set_draw_value(true);
        intensity_scale.set_value_pos(PositionType::Right);
        
        // Create apply button
        let apply_button = Button::with_label("Apply Filter");
        vbox.append(&apply_button);
        
        // Create the list store for applied filters
        // Columns: 0 = ID, 1 = Filter Name, 2 = Parameters
        let filters_store = ListStore::new(&[
            String::static_type(),  // Filter name
            String::static_type(),  // Filter parameters
        ]);
        
        // Create the tree view
        let tree_view = TreeView::with_model(&filters_store);
        tree_view.set_headers_visible(true);
        
        // Create the filter name column
        let name_renderer = CellRendererText::new();
        let name_column = TreeViewColumn::new();
        name_column.set_title("Filter");
        name_column.pack_start(&name_renderer, true);
        name_column.add_attribute(&name_renderer, "text", 0);
        tree_view.append_column(&name_column);
        
        // Create the parameters column
        let params_renderer = CellRendererText::new();
        let params_column = TreeViewColumn::new();
        params_column.set_title("Parameters");
        params_column.pack_start(&params_renderer, true);
        params_column.add_attribute(&params_renderer, "text", 1);
        tree_view.append_column(&params_column);
        
        // Add the tree view to a scrolled window
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&tree_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_margin_bottom(10);
        vbox.append(&scrolled_window);
        
        // Create buttons box
        let buttons_box = GtkBox::new(Orientation::Horizontal, 5);
        
        // Preview button
        let preview_button = Button::with_label("Preview");
        buttons_box.append(&preview_button);
        
        // Remove button
        let remove_button = Button::with_label("Remove");
        buttons_box.append(&remove_button);
        
        // Add the buttons box to the main widget
        vbox.append(&buttons_box);
        
        // Clone widgets for use in closures
        let filter_combo_for_category = filter_combo.clone();
        let filter_params_box_for_category = filter_params_box.clone();
        let intensity_scale_for_category = intensity_scale.clone();
        let filters_store_clone = filters_store.clone();
        let tree_view_clone = tree_view.clone();
        
        // Handle category selection
        category_combo.connect_changed(move |combo| {
            if let Some(category_id) = combo.active_id() {
                filter_combo_for_category.remove_all();
                
                match category_id.as_str() {
                    "color" => {
                        filter_combo_for_category.append(Some("brightness"), "Brightness");
                        filter_combo_for_category.append(Some("contrast"), "Contrast");
                        filter_combo_for_category.append(Some("saturation"), "Saturation");
                        filter_combo_for_category.append(Some("hue"), "Hue");
                        filter_combo_for_category.append(Some("invert"), "Invert");
                    },
                    "artistic" => {
                        filter_combo_for_category.append(Some("oil"), "Oil Paint");
                        filter_combo_for_category.append(Some("watercolor"), "Watercolor");
                        filter_combo_for_category.append(Some("pencil"), "Pencil Sketch");
                    },
                    "distort" => {
                        filter_combo_for_category.append(Some("wave"), "Wave");
                        filter_combo_for_category.append(Some("twirl"), "Twirl");
                        filter_combo_for_category.append(Some("ripple"), "Ripple");
                    },
                    _ => {}
                }
                
                filter_combo_for_category.set_active(Some(0));
            }
        });
        
        // Clone widgets for use in filter selection closure
        let filter_params_box_for_filter = filter_params_box.clone();
        let intensity_scale_for_filter = intensity_scale.clone();
        
        // Handle filter selection
        filter_combo.connect_changed(move |combo| {
            // Clear existing parameters
            while let Some(child) = filter_params_box_for_filter.first_child() {
                filter_params_box_for_filter.remove(&child);
            }
            
            if let Some(filter_id) = combo.active_id() {
                // Add intensity scale for filters that use it
                match filter_id.as_str() {
                    "brightness" | "contrast" | "saturation" | "hue" | "invert" => {
                        let label = Label::new(Some("Intensity:"));
                        filter_params_box_for_filter.append(&label);
                        filter_params_box_for_filter.append(&intensity_scale_for_filter);
                    },
                    _ => {}
                }
            }
        });
        
        // Clone widgets for use in apply button closure
        let filter_combo_for_apply = filter_combo.clone();
        let intensity_scale_for_apply = intensity_scale.clone();
        
        // Handle apply button click
        apply_button.connect_clicked(move |_| {
            if let Some(filter_id) = filter_combo_for_apply.active_id() {
                let intensity = intensity_scale_for_apply.value();
                // TODO: Apply the selected filter with the given intensity
            }
        });
        
        // Connect remove button
        remove_button.connect_clicked(move |_| {
            info!("Remove button clicked");
            
            // Get the selected filter from the tree view
            if let Some((model, iter)) = tree_view_clone.selection().selected() {
                if let Ok(filter_store) = model.downcast::<ListStore>() {
                    filter_store.remove(&iter);
                    info!("Removed filter from list");
                }
            } else {
                info!("No filter selected for removal");
            }
        });
        
        info!("Filters panel created");
        FiltersPanel {
            widget: vbox,
            filters_store,
            tree_view,
            preview_button,
            apply_button,
            intensity_scale,
            filter_params_box,
            category_combo,
            filter_combo,
        }
    }
    
    pub fn get_widget(&self) -> GtkBox {
        self.widget.clone()
    }
} 