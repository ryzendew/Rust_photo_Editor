use crate::core::canvas::Tool;
use super::ToolImpl;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VectorMode {
    Path,
    Rectangle,
    Ellipse,
    Polygon,
    Star,
    Pen,
}

pub struct VectorTool {
    name: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    active: bool,
    start_x: f64,
    start_y: f64,
    current_x: f64,
    current_y: f64,
    is_drawing: bool,
    mode: VectorMode,
    stroke_width: f64,
    stroke_color: (f64, f64, f64, f64), // RGBA
    fill_color: (f64, f64, f64, f64),   // RGBA
    points: Vec<(f64, f64)>,
}

impl VectorTool {
    pub fn new() -> Self {
        Self {
            name: "Vector",
            icon: "insert-object-symbolic",
            tooltip: "Create vector shapes and paths",
            active: false,
            start_x: 0.0,
            start_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            is_drawing: false,
            mode: VectorMode::Path,
            stroke_width: 2.0,
            stroke_color: (0.0, 0.0, 0.0, 1.0), // Black
            fill_color: (1.0, 1.0, 1.0, 0.0),   // Transparent white
            points: Vec::new(),
        }
    }
    
    pub fn set_mode(&mut self, mode: VectorMode) {
        self.mode = mode;
    }
    
    pub fn set_stroke_width(&mut self, width: f64) {
        self.stroke_width = width;
    }
    
    pub fn set_stroke_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.stroke_color = (r, g, b, a);
    }
    
    pub fn set_fill_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.fill_color = (r, g, b, a);
    }
}

impl ToolImpl for VectorTool {
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn icon(&self) -> &'static str {
        self.icon
    }
    
    fn tooltip(&self) -> &'static str {
        self.tooltip
    }
    
    fn tool_type(&self) -> Tool {
        Tool::Vector
    }
    
    fn activate(&mut self) {
        self.active = true;
    }
    
    fn deactivate(&mut self) {
        self.active = false;
    }
    
    fn handle_mouse_down(&mut self, x: f64, y: f64, _button: u32) {
        self.start_x = x;
        self.start_y = y;
        self.current_x = x;
        self.current_y = y;
        self.is_drawing = true;
        
        match self.mode {
            VectorMode::Path | VectorMode::Pen => {
                self.points.clear();
                self.points.push((x, y));
            },
            _ => {
                // For shapes, just store the start point
            }
        }
    }
    
    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.is_drawing {
            self.current_x = x;
            self.current_y = y;
            
            match self.mode {
                VectorMode::Path | VectorMode::Pen => {
                    self.points.push((x, y));
                },
                _ => {
                    // For shapes, just update the current point
                }
            }
        }
    }
    
    fn handle_mouse_up(&mut self, x: f64, y: f64, _button: u32) {
        self.current_x = x;
        self.current_y = y;
        self.is_drawing = false;
        
        // Finalize the vector object and add it to the document
        match self.mode {
            VectorMode::Path | VectorMode::Pen => {
                // Create a path with the collected points
                if !self.points.is_empty() {
                    // Add the path to the document
                    // In a real implementation, this would create a permanent
                    // vector object and add it to the document
                }
                self.points.clear();
            },
            VectorMode::Rectangle => {
                // Create a rectangle
                let x = self.start_x.min(self.current_x);
                let y = self.start_y.min(self.current_y);
                let width = (self.current_x - self.start_x).abs();
                let height = (self.current_y - self.start_y).abs();
                
                // Add the rectangle to the document
            },
            VectorMode::Ellipse => {
                // Create an ellipse
                let center_x = (self.start_x + self.current_x) / 2.0;
                let center_y = (self.start_y + self.current_y) / 2.0;
                let radius_x = (self.current_x - self.start_x).abs() / 2.0;
                let radius_y = (self.current_y - self.start_y).abs() / 2.0;
                
                // Add the ellipse to the document
            },
            VectorMode::Polygon => {
                // Create a polygon
                // In a real implementation, this would involve creating a 
                // regular polygon with the specified number of sides
            },
            VectorMode::Star => {
                // Create a star
                // In a real implementation, this would involve creating a 
                // star shape with the specified number of points
            },
        }
    }
    
    fn handle_key_press(&mut self, key: &str) {
        match key {
            "Shift" => {
                // Constrain shape
            },
            "Alt" => {
                // Draw from center
            },
            _ => {},
        }
    }
    
    fn handle_key_release(&mut self, key: &str) {
        match key {
            "Shift" => {
                // Stop constraining shape
            },
            "Alt" => {
                // Stop drawing from center
            },
            _ => {},
        }
    }
} 