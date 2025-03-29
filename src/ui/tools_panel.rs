use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, FlowBox, Image, Label, Orientation, ScrolledWindow,
    ToggleButton,
};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

use crate::tools::{ToolType, ToolManager, ToolManagerExt};

#[derive(Clone)]
pub struct ToolsPanel {
    pub widget: GtkBox,
    pub tools_box: GtkBox,
    tool_buttons: Vec<ToggleButton>,
    tool_manager: Option<Rc<RefCell<ToolManager>>>,
}

impl ToolsPanel {
    pub fn new(tool_manager: Rc<RefCell<ToolManager>>) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        let tools_box = GtkBox::new(Orientation::Vertical, 0);
        widget.append(&tools_box);

        let mut panel = Self {
            widget,
            tools_box,
            tool_buttons: Vec::new(),
            tool_manager: Some(tool_manager),
        };

        panel.build_tools();
        panel
    }
    
    pub fn get_widget(&self) -> GtkBox {
        self.widget.clone()
    }
    
    pub fn add_tool_button(&mut self, label_text: &str, icon_name: &str, tool_type: ToolType) {
        let button = ToggleButton::new();
        button.set_widget_name(&tool_type.to_string());
        
        let icon = Image::from_icon_name(icon_name);
        icon.set_icon_size(gtk4::IconSize::Large);
        button.set_child(Some(&icon));
        
        button.set_tooltip_text(Some(label_text));
        button.add_css_class("tool-button");
        
        if let Some(tool_manager) = &self.tool_manager {
            let tool_manager = tool_manager.clone();
            button.connect_toggled(move |btn| {
                if btn.is_active() {
                    tool_manager.borrow_mut().set_active_tool(tool_type);
                }
            });
        }
        
        self.widget.append(&button);
        self.tool_buttons.push(button);
    }
    
    pub fn build_tools(&mut self) {
        // Selection tools
        self.add_tool_button("Selection", "edit-select-all-symbolic", ToolType::Selection);
        self.add_tool_button("Rectangle Selection", "edit-select-all-symbolic", ToolType::RectangleSelection);
        self.add_tool_button("Ellipse Selection", "edit-select-all-symbolic", ToolType::EllipseSelection);
        self.add_tool_button("Lasso Selection", "edit-cut-symbolic", ToolType::LassoSelection);
        self.add_tool_button("Magic Wand Selection", "edit-find-symbolic", ToolType::MagicWandSelection);
        self.add_tool_button("Move", "object-move-symbolic", ToolType::Move);

        // Paint tools
        self.add_tool_button("Brush", "edit-symbolic", ToolType::Brush);
        self.add_tool_button("Eraser", "edit-clear-symbolic", ToolType::Eraser);
        self.add_tool_button("Clone", "edit-copy-symbolic", ToolType::Clone);
        self.add_tool_button("Heal", "applications-science-symbolic", ToolType::Heal);
        self.add_tool_button("Fill", "color-fill-symbolic", ToolType::Fill);

        // Vector tools
        self.add_tool_button("Rectangle", "draw-rectangle-symbolic", ToolType::VectorRectangle);
        self.add_tool_button("Ellipse", "draw-ellipse-symbolic", ToolType::VectorEllipse);
        self.add_tool_button("Path", "draw-path-symbolic", ToolType::VectorPath);
        self.add_tool_button("Text", "insert-text-symbolic", ToolType::VectorText);

        // Other tools
        self.add_tool_button("Crop", "edit-cut-symbolic", ToolType::Crop);
        self.add_tool_button("Text", "insert-text-symbolic", ToolType::Text);
        self.add_tool_button("Gradient", "color-gradient-symbolic", ToolType::Gradient);
        self.add_tool_button("Color Picker", "color-select-symbolic", ToolType::ColorPicker);
        self.add_tool_button("Zoom", "zoom-fit-best-symbolic", ToolType::Zoom);
        self.add_tool_button("Pan", "grab-symbolic", ToolType::Pan);
    }
    
    pub fn set_tool_manager(&mut self, tool_manager: Rc<RefCell<ToolManager>>) {
        self.tool_manager = Some(tool_manager);
    }
    
    pub fn set_active_tool(&self, tool_type: ToolType) {
        for button in &self.tool_buttons {
            button.set_active(false);
        }
        if let Some(button) = self.tool_buttons.iter().find(|b| {
            b.widget_name() == tool_type.to_string()
        }) {
            button.set_active(true);
        }
    }
} 