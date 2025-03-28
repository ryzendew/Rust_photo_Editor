use cairo::Context;
use crate::core::{Canvas, Point, Selection};
use crate::vector::{VectorShape, SelectionState};
use crate::vector::document::VectorDocument;
use crate::vector::shape::VectorShape as ShapeImpl;
use crate::vector::text::TextShape;
use image::{ImageBuffer, Rgba, GenericImageView};
use super::ToolImpl;

/// Types of selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionType {
    Rectangle,
    Ellipse,
    Lasso,
    MagicWand,
}

/// Tool for creating and manipulating selections
#[derive(Clone)]
pub struct SelectionTool {
    pub selection_type: SelectionType,
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub is_active: bool,
    pub is_selecting: bool,
    pub selection: Option<Selection>,
    pub points: Vec<Point>,
}

impl SelectionTool {
    pub fn new() -> Self {
        Self {
            selection_type: SelectionType::Rectangle,
            start_point: None,
            end_point: None,
            is_active: false,
            is_selecting: false,
            selection: None,
            points: Vec::new(),
        }
    }
    
    pub fn cursor(&self) -> &'static str {
        match self.selection_type {
            SelectionType::Rectangle => "crosshair",
            SelectionType::Ellipse => "crosshair",
            SelectionType::Lasso => "crosshair",
            SelectionType::MagicWand => "cell",
        }
    }
    
    pub fn active(&self) -> bool {
        self.is_active
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
    
    pub fn set_selection_type(&mut self, selection_type: SelectionType) {
        self.selection_type = selection_type;
    }
    
    pub fn get_selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }
    
    pub fn mouse_down(&mut self, x: f64, y: f64, button: u32) {
        if !self.is_active || button != 1 {
            return;
        }
        
        self.is_selecting = true;
        self.start_point = Some(Point::new(x, y));
        self.end_point = Some(Point::new(x, y));
        
        match self.selection_type {
            SelectionType::Lasso => {
                self.points.clear();
                self.points.push(Point::new(x, y));
            },
            _ => {}
        }
    }
    
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if !self.is_selecting {
            return;
        }
        
        self.end_point = Some(Point::new(x, y));
        
        match self.selection_type {
            SelectionType::Lasso => {
                self.points.push(Point::new(x, y));
            },
            _ => {}
        }
    }
    
    pub fn mouse_up(&mut self, x: f64, y: f64, button: u32) {
        if !self.is_selecting || button != 1 {
            return;
        }
        
        self.is_selecting = false;
        self.end_point = Some(Point::new(x, y));
        
        if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
            // If the selection is too small, clear it
            if (end.x - start.x).abs() < 3.0 && (end.y - start.y).abs() < 3.0 {
                self.selection = None;
                return;
            }
            
            // Create selection based on the type
            self.create_selection(start, end);
        }
    }
    
    pub fn key_press(&mut self, key: &str) {
        if key == "Escape" {
            self.reset();
        }
    }
    
    pub fn reset(&mut self) {
        self.is_selecting = false;
        self.start_point = None;
        self.end_point = None;
        self.selection = None;
        self.points.clear();
    }
    
    fn create_selection(&mut self, start: Point, end: Point) {
        // Calculate the selection rectangle
        let x = start.x.min(end.x) as u32;
        let y = start.y.min(end.y) as u32;
        let width = (start.x - end.x).abs() as u32;
        let height = (start.y - end.y).abs() as u32;
        
        // Create selection based on type
        match self.selection_type {
            SelectionType::Rectangle => {
                // Make a rectangle selection with width and height of the canvas
                let canvas_width = 2000; // Should be actual canvas width
                let canvas_height = 2000; // Should be actual canvas height
                self.selection = Some(Selection::rectangle(x.into(), y.into(), width, height, canvas_width, canvas_height));
            },
            SelectionType::Ellipse => {
                // Make an ellipse selection with width and height of the canvas
                let canvas_width = 2000; // Should be actual canvas width
                let canvas_height = 2000; // Should be actual canvas height
                self.selection = Some(Selection::ellipse(x.into(), y.into(), width, height, canvas_width, canvas_height));
            },
            SelectionType::Lasso => {
                // Lasso selection is not implemented yet - use rectangle as fallback
                let canvas_width = 2000; // Should be actual canvas width
                let canvas_height = 2000; // Should be actual canvas height
                self.selection = Some(Selection::rectangle(x.into(), y.into(), width, height, canvas_width, canvas_height));
            },
            SelectionType::MagicWand => {
                // Magic wand is not implemented yet - use rectangle as fallback
                let canvas_width = 2000; // Should be actual canvas width
                let canvas_height = 2000; // Should be actual canvas height
                self.selection = Some(Selection::rectangle(x.into(), y.into(), width, height, canvas_width, canvas_height));
            },
        }
    }
    
    pub fn draw_preview(&self, context: &Context, canvas: &Canvas) {
        if !self.is_active {
            return;
        }
        
        context.save();
        
        // If we're in vector mode, display the selection differently
        if canvas.has_vector_mode {
            if let (Some(vector_doc), Some(start), Some(end)) = (&canvas.vector_document, self.start_point, self.end_point) {
                // We are selecting vector objects, draw a selection rectangle
                let x = start.x.min(end.x);
                let y = start.y.min(end.y);
                let width = (start.x - end.x).abs();
                let height = (start.y - end.y).abs();
                
                context.set_source_rgba(0.0, 0.7, 1.0, 0.3);
                context.rectangle(x, y, width, height);
                context.fill();
                
                context.set_source_rgba(0.0, 0.7, 1.0, 0.8);
                context.set_line_width(1.0);
                context.set_dash(&[3.0, 3.0], 0.0);
                context.rectangle(x, y, width, height);
                context.stroke();
            }
        } else {
            // Standard raster selection
            if self.is_selecting {
                if let (Some(start), Some(end)) = (self.start_point, self.end_point) {
                    match self.selection_type {
                        SelectionType::Rectangle => {
                            // Draw rectangle selection
                            let x = start.x.min(end.x);
                            let y = start.y.min(end.y);
                            let width = (start.x - end.x).abs();
                            let height = (start.y - end.y).abs();
                            
                            context.set_source_rgba(0.0, 0.7, 1.0, 0.3);
                            context.rectangle(x, y, width, height);
                            context.fill();
                            
                            context.set_source_rgba(0.0, 0.7, 1.0, 0.8);
                            context.set_line_width(1.0);
                            context.set_dash(&[3.0, 3.0], 0.0);
                            context.rectangle(x, y, width, height);
                            context.stroke();
                        },
                        SelectionType::Ellipse => {
                            // Draw ellipse selection
                            let x = start.x.min(end.x);
                            let y = start.y.min(end.y);
                            let width = (start.x - end.x).abs();
                            let height = (start.y - end.y).abs();
                            
                            let center_x = x + width / 2.0;
                            let center_y = y + height / 2.0;
                            let radius_x = width / 2.0;
                            let radius_y = height / 2.0;
                            
                            context.save();
                            context.translate(center_x, center_y);
                            context.scale(radius_x, radius_y);
                            
                            context.set_source_rgba(0.0, 0.7, 1.0, 0.3);
                            context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                            context.fill();
                            
                            context.set_source_rgba(0.0, 0.7, 1.0, 0.8);
                            context.set_line_width(1.0 / radius_x.min(radius_y));
                            context.set_dash(&[3.0 / radius_x.min(radius_y), 3.0 / radius_x.min(radius_y)], 0.0);
                            context.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                            context.stroke();
                            
                            context.restore();
                        },
                        SelectionType::Lasso => {
                            // Draw lasso selection
                            if self.points.len() > 1 {
                                context.set_source_rgba(0.0, 0.7, 1.0, 0.3);
                                context.move_to(self.points[0].x, self.points[0].y);
                                
                                for i in 1..self.points.len() {
                                    context.line_to(self.points[i].x, self.points[i].y);
                                }
                                
                                context.close_path();
                                context.fill();
                                
                                context.set_source_rgba(0.0, 0.7, 1.0, 0.8);
                                context.set_line_width(1.0);
                                context.set_dash(&[3.0, 3.0], 0.0);
                                
                                context.move_to(self.points[0].x, self.points[0].y);
                                
                                for i in 1..self.points.len() {
                                    context.line_to(self.points[i].x, self.points[i].y);
                                }
                                
                                context.close_path();
                                context.stroke();
                            }
                        },
                        SelectionType::MagicWand => {
                            // Magic wand preview is not implemented yet
                        },
                    }
                }
            }
            
            // Draw existing selection
            if let Some(selection) = &self.selection {
                selection.render_outline(context);
            }
        }
        
        context.restore();
    }
    
    /// Select vector objects that intersect with the selection rectangle
    pub fn select_vector_objects(&self, canvas: &mut Canvas) {
        if !canvas.has_vector_mode {
            return;
        }
        
        if let (Some(vector_doc), Some(start), Some(end)) = (&mut canvas.vector_document, self.start_point, self.end_point) {
            let x = start.x.min(end.x);
            let y = start.y.min(end.y);
            let width = (start.x - end.x).abs();
            let height = (start.y - end.y).abs();
            
            // Since we can't directly select objects in VectorDocument through 
            // the public API, we'll create a rectangle shape to represent the selection
            let shape = VectorShape::rectangle(x, y, width, height);
            
            // Add the shape to the active layer
            vector_doc.add_shape(shape);
        }
    }
} 