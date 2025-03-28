use crate::core::canvas::Tool;
use super::ToolImpl;

pub struct PaintTool {
    name: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    active: bool,
    last_x: f64,
    last_y: f64,
    current_x: f64,
    current_y: f64,
    is_painting: bool,
    brush_size: f64,
    brush_opacity: f64,
    pressure_sensitivity: bool,
}

impl PaintTool {
    pub fn new() -> Self {
        Self {
            name: "Paint",
            icon: "applications-graphics-symbolic",
            tooltip: "Paint with a brush",
            active: false,
            last_x: 0.0,
            last_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            is_painting: false,
            brush_size: 20.0,
            brush_opacity: 1.0,
            pressure_sensitivity: true,
        }
    }
    
    pub fn set_brush_size(&mut self, size: f64) {
        self.brush_size = size;
    }
    
    pub fn set_brush_opacity(&mut self, opacity: f64) {
        self.brush_opacity = opacity.clamp(0.0, 1.0);
    }
    
    pub fn set_pressure_sensitivity(&mut self, enabled: bool) {
        self.pressure_sensitivity = enabled;
    }
}

impl ToolImpl for PaintTool {
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
        Tool::Paint
    }
    
    fn activate(&mut self) {
        self.active = true;
    }
    
    fn deactivate(&mut self) {
        self.active = false;
    }
    
    fn handle_mouse_down(&mut self, x: f64, y: f64, _button: u32) {
        self.last_x = x;
        self.last_y = y;
        self.current_x = x;
        self.current_y = y;
        self.is_painting = true;
        
        // Draw a dot at the initial position
    }
    
    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        if self.is_painting {
            self.last_x = self.current_x;
            self.last_y = self.current_y;
            self.current_x = x;
            self.current_y = y;
            
            // Draw a line from last position to current position
            // using vector paths for clean scalable strokes
        }
    }
    
    fn handle_mouse_up(&mut self, x: f64, y: f64, _button: u32) {
        self.current_x = x;
        self.current_y = y;
        self.is_painting = false;
        
        // Finalize the brush stroke
    }
    
    fn handle_key_press(&mut self, key: &str) {
        match key {
            "[" => {
                // Decrease brush size
                self.brush_size = (self.brush_size - 5.0).max(1.0);
            },
            "]" => {
                // Increase brush size
                self.brush_size = (self.brush_size + 5.0).min(500.0);
            },
            _ => {},
        }
    }
    
    fn handle_key_release(&mut self, _key: &str) {
        // No special handling needed for key release
    }
} 