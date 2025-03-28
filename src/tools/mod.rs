// Tools module: Contains all tools for image editing

mod selection;
mod brush;
mod clone;
mod heal;
mod crop;
mod text;
mod gradient;
mod vector_tools;

pub use selection::{SelectionTool, SelectionType};
pub use brush::{BrushTool, BrushType, BrushSettings};
pub use clone::{CloneTool, CloneSettings};
pub use heal::{HealTool, HealSettings};
pub use crop::CropTool;
pub use text::TextTool;
pub use gradient::{GradientTool, GradientType};
pub use vector_tools::{RectangleTool, EllipseTool, PathTool, TextTool as VectorTextTool};

use cairo::Context;
use image::{DynamicImage, ImageBuffer, Rgba};
use crate::core::{Canvas, Layer, LayerManager, Selection, Color};
use crate::core::Point as CorePoint;
use crate::core::selection::Selection as CoreSelection;
use crate::core::canvas::Selection as CanvasSelection;
use crate::vector::{Point as VectorPoint, VectorDocument, Rect};
use std::str::FromStr;
use std::cell::RefMut;

/// The different types of tools available in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    // Selection tools
    Move,
    Selection,
    RectangleSelection,
    EllipseSelection,
    LassoSelection,
    MagicWandSelection,
    
    // Paint tools
    Brush,
    Eraser,
    Clone,
    Heal,
    Fill,
    
    // Vector tools
    VectorRectangle,
    VectorEllipse,
    VectorPath,
    VectorText,
    
    // Other tools
    Crop,
    Text,
    Gradient,
    ColorPicker,
    Zoom,
    Pan,
}

/// Custom error type for ToolType parsing
#[derive(Debug, Clone)]
pub struct ToolTypeParseError;

impl std::fmt::Display for ToolTypeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid tool type")
    }
}

impl std::error::Error for ToolTypeParseError {}

impl FromStr for ToolType {
    type Err = ToolTypeParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Move" => Ok(ToolType::Move),
            "Selection" => Ok(ToolType::Selection),
            "RectangleSelection" => Ok(ToolType::RectangleSelection),
            "EllipseSelection" => Ok(ToolType::EllipseSelection),
            "LassoSelection" => Ok(ToolType::LassoSelection),
            "MagicWandSelection" => Ok(ToolType::MagicWandSelection),
            "Brush" => Ok(ToolType::Brush),
            "Eraser" => Ok(ToolType::Eraser),
            "Clone" => Ok(ToolType::Clone),
            "Heal" => Ok(ToolType::Heal),
            "Fill" => Ok(ToolType::Fill),
            "VectorRectangle" => Ok(ToolType::VectorRectangle),
            "VectorEllipse" => Ok(ToolType::VectorEllipse),
            "VectorPath" => Ok(ToolType::VectorPath),
            "VectorText" => Ok(ToolType::VectorText),
            "Crop" => Ok(ToolType::Crop),
            "Text" => Ok(ToolType::Text),
            "Gradient" => Ok(ToolType::Gradient),
            "ColorPicker" => Ok(ToolType::ColorPicker),
            "Zoom" => Ok(ToolType::Zoom),
            "Pan" => Ok(ToolType::Pan),
            _ => Err(ToolTypeParseError),
        }
    }
}

/// Base trait for all tools
pub trait Tool {
    fn tool_type(&self) -> ToolType;
    fn cursor(&self) -> &'static str;
    fn active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    fn mouse_down(&mut self, x: f64, y: f64, button: u32);
    fn mouse_move(&mut self, x: f64, y: f64);
    fn mouse_up(&mut self, x: f64, y: f64, button: u32);
    fn key_press(&mut self, key: &str);
    fn draw_preview(&self, context: &Context, canvas: &Canvas);
}

/// Trait for tool implementations
pub trait ToolImpl {
    /// Called when the mouse button is pressed
    fn on_mouse_down(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool;
    
    /// Called when the mouse is dragged with button pressed
    fn on_mouse_drag(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool;
    
    /// Called when the mouse button is released
    fn on_mouse_up(&mut self, canvas: &mut Canvas, x: f64, y: f64) -> bool;
    
    /// Gets the cursor type to display
    fn get_cursor(&self) -> Option<String> {
        None
    }
    
    /// Optional method to draw tool-specific UI elements
    fn draw(&self, canvas: &Canvas, context: &cairo::Context) {}
}

/// A collection of all available tools
pub struct ToolManager {
    pub active_tool: ToolType,
    pub selection_tool: SelectionTool,
    pub brush_tool: BrushTool,
    pub clone_tool: CloneTool,
    pub heal_tool: HealTool,
    pub crop_tool: CropTool,
    pub text_tool: TextTool,
    pub gradient_tool: GradientTool,
    pub rectangle_tool: RectangleTool,
    pub ellipse_tool: EllipseTool,
    pub path_tool: PathTool,
    pub vector_text_tool: VectorTextTool,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            active_tool: ToolType::Brush,
            selection_tool: SelectionTool::new(),
            brush_tool: BrushTool::new(),
            clone_tool: CloneTool::new(),
            heal_tool: HealTool::new(),
            crop_tool: CropTool::new(),
            text_tool: TextTool::new(),
            gradient_tool: GradientTool::new(),
            rectangle_tool: RectangleTool::new(),
            ellipse_tool: EllipseTool::new(),
            path_tool: PathTool::new(),
            vector_text_tool: VectorTextTool::new(),
        }
    }
    
    pub fn set_active_tool(&mut self, tool_type: ToolType) {
        // Deactivate current tool
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.set_active(false),
            
            ToolType::Brush => self.brush_tool.set_active(false),
            ToolType::Clone => self.clone_tool.set_active(false),
            ToolType::Heal => self.heal_tool.set_active(false),
            ToolType::Crop => self.crop_tool.set_active(false),
            ToolType::Text => self.text_tool.set_active(false),
            ToolType::Gradient => self.gradient_tool.set_active(false),
            
            ToolType::VectorRectangle => self.rectangle_tool.cancel(),
            ToolType::VectorEllipse => self.ellipse_tool.cancel(),
            ToolType::VectorPath => self.path_tool.cancel(),
            ToolType::VectorText => self.vector_text_tool.reset(),
            
            _ => {}
        }
        
        // Activate new tool
        match tool_type {
            ToolType::RectangleSelection => {
                self.selection_tool.set_selection_type(SelectionType::Rectangle);
                self.selection_tool.set_active(true);
            },
            ToolType::EllipseSelection => {
                self.selection_tool.set_selection_type(SelectionType::Ellipse);
                self.selection_tool.set_active(true);
            },
            ToolType::LassoSelection => {
                self.selection_tool.set_selection_type(SelectionType::Lasso);
                self.selection_tool.set_active(true);
            },
            ToolType::MagicWandSelection => {
                self.selection_tool.set_selection_type(SelectionType::MagicWand);
                self.selection_tool.set_active(true);
            },
            
            ToolType::Brush => self.brush_tool.set_active(true),
            ToolType::Clone => self.clone_tool.set_active(true),
            ToolType::Heal => self.heal_tool.set_active(true),
            ToolType::Crop => self.crop_tool.set_active(true),
            ToolType::Text => self.text_tool.set_active(true),
            ToolType::Gradient => self.gradient_tool.set_active(true),
            
            _ => {}
        }
        
        self.active_tool = tool_type;
    }
    
    pub fn get_active_tool(&self) -> ToolType {
        self.active_tool
    }
    
    pub fn get_cursor(&self) -> &'static str {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.cursor(),
            
            ToolType::Brush => self.brush_tool.cursor(),
            ToolType::Clone => self.clone_tool.cursor(),
            ToolType::Heal => self.heal_tool.cursor(),
            ToolType::Crop => self.crop_tool.cursor(),
            ToolType::Text => self.text_tool.cursor(),
            ToolType::Gradient => self.gradient_tool.cursor(),
            
            ToolType::VectorRectangle => "crosshair",
            ToolType::VectorEllipse => "crosshair",
            ToolType::VectorPath => "crosshair",
            ToolType::VectorText => "text",
            
            ToolType::ColorPicker => "cell",
            ToolType::Zoom => "zoom-in",
            ToolType::Pan => "grab",
            _ => "default",
        }
    }
    
    pub fn mouse_down(&mut self, x: f64, y: f64, button: u32, canvas: &mut Canvas) {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.mouse_down(x, y, button),
            
            ToolType::Brush => self.brush_tool.mouse_down(x, y, button),
            ToolType::Clone => self.clone_tool.mouse_down(x, y, button),
            ToolType::Heal => self.heal_tool.mouse_down(x, y, button),
            ToolType::Crop => self.crop_tool.mouse_down(x, y, button),
            ToolType::Text => self.text_tool.mouse_down(x, y, button),
            ToolType::Gradient => self.gradient_tool.mouse_down(x, y, button),
            
            ToolType::VectorRectangle => self.rectangle_tool.start(x, y),
            ToolType::VectorEllipse => self.ellipse_tool.start(x, y),
            ToolType::VectorPath => self.path_tool.start(x, y),
            ToolType::VectorText => {
                // Handle vector text tool click
                if let Some(vector_doc) = &mut canvas.vector_document {
                    self.vector_text_tool.click(x, y, vector_doc);
                }
            },
            
            _ => {}
        }
    }
    
    pub fn mouse_move(&mut self, x: f64, y: f64, canvas: &mut Canvas) {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.mouse_move(x, y),
            
            ToolType::Brush => self.brush_tool.mouse_move(x, y),
            ToolType::Clone => self.clone_tool.mouse_move(x, y),
            ToolType::Heal => self.heal_tool.mouse_move(x, y),
            ToolType::Crop => self.crop_tool.mouse_move(x, y),
            ToolType::Text => self.text_tool.mouse_move(x, y),
            ToolType::Gradient => self.gradient_tool.mouse_move(x, y),
            
            ToolType::VectorRectangle => self.rectangle_tool.update(x, y),
            ToolType::VectorEllipse => self.ellipse_tool.update(x, y),
            ToolType::VectorPath => {
                // Just update preview, don't add points on move
            },
            
            _ => {}
        }
    }
    
    pub fn mouse_up(&mut self, x: f64, y: f64, button: u32, canvas: &mut Canvas) {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => {
                self.selection_tool.mouse_up(x, y, button);
                // Apply selection to canvas
                if let Some(selection) = self.selection_tool.get_selection() {
                    // Convert from CoreSelection to CanvasSelection
                    let canvas_selection = CanvasSelection::new(
                        selection.x as i32, 
                        selection.y as i32,
                        selection.width as i32,
                        selection.height as i32
                    );
                    canvas.set_selection(canvas_selection);
                }
            },
            
            ToolType::Brush => self.brush_tool.mouse_up(x, y, button),
            ToolType::Clone => self.clone_tool.mouse_up(x, y, button),
            ToolType::Heal => self.heal_tool.mouse_up(x, y, button),
            ToolType::Crop => {
                self.crop_tool.mouse_up(x, y, button);
                if self.crop_tool.is_complete() {
                    // Apply crop to canvas
                    if let Some(rect) = self.crop_tool.get_crop_rect() {
                        canvas.crop(rect.x as u32, rect.y as u32, rect.width as u32, rect.height as u32);
                    }
                    self.crop_tool.reset();
                }
            },
            ToolType::Text => self.text_tool.mouse_up(x, y, button),
            ToolType::Gradient => self.gradient_tool.mouse_up(x, y, button),
            
            ToolType::VectorRectangle => {
                if let Some(vector_doc) = &mut canvas.vector_document {
                    self.rectangle_tool.end(vector_doc);
                }
            },
            ToolType::VectorEllipse => {
                if let Some(vector_doc) = &mut canvas.vector_document {
                    self.ellipse_tool.end(vector_doc);
                }
            },
            ToolType::VectorPath => {
                if button == 3 { // Right click
                    if let Some(vector_doc) = &mut canvas.vector_document {
                        self.path_tool.end(true, vector_doc);
                    }
                } else {
                    // On left click, just add a point
                    self.path_tool.add_point(x, y);
                }
            },
            
            _ => {}
        }
    }
    
    pub fn key_press(&mut self, key: &str, canvas: &mut Canvas) {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.key_press(key),
            
            ToolType::Brush => self.brush_tool.key_press(key),
            ToolType::Clone => self.clone_tool.key_press(key),
            ToolType::Heal => self.heal_tool.key_press(key),
            ToolType::Crop => self.crop_tool.key_press(key),
            ToolType::Text => self.text_tool.key_press(key),
            ToolType::Gradient => self.gradient_tool.key_press(key),
            
            ToolType::VectorPath => {
                if key == "Escape" {
                    self.path_tool.cancel();
                } else if key == "Return" {
                    if let Some(vector_doc) = &mut canvas.vector_document {
                        self.path_tool.end(false, vector_doc);
                    }
                }
            },
            
            _ => {}
        }
    }
    
    pub fn draw_preview(&self, context: &Context, canvas: &Canvas) {
        match self.active_tool {
            ToolType::RectangleSelection |
            ToolType::EllipseSelection |
            ToolType::LassoSelection |
            ToolType::MagicWandSelection => self.selection_tool.draw_preview(context, canvas),
            
            ToolType::Brush => self.brush_tool.draw_preview(context, canvas),
            ToolType::Clone => self.clone_tool.draw_preview(context, canvas),
            ToolType::Heal => self.heal_tool.draw_preview(context, canvas),
            ToolType::Crop => self.crop_tool.draw_preview(context, canvas),
            ToolType::Text => self.text_tool.draw_preview(context, canvas),
            ToolType::Gradient => self.gradient_tool.draw_preview(context, canvas),
            
            ToolType::VectorRectangle => self.rectangle_tool.draw_preview(context),
            ToolType::VectorEllipse => self.ellipse_tool.draw_preview(context),
            ToolType::VectorPath => {
                // Get current mouse position for preview
                let (x, y) = canvas.get_mouse_position();
                self.path_tool.draw_preview(context, x, y);
            },
            ToolType::VectorText => {
                // Get current mouse position for preview
                let (x, y) = canvas.get_mouse_position();
                self.vector_text_tool.draw_preview(context, x, y);
            },
            
            _ => {}
        }
    }
}

impl Clone for ToolManager {
    fn clone(&self) -> Self {
        Self {
            active_tool: self.active_tool,
            selection_tool: self.selection_tool.clone(),
            brush_tool: self.brush_tool.clone(),
            clone_tool: self.clone_tool.clone(),
            heal_tool: self.heal_tool.clone(),
            crop_tool: self.crop_tool.clone(),
            text_tool: self.text_tool.clone(),
            gradient_tool: self.gradient_tool.clone(),
            rectangle_tool: self.rectangle_tool.clone(),
            ellipse_tool: self.ellipse_tool.clone(),
            path_tool: self.path_tool.clone(),
            vector_text_tool: self.vector_text_tool.clone(),
        }
    }
}

// Remove duplicate tool imports
// pub use brush::BrushTool;
// pub use clone::CloneTool;
// pub use heal::HealTool;
// pub use crop::CropTool;
// pub use selection::SelectionTool;
// pub use gradient::GradientTool;
// pub use text::TextTool;
pub use transform::TransformTool;
pub use paint::PaintTool;
pub use vector::VectorTool;
// pub use vector_tools::*; // Already imported above

// Add extension method for RefMut<ToolManager>
pub trait ToolManagerExt {
    fn select_tool(&mut self, tool_type: ToolType);
}

impl ToolManagerExt for RefMut<'_, ToolManager> {
    fn select_tool(&mut self, tool_type: ToolType) {
        self.set_active_tool(tool_type);
    }
} 