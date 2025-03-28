use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, FlowBox, Image, Label, Orientation, ScrolledWindow,
    ToggleButton,
};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

use crate::tools::{ToolType, ToolManager, ToolManagerExt};

pub struct ToolsPanel {
    pub widget: GtkBox,
    pub tools_flow: FlowBox,
    pub tool_manager: Option<Rc<RefCell<ToolManager>>>,
    pub active_tool_button: Option<ToggleButton>,
}

impl ToolsPanel {
    pub fn new() -> Self {
        info!("Creating tools panel");
        
        // Create the main box
        let widget = GtkBox::new(Orientation::Vertical, 5);
        widget.set_margin_start(10);
        widget.set_margin_end(10);
        widget.set_margin_top(10);
        widget.set_margin_bottom(10);
        
        // Create a label for the panel
        let label = Label::new(Some("Tools"));
        label.set_halign(gtk4::Align::Start);
        label.set_margin_bottom(5);
        widget.append(&label);
        
        // Create a flow box for tools
        let tools_flow = FlowBox::new();
        tools_flow.set_selection_mode(gtk4::SelectionMode::None);
        tools_flow.set_max_children_per_line(3);
        tools_flow.set_min_children_per_line(3);
        tools_flow.set_homogeneous(true);
        
        // Add the tools flow to a scrolled window
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&tools_flow));
        scrolled_window.set_vexpand(true);
        widget.append(&scrolled_window);
        
        // Create the tools panel
        let panel = ToolsPanel {
            widget,
            tools_flow,
            tool_manager: None,
            active_tool_button: None,
        };
        
        // Add the tools
        panel.add_tools();
        
        info!("Tools panel created");
        panel
    }
    
    pub fn get_widget(&self) -> GtkBox {
        self.widget.clone()
    }
    
    fn add_tools(&self) {
        // Create toggle buttons for each tool
        self.add_tool_button("Move", "edit-select-all-symbolic", ToolType::Move);
        self.add_tool_button("Select", "edit-select-symbolic", ToolType::Selection);
        self.add_tool_button("Crop", "edit-cut-symbolic", ToolType::Crop);
        self.add_tool_button("Brush", "edit-clear-symbolic", ToolType::Brush);
        self.add_tool_button("Eraser", "edit-delete-symbolic", ToolType::Eraser);
        self.add_tool_button("Text", "format-text-symbolic", ToolType::Text);
        self.add_tool_button("Fill", "color-fill-symbolic", ToolType::Fill);
        self.add_tool_button("Gradient", "gradient-symbolic", ToolType::Gradient);
        self.add_tool_button("Heal", "healing-symbolic", ToolType::Heal);
    }
    
    fn add_tool_button(&self, label_text: &str, icon_name: &str, tool_type: ToolType) {
        // Create a toggle button
        let button = ToggleButton::new();
        button.set_tooltip_text(Some(label_text));
        
        // Create a box for icon and label
        let box_layout = GtkBox::new(Orientation::Vertical, 2);
        
        // Add icon to the button
        let icon = Image::from_icon_name(icon_name);
        box_layout.append(&icon);
        
        // Add label to the button
        let label = Label::new(Some(label_text));
        box_layout.append(&label);
        
        // Set the box as the button's child
        button.set_child(Some(&box_layout));
        
        // Store the tool type in the button's data
        button.set_widget_name(&format!("tool-{:?}", tool_type));
        
        // Connect the toggle signal
        let button_clone = button.clone();
        button.connect_toggled(move |btn| {
            if btn.is_active() {
                // The tool was selected
                info!("Tool selected: {:?}", tool_type);
                // Here you would notify the tool manager
            }
        });
        
        // Add the button to the flow box
        let flow_child = gtk4::FlowBoxChild::new();
        flow_child.set_child(Some(&button));
        self.tools_flow.insert(&flow_child, -1);
    }
    
    pub fn set_tool_manager(&mut self, tool_manager: Rc<RefCell<ToolManager>>) {
        info!("Setting tool manager in tools panel");
        self.tool_manager = Some(tool_manager);
        
        // Connect the tool manager to the buttons
        self.connect_tool_buttons();
    }
    
    fn connect_tool_buttons(&self) {
        if let Some(tool_manager) = &self.tool_manager {
            // Get the number of children in the flow box
            let n_children = self.tools_flow.observe_children().n_items();
            
            // Iterate through all the children
            for i in 0..n_children {
                if let Some(child) = self.tools_flow.observe_children().item(i) {
                    // Get the FlowBoxChild
                    if let Some(flow_child) = child.downcast_ref::<gtk4::FlowBoxChild>() {
                        // Get the button from the child
                        if let Some(child_widget) = flow_child.child() {
                            if let Some(button) = child_widget.downcast_ref::<ToggleButton>() {
                                let name = button.widget_name();
                                if name.starts_with("tool-") {
                                    // Parse the tool type from the button name
                                    let tool_type_str = name["tool-".len()..].to_string();
                                    if let Ok(tool_type) = tool_type_str.parse::<ToolType>() {
                                        // Connect the button to switch tools
                                        let tool_mgr = tool_manager.clone();
                                        let button_clone = button.clone();
                                        button_clone.connect_toggled(move |btn| {
                                            if btn.is_active() {
                                                let mut tm = tool_mgr.borrow_mut();
                                                tm.select_tool(tool_type);
                                                info!("Tool activated: {:?}", tool_type);
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn set_active_tool(&mut self, tool_type: ToolType) {
        info!("Setting active tool in tools panel: {:?}", tool_type);
        
        // If there was a previously active button, deactivate it
        if let Some(button) = &self.active_tool_button {
            button.set_active(false);
        }
        
        // Find the button for the specified tool
        let tool_name = format!("tool-{:?}", tool_type);
        let n_children = self.tools_flow.observe_children().n_items();
            
        for i in 0..n_children {
            if let Some(child) = self.tools_flow.observe_children().item(i) {
                if let Some(flow_child) = child.downcast_ref::<gtk4::FlowBoxChild>() {
                    if let Some(child_widget) = flow_child.child() {
                        if let Some(button) = child_widget.downcast_ref::<ToggleButton>() {
                            if button.widget_name() == tool_name {
                                // Activate this button
                                button.set_active(true);
                                self.active_tool_button = Some(button.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
} 