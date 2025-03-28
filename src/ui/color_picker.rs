use crate::core::Canvas;
use crate::vector::shape::Color;
use cairo::Context;

/// A tool for selecting colors from the canvas
#[derive(Clone)]
pub struct ColorPicker {
    pub current_color: Color,
    pub last_x: Option<f64>,
    pub last_y: Option<f64>,
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            current_color: Color::new(0.0, 0.0, 0.0, 1.0),
            last_x: None,
            last_y: None,
        }
    }
    
    pub fn pick_color(&mut self, _canvas: &Canvas, x: f64, y: f64) -> Color {
        // Simple implementation - would need to actually sample from the canvas in real app
        self.last_x = Some(x);
        self.last_y = Some(y);
        self.current_color.clone()
    }
    
    pub fn draw_preview(&self, context: &Context, _canvas: &Canvas) {
        if let (Some(x), Some(y)) = (self.last_x, self.last_y) {
            // Draw a crosshair at the current point
            context.save().unwrap();
            context.set_source_rgba(0.0, 0.0, 0.0, 0.8);
            
            // Draw horizontal line
            context.move_to(x - 10.0, y);
            context.line_to(x + 10.0, y);
            
            // Draw vertical line
            context.move_to(x, y - 10.0);
            context.line_to(x, y + 10.0);
            
            context.set_line_width(1.0);
            context.stroke().unwrap();
            context.restore().unwrap();
        }
    }
} 